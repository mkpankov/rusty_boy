#![feature(phase)]
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate scan;

extern crate serialize;

extern crate scan_util;

extern crate term;
extern crate time;

use serialize::json;
use std::io::fs;
use std::io;
use std::num::Float as Float;
use std::num::{pow, from_int, from_uint, from_u64, from_f64};
use std::rand;
use std::rand::{Rng, TaskRng};
use std::rand::distributions::{IndependentSample, Range};
use time::precise_time_ns;

static BASE_POINTS: int = 1i;

fn time_multiplier(time: f64) -> f64 {
    let x = time;

    let y =  match x {
        x if x < 0.25 => 5.,
        x if x > 20.  => 0.,
        x if x > 10.  => 0.1,
        x if x > 5.   => 1.,
        _ => 1. / x
    };
    info!("tm({}) -> {}", x, y)

    y
}

fn complexity_multiplier(r: Round) -> f64 {
    let answer = r.a + r.b;
    let f : f64 = from_int(answer)
        .expect("Couldn't convert answer to float");
    let l = Float::log10(f);
    let fc = match r.description.as_slice() {
        "+" => 2.,
        "-" => 3.,
        "*" => 4.,
        "/" => 8.,
        _   => panic!("Couldn't calculate complexity for unknown function")
    };
    l * fc
}

fn full_multiplier(r: Round) -> uint {
    let time_us = r.end - r.start;
    let time = time_us / pow(10, 9);
    info!("time_us: {}, time: {}", time_us, time);
    let tm =
        time_multiplier(from_u64(time)
                        .expect("Time of trial can't be converted to f64"));
    let cm =
        complexity_multiplier(r);
    from_f64(
        Float::round (10. * tm * cm))
        .expect("Full multiplier can't be converted to int")
}

#[allow(dead_code)]
fn compute_mean(times: Vec<u64>) -> f64 {
    let n : f64 = from_uint(times.len())
        .expect("Couldn't convert length of times in compute_mean");
    let sum : f64 = from_u64(times.iter().fold(0, |a, &e| a + e))
        .expect("Couldn't convert sum of times in compute_mean");
    sum / n
}

fn compute_median(mut times: Vec<u64>) -> f64 {
    times.sort();
    match times.len() {
        n if n % 2 == 0 => from_u64( (times[n/2] + times[n/2 - 1]) / 2 )
            .expect("Couldn't convert median in case of even length"),
        n               => from_u64(  times[n/2] )
            .expect("Couldn't convert median in case of odd length"),
    }
}

struct SymbolMap<'a> {
    invitation: &'a str,
    checkmark: &'a str,
    wrongmark: &'a str,
}

fn setup_symbols<'a>() -> SymbolMap<'a> {
    if std::os::args().iter().any(|x| x.as_slice() == "--unicode") {
        SymbolMap {
            invitation: "□",
            checkmark: "✓",
            wrongmark: "✗",
        }
    } else {
        SymbolMap {
            invitation: "o",
            checkmark: "V",
            wrongmark: "X",
        }
    }
}

#[deriving(PartialEq, Eq, PartialOrd, Ord, Show, FromPrimitive)]
enum Kind {
    Add_ = 0,
    Sub_,
    Mul_,
}


fn rand_kind<R: Rng>(low: Kind, high: Kind, rng: &mut R) -> Kind {
    let r = Range::new(low as uint, high as uint);
    from_uint(r.ind_sample(rng))
        .expect("Couldn't convert uint to Kind in rand_kind")
}


