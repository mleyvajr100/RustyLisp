use std::collections::HashMap;
use std::rc::Rc;
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
}

fn check_arguments(args: &Vec<LispExpression>, number_of_args: usize) {
    if args.len() != number_of_args {
        panic!("special form was not supplied with correct number of arugments");
    }
}

const REQUIRED_DEFINE_ARGUMENTS: usize = 3;
const REQUIRED_LAMBDA_ARGUMENTS: usize = 3;
const REQUIRED_IF_ARGUMENTS: usize = 4;

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
}
