use std::fmt::Debug;

use std::iter::zip;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::lisp_expression::LispExpression;
use crate::evaluate::{LispOutput, Environment, evaluate};


pub trait LispFunctionCall {
    fn call(&self, args: Vec<LispOutput>) -> LispOutput;
}


// -------------- BUILT IN FUNCTION --------------
#[derive(Clone)]
pub struct BuiltInFunction {
    function: Rc<dyn Fn(Vec<LispOutput>) -> LispOutput>,
}


impl std::fmt::Debug for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltInFunction")
    }
}

impl PartialEq for BuiltInFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.function, &other.function)
    }
}

impl LispFunctionCall for BuiltInFunction {
    fn call(&self, args: Vec<LispOutput>) -> LispOutput {
        return (self.function)(args);
    }
}

impl BuiltInFunction {
    pub fn new(built_in_func: Rc<dyn Fn(Vec<LispOutput>) -> LispOutput>) -> Self {
        return BuiltInFunction {
            function: built_in_func,
        }
    }
}


// -------------- USER FUNCTION --------------
#[derive(Debug, Clone)]
pub struct Function {
    parameters: Vec<String>,
    body: LispExpression,
    enclosing_frame: Weak<RefCell<Environment>>,
}


impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
            && self.body == other.body
            && Rc::ptr_eq(&self.enclosing_frame.upgrade().unwrap(), &other.enclosing_frame.upgrade().unwrap())
    }
}

impl LispFunctionCall for Function {
    fn call(&self, args: Vec<LispOutput>) -> LispOutput {
        
        let mut bindings = HashMap::new();

        for (param, arg) in zip(&self.parameters, args) {
            bindings.insert(param.clone(), arg);
        }

        let mut new_env = Rc::new(RefCell::new(
            Environment {
                parent_env: Some(self.enclosing_frame.upgrade().unwrap().clone()),
                bindings,
            }
        ));

        return evaluate(&self.body, &mut new_env);
    }
}

impl Function {
    pub fn build(
        parameters: LispExpression, 
        body: LispExpression, 
        enclosing_frame: Rc<RefCell<Environment>>
    ) -> Self {
            let mut params = vec![];
            if let LispExpression::List(param_expressions) = parameters {
                for param_expr in &param_expressions {
                    match &param_expr {
                        LispExpression::Symbol(param) => params.push(param.clone()),
                        _ => panic!("one or more parameters is not a LispExpression symbol"),
                    };
                }
            } else {
                panic!("parameters should be a list");
            }
            return Self {
                parameters: params,
                body,
                enclosing_frame: Rc::downgrade(&enclosing_frame),
            };
    }
}


// -------------- LISP FUNCTION ENUM WRAPPER --------------
#[derive(Debug, Clone, PartialEq)]
pub enum LispFunction {
    BuiltInFunction(BuiltInFunction),
    Function(Function),
}

impl LispFunctionCall for LispFunction {
    fn call(&self, args: Vec<LispOutput>) -> LispOutput {
        match self {
            LispFunction::BuiltInFunction(function) => function.call(args),
            LispFunction::Function(func) => func.call(args),
        }
    }
}