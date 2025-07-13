use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::expr::Literal;

#[derive(Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub env: HashMap<String, Literal>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            env: HashMap::new(),
        }
    }

    pub fn assign(&mut self, name: &String, value: &Literal) -> Result<(), String> {
        if self.env.contains_key(name) {
            self.env.insert(name.clone(), value.clone());
            Ok(())
        } else if self.enclosing.is_some() {
            self.enclosing
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow_mut()
                .assign(name, value)
        } else {
            Err(format!("Undefined variable {}", name))
        }
    }
    pub fn define(&mut self, name: String, value: Literal) {
        self.env.insert(name, value);
    }
    pub fn get(&mut self, name: String) -> Result<Literal, String> {
        let key = self.env.contains_key(&name);
        if key {
            Ok(self.env.get(&name).unwrap().clone())
        } else if self.enclosing.is_some() {
            let data = self
                .enclosing
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow_mut()
                .get(name)?;
            Ok(data)
        } else {
            Err("no such variable".to_string())
        }
    }
}
