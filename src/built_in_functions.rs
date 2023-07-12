use std::rc::Rc;
use std::collections::HashMap;

use crate::evaluate::LispOutput;
use crate::functions::{LispFunction, BuiltInFunction};


const MINIMUM_REQUIRED_DIVISION_ARGUMENTS: usize = 2;

fn unwrap_lisp_outputs(args: Vec<LispOutput>) -> impl Iterator<Item = i64> {
    return args.into_iter().map(|output| {
        if let LispOutput::Integer(num) = output {
            return num;
        };
        panic!("Only expecting integer arguments");
    });
}

fn add(args: Vec<LispOutput>) -> LispOutput {
    return LispOutput::Integer(unwrap_lisp_outputs(args).sum());
}

fn sub(args: Vec<LispOutput>) -> LispOutput {
    if args.len() == 1 {
        match &args[0] {
            LispOutput::Integer(num) => LispOutput::Integer(-num),
            _ => panic!("Only expecting integer arugments"),
        };
    }

    let mut integers = unwrap_lisp_outputs(args);
    let first_val = integers.next().unwrap();
    return LispOutput::Integer(
        first_val - integers.sum::<i64>()
    );
}

fn mul(args: Vec<LispOutput>) -> LispOutput {
    return LispOutput::Integer(
        unwrap_lisp_outputs(args).fold(1, |acc, next| acc * next)
    );
}

fn div(args: Vec<LispOutput>) -> LispOutput {
    let length = args.len();

    if length < MINIMUM_REQUIRED_DIVISION_ARGUMENTS {
        panic!("Need two or more arguments to apply division function");
    }

    let mut integers = unwrap_lisp_outputs(args);
    let first_val = integers.next().unwrap();
    return LispOutput::Integer(
        first_val / integers.fold(1, |acc, next| acc * next)
    );
    
}

fn convert_to_built_in(func: Rc<dyn Fn(Vec<LispOutput>) -> LispOutput>) -> LispOutput {
    return LispOutput::Lambda(LispFunction::BuiltInFunction(BuiltInFunction::new(func)));
}

pub fn built_in_function_bindings() -> HashMap<String, LispOutput> {
    return HashMap::from([
        ("+".to_string(), convert_to_built_in(Rc::new(add))),
        ("-".to_string(), convert_to_built_in(Rc::new(sub))),
        ("*".to_string(), convert_to_built_in(Rc::new(mul))),
        ("/".to_string(), convert_to_built_in(Rc::new(div))),
    ]);


}