use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::functions::{LispFunction, LispFunctionCall, Function};
use crate::lisp_expression::LispExpression;


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
