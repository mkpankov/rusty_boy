extern crate time;

use std::io;
use std::num::pow;
use time::get_time;


fn main() {
    loop {

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