fn handle_input<'a>(
    r: Round,
    s: State,
    sm: SymbolMap,
) -> State<'a>
{
    let diff_ms = (r.end - r.start) / pow(10, 6);

    let trimmed = r.input.clone().trim_chars(['\r', '\n'].as_slice());
    let new_is_finished;
    let mut new_times = s.times.clone();
    new_times.push(diff_ms);
    let maybe_c_user : Option<int> = from_str(trimmed);
    let new_attempts;
    let new_combo;

    new_attempts = s.attempts + 1;
    if trimmed == "q" || trimmed == "quit" || new_attempts >= 10 {
        new_is_finished = true;
    } else {
        new_is_finished = false;
    }

    match maybe_c_user {
        Some(c_user) => {
            let c_real : int = (*r.function)(&r.a, &r.b);
            let is_correct = c_user == c_real;
            new_combo = s.combo + 1;
            let mult = full_multiplier(r);
            let pending = BASE_POINTS * from_uint(mult)
                .expect("Couldn't convert multiplier to int");

            let combed = pending * from_uint(s.combo)
                .expect("Couldn't convert combo to int");
            let new_score = s.score + combed;

            let new_s = if is_correct {
                State {
                    times: new_times,
                    correct: s.correct + 1,
                    incorrect: s.incorrect,
                    attempts: new_attempts,
                    combo: new_combo,
                    max_combo: new_combo,
                    score: new_score,
                    is_finished: new_is_finished,
                }
            } else {
                produce_incorrect(&s)
            };

            do_output(&s, &sm,
                      is_correct, pending, new_s.score - s.score,
                      new_score, c_real, mult);
            info!(" {} ms", diff_ms);

            new_s
        },
        None => {
            println!("You didn't input a number.");
            produce_incorrect(&s)
        },
    }
}

fn do_output(s: &State, sm: &SymbolMap,
             is_correct: bool,
             pending: int, combed: int, new_score: int,
             c_real: int, mult: uint) {

    let explanation =
        if mult == 0 {
            "(timeout)"
        } else {
            ""
        };

    let message =
        if is_correct {
            format!(" {:+8}×{:02} = {:+10}! {}",
                    pending, s.combo, combed, explanation)
        } else {
            format!(" {:+8}^W {}.",
                    combed, c_real)
        };
    let color =
        if is_correct {
            term::color::GREEN
        } else {
            term::color::RED
        };
    let mark =
        if is_correct {
            sm.checkmark
        } else {
            sm.wrongmark
        };
    let maybe_term = term::stdout();

    if maybe_term.is_some() {
        let mut term = maybe_term
            .expect("Impossible happened: maybe_term is Some(_) but we couldn't unwrap it");

        // Remember Result<_> panics with Err(message) if it's not Ok(_)
        term.fg(color)
            .unwrap();
        term.attr(term::attr::Bold)
            .unwrap();
        (write!(term, "{:1}", mark))
            .unwrap();
        term.reset()
            .unwrap();
    } else {
        print!("{:1}", mark);
    }

    let maybe_term2 = term::stdout();
    if is_correct {
        if maybe_term2.is_some() {
            let mut term = maybe_term2
                .expect("Impossible happened: maybe_term2 is Some(_) but we couldn't unwrap it");
            (write!(term, "{:15}", message.slice_to(15)))
                .unwrap();
            let c = choose_color(combed);
            term.fg(c)
                .unwrap();
            term.attr(term::attr::Bold)
                .unwrap();
            (write!(term, "{:10}", message.slice(16,26)))
                .unwrap();
            term.reset()
                .unwrap();
            (write!(term, "{:22}{:32}\n", "", new_score))
                .unwrap();
        } else {
            println!("{:47}{:32}", message, new_score);
        }
    } else {
        println!("{:47}{:32}", message, new_score);
    }
}

fn choose_color(points: int) -> term::color::Color {
    let points_f: f64 = from_int(points).expect("Couldn't convert points to float");
    let l = Float::log10(points_f);
    let r: int = from_f64(Float::round(l)).expect("Couldn't convert modifier to int");
    match r {
        0...1 => term::color::BLUE,
        2 => term::color::GREEN,
        3 => term::color::YELLOW,
        4 => term::color::RED,
        _ => term::color::WHITE,
    }
}

#[deriving(Decodable, Encodable, Show)]
struct Level {
    functions: Vec<String>,
    operands_digits: Vec<int>,
    timeout: int,
}

