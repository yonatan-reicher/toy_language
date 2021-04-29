use crate::ast::TypeAst;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::ast::Ast;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Int,
    Bool,
    Function(Rc<(Type, Type)>),
}

pub struct TypeChecker {
    vars: RefCell<HashMap<String, Type>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self { vars: RefCell::new(HashMap::new()) }
    }

    pub fn var<F, T>(&self, name: String, t: Type, f: F) -> T
        where F: FnOnce(&TypeChecker) -> T 
    {
        self.vars.borrow_mut().insert(name.clone(), t);
        let ret = f(self);
        self.vars.borrow_mut().remove(&name);
        ret
    }

    pub fn get_var(&self, name: &str) -> Option<Type> {
        self.vars.borrow().get(name).cloned()
    }

    pub fn eval(&self, type_ast: &TypeAst) -> Result<Type, ()> {
        match type_ast {
            TypeAst::Int(_) => Ok(Type::Int),
            TypeAst::Bool(_) => Ok(Type::Bool),
            TypeAst::Function(l, r) => {
                let func = (self.eval(l)?, self.eval(r)?);
                Ok(Type::Function(Rc::new(func)))
            }
        }
    }

    pub fn typecheck(&self, ast: &Ast) -> Result<Type, ()> {
        match ast {
            Ast::Paren(e, _) => self.typecheck(e.as_ref()),
            Ast::Var(name, _) => self.get_var(name).ok_or(()),
            Ast::IntLiteral(_, _) => Ok(Type::Int),
            Ast::BoolLiteral(_, _) => Ok(Type::Bool),
            Ast::And(left, right) => {
                let left_type = self.typecheck(left.as_ref())?;
                let right_type = self.typecheck(right.as_ref())?;
                match (left_type, right_type) {
                    (Type::Bool, Type::Bool) => Ok(Type::Bool),
                    _ => Err(()),
                }
            }
            Ast::Eq(left, right) => {
                let left_type = self.typecheck(left.as_ref())?;
                let right_type = self.typecheck(right.as_ref())?;
                match (left_type, right_type) {
                    (Type::Int, Type::Int) => Ok(Type::Bool),
                    (Type::Bool, Type::Bool) => Ok(Type::Bool),
                    _ => Err(()),
                }
            }
            Ast::Add(left, right) => {
                let left_type = self.typecheck(left.as_ref())?;
                let right_type = self.typecheck(right.as_ref())?;
                match (left_type, right_type) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    _ => Err(()),
                }
            }
            Ast::Let { name, right, body, .. } => {
                let var_type = self.typecheck(right.as_ref())?;
                self.var(name.clone(), var_type, move |checker| {
                    checker.typecheck(body.as_ref())
                })
            }
            Ast::Function { input: (input_var, _), input_type, ret, .. } => {
                self.var(input_var.clone(), self.eval(input_type)?, move |t| {
                    t.typecheck(ret.as_ref())
                })
            }
        }
    }
}



