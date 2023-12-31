use std::collections::HashMap;
use std::rc::Rc;
use std::boxed::Box;
use std::cell::RefCell;

use crate::lisp_expression::LispExpression;
use crate::built_in_functions::built_in_function_bindings;
use crate::functions::{LispFunction, LispFunctionCall, Function};


#[derive(Debug, Clone, PartialEq)]
pub enum LispOutput {
    Void,
    Integer(i64),
    Bool(bool),
    Lambda(LispFunction),
    List(Box<LispList>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LispList {
    Cons(LispOutput, Box<LispList>),
    Nil,
}

impl LispList {
    pub fn build(mut args: impl Iterator<Item=LispOutput>) -> Self {
        let first = args.next();
        match first {
            Some(val) => LispList::Cons(val, Box::new(LispList::build(args))),
            None => LispList::Nil,
        }
    }

    pub fn get_car(&self) -> LispOutput {
        match self {
            LispList::Cons(car, _) => car.clone(),
            LispList::Nil => panic!("lisp list is empty!"),
        }
    }

    pub fn get_cdr(&self) -> LispOutput {
        match self {
            LispList::Cons(_, cdr) => LispOutput::List(cdr.clone()),
            LispList::Nil => panic!("lisp list is empty!"),
        }
    }

    pub fn length(&self) -> LispOutput {
        fn get_length(list: &LispList) -> i64 {
            match list {
                LispList::Nil => 0,
                LispList::Cons(_, cdr) => get_length(cdr) + 1,
            }
        }
        return LispOutput::Integer(get_length(self));
    }

    pub fn get(&self, index: i64) -> LispOutput {
        match self {
            LispList::Nil => panic!("index out of bounds!"),
            LispList::Cons(car, cdr) => {
                if index == 0 {
                    return car.clone();
                }

                cdr.get(index - 1)
            }
        }
    }

    pub fn append(lists: Vec<LispList>) -> LispList {
        if lists.len() == 0 {
            return LispList::Nil;
        }

        match &lists[0] {
            LispList::Nil => LispList::append(lists[1..].to_vec()),
            LispList::Cons(car, cdr) => {
                let mut new_args = vec![*cdr.clone()];
                new_args.append(&mut lists[1..].to_vec());
                let rest = Box::new(LispList::append(new_args));
                
                return LispList::Cons(car.clone(), rest);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub bindings: HashMap<String, LispOutput>,
    pub parent_env: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            bindings: HashMap::new(),
            parent_env: None,
        }
    }

    pub fn build(
        bindings: HashMap<String, LispOutput>, 
        parent_env: Option<Rc<RefCell<Environment>>>) -> Self {
            return Environment {
                bindings,
                parent_env,
            }
    }

    pub fn built_ins_env() -> Self {
        return Self::build(
            built_in_function_bindings(),
            None,
        );
    }

    pub fn global_env() -> Self {
        return Self::build(
            HashMap::new(),
            Some(Rc::new(RefCell::new(Self::built_ins_env()))),
        );
    }

    fn get(&self, var: &String) -> LispOutput {
        
        let val = self.bindings.get(var);

        if val == None {
            if self.parent_env == None {
                panic!("variable not found in any environment");
            }
            return self.parent_env.as_ref().unwrap().borrow().get(var);
        }
        return val.unwrap().clone();
    }

    fn set(&mut self, var: &String, val: &LispOutput) {
        self.bindings.insert(var.clone(), val.clone());
    }

    fn del(&mut self, var: &String) -> LispOutput {
        if !self.bindings.contains_key(var) {
            panic!("variable not found in environment!");
        }
        return self.bindings.remove(var).unwrap();
    }

    fn set_bang(&mut self, var: &String, val: LispOutput) -> LispOutput {
        if self.bindings.contains_key(var) {
            self.bindings.insert(var.clone(), val.clone());
            return val;
        }

        match &self.parent_env {
            Some(env) => env.borrow_mut().set_bang(var, val),
            None => panic!("variable does not exist in any environment!"),
        }
    }
}

fn check_arguments(args: &Vec<LispExpression>, number_of_args: usize) {
    if args.len() != number_of_args {
        panic!("special form was not supplied with correct number of arugments");
    }
}

const REQUIRED_DEFINE_ARGUMENTS: usize = 3;
const REQUIRED_LAMBDA_ARGUMENTS: usize = 3;
const REQUIRED_IF_ARGUMENTS: usize = 4;
const REQUIRED_DEL_ARGUMENTS: usize = 2;
const REQUIRED_LET_ARGUMENTS: usize = 3;
const REQUIRED_SET_BANG_ARGUMENTS: usize = 3;

pub fn evaluate(tree: &LispExpression, env: &mut Rc<RefCell<Environment>>) -> LispOutput {
    match tree {
        LispExpression::Integer(num) => LispOutput::Integer(num.clone()),
        LispExpression::Symbol(var) => env.borrow_mut().get(&var),
        LispExpression::List(expressions) => {
            if expressions.len() == 0 {
                panic!("list of expression cannot be empty!");
            }

            if let LispExpression::Symbol(built_in) = &expressions[0] {
                match &built_in[..] {
                    "define" => {
                        check_arguments(&expressions, REQUIRED_DEFINE_ARGUMENTS);
                        let var = match &expressions[1] {
                            LispExpression::Symbol(symbol) => symbol,
                            _ => panic!("var must be LispExpression Symbol"),
                        };
        
                        let val = evaluate(&expressions[2], env);

                        env.borrow_mut().set(&var, &val);
        
                        return val;
                    },
                    "lambda" => {
                        check_arguments(&expressions, REQUIRED_LAMBDA_ARGUMENTS);
                        let parameters = &expressions[1];
                        let body = &expressions[2];

                        return LispOutput::Lambda(
                            LispFunction::Function(
                                Function::build(parameters.clone(), body.clone(), env.clone())
                            )
                        );
                    },
                    "if" => {
                        check_arguments(&expressions, REQUIRED_IF_ARGUMENTS);
                        let condition = &expressions[1];
                        
                        if evaluate(condition, env) == LispOutput::Bool(true) {
                            let true_expr = &expressions[2];
                            return evaluate(true_expr, env);
                        } else {
                            let false_expr = &expressions[3];
                            return evaluate(false_expr, env);
                        }
                    },
                    "and" => {
                        for expr in &expressions[1..] {
                            let clause_bool = evaluate(expr, env);
                            if clause_bool == LispOutput::Bool(false) {
                                return clause_bool;
                            }
                        }
                        return LispOutput::Bool(true);
                    },
                    "or" => {
                        for expr in &expressions[1..] {
                            let clause_bool = evaluate(expr, env);
                            if clause_bool == LispOutput::Bool(true) {
                                return clause_bool;
                            }
                        }
                        return LispOutput::Bool(false);
                    },
                    "del" => {
                        check_arguments(&expressions, REQUIRED_DEL_ARGUMENTS);
                        if let LispExpression::Symbol(symbol) = &expressions[1] {
                            return env.borrow_mut().del(&symbol);
                        }
                        panic!("expecting a symbol when removing a binding!");
                    },
                    "let" => {
                        check_arguments(&expressions, REQUIRED_LET_ARGUMENTS);

                        let mut bindings = HashMap::new();

                        if let LispExpression::List(definitions) = &expressions[1] {
                            for def in definitions {
                                if let LispExpression::List(binding) = &def {
                                    let var = match &binding[0] {
                                        LispExpression::Symbol(symbol) => symbol,
                                        _ => panic!("expecting first element of binding to be symbol!"),
                                    };
                                    let expr = &binding[1];

                                    bindings.insert(var.clone(), evaluate(expr, env));
                                } else {
                                    panic!("each binding should be a LispExpression List!");
                                }
                            }
                        } else {
                            panic!("expecting list of bindings");
                        }

                        let mut new_env = Rc::new(RefCell::new(Environment::build(
                            bindings,
                            Some(env.clone()),
                        )));

                        return evaluate(&expressions[2], &mut new_env);
                    },
                    "set!" => {
                        check_arguments(&expressions, REQUIRED_SET_BANG_ARGUMENTS);
                        let variable = match &expressions[1] {
                            LispExpression::Symbol(variable) => variable,
                            _ => panic!("expecting variable to be String type!"),
                        };
                        let value = evaluate(&expressions[2], env);
                        return env.borrow_mut().set_bang(variable, value);
                    },
                    _ => {},
                }
            }

            let mut expr_iterator = expressions.iter();
            let function = match evaluate(
                expr_iterator.next().unwrap(), 
                env) {
                    LispOutput::Lambda(output) => output,
                    _ => panic!("expected function for first expression of list"),
            };
            let args = expr_iterator.map(|expr| evaluate(expr, env)).collect();
            return function.call(args);
        },
    }
}


// ============== TESTS ===============

#[cfg(test)]
mod tests {
    use super::*;

    fn create_empty_environment() -> Rc<RefCell<Environment>> {
        return Rc::new(RefCell::new(Environment::new()));
    }

    fn create_global_environment() -> Rc<RefCell<Environment>> {
        return Rc::new(RefCell::new(Environment::global_env()));
    }

    #[test]
    fn single_integer() {
        let lisp_integer = LispExpression::Integer(1);
        let mut env = create_empty_environment();

        let expected = LispOutput::Integer(1);
        let result = evaluate(&lisp_integer, &mut env);
        
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_defintion() {
        let mut env = create_empty_environment();

        let lisp_definition = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);

        let expected = LispOutput::Integer(2);
        let defintion_result = evaluate(&lisp_definition, &mut env);

        assert_eq!(expected, defintion_result);

        let lisp_x = LispExpression::Symbol("x".to_string());
        let x_result = evaluate(&lisp_x, &mut env);

        assert_eq!(expected, x_result);
    }

    #[test]
    #[should_panic]
    fn variable_not_found() {
        let mut env = create_empty_environment();
        let nonexistent_variable = LispExpression::Symbol("x".to_string());
        evaluate(&nonexistent_variable, &mut env);
    }

    #[test]
    fn simple_lambda() {
        let mut env = create_global_environment();
        let add_one = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("add_one".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("lambda".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("x".to_string()),
                ]),
                LispExpression::List(vec![
                    LispExpression::Symbol("+".to_string()),
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Integer(1),
                ]),
            ]),
        ]);

