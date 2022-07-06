use std::env;
use std::process;

use leelo::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = leelo::run(config) {
        println!("Application Error: {}", e);

        process::exit(1);
    }
}