fn produce_incorrect<'a, 'b>(s: &State<'a>) -> State<'b> {
    let new_attempts = s.attempts + 1;
    let new_is_finished = new_attempts >= 10;
    State {
        times: s.times.clone(),
        correct: s.correct,
        incorrect: s.incorrect + 1,
        attempts: new_attempts,
        combo: 1,
        max_combo: s.max_combo,
        score: s.score - BASE_POINTS,
        is_finished: new_is_finished,
    }
}

struct Round<'a> {
    input: &'a str,

    a: int,
    b: int,
    function: &'a fn(&int, &int) -> int,
    description: String,

    start: u64,
    end: u64,
}

struct State<'a> {
    times: Vec<u64>,

    correct: uint,
    incorrect: uint,
    attempts: uint,

    combo: uint,
    max_combo: uint,

    score: int,

    is_finished: bool,
}

fn read_level(path: &Path) -> Result<Level, String> {
    let mut file = match std::io::File::open(path) {
        Err(why) =>
            return Err(format!("couldn't open {}: {}", path.display(), why)),
        Ok(file) => file,
    };

    let level_encoded;
    match file.read_to_string() {
        Err(why) =>
            return Err(format!("couldn't read {}: {}", path.display(), why)),
        Ok(string) => level_encoded = string,
    };

    let level_decoded: json::DecodeResult<Level> =
        json::decode(level_encoded.as_slice());
    match level_decoded {
        Err(why) => Err(format!("{}", why)),
        Ok(level) => Ok(level),
    }
}

struct Game {
    ranges_operands: Vec<Range<int>>,
    range_kind: Range<uint>,
    rng_a: TaskRng,
    rng_b: TaskRng,
    rng_kind: TaskRng,
    functions: Vec<(fn(&int, &int) -> int, String)>,
}

fn setup_game<'a>(l: Level) -> Game {
    let mut ranges_operands = vec![];
    for i in l.operands_digits.iter() {
        ranges_operands.push(Range::new(1, 10 ** i));
    }
    // TODO: Setup a proper mapping of allowed functions
    let range_kind = Range::new(0, l.functions.len());
    let rng_a =    rand::task_rng();
    let rng_b =    rand::task_rng();
    let rng_kind = rand::task_rng();

    let mut functions = vec![];
    for f in l.functions.iter() {
        match f.as_slice() {
            o @ "+" => functions.push((Add::add, o.to_string())),
            o @ "-" => functions.push((Sub::sub, o.to_string())),
            o @ "*" => functions.push((Mul::mul, o.to_string())),
            o @ "/" => functions.push((Div::div, o.to_string())),
            _   => continue,
        }
    }

    Game {
        ranges_operands: ranges_operands,
        range_kind: range_kind,
        rng_a: rng_a,
        rng_b: rng_b,
        rng_kind: rng_kind,
        functions: functions,
    }
}


fn choose_load_level() -> Result<Level, String> {
    let maybe_level;
    let level_dir = Path::new(".");
    let maybe_files = fs::readdir(&level_dir);
    match maybe_files {
        Err(why) => panic!("Failed to read {} directory: {}",
                           level_dir.display(), why),
        Ok(files) => {
            let levels: Vec<&Path> = files.iter().filter(
                |p| p.filename_str()
                    .expect("Couldn't represent filename as str")
                    .ends_with(".lvl.json")).collect();
            let levels_displays : Vec<std::path::Display<Path>> =
                levels.iter().map(|p| p.display()).collect();
            println!("Found levels:");
            for (i, l) in levels_displays.iter().enumerate() {
                println!("{}. {}", i + 1, l);
            }
            print!("Select one: ");
            let result = io::stdio::stdin().read_line();
            match result {
                Err(_) => panic!("Failed to read the choice."),
                Ok(string) => {
                    let trimmed =
                        string.as_slice().trim_chars(['\r', '\n'].as_slice());
                    let maybe_choice: Option<uint> = from_str(trimmed);
                    match maybe_choice {
                        None => panic!("Failed to parse unsigned integer from choice"),
                        Some(choice) => {
                            maybe_level = read_level(levels[choice - 1]);
                        }
                    }
                }
            }
        },
    }
    maybe_level
}


