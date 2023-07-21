use crate::parser::parse;
use crate::tokenizer::tokenize;
use crate::evaluate::{evaluate, Environment, LispOutput};

use std::io;
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;

pub mod parser;
pub mod evaluate;
pub mod tokenizer;
pub mod lisp_expression;
pub mod functions;
pub mod built_in_functions;

fn main() {
    // let mut env = Rc::new(RefCell::new(Environment::global_env()));

    // let add_one = evaluate(&parse(&tokenize("(define add_one (lambda (y) (+ y 1)))")), &mut env);
    // let add_one_twice = evaluate(&parse(&tokenize("(add_one (add_one 2))")), &mut env);

    // let anonymous_lambda_result = evaluate(&parse(&tokenize("((lambda (y) (+ y 1)) 5)")), &mut env);
    // let comparisons = evaluate(&parse(&tokenize("(equal? 5 5 5)")), &mut env);

    // let if_statement = evaluate(&parse(&tokenize("(if (< 1 2 3) 1 0)")), &mut env);

    // let list_statement = evaluate(&parse(&tokenize("(list 1 2 3 4)")), &mut env);
    // let car_statement = evaluate(&parse(&tokenize("(car (list 1 2 3 4))")), &mut env);
    // let cdr_statement = evaluate(&parse(&tokenize("(car (cdr (list 1 2 3 4)))")), &mut env);

    // println!("add_one lambda {:?}", add_one);
    // println!("Should get 2 + 1 + 1 = 4: {:?}", add_one_twice);
    // println!("Expecting 6: {:?}", anonymous_lambda_result);
    // println!("Expecting False: {:?}", comparisons);
    // println!("Expecting 1: {:?}", if_statement);

    // println!("Expecting LispList with 1, 2, 3, 4: {:?}", list_statement);
    // println!("Expecting to car of list, should be 1: {:?}", car_statement);
    // println!("Expecting second element of list, should be 2: {:?}", cdr_statement);

    repl();
}

fn read_string() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("can not read user input");
    input
}

fn repl() {
    let mut env = Rc::new(RefCell::new(Environment::global_env()));
    loop {
        print!(">>> ");
        let _ = io::stdout().flush();
        let input = read_string();

        if &input == "exit" {
            break;
        }

        let output = evaluate(&parse(&tokenize(&input)), &mut env);


        match output {
            LispOutput::Integer(num) => println!("{:?}", num),
            LispOutput::Bool(bool_val) => println!("{:?}", bool_val),
            LispOutput::Lambda(func) => println!("{:?}", func),
            LispOutput::List(list) => println!("{:?}", *list),
            LispOutput::Void => println!("void"),
        };
    }
}