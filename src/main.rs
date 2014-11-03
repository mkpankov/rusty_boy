extern crate time;

use std::io;
use std::io::Timer;
use std::time::Duration;
use std::num::pow;
use std::rand::distributions::{IndependentSample, Range};

use time::precise_time_ns;


fn main() {
    loop {
        let range = Range::new(1000 * pow(10, 6), 1500 * pow (10, 6));
        let mut rng = std::rand::task_rng();
        let draw_delay_offset : u64 = range.ind_sample(&mut rng);
        let draw_delay = 500 * pow(10, 6) + draw_delay_offset;

        println!("Setting timer for {}", draw_delay.to_i64().unwrap());
        let mut timer = Timer::new().unwrap();
        let timeout = timer.oneshot(
            Duration::nanoseconds(draw_delay.to_i64().unwrap()));

        timeout.recv();
        println!("Draw!");

        let t_comp  = time::precise_time_ns();
        let slack   = 500 * pow(10, 6);
        println!("I'm gonna shoot you in {}, at {}!", slack, t_comp + slack);

        let r = io::stdio::stdin().read_char();
        let t_human  = time::precise_time_ns();
        println!("Your time: {}, that was in {}", t_human, t_human - t_comp);

        if t_comp + slack > t_human {
            println!("You shot me!")
        } else {
            println!("I shot you.")
        }

        match r {
            Ok(_) => continue,
            Err(_) => break,
        }

    }
}
