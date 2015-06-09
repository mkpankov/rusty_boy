#[macro_use]
extern crate log;
extern crate env_logger;

extern crate num;

extern crate rand;

extern crate rustc_serialize;

extern crate term;
extern crate time;

use rand::Rng;
use rand::distributions::range::Range;
use rand::ThreadRng;

use rustc_serialize::json;
use std::io;
use std::path::Path;
use time::precise_time_ns;

static BASE_POINTS: isize = 1isize;

fn time_multiplier(time: f64) -> f64 {
    let x = time;

    let y =  match x {
        x if x < 0.25 => 5.,
        x if x > 20.  => 0.,
        x if x > 10.  => 0.1,
        x if x > 5.   => 1.,
        _ => 1. / x
    };
    info!("tm({}) -> {}", x, y);

    y
}

fn complexity_multiplier(r: Round) -> f64 {
    let answer = r.a + r.b;
    let f : f64 = num::FromPrimitive::from_isize(answer)
        .expect("Couldn't convert answer to float");
    let l = num::Float::log10(f);
    let fc = match &*r.description {
        "+" => 2.,
        "-" => 3.,
        "*" => 4.,
        "/" => 8.,
        _   => panic!("Couldn't calculate complexity for unknown function")
    };
    l * fc
}

fn full_multiplier(r: Round) -> usize {
    let time_us = r.end - r.start;
    let time = time_us / 10u64.pow(9);
    info!("time_us: {}, time: {}", time_us, time);
    let tm =
        time_multiplier(
            num::FromPrimitive::from_u64(time)
                .expect("Time of trial can't be converted to f64"));
    let cm =
        complexity_multiplier(r);
    num::FromPrimitive::from_f64(
        num::Float::round (10. * tm * cm))
        .expect("Full multiplier can't be converted to isize")
}

#[allow(dead_code)]
fn compute_mean(times: Vec<u64>) -> f64 {
    let n : f64 =
        num::FromPrimitive::from_usize(times.len())
        .expect("Couldn't convert length of times in compute_mean");
    let sum : f64 =
        num::FromPrimitive::from_u64(times.iter().fold(0, |a, &e| a + e))
        .expect("Couldn't convert sum of times in compute_mean");
    sum / n
}

fn compute_median(mut times: Vec<u64>) -> f64 {
    times.sort();
    match times.len() {
        n if n % 2 == 0 =>
            num::FromPrimitive::from_u64( (times[n/2] + times[n/2 - 1]) / 2 )
            .expect("Couldn't convert median in case of even length"),
        n               =>
            num::FromPrimitive::from_u64(  times[n/2] )
            .expect("Couldn't convert median in case of odd length"),
    }
}

struct SymbolMap<'a> {
    invitation: &'a str,
    checkmark: &'a str,
    wrongmark: &'a str,
}

