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

    let mut numbers = unwrap_lisp_outputs(args);
    let first_val = numbers.next().unwrap();
    return LispOutput::Integer(
        first_val - numbers.sum::<i64>()
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

    let mut numbers = unwrap_lisp_outputs(args);
    let first_val = numbers.next().unwrap();
    return LispOutput::Integer(
        first_val / numbers.fold(1, |acc, next| acc * next)
    );
    
}

fn comparator(func: Rc<dyn Fn(i64, i64) -> bool>) -> Rc<dyn Fn(Vec<LispOutput>) -> LispOutput> {

    let apply_func = move |args| {
        let numbers: Vec<i64> = unwrap_lisp_outputs(args).collect();

        for i in 0..numbers.len() - 1 {
            let current = numbers[i];
            let next = numbers[i + 1];

            if !func(current, next) {
                return LispOutput::Bool(false);
            }
        }
        return LispOutput::Bool(true);
    };

    return Rc::new(apply_func);
}

fn equal_compare(args: Vec<LispOutput>) -> LispOutput {
    return comparator(Rc::new(|a, b| a == b))(args);
}

fn less_than_compare(args: Vec<LispOutput>) -> LispOutput {
    return comparator(Rc::new(|a, b| a < b))(args);
}

fn less_than_or_equal_compare(args: Vec<LispOutput>) -> LispOutput {
    return comparator(Rc::new(|a, b| a <= b))(args);
}

fn greater_than_compare(args: Vec<LispOutput>) -> LispOutput {
    return comparator(Rc::new(|a, b| a > b))(args);
}

fn greater_than_or_equal_compare(args: Vec<LispOutput>) -> LispOutput {
    return comparator(Rc::new(|a, b| a >= b))(args);
}


// ============== FUNCTION BUILDINGS FUNCTIONS ===============

fn convert_to_built_in(func: Rc<dyn Fn(Vec<LispOutput>) -> LispOutput>) -> LispOutput {
    return LispOutput::Lambda(LispFunction::BuiltInFunction(BuiltInFunction::new(func)));
}

pub fn built_in_function_bindings() -> HashMap<String, LispOutput> {
    return HashMap::from([
        ("+".to_string(), convert_to_built_in(Rc::new(add))),
        ("-".to_string(), convert_to_built_in(Rc::new(sub))),
        ("*".to_string(), convert_to_built_in(Rc::new(mul))),
        ("/".to_string(), convert_to_built_in(Rc::new(div))),
        ("equal?".to_string(), convert_to_built_in(Rc::new(equal_compare))),
        ("<".to_string(), convert_to_built_in(Rc::new(less_than_compare))),
        ("<=".to_string(), convert_to_built_in(Rc::new(less_than_or_equal_compare))),
        (">".to_string(), convert_to_built_in(Rc::new(greater_than_compare))),
        (">=".to_string(), convert_to_built_in(Rc::new(greater_than_or_equal_compare))),
        ("#t".to_string(), LispOutput::Bool(true)),
        ("#f".to_string(), LispOutput::Bool(false)),
    ]);


}