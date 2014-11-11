#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate time;

use std::io;
use std::num::{pow, from_int, from_u64, from_f64};
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

fn full_multiplier(time: int) -> int {
    let tm =
        time_multiplier(from_int(time).expect("Time of trial can't be converted to f64"));
    from_f64(
        std::num::Float::round (1000. * tm)).expect("Full multiplier can't be converted to int")
}

fn main() {
    let mut score = 0i;
    let mut combo = 0i;
    let mut max_combo = 0i;
    let mut correct = 0i;
    let mut incorrect = 0i;
    let mut times : Vec<u64> = vec![];

    loop {
        #[deriving(PartialEq, Eq, PartialOrd, Ord)]
        enum Kind {
            Add_ = 0,
            Sub_,
            Mul_,
        };

        impl std::rand::Rand for Kind {
            fn rand<R: Rng>(rng: &mut R) -> Kind {
                let range = Range::new(1i, 3);
                let kind_num = range.ind_sample(rng);
                match kind_num {
                    1 => Add_,
                    2 => Sub_,
                    3 => Mul_,
                    _ => panic!("we couldn't get anything else from rng"),
                }
            }
        }

        let functions : &[(fn(&int, &int) -> int, &str)] =
            &[(Add::add, "+"), (Sub::sub, "-"), (Mul::mul, "*")];

        let range_operands = Range::new(1, 30);
        let mut rng_a =    rand::task_rng();
        let mut rng_b =    rand::task_rng();
        let mut rng_kind = rand::task_rng();

        let a = range_operands.ind_sample(&mut rng_a);
        let b = range_operands.ind_sample(&mut rng_b);
        let kind : Kind = rng_kind.gen();
        let (function, description) = functions[kind as uint];

        print!("□   {} {} {} = ", a, description, b);

        let start = precise_time_ns();
        let result = io::stdio::stdin().read_line();
        let end   = precise_time_ns();
        let diff_ms = (end - start) / pow(10, 6);
        let diff_s  = (end - start) / pow(10, 9);
        let diff_s_int = from_u64(diff_s).expect("Time of trial can't be converted to int");

        match result {
            Ok(string) => {
                let trimmed = string.as_slice().trim_chars(['\r', '\n'].as_slice());
                if trimmed == "q" {
                    break;
                }
                times.push(diff_ms);
                let maybe_c_user : Option<int> =
                    std::from_str::from_str(trimmed);
                match maybe_c_user {
                    Some(c_user) => {
                        let c_real = function(&a, &b);
                        let message =
                            if c_user == c_real {
                                correct += 1;
                                combo += 1;
                                if combo > max_combo {
                                    max_combo = combo;
                                }
                                let mult = full_multiplier(diff_s_int);
                                let explanation = if mult == 0 {
                                    "(timeout)"
                                } else {
                                    ""
                                };
                                let pending =
                                    1000 * mult;
                                let combed = pending * combo;
                                score += combed;
                                format!("✓ {:+8}×{:02} = {:+10}! {}",
                                        pending, combo, combed, explanation)
                            } else {
                                incorrect += 1;
                                combo = 0;
                                let pending = -1000;
                                score += pending;
                                if score < 0 {
                                    score = 0;
                                };
                                format!("✗ {:+8}^W {}.",
                                        pending, c_real)
                            };
                        println!("{:48}{:32}", message, score);
                        info!(" {} ms", diff_ms);
                    },
                    None => {
                        println!("You didn't input a number. Try again.");
                    },
                }
            },
            Err(_) => break,
        };
    }

    let mut total = 0;
    let mut num = 0u64;
    for t in times.iter() {
        total += *t;
        num += 1;
    };
    let total_time_f64 : f64 =
        from_u64(total).expect("Total time can't be converted to f64");
    let number_of_tries_f64 : f64 =
        from_u64(num).expect("Number of tries can't be converted to f64");
    let average_time : f64 = total_time_f64 / number_of_tries_f64;
    let total_trials = incorrect + correct;
    let rate : f64 = 100.
      * from_int(correct)     .expect("Number of correct trials can't be converted to f64")
      / from_int(total_trials).expect("Total number of trials can't be converted to f64");

    println!("====\n\
             Your score: {}\n\
             Correct answers: {} ({rate:.0f} %), incorrect: {}, total: {}.\n\
             Average time: {:.2f} s.",
             score, correct, incorrect, correct + incorrect, average_time / 1000.,
             rate=rate);
}
