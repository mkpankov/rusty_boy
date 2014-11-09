#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate time;

use std::io;
use std::num::pow;
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
    let tm = time_multiplier(std::num::from_int(time).unwrap());
    std::num::from_f64(
        std::num::Float::round (1000. * tm)).unwrap()
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
        let mut rng_a = rand::task_rng();
        let mut rng_b = rand::task_rng();
        let mut rng_kind = rand::task_rng();

        let a = range_operands.ind_sample(&mut rng_a);
        let b = range_operands.ind_sample(&mut rng_b);
        let kind : Kind = rng_kind.gen();
        let (function, description) = functions[kind as uint];

        print!("Solve this: {} {} {} = ", a, description, b);

        let start = precise_time_ns();
        let result = io::stdio::stdin().read_line();
        let end   = precise_time_ns();
        let diff_ms = (end - start) / pow(10, 6);
        let diff_s  = (end - start) / pow(10, 9);
        let diff_s_int = std::num::from_u64(diff_s).unwrap();
        times.push(diff_ms);

        match result {
            Ok(string) => {
                let trimmed = string.as_slice().trim_chars(['\r', '\n'].as_slice());
                if trimmed == "q" {
                    break;
                }
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
                                let pending =
                                    1000 * full_multiplier(diff_s_int);
                                let combed = pending * combo;
                                score += combed;
                                format!("  Correct! {:+8}×{:02} = {:+10}!",
                                        pending, combo, combed)
                            } else {
                                incorrect += 1;
                                combo = 0;
                                let pending = -1000;
                                score += pending;
                                if score < 0 {
                                    score = 0;
                                };
                                format!("Incorrect! {:+8}^W {}.",
                                        pending, c_real)
                            };
                        println!("{:36}{:44}", message, score);
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
    let average : f64 = std::num::from_u64(total / num).unwrap();
    let rate : f64 =
        100f64 * std::num::from_int(correct).unwrap() /
        std::num::from_int(incorrect + correct).unwrap();

    println!("====\n\
             Your score: {}\n\
             Correct answers: {} ({rate:.0f} %), incorrect: {}, total: {}.\n\
             Average time: {:.2f} s.",
             score, correct, incorrect, correct + incorrect, average / 1000_f64,
             rate=rate);
}
