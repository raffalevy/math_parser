#![allow(dead_code, unused_variables)]

mod lexer;
mod math_parser;
mod math_parser_2;
mod math_parser_tree;
mod ast;

use std::io::Error;
use std::io::Write;
use std::io::{stdin, stdout, BufRead, BufReader};
use std::fs::File;

fn main() {
    // run_prompt_multiline();
    run_prompt_math();
    // let args: Vec<String> = ::std::env::args().collect();
    // run_file(&args[1]).unwrap();
}

fn run_prompt() {
    let stdin = stdin();
    let mut stdout = stdout();
    let mut lines = stdin.lock().lines();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        match lines.next() {
            Some(Ok(line)) => run(line),
            Some(Err(err)) => eprintln!("{}", err),
            None => {
                println!("Exiting.");
                break;
            }
        };
    }
}

fn run_prompt_multiline() {
    let stdin = stdin();
    let mut stdout = stdout();
    let mut lines = stdin.lock().lines();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        let mut buf = String::new();
        loop {
            match lines.next() {
                Some(Ok(line)) => {
                    buf.push_str(&line);
                    buf.push('\n');
                }
                Some(Err(err)) => eprintln!("{}", err),
                None => {
                    break;
                }
            }
        }
        run(buf);
    }
}

fn run(source: String) {
    let tokens = lexer::scan(source);
    for token in &tokens {
        println!("{:?}", token)
    }
}

fn run_prompt_math() {
    let stdin = stdin();
    let mut stdout = stdout();
    let mut lines = stdin.lock().lines();
    // let mut context = math_parser_2::EvalContext::new();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        match lines.next() {
            // Some(Ok(line)) => match context.eval(&line) {
            Some(Ok(line)) => match math_parser_tree::parse_repl(&line) {
                Ok(x) => println!("Result: {:#?}", x),
                Err(err) => println!("Error: {}", err),
            },
            Some(Err(err)) => eprintln!("{}", err),
            None => {
                println!("Exiting.");
                break;
            }
        };
    }
}

fn run_file(path: &str) -> Result<(), Error> {
    let file = File::open(path)?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    let mut context = math_parser_2::EvalContext::new();
    // match context.eval(&contents) {
    //     Ok(x) => println!("Result: {}", x),
    //     Err(err) => println!("Error: {}", err),
    // };
    let reader = BufReader::new(file);
    let mut last: Option<f64> = None;
    for l in reader.lines() {
        match l {
            Ok(line) => match context.eval(&line) {
                Ok(res) => last = Some(res),
                Err(err) => println!("Error: {}", err),
            },
            Err(_) => (),
        }
    }
    match last {
        Some(res) => println!("Result: {}", res),
        None => (),
    }
    Ok(())
}