        evaluate(&add_one, &mut env);

        let two_plus_one = LispExpression::List(vec![
            LispExpression::Symbol("add_one".to_string()),
            LispExpression::Integer(2),
        ]);

        let result = evaluate(&two_plus_one, &mut env);
        let expected = LispOutput::Integer(3);

        assert_eq!(expected, result);
    }

    #[test]
    fn simple_if_statement() {
        let mut env = create_global_environment();
        let always_true_expression = LispExpression::List(vec![
            LispExpression::Symbol("if".to_string()),
            LispExpression::Symbol("#t".to_string()),
            LispExpression::Integer(1),
            LispExpression::Integer(0),
        ]);

        let always_false_expression = LispExpression::List(vec![
            LispExpression::Symbol("if".to_string()),
            LispExpression::Symbol("#f".to_string()),
            LispExpression::Integer(1),
            LispExpression::Integer(0),
        ]);

        let true_result = evaluate(&always_true_expression, &mut env);
        let false_result = evaluate(&always_false_expression, &mut env);

        assert_eq!(LispOutput::Integer(1), true_result);
        assert_eq!(LispOutput::Integer(0), false_result);
    }

    #[test]
    fn simple_and_statement() {
        let mut env = create_global_environment();
        let single_true_expression = LispExpression::List(vec![
            LispExpression::Symbol("and".to_string()),
            LispExpression::Symbol("#t".to_string()),
        ]);

        let single_false_expression = LispExpression::List(vec![
            LispExpression::Symbol("and".to_string()),
            LispExpression::Symbol("#f".to_string()),
        ]);

        let nested_and_expression = LispExpression::List(vec![
            LispExpression::Symbol("and".to_string()),
            LispExpression::Symbol("#t".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("equal?".to_string()),
                LispExpression::Integer(10),
                LispExpression::List(vec![
                    LispExpression::Symbol("+".to_string()),
                    LispExpression::Integer(1),
                    LispExpression::Integer(2),
                    LispExpression::Integer(3),
                    LispExpression::Integer(4),
                ]),
            ]),
        ]);

        let true_result = evaluate(&single_true_expression, &mut env);
        let false_result = evaluate(&single_false_expression, &mut env);
        let nested_result = evaluate(&nested_and_expression, &mut env);

        assert_eq!(LispOutput::Bool(true), true_result);
        assert_eq!(LispOutput::Bool(false), false_result);
        assert_eq!(LispOutput::Bool(true), nested_result);
    }

    #[test]
    fn short_circuiting_and() {
        let mut env = create_global_environment();
        let nested_and_expression = LispExpression::List(vec![
            LispExpression::Symbol("and".to_string()),
            LispExpression::Symbol("#f".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("add_one".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("lambda".to_string()),
                    LispExpression::List(vec![
                        LispExpression::Symbol("x".to_string()),
                    ]),
                    LispExpression::List(vec![
                        LispExpression::Symbol("+".to_string()),
                        LispExpression::Symbol("x".to_string()),
                        LispExpression::Integer(1),
                    ]),
                ]),
            ]),
        ]);

        let nested_result = evaluate(&nested_and_expression, &mut env);

        // add_one function should not be defined, since it is expected that
        // the and short circuiting occurred at the first true expression
        let borrowed_env = env.borrow();
        let add_one_func = borrowed_env.bindings.get("add_one");
        
        match add_one_func {
            Some(_) => panic!("function should not be defined!"),
            None => {},
        };

        assert_eq!(LispOutput::Bool(false), nested_result);
    }

    #[test]
    fn non_short_circuiting_and() {
        let mut env = create_global_environment();
        let nested_and_expression = LispExpression::List(vec![
            LispExpression::Symbol("and".to_string()),
            LispExpression::Symbol("#t".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("add_one".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("lambda".to_string()),
                    LispExpression::List(vec![
                        LispExpression::Symbol("x".to_string()),
                    ]),
                    LispExpression::List(vec![
                        LispExpression::Symbol("+".to_string()),
                        LispExpression::Symbol("x".to_string()),
                        LispExpression::Integer(1),
                    ]),
                ]),
            ]),
            LispExpression::List(vec![
                LispExpression::Symbol("equal?".to_string()),
                LispExpression::Integer(10),
                LispExpression::List(vec![
                    LispExpression::Symbol("+".to_string()),
                    LispExpression::Integer(1),
                    LispExpression::Integer(2),
                    LispExpression::Integer(3),
                    LispExpression::Integer(4),
                ]),
            ]),
        ]);

        let nested_result = evaluate(&nested_and_expression, &mut env);

        // add_one function should not be defined, since it is expected that
        // the and short circuiting occurred at the first true expression
        let borrowed_env = env.borrow();
        let add_one_func = borrowed_env.bindings.get("add_one");
        
        match add_one_func {
            Some(_) => {},
            None => { panic!("function should not be defined!") },
        };

        assert_eq!(LispOutput::Bool(true), nested_result);
    }

    #[test]
    fn short_circuiting_or() {
        let mut env = create_global_environment();
        let nested_and_expression = LispExpression::List(vec![
            LispExpression::Symbol("or".to_string()),
            LispExpression::Symbol("#t".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("add_one".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("lambda".to_string()),
                    LispExpression::List(vec![
                        LispExpression::Symbol("x".to_string()),
                    ]),
                    LispExpression::List(vec![
                        LispExpression::Symbol("+".to_string()),
                        LispExpression::Symbol("x".to_string()),
                        LispExpression::Integer(1),
                    ]),
                ]),
            ]),
        ]);

        let nested_result = evaluate(&nested_and_expression, &mut env);

        // add_one function should not be defined, since it is expected that
        // the or short circuiting occurred at the first true expression
        let borrowed_env = env.borrow();
        let add_one_func = borrowed_env.bindings.get("add_one");
        
        match add_one_func {
            Some(_) => panic!("function should not be defined!"),
            None => {},
        }

        assert_eq!(LispOutput::Bool(true), nested_result);
    }

    #[test]
    fn empty_list() {
        let mut env = create_global_environment();
        let emtpy_list_expression = LispExpression::List(vec![
            LispExpression::Symbol("list".to_string()),
        ]);

        let expected = LispOutput::List(Box::new(LispList::Nil));
        let result = evaluate(&emtpy_list_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn single_element_list() {
        let mut env = create_global_environment();
        let list_expression = LispExpression::List(vec![
            LispExpression::Symbol("list".to_string()),
            LispExpression::Integer(3),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(3),
                    Box::new(LispList::Nil)
                )
            )
        );

        let result = evaluate(&list_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn multiple_element_list() {
        let mut env = create_global_environment();
        let list_expression = LispExpression::List(vec![
            LispExpression::Symbol("list".to_string()),
            LispExpression::Integer(1),
            LispExpression::Integer(2),
            LispExpression::Integer(3),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(1),
                    Box::new(
                        LispList::Cons(
                            LispOutput::Integer(2),
                            Box::new(
                                LispList::Cons(
                                    LispOutput::Integer(3),
                                    Box::new(LispList::Nil)
                                )
                            )
                        )
                    )
                )
            )
        );

        let result = evaluate(&list_expression, &mut env);
        assert_eq!(expected, result);

        let get_car_expression = LispExpression::List(vec![
            LispExpression::Symbol("car".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
        ]);

        let expected = LispOutput::Integer(1);
        let result = evaluate(&get_car_expression, &mut env);
        assert_eq!(expected, result);


        let get_cdr_expression = LispExpression::List(vec![
            LispExpression::Symbol("cdr".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(2),
                    Box::new(
                        LispList::Cons(
                            LispOutput::Integer(3),
                            Box::new(LispList::Nil)
                        )
                    )
                )
            )
        );

        let result = evaluate(&get_cdr_expression, &mut env);
        assert_eq!(expected, result);
    }

    #[test]
    fn is_list() {
        let mut env = create_global_environment();
        let list_expression = LispExpression::List(vec![
            LispExpression::Symbol("list?".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
        ]);
        let list_expected = LispOutput::Bool(true);
        
        let function_expression = LispExpression::List(vec![
            LispExpression::Symbol("list?".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("add_one".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("lambda".to_string()),
                    LispExpression::List(vec![
                        LispExpression::Symbol("x".to_string()),
                    ]),
                    LispExpression::List(vec![
                        LispExpression::Symbol("+".to_string()),
                        LispExpression::Symbol("x".to_string()),
                        LispExpression::Integer(1),
                    ]),
                ]),
            ]), 
        ]);
        let function_expected = LispOutput::Bool(false);
                        
        let integer_expression = LispExpression::List(vec![
            LispExpression::Symbol("list?".to_string()),
            LispExpression::Integer(3),
        ]);
        let integer_expected = LispOutput::Bool(false);

        let bool_expression = LispExpression::List(vec![
            LispExpression::Symbol("list?".to_string()),
            LispExpression::Symbol("#t".to_string()),
        ]);
        let bool_expected = LispOutput::Bool(false);

        assert_eq!(list_expected, evaluate(&list_expression, &mut env));
        assert_eq!(function_expected, evaluate(&function_expression, &mut env));
        assert_eq!(integer_expected, evaluate(&integer_expression, &mut env));
        assert_eq!(bool_expected, evaluate(&bool_expression, &mut env));
    }

    #[test]
    fn empty_list_length() {
        let mut env = create_global_environment();
        let empty_list_length_expression = LispExpression::List(vec![
            LispExpression::Symbol("length".to_string()),
            LispExpression::Symbol("nil".to_string()),
        ]);
        
        let expected = LispOutput::Integer(0);

        assert_eq!(expected, evaluate(&empty_list_length_expression, &mut env));
    }

    #[test]
    fn single_element_list_length() {
        let mut env = create_global_environment();
        let list_length_expression = LispExpression::List(vec![
            LispExpression::Symbol("length".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(3),
            ]),
        ]);
        
        let expected = LispOutput::Integer(1);

        assert_eq!(expected, evaluate(&list_length_expression, &mut env));
    }

    #[test]
    fn multi_element_list_length() {
        let mut env = create_global_environment();
        let list_length_expression = LispExpression::List(vec![
            LispExpression::Symbol("length".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Symbol("#t".to_string()),
                LispExpression::Symbol("nil".to_string()),
                LispExpression::Integer(4),
                LispExpression::Integer(5),
            ]),
        ]);
        
        let expected = LispOutput::Integer(5);

        assert_eq!(expected, evaluate(&list_length_expression, &mut env));
    }

    #[test]
    fn indexing_into_single_element_list() {
        let mut env = create_global_environment();
        let list_ref_expression = LispExpression::List(vec![
            LispExpression::Symbol("list-ref".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
            ]),
            LispExpression::Integer(0),
        ]);

        let expected = LispOutput::Integer(1);
        let result = evaluate(&list_ref_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn indexing_into_multi_element_list() {
        let mut env = create_global_environment();
        let list_ref_expression = LispExpression::List(vec![
            LispExpression::Symbol("list-ref".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
                LispExpression::Integer(4),
                LispExpression::Integer(5),
            ]),
            LispExpression::Integer(3),
        ]);

        let expected = LispOutput::Integer(4);
        let result = evaluate(&list_ref_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn indexing_into_empty_list() {
        let mut env = create_global_environment();
        let list_ref_expression = LispExpression::List(vec![
            LispExpression::Symbol("list-ref".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Integer(0),
        ]);

        evaluate(&list_ref_expression, &mut env);
    }

    #[test]
    #[should_panic]
    fn indexing_out_of_bounds_non_empty_list() {
        let mut env = create_global_environment();
        let list_ref_expression = LispExpression::List(vec![
            LispExpression::Symbol("list-ref".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
            LispExpression::Integer(5),
        ]);

        evaluate(&list_ref_expression, &mut env);
    }

    #[test]
    fn appending_no_list() {
        let mut env = create_global_environment();
        let append_empty_expression = LispExpression::List(vec![
            LispExpression::Symbol("append".to_string()),
        ]);

        let expected = LispOutput::List(Box::new(LispList::Nil));
        let result = evaluate(&append_empty_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn appending_single_empty_list() {
        let mut env = create_global_environment();
        let append_empty_expression = LispExpression::List(vec![
            LispExpression::Symbol("append".to_string()),
            LispExpression::Symbol("nil".to_string()),
        ]);

        let expected = LispOutput::List(Box::new(LispList::Nil));
        let result = evaluate(&append_empty_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn appending_single_non_empty_list() {
        let mut env = create_global_environment();
        let append_empty_expression = LispExpression::List(vec![
            LispExpression::Symbol("append".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(1),
                    Box::new(
                        LispList::Cons(
                            LispOutput::Integer(2),
                            Box::new(
                                LispList::Cons(
                                    LispOutput::Integer(3),
                                    Box::new(LispList::Nil)
                                )
                            )
                        )
                    )
                )
            )
        );

        let result = evaluate(&append_empty_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn appending_non_empty_list_with_empty_lists() {
        let mut env = create_global_environment();
        let append_empty_expression = LispExpression::List(vec![
            LispExpression::Symbol("append".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("nil".to_string()),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(1),
                    Box::new(
                        LispList::Cons(
                            LispOutput::Integer(2),
                            Box::new(
                                LispList::Cons(
                                    LispOutput::Integer(3),
                                    Box::new(LispList::Nil)
                                )
                            )
                        )
                    )
                )
            )
        );

        let result = evaluate(&append_empty_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn appending_two_non_empty_lists() {
        let mut env = create_global_environment();
        let append_empty_expression = LispExpression::List(vec![
            LispExpression::Symbol("append".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
            ]),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(4),
                LispExpression::Integer(5),
            ]),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(1),
                    Box::new(
                        LispList::Cons(
                            LispOutput::Integer(2),
                            Box::new(
                                LispList::Cons(
                                    LispOutput::Integer(3),
                                    Box::new(
                                        LispList::Cons(
                                            LispOutput::Integer(4),
                                            Box::new(
                                                LispList::Cons(
                                                    LispOutput::Integer(5),
                                                    Box::new(LispList::Nil)
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        );

        let result = evaluate(&append_empty_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn appending_multiple_non_empty_lists_and_empty_lists() {
        let mut env = create_global_environment();
        let append_empty_expression = LispExpression::List(vec![
            LispExpression::Symbol("append".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
            ]),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(3),
            ]),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(4),
                LispExpression::Integer(5),
            ]),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("nil".to_string()),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(1),
                    Box::new(
                        LispList::Cons(
                            LispOutput::Integer(2),
                            Box::new(
                                LispList::Cons(
                                    LispOutput::Integer(3),
                                    Box::new(
                                        LispList::Cons(
                                            LispOutput::Integer(4),
                                            Box::new(
                                                LispList::Cons(
                                                    LispOutput::Integer(5),
                                                    Box::new(LispList::Nil)
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        );

        let result = evaluate(&append_empty_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn map_on_non_list() {
        let mut env = create_global_environment();
        let map_expression = LispExpression::List(vec![
            LispExpression::Symbol("map".to_string()),
            LispExpression::Integer(1),
            LispExpression::Symbol("+".to_string()),
        ]);

        evaluate(&map_expression, &mut env);
    }

    #[test]
    fn map_on_empty_list() {
        let mut env = create_global_environment();
        let map_expression = LispExpression::List(vec![
            LispExpression::Symbol("map".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("+".to_string()),
        ]);

        let expected = LispOutput::List(Box::new(LispList::Nil));
        let result = evaluate(&map_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn map_on_single_element_list() {
        let mut env = create_global_environment();
        let map_expression = LispExpression::List(vec![
            LispExpression::Symbol("map".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(3),
            ]),
            LispExpression::Symbol("-".to_string()),
        ]);

        let expected = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(-3),
                    Box::new(
                        LispList::Nil,
                    )
                )
            )
        );
        let result = evaluate(&map_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn filter_on_non_list() {
        let mut env = create_global_environment();
        let filter_expression = LispExpression::List(vec![
            LispExpression::Symbol("filter".to_string()),
            LispExpression::Integer(1),
            LispExpression::Symbol("+".to_string()),
        ]);

        evaluate(&filter_expression, &mut env);
    }

    #[test]
    fn filter_on_empty_list() {
        let mut env = create_global_environment();
        let filter_expression = LispExpression::List(vec![
            LispExpression::Symbol("filter".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("+".to_string()),
        ]);

        let expected = LispOutput::List(Box::new(LispList::Nil));
        let result = evaluate(&filter_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn filter_on_single_element_list() {
        let mut env = create_global_environment();

        let greater_than_one_func = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("greater_than_one".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("lambda".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("x".to_string()),
                ]),
                LispExpression::List(vec![
                    LispExpression::Symbol(">".to_string()),
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Integer(1),
                ])
            ])
        ]);

        evaluate(&greater_than_one_func, &mut env);

        let filter_expression_false = LispExpression::List(vec![
            LispExpression::Symbol("filter".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(0),
            ]),
            LispExpression::Symbol("greater_than_one".to_string()),
        ]);

        let filter_expression_true = LispExpression::List(vec![
            LispExpression::Symbol("filter".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(3),
            ]),
            LispExpression::Symbol("greater_than_one".to_string()),
        ]);

        let expected_filter_false = LispOutput::List(Box::new(LispList::Nil));
        let expected_filter_true = LispOutput::List(
            Box::new(
                LispList::Cons(
                    LispOutput::Integer(3),
                    Box::new(
                        LispList::Nil,
                    )
                )
            )
        );

        let result_false = evaluate(&filter_expression_false, &mut env);
        let result_true = evaluate(&filter_expression_true, &mut env);

        assert_eq!(expected_filter_false, result_false);
        assert_eq!(expected_filter_true, result_true);
    }

    #[test]
    #[should_panic]
    fn reduce_on_non_list() {
        let mut env = create_global_environment();
        let reduce_expression = LispExpression::List(vec![
            LispExpression::Symbol("reduce".to_string()),
            LispExpression::Integer(1),
            LispExpression::Symbol("+".to_string()),
            LispExpression::Integer(1),
        ]);

        evaluate(&reduce_expression, &mut env);
    }

    #[test]
    fn reduce_on_empty_list() {
        let mut env = create_global_environment();
        let reduce_expression = LispExpression::List(vec![
            LispExpression::Symbol("reduce".to_string()),
            LispExpression::Symbol("nil".to_string()),
            LispExpression::Symbol("+".to_string()),
            LispExpression::Integer(0),
        ]);

        let expected = LispOutput::Integer(0);
        let result = evaluate(&reduce_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn reduce_on_single_element_list() {
        let mut env = create_global_environment();

        let reduce_expression = LispExpression::List(vec![
            LispExpression::Symbol("reduce".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
            ]),
            LispExpression::Symbol("+".to_string()),
            LispExpression::Integer(0),
        ]);

        let expected = LispOutput::Integer(1);

        let result = evaluate(&reduce_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn reduce_on_multi_element_list() {
        let mut env = create_global_environment();

        let reduce_expression = LispExpression::List(vec![
            LispExpression::Symbol("reduce".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("list".to_string()),
                LispExpression::Integer(1),
                LispExpression::Integer(2),
                LispExpression::Integer(3),
                LispExpression::Integer(4),
                LispExpression::Integer(5),
            ]),
            LispExpression::Symbol("+".to_string()),
            LispExpression::Integer(0),
        ]);

        let expected = LispOutput::Integer(15);

        let result = evaluate(&reduce_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn begin_empty_arguments() {
        let mut env = create_global_environment();

        let begin_expression = LispExpression::List(vec![
            LispExpression::Symbol("begin".to_string()),
        ]);
        
        evaluate(&begin_expression, &mut env);
    }

    #[test]
    fn begin_single_argument() {
        let mut env = create_global_environment();

        let begin_expression = LispExpression::List(vec![
            LispExpression::Symbol("begin".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("x".to_string()),
                LispExpression::Integer(2),
            ]),
            LispExpression::Symbol("x".to_string()),
        ]);

        let expected = LispOutput::Integer(2);

        let result = evaluate(&begin_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn begin_multiple_arguments() {
        let mut env = create_global_environment();

        let add_one_func = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("add_one".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("lambda".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("x".to_string()),
                ]),
                LispExpression::List(vec![
                    LispExpression::Symbol("+".to_string()),
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Integer(1),
                ])
            ])
        ]);

        // x = 2
        // y = x + 3  - should evaluate to 5
        // add_one(y) - should evaluate to 6
        let begin_expression = LispExpression::List(vec![
            LispExpression::Symbol("begin".to_string()),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("x".to_string()),
                LispExpression::Integer(2),
            ]),
            LispExpression::List(vec![
                LispExpression::Symbol("define".to_string()),
                LispExpression::Symbol("y".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("+".to_string()),
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Integer(3),
                ]),
            ]),
            LispExpression::List(vec![
                add_one_func,
                LispExpression::Symbol("y".to_string()),
            ])
        ]);

        let expected = LispOutput::Integer(6);

        let result = evaluate(&begin_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn del_non_existent_object() {
        let mut env = create_global_environment();

        let del_expression = LispExpression::List(vec![
            LispExpression::Symbol("del".to_string()),
            LispExpression::Symbol("add_one".to_string()),
        ]);

        evaluate(&del_expression, &mut env);
    }

    #[test]
    fn del_variable_definition() {
        let mut env = create_global_environment();

        let define_var = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);

        evaluate(&define_var, &mut env);

        let del_expression = LispExpression::List(vec![
            LispExpression::Symbol("del".to_string()),
            LispExpression::Symbol("x".to_string()),
        ]);

        let expected = LispOutput::Integer(2);
        let result = evaluate(&del_expression, &mut env);
        
        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn del_variable_definition_twice() {
        let mut env = create_global_environment();

        let define_var = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);

        evaluate(&define_var, &mut env);

        let del_expression = LispExpression::List(vec![
            LispExpression::Symbol("del".to_string()),
            LispExpression::Symbol("x".to_string()),
        ]);

        let expected = LispOutput::Integer(2);
        let result = evaluate(&del_expression, &mut env);
        
        assert_eq!(expected, result);

        evaluate(&del_expression, &mut env);
    }

    #[test]
    fn let_simple_variable_definition() {
        let mut env = create_global_environment();

        let let_expression = LispExpression::List(vec![
            LispExpression::Symbol("let".to_string()),
            LispExpression::List(vec![
                LispExpression::List(vec![
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Integer(2),
                ])
            ]),
            LispExpression::Symbol("x".to_string()),
        ]);

        let expected = LispOutput::Integer(2);
        let result = evaluate(&let_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    fn let_binary_operations() {
        let mut env = create_global_environment();

        let let_expression = LispExpression::List(vec![
            LispExpression::Symbol("let".to_string()),
            LispExpression::List(vec![
                LispExpression::List(vec![
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Integer(2),
                ]),
                LispExpression::List(vec![
                    LispExpression::Symbol("y".to_string()),
                    LispExpression::Integer(3),
                ]),
                LispExpression::List(vec![
                    LispExpression::Symbol("z".to_string()),
                    LispExpression::Integer(6),
                ]),
            ]),
            LispExpression::List(vec![
                LispExpression::Symbol("equal?".to_string()),
                LispExpression::List(vec![
                    LispExpression::Symbol("*".to_string()),
                    LispExpression::Symbol("x".to_string()),
                    LispExpression::Symbol("y".to_string()),
                ]),
                LispExpression::Symbol("z".to_string()),
            ]),
        ]);

        let expected = LispOutput::Bool(true);
        let result = evaluate(&let_expression, &mut env);

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn set_bang_non_existent_variable() {
        let mut env = create_global_environment();

        let set_bang_expression = LispExpression::List(vec![
            LispExpression::Symbol("set!".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);

        evaluate(&set_bang_expression, &mut env);
    }

    #[test]
    fn set_bang_single_variable() {
        let mut env = create_global_environment();

        let define_x = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);

        evaluate(&define_x, &mut env);

        let get_x = LispExpression::Symbol("x".to_string());
        let expected_x_before = LispOutput::Integer(2);

        assert_eq!(expected_x_before, evaluate(&get_x, &mut env));
        
        let set_bang_expression = LispExpression::List(vec![
            LispExpression::Symbol("set!".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(5),
        ]);

        let set_bang_result = evaluate(&set_bang_expression, &mut env);
        let expected_x_after = LispOutput::Integer(5);

        assert_eq!(expected_x_after, set_bang_result);
        assert_eq!(expected_x_after, evaluate(&get_x, &mut env));
    }
}
