use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            let prog = &args[1];
            let what = &args[2];
            println!("Program: {}", prog);
            println!("What: {}", what);
        }
        _ => println!("Too few or too many arguments"),
    }
}
