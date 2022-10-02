#![forbid(unsafe_code)]

mod calculator;
mod controller;
use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();

    let mut controller = controller::Controller::new(controller::OutputFormat::Standard);
    let res = controller.execute(args[1].clone());

    println!("{}", res);
    
}