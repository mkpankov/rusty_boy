#![feature(phase)]
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate scan;
extern crate scan_util;

extern crate term;
extern crate time;

use std::io;
use std::num::{pow, from_int, from_uint, from_u64, from_f64};
use std::rand;
use std::rand::Rng;
use std::rand::distributions::{IndependentSample, Range};
use time::precise_time_ns;

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

fn full_multiplier(time: int) -> uint {
    let tm =
        time_multiplier(from_int(time).expect("Time of trial can't be converted to f64"));
    from_f64(
        std::num::Float::round (10. * tm)).expect("Full multiplier can't be converted to int")
}

#[allow(dead_code)]
fn compute_mean(times: Vec<u64>) -> f64 {
    let n : f64 = from_uint(times.len()).unwrap();
    let sum : f64 = from_u64(times.iter().fold(0, |a, &e| a + e)).unwrap();
    sum / n
}

fn compute_median(mut times: Vec<u64>) -> f64 {
    times.sort();
    match times.len() {
        n if n % 2 == 0 => from_u64( (times[n/2] + times[n/2 - 1]) / 2 ).unwrap(),
        n               => from_u64(  times[n/2] ).unwrap(),
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

#[deriving(PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Add_ = 0,
    Sub_,
    Mul_,
}

impl std::rand::Rand for Kind {
    fn rand<R: Rng>(rng: &mut R) -> Kind {
        let range = Range::new(1i, 4);
        let kind_num = range.ind_sample(rng);
        match kind_num {
            1 => Add_,
            2 => Sub_,
            3 => Mul_,
            _ => panic!("we couldn't get anything else from rng"),
        }
    }
}

fn handle_input(
    string: &str,
    times: &mut Vec<u64>,
    start: u64,
    end: u64,
    correct: &mut uint,
    incorrect: &mut uint,
    a: int,
    b: int,
    function: fn(&int, &int) -> int,
    combo: &mut uint,
    attempts: &mut uint,
    max_combo: &mut uint,
    score: &mut uint,
    sm: SymbolMap,
) -> bool
{
    let diff_ms = (end - start) / pow(10, 6);
    let diff_s  = (end - start) / pow(10, 9);
    let diff_s_int = from_u64(diff_s).expect("Time of trial can't be converted to int");

    let trimmed = string.as_slice().trim_chars(['\r', '\n'].as_slice());
    if trimmed == "q" {
        return true;
    }
    times.push(diff_ms);
    let color;
    let mark;
    let maybe_c_user : Option<int> = from_str(trimmed);
    match maybe_c_user {
        Some(c_user) => {
            let c_real : int = function(&a, &b);
            let message =
                if c_user == c_real {
                    *correct += 1;
                    *combo += 1;
                    *attempts += 1;
                    if combo > max_combo {
                        *max_combo = *combo;
                    }
                    let mult = full_multiplier(diff_s_int);
                    let explanation = if mult == 0 {
                        "(timeout)"
                    } else {
                        ""
                    };
                    let pending =
                        1000i * from_uint(mult).unwrap();
                    let combed = pending * from_uint(*combo).unwrap();
                    *score += from_int(combed).unwrap();
                    color = term::color::GREEN;
                    mark = sm.checkmark;
                    format!(" {:+8}×{:02} = {:+10}! {}",
                            pending, combo, combed, explanation)
                } else {
                    *incorrect += 1;
                    *combo = 0;
                    *attempts += 1;
                    let pending = -1000i;
                    *score += from_int(pending).unwrap();
                    color = term::color::RED;
                    mark = sm.wrongmark;
                    format!(" {:+8}^W {}.",
                            pending, c_real)
                };
            let maybe_term = term::stdout();

            if maybe_term.is_some() {
                let mut term = term::stdout().unwrap();
                term.fg(color).unwrap();
                (write!(term, "{:1}", mark)).unwrap();
                term.reset().unwrap();
            } else {
                print!("{:1}", mark);
            }

            println!("{:47}{:32}", message, score);
            info!(" {} ms", diff_ms);
        },
        None => {
            println!("You didn't input a number. Try again.");
        },
    }
    return false;
}

fn main() {
    let mut score = 0u;
    let mut combo = 0u;
    let mut max_combo = 0u;
    let mut correct = 0u;
    let mut incorrect = 0u;
    let mut times : Vec<u64> = vec![];
    let mut attempts = 0u;

    let sm = setup_symbols();
    let range_operands = Range::new(1, 30);
    let mut rng_a =    rand::task_rng();
    let mut rng_b =    rand::task_rng();
    let mut rng_kind = rand::task_rng();
    let functions : &[(fn(&int, &int) -> int, &str)] =
        &[(Add::add, "+"), (Sub::sub, "-"), (Mul::mul, "*")];

    loop {
        let a = range_operands.ind_sample(&mut rng_a);
        let b = range_operands.ind_sample(&mut rng_b);
        let kind : Kind = rng_kind.gen();
        let (function, description) = functions[kind as uint];

        print!("{}   {} {} {} = ", sm.invitation, a, description, b);

        let start = precise_time_ns();
        let result = io::stdio::stdin().read_line();
        let end   = precise_time_ns();

        match result {
            Ok(string) => {
                if handle_input(string.as_slice(), &mut times, start, end,
                                &mut correct, &mut incorrect,
                                a, b, function, &mut combo, &mut attempts,
                                &mut max_combo, &mut score, sm) {
                    break;
                }
            },
            Err(_) => break,
        };
        if attempts >= 10 {
            break;
        }
    }

    let time_stat : f64 = if times.len() != 0 {
        compute_median(times)
    } else {
        0.
    };
    let total_trials = incorrect + correct;
    let rate : f64 = if total_trials != 0 {
        100.
      * from_uint(correct)     .expect("Number of correct trials can't be converted to f64")
      / from_uint(total_trials).expect("Total number of trials can't be converted to f64")
    } else {
        0.
    };

    println!("====\n\
             Your score: {}\n\
             Correct answers: {} ({rate:.0f} %), incorrect: {}, total: {}.\n\
             Median time: {:.2f} s.",
             score, correct, incorrect, correct + incorrect, time_stat / 1000.,
             rate=rate);

    let mut recs = read_records();
    process_records(&mut recs, score);
    write_records(recs);
}

#[deriving(Show)]
struct Record {
    points: uint,
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
            "player: \"" player: &str "\", points: \"" points: uint "\"" => {
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


fn insert_record(recs: &mut Vec<Record>, saved: Option<Record>, new: uint) {
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
                p_a.cmp(&p_b));
        },
        Err(_) => {
            match saved {
                Some(saved) => recs.push(saved),
                None => (),
            }
        }
    }
}


fn process_records(recs: &mut Vec<Record>, new : uint) {
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
                let line = format!("player: \"{:s}\", points: \"{:u}\"\n",
                                   *player, points);
                file.write(line.as_bytes()).unwrap();
            }
        }
    }
}
