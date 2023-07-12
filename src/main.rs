use crate::parser::parse;
use crate::tokenizer::tokenize;
use crate::evaluate::{evaluate, Environment};
use crate::built_in_functions::built_in_function_bindings;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub mod parser;
pub mod evaluate;
pub mod tokenizer;
pub mod lisp_expression;
pub mod functions;
pub mod built_in_functions;

fn main() {

    let bindings = built_in_function_bindings();

    let built_in_env = Rc::new(RefCell::new(Environment::build(bindings, None)));
    let mut env = Rc::new(RefCell::new(Environment::build(HashMap::new(), Some(built_in_env.clone()))));

    let add_one = evaluate(&parse(&tokenize("(define add_one (lambda (y) (+ y 1)))")), &mut env);
    let add_one_twice = evaluate(&parse(&tokenize("(add_one (add_one 2))")), &mut env);

    println!("add_one lambda {:?}", add_one);
    println!("Should get 2 + 1 + 1 = 4: {:?}", add_one_twice);
    
}