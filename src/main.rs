use crate::parser::parse;
use crate::tokenizer::tokenize;
use crate::evaluate::{evaluate, Environment};

use std::rc::Rc;
use std::cell::RefCell;

pub mod parser;
pub mod evaluate;
pub mod tokenizer;
pub mod lisp_expression;
pub mod functions;
pub mod built_in_functions;

fn main() {
    let mut env = Rc::new(RefCell::new(Environment::global_env()));

    let add_one = evaluate(&parse(&tokenize("(define add_one (lambda (y) (+ y 1)))")), &mut env);
    let add_one_twice = evaluate(&parse(&tokenize("(add_one (add_one 2))")), &mut env);

    let anonymous_lambda_result = evaluate(&parse(&tokenize("((lambda (y) (+ y 1)) 5)")), &mut env);
    let comparisons = evaluate(&parse(&tokenize("(equal? 5 5 5)")), &mut env);

    println!("add_one lambda {:?}", add_one);
    println!("Should get 2 + 1 + 1 = 4: {:?}", add_one_twice);
    println!("Expecting 6: {:?}", anonymous_lambda_result);
    println!("Expecting False: {:?}", comparisons);
    
}