fn setup_symbols<'a>() -> SymbolMap<'a> {
    if std::env::args().any(|x| x == "--unicode") {
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kind {
    Add_ = 0,
    Sub_,
    Mul_,
}


impl From<usize> for Kind {
    fn from(x: usize) -> Kind {
        use Kind::*;
        match x {
            0 => Add_,
            1 => Sub_,
            2 => Mul_,
            _ => panic!("can't convert it"),
        }
    }
}


fn rand_kind<R: Rng>(low: Kind, high: Kind, rng: &mut R) -> Kind {
    use rand::distributions::IndependentSample;

    let r = Range::new(low as usize, high as usize);
    let n: usize = r.ind_sample(rng);
    Kind::from(n)
}


fn handle_input<'a>(
    r: Round,
    s: State,
    sm: &SymbolMap,
) -> State
{
    let diff_ms = (r.end - r.start) / 10u64.pow(6);

    let newlines: &[_] = &['\r', '\n'];
    let trimmed = r.input.clone().trim_matches(newlines);
    let new_is_finished;
    let mut new_times = s.times.clone();
    new_times.push(diff_ms);
    let maybe_c_user: Result<isize, _> = trimmed.parse();
    let new_attempts;
    let new_combo;

    new_attempts = s.attempts + 1;
    if trimmed == "q" || trimmed == "quit" || new_attempts >= 10 {
        new_is_finished = true;
    } else {
        new_is_finished = false;
    }

    match maybe_c_user {
        Ok(c_user) => {
            let c_real : isize = (*r.function)(r.a, r.b);
            let is_correct = c_user == c_real;
            new_combo = s.combo + 1;
            let mult = full_multiplier(r);
            let pending = BASE_POINTS * mult as isize;

            let combed: isize = pending * s.combo as isize;
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
        Err(_) => {
            println!("You didn't input a number.");
            produce_incorrect(&s)
        },
    }
}

fn do_output(s: &State, sm: &SymbolMap,
             is_correct: bool,
             pending: isize, combed: isize, new_score: isize,
             c_real: isize, mult: usize) {

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
    let mut maybe_term = term::stdout();

    match maybe_term {
        None => {
            print!("{:1}", mark);
            println!("{:47}{:32}", message, new_score);
        },
        Some(ref mut term) => {
            // Remember Result<_> panics with Err(message) if it's not Ok(_)
            term.fg(color)
                .unwrap();
            term.attr(term::Attr::Bold)
                .unwrap();
            (write!(term, "{:1}", mark))
                .unwrap();
            term.reset()
                .unwrap();

            if is_correct {
                (write!(term, "{:15}", &message[0..15]))
                    .unwrap();
                let c = choose_color(combed);
                term.fg(c)
                    .unwrap();
                term.attr(term::Attr::Bold)
                    .unwrap();
                (write!(term, "{:10}", &message[16..26]))
                    .unwrap();
                term.reset()
                    .unwrap();
                (write!(term, "{:22}{:32}\n", "", new_score))
                    .unwrap();
            } else {
                println!("{:47}{:32}", message, new_score);
            }
        },
    }
}

fn choose_color(points: isize) -> term::color::Color {
    let points_f: f64 =
        num::FromPrimitive::from_isize(points)
        .expect("Couldn't convert points to float");
    let l = num::Float::log10(points_f);
    let r: isize =
        num::FromPrimitive::from_f64(num::Float::round(l))
        .expect("Couldn't convert modifier to isize");
    match r {
        0...1 => term::color::BLUE,
        2 => term::color::GREEN,
        3 => term::color::YELLOW,
        4 => term::color::RED,
        _ => term::color::WHITE,
    }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Level {
    functions: Vec<String>,
    operands_digits: Vec<isize>,
    timeout: isize,
}

fn produce_incorrect<'a, 'b>(s: &State) -> State {
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

    a: isize,
    b: isize,
    function: &'a Fn(isize, isize) -> isize,
    description: String,

    start: u64,
    end: u64,
}

struct State {
    times: Vec<u64>,

    correct: usize,
    incorrect: usize,
    attempts: usize,

    combo: usize,
    max_combo: usize,

    score: isize,

    is_finished: bool,
}

fn read_level(path: &std::path::PathBuf) -> Result<Level, String> {
    use std::io::Read;

    let mut file = match std::fs::File::open(&path) {
        Err(why) =>
            return Err(format!("couldn't open {}: {}", path.display(), why)),
        Ok(file) => file,
    };

    let level_encoded;
    let mut string = String::new();
    let r = file.read_to_string(&mut string);
    match r {
        Err(why) =>
            return Err(format!("couldn't read {}: {}", path.display(), why)),
        Ok(_) => level_encoded = string,
    };

    let level_decoded: json::DecodeResult<Level> =
        json::decode(&level_encoded);
    match level_decoded {
        Err(why) => Err(format!("{}", why)),
        Ok(level) => Ok(level),
    }
}

struct Game {
    ranges_operands: Vec<Range<isize>>,
    range_kind: Range<usize>,
    rng_a: ThreadRng,
    rng_b: ThreadRng,
    rng_kind: ThreadRng,
    functions: Vec<(fn(isize, isize) -> isize, String)>,
}

fn setup_game<'a>(l: Level) -> Game {
    let mut ranges_operands = vec![];
    for i in l.operands_digits.iter() {
        ranges_operands.push(Range::new(1, 10 ** i));
    }
    // TODO: Setup a proper mapping of allowed functions
    let range_kind = Range::new(0, l.functions.len());
    let rng_a =    rand::thread_rng();
    let rng_b =    rand::thread_rng();
    let rng_kind = rand::thread_rng();

    let mut functions: Vec<(fn(isize, isize) -> isize, String)> = vec![];
    for f in l.functions {
        use std::ops::{Add, Sub, Mul, Div};
        match &*f {
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
    use std::path::PathBuf;

    let maybe_level;
    let level_dir = Path::new(".");
    let maybe_files = std::fs::read_dir(&level_dir);
    match maybe_files {
        Err(why) => panic!("Failed to read {} directory: {}",
                           level_dir.display(), why),
        Ok(files) => {
            let levels: Vec<PathBuf> = files
                .map(|p| p.ok().expect("Couldn't represent filename as str")
                     .path())
                .filter(|p| p.ends_with(".lvl.json"))
                .collect();
            let levels_displays : Vec<std::path::Display> =
                levels.iter().map(|p| p.display()).collect();
            println!("Found levels:");
            for (i, l) in levels_displays.iter().enumerate() {
                println!("{}. {}", i + 1, l);
            }
            print!("Select one: ");

            let mut string = String::new();
            let result = io::stdin().read_line(&mut string);
            match result {
                Err(_) => panic!("Failed to read the choice."),
                Ok(_) => {
                    let newlines: &[_] = &['\r', '\n'];
                    let trimmed =
                        &string.trim_matches(newlines);
                    let maybe_choice: Result<usize, _> = trimmed.parse();
                    match maybe_choice {
                        Err(_) => panic!("Failed to parse unsigned integer from choice"),
                        Ok(choice) => {
                            maybe_level = read_level(&levels[choice - 1]);
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
        use rand::distributions::IndependentSample;

        let a = game.ranges_operands[0].ind_sample(&mut game.rng_a);
        let b = game.ranges_operands[1].ind_sample(&mut game.rng_b);
        let kind = game.range_kind.ind_sample(&mut game.rng_kind);
        let (ref function, ref description) = game.functions[kind];

        print!("{}   {} {} {} = ", sm.invitation, a, description, b);

        let start = precise_time_ns();
        let mut string = String::new();
        let result = io::stdin().read_line(&mut string);
        let end   = precise_time_ns();

        match result {
            Ok(_) => {
                last_state = handle_input(
                    Round {
                        input: &string,
                        start: start,
                        end: end,
                        a: a,
                        b: b,
                        function: function,
                        description: description.to_string(),
                    },
                    last_state,
                    &sm);
                if last_state.is_finished {
                    break;
                }
            },
            Err(_) => break,
        };
    }
    process_results(last_state);
}

fn process_results(s: State) {
    let time_stat : f64 = if s.times.len() != 0 {
        compute_median(s.times)
    } else {
        0.
    };
    let total_trials = s.incorrect + s.correct;
    let rate : f64 = if total_trials != 0 {
        100.
      * s.correct as f64
      / total_trials as f64
    } else {
        0.
    };

    println!("====\n\
             Your score: {}\n\
             Correct answers: {} ({rate:.0} %), incorrect: {}, total: {}.\n\
             Median time: {:.2} s.",
             s.score, s.correct, s.incorrect, s.correct + s.incorrect,
             time_stat / 1000., rate=rate);

    let mut recs = read_records();
    process_records(&mut recs, s.score);
    write_records(recs);
}

#[derive(Debug, RustcDecodable)]
struct Record {
    points: isize,
    player: String,
}


fn read_records() -> Vec<Record> {
    use std::io::BufReader;
    use std::fs::File;
    use std::io::Read;
    use rustc_serialize::json;

    let path = Path::new("records");
    let mut file = BufReader::new(File::open(&path).unwrap());
    let mut buffer = String::new();

    file.read_to_string(&mut buffer).unwrap();

    let decode_result: json::DecodeResult<Vec<Record>> = json::decode(&buffer);
    let records = decode_result.unwrap();

    records
}


fn insert_record(recs: &mut Vec<Record>, saved: Option<Record>, new: isize) {
    let mut stdin = std::io::stdin();
    print!("Enter your name: ");
    let mut string = String::new();
    let line = stdin.read_line(&mut string);
    match line {
        Ok(_) => {
            let newlines: &[_] = &['\r', '\n'];
            let name = &string.trim_matches(newlines);
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


fn process_records(recs: &mut Vec<Record>, new : isize) {
    let n = recs.len();
    if n >= 10 {
        match &mut recs[n - 1] {
            &mut Record { points: old, .. } => {
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
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(&Path::new("records")).unwrap();
    for r in recs.iter() {
        match r {
            &Record { ref player, points } => {
                let line = format!("player: \"{}\", points: \"{}\"\n",
                                   *player, points);
                file.write(line.as_bytes()).unwrap();
            }
        }
    }
}
