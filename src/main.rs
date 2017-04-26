extern crate secd;

use std::env;

fn main() {
    let mut args = env::args();
    if args.len() == 2 {
        println!("{}",
                 secd::run_lisp_file(&args.nth(1).unwrap()).expect("main"));
    } else {
        println!("expected 1 file");
    }
}
