use std::rc::Rc;
use std::boxed::Box;
use std::collections::HashMap;

use crate::evaluate::{LispOutput, LispList};
use crate::functions::{LispFunction, BuiltInFunction, LispFunctionCall};


const MINIMUM_REQUIRED_DIVISION_ARGUMENTS: usize = 2;
const REQUIRED_CAR_ARGUMENTS: usize = 1;
const REQUIRED_CDR_ARGUMENTS: usize = 1;
const REQUIRED_IS_LIST_ARGUMENTS: usize = 1;
const REQUIRED_LIST_LENGTH_ARGUMENTS: usize = 1;
const REQUIRED_LIST_REF_ARGUMENTS: usize = 2;
const REQUIRED_MAP_ARGUMENTS: usize = 2;
const REQUIRED_FILTER_ARGUMENTS: usize = 2;


fn unwrap_lisp_outputs(args: Vec<LispOutput>) -> impl Iterator<Item = i64> {
    return args.into_iter().map(|output| {
        if let LispOutput::Integer(num) = output {
            return num;
        };
        panic!("Only expecting integer arguments");
    });
}

fn check_output_arguments(args: &Vec<LispOutput>, number_of_args: usize) {
    if args.len() != number_of_args {
        panic!("incorrect nubmer of arguements: got {}, expected {}", args.len(), number_of_args);
    }
}


// ============== ARITHMETIC BUILT-INS ===============

fn add(args: Vec<LispOutput>) -> LispOutput {
    return LispOutput::Integer(unwrap_lisp_outputs(args).sum());
}

fn sub(args: Vec<LispOutput>) -> LispOutput {
    if args.len() == 1 {
        return match &args[0] {
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


// ============== LOGIC BUILT-INS ===============

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

// ============== LIST BUILT-INS ===============

fn make_list(args: Vec<LispOutput>) -> LispOutput {
    return LispOutput::List(Box::new(LispList::build(args.into_iter())));
}

fn car_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_CAR_ARGUMENTS);
    match &args[0] {
        LispOutput::List(cons_cell) => cons_cell.get_car(),
        _ => panic!("expecting a cons cell!"),
    }
}

fn cdr_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_CDR_ARGUMENTS);

    match &args[0] {
        LispOutput::List(cons_cell) => cons_cell.get_cdr(),
        _ => panic!("expecting a cons cell!"),
    }
}

fn is_list_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_IS_LIST_ARGUMENTS);

    match args[0] {
        LispOutput::List(_) => LispOutput::Bool(true),
        _ => LispOutput::Bool(false),
    }
}

fn list_length_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_LIST_LENGTH_ARGUMENTS);

    match &args[0] {
        LispOutput::List(cons_cell) => cons_cell.length(),
        _ => panic!("expecting lisp list to get length"),
    }
}

fn list_ref_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_LIST_REF_ARGUMENTS);

    
    let index = match args[1] {
        LispOutput::Integer(num) => num,
        _ => panic!("expecting an integer to use as index in list")
    };
    
    if index < 0 {
        panic!("negative indicies are not supported!");
    }

    match &args[0] {
        LispOutput::List(cons_cell) => cons_cell.get(index),
        _ => panic!("expecting a cons cell to index into"),
    }
}

fn append_func(args: Vec<LispOutput>) -> LispOutput {
    let lists = args.into_iter().map(|output| {
        match output {
            LispOutput::List(list) => *list,
            _ => panic!("expecting only lisp lists for append built-in!"),
        }
    }).collect();
    return LispOutput::List(Box::new(LispList::append(lists)));
}

fn map_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_MAP_ARGUMENTS);

    fn apply_map(list: LispList, func: impl LispFunctionCall) -> LispList {
        match list {
            LispList::Nil => LispList::Nil,
            LispList::Cons(car, cdr) => LispList::Cons(
                func.call(vec![car.clone()]), 
                Box::new(apply_map(*cdr, func))
            )
        }
    }

    let function = match &args[1] {
        LispOutput::Lambda(func) => func.clone(),
        _ => panic!("expecting second argument to be lisp function!"),
    };

    match &args[0] {
        LispOutput::List(list) => LispOutput::List(Box::new(apply_map(*list.clone(), function))),
        _ => panic!("expecting first argument to be lisp list!"),
    }
}

fn filter_func(args: Vec<LispOutput>) -> LispOutput {
    check_output_arguments(&args, REQUIRED_FILTER_ARGUMENTS);

    fn apply_filter(list: LispList, func: impl LispFunctionCall) -> LispList {
        match list {
            LispList::Nil => LispList::Nil,
            LispList::Cons(car, cdr) => {
                if let LispOutput::Bool(should_keep) = func.call(vec![car.clone()]) {
                    if !should_keep {
                        return apply_filter(*cdr, func);
                    }

                    return LispList::Cons(
                        car, 
                        Box::new(apply_filter(*cdr, func))
                    );
                }

                panic!("expecting element to evaluate to boolean!");
            },
        }
    }

    let function = match &args[1] {
        LispOutput::Lambda(func) => func.clone(),
        _ => panic!("expecting second argument to be lisp function!"),
    };

    match &args[0] {
        LispOutput::List(list) => LispOutput::List(Box::new(apply_filter(*list.clone(), function))),
        _ => panic!("expecting first argument to be lisp list!"),
    }
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
        ("nil".to_string(), LispOutput::List(Box::new(LispList::Nil))),
        ("list".to_string(), convert_to_built_in(Rc::new(make_list))),
        ("car".to_string(), convert_to_built_in(Rc::new(car_func))),
        ("cdr".to_string(), convert_to_built_in(Rc::new(cdr_func))),
        ("list?".to_string(), convert_to_built_in(Rc::new(is_list_func))),
        ("length".to_string(), convert_to_built_in(Rc::new(list_length_func))),
        ("list-ref".to_string(), convert_to_built_in(Rc::new(list_ref_func))),
        ("append".to_string(), convert_to_built_in(Rc::new(append_func))),
        ("map".to_string(), convert_to_built_in(Rc::new(map_func))),
        ("filter".to_string(), convert_to_built_in(Rc::new(filter_func))),
    ]);


}