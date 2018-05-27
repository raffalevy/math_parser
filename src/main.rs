#![allow(dead_code, unused_variables, unused_imports)]

mod lexer;
mod math_parser;
mod math_parser_2;
mod parser;
mod ast;
mod visitor;
mod eval;
mod vm;
mod compiler;

use std::env::args;
use std::io::Error;
use std::io::Write;
use std::io::{stdin, stdout, BufRead, BufReader, Read};
use std::fs::File;

fn main() {
    // run_prompt_multiline();
    // run_prompt_math();

    let args: Vec<String> = args().collect();
    run_file(&args[1]).unwrap();

    // let mut vm1 = vm::VM::new(&[
    //     vm::CALL,
    //     15,0,0,0,0,0,0,0,
    //     vm::CONST_U8,
    //     14,
    //     vm::U8_2_F64,
    //     vm::PRINT_F64,
    //     vm::POP_F64,
    //     vm::EXIT,
    //     vm::SET_CTX,
    //     vm::CONST_U8,
    //     8,
    //     vm::U8_2_F64,
    //     vm::PRINT_F64,
    //     vm::POP_F64,
    //     vm::RET_CTX,
    //     vm::RET
    // ]);
    // vm1.run();
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
    let mut context = eval::EvalContext::new();
    // let mut context = math_parser_2::EvalContext::new();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        match lines.next() {
            // Some(Ok(line)) => match context.eval(&line) {
            Some(Ok(line)) => match parser::parse_repl(&line) {
                Ok(x) => {
                    println!("Parse tree: {:#?}", x);
                    println!(
                        "Eval result: {:?}",
                        context
                            .eval_repltree(&x)
                            .map(|opt| opt.map(|num| format!("{:e}", num)))
                    );
                }
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
    #[allow(unused_mut)]
    let mut ctx = eval::EvalContext::new();
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // let res = parser::parse_file(&contents);
    // println!("{:#?}", res);
    if let Ok(tree) = parser::parse_file(&contents) {
        println!("{:#?}", tree);
        // let res = ctx.eval_file(&tree);
        // let mapped = res.map(|opt| opt.map(|val| format!("{:e}", val)));
        // println!("{:?}", mapped);
        // println!("{:?}", res);
        let mut compiled = compiler::compile(&tree);
        println!("Compiled: {:?}", compiled);
        let mut vm1 = vm::VM::new(&compiled);
        vm1.run();
    }
    // println!("{:#?}", tree);

    Ok(())
}
