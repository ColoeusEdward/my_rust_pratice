use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    // let mut guess = String::new();
    println!("Hello, world!");
    println!("input your gess number");
    let secret_number = rand::thread_rng().gen_range(1..=100);
    // println!("the secret number is {secret_number}");
    loop {
        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        // let temp: String = guess.trim().parse().expect("ffffk");
        // println!("temp {temp}");
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };
        println!("your guess is {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
