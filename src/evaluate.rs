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
                        let var = match &expressions[1] {
                            LispExpression::Symbol(symbol) => symbol,
                            _ => panic!("var must be LispExpression Symbol"),
                        };
        
                        let val = evaluate(&expressions[2], env);

                        env.borrow_mut().set(&var, &val);
        
                        return val;
                    },
                    "lambda" => {
                        let parameters = &expressions[1];
                        let body = &expressions[2];

                        return LispOutput::Lambda(
                            LispFunction::Function(
                                Function::build(parameters.clone(), body.clone(), env.clone())
                            )
                        );
                    }
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
    use crate::built_in_functions::built_in_function_bindings;

    fn create_empty_environment() -> Rc<RefCell<Environment>> {
        return Rc::new(RefCell::new(Environment::new()));
    }

    fn create_environment_with_built_ins() -> Rc<RefCell<Environment>> {
        return Rc::new(RefCell::new(Environment::build(built_in_function_bindings(), None)));
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
        let mut env = create_environment_with_built_ins();
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
}
