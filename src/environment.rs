use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::expr::Literal;

#[derive(Clone,Debug)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub env: HashMap<String, Literal>,
}
impl Environment {
 
    pub fn new_main() -> Self {
        Self {
            enclosing: Some(Rc::new(RefCell::new(Environment::new()))),
            env: HashMap::new(),
        }
    }
    pub fn new() -> Self {
        Self {
            enclosing: None,
            env: HashMap::new(),
        }
    }

     pub fn deep_clone(&self) -> Environment {
        Environment {
            env: self.env.clone(),
            enclosing: self.enclosing
                .as_ref()
                .map(|parent| Rc::new(RefCell::new(parent.clone().borrow_mut().deep_clone()))),
        }
    }


     pub fn enclose(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(parent),
            env: HashMap::new(),
        }))
    }

    

    pub fn get_At(
        env: Rc<RefCell<Self>>,
        distance: usize,
        name: String,
    ) -> Result<Literal, String> {
        Self::ancestor(env, distance).clone().borrow_mut().get(name)
    }




    pub fn ASSIGN_AT(env: Rc<RefCell<Self>>, distance: usize, name: &String, value: Literal) {
        Self::ancestor(env, distance)
            .clone()
            .borrow_mut()
            .env
            .insert(name.clone(), value);
    }
    pub fn ancestor(env: Rc<RefCell<Self>>, distance: usize) -> Rc<RefCell<Environment>> {
        let mut env = env.clone();
        for _ in 0..distance {
            if let Some(a) = env.clone().borrow_mut().enclosing.as_ref(){
              env = a.clone();
            }
        }
        env
    }
    pub fn assign(&mut self, name: &String, value: &Literal) -> Result<(), String> {
        if  self.env.contains_key(name) {
           self.env.insert(name.clone(), value.clone());
            Ok(())

        }
        else if self.enclosing.is_some(){
 self.enclosing
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow_mut()
                .assign(name, value)
         } 

        else {
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