fn main() {
    let sm = setup_symbols();
    let maybe_level = choose_load_level();

    let mut game;
    match maybe_level {
        Err(why) => panic!("{}", why),
        Ok(level) => game = setup_game(level),
    };

    let initial_state =
        State {
            times: vec![],
            correct: 0,
            incorrect: 0,
            combo: 1,
            max_combo: 0,
            attempts: 0,
            score: 0,
            is_finished: false,
        };
    let mut last_state = initial_state;

    loop {
        let a = game.ranges_operands[0].ind_sample(&mut game.rng_a);
        let b = game.ranges_operands[1].ind_sample(&mut game.rng_b);
        let kind = game.range_kind.ind_sample(&mut game.rng_kind);
        let (ref function, ref description) = game.functions[kind];

        print!("{}   {} {} {} = ", sm.invitation, a, description, b);

        let start = precise_time_ns();
        let result = io::stdio::stdin().read_line();
        let end   = precise_time_ns();

        match result {
            Ok(string) => {
                last_state = handle_input(
                    Round {
                        input: string.as_slice(),
                        start: start,
                        end: end,
                        a: a,
                        b: b,
                        function: function,
                        description: description.to_string(),
                    },
                    last_state,
                    sm);
                if last_state.is_finished {
                    break;
                }
            },
            Err(_) => break,
        };
    }
    process_results(last_state);
}

fn process_results<'a>(s: State<'a>) {
    let time_stat : f64 = if s.times.len() != 0 {
        compute_median(s.times)
    } else {
        0.
    };
    let total_trials = s.incorrect + s.correct;
    let rate : f64 = if total_trials != 0 {
        100.
      * from_uint(s.correct)     .expect("Number of correct trials can't be converted to f64")
      / from_uint(total_trials).expect("Total number of trials can't be converted to f64")
    } else {
        0.
    };

    println!("====\n\
             Your score: {}\n\
             Correct answers: {} ({rate:.0f} %), incorrect: {}, total: {}.\n\
             Median time: {:.2f} s.",
             s.score, s.correct, s.incorrect, s.correct + s.incorrect,
             time_stat / 1000., rate=rate);

    let mut recs = read_records();
    process_records(&mut recs, s.score);
    write_records(recs);
}

#[deriving(Show)]
struct Record {
    points: int,
    player: String,
}


fn read_records() -> Vec<Record> {
    use std::io::BufferedReader;
    use std::io::File;

    let path = Path::new("records");
    let mut file = BufferedReader::new(File::open(&path));
    let mut records: Vec<Record> = vec![];

    loop {
        let record : Record;
        let res = scanln_from! {
            &mut file,
            "player: \"" player: &str "\", points: \"" points: int "\"" => {
                record = Record { player : player.to_string(),
                                  points : points };
                records.push(record)
            },
        };
        match res {
            Ok(_) => {
                info!("Read and parsed a record.");
            },
            Err(_) => break,
        }
    }

    records
}


fn insert_record(recs: &mut Vec<Record>, saved: Option<Record>, new: int) {
    let mut stdin = std::io::stdio::stdin();
    print!("Enter your name: ");
    let line = stdin.read_line();
    match line {
        Ok(line) => {
            let name = line.as_slice().trim_chars(['\r', '\n'].as_slice());
            let name_ = name.to_string();

            recs.push( Record { points: new, player: name_ } );
            recs.sort_by(
                |&Record { points: p_a, .. }, &Record { points: p_b, .. }|
                p_b.cmp(&p_a));
        },
        Err(_) => {
            match saved {
                Some(saved) => recs.push(saved),
                None => (),
            }
        }
    }
}


fn process_records(recs: &mut Vec<Record>, new : int) {
    let n = recs.len();
    if n >= 10 {
        match &mut recs[1] {
            &Record { points: old, .. } => {
                if old < new {
                    let saved = recs.pop();

                    insert_record(recs, saved, new);
                }
            }
        }
    } else {
        insert_record(recs, None, new);
    }
}


fn write_records(recs: Vec<Record>) {
    use std::io::File;

    let mut file = File::create(&Path::new("records"));
    for r in recs.iter() {
        match r {
            &Record { ref player, points } => {
                let line = format!("player: \"{:s}\", points: \"{}\"\n",
                                   *player, points);
                file.write(line.as_bytes()).unwrap();
            }
        }
    }
}
