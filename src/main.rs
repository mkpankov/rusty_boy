use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};

fn main() {
    loop {
        let range = Range::new(1, 30);
        let mut rng = rand::task_rng();
        let a = range.ind_sample(&mut rng);
        let b = range.ind_sample(&mut rng);

        println!("Solve this: {} + {} = ?", a, b)

        let result = io::stdio::stdin().read_line();

        match result {
            Ok(mut string) => {
                let n = string.len() - 1;
                string.remove(n);
                let maybe_c_user : Option<int> =
                    std::from_str::from_str(string.as_slice());
                match maybe_c_user {
                    Some(c_user) => {
                        let c_real = a + b;
                        let reference;
                        let message =
                            if c_user == c_real {
                                "Correct!"
                            } else {
                                reference = format!("Incorrect! Correct answer is {}",
                                                    c_real);
                                reference.as_slice()

                            };
                        println!("{}", message);
                    },
                    None => {
                        println!("You didn't input a number. Try again.");
                    },
                }
            },
            Err(_) => break,
        };
    }
}
