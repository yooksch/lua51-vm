use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use rand::RngCore;

use super::{LuaResult, value::LuaValue};

// Rust:tm:
type HandlerFn = Arc<Mutex<Box<dyn FnMut(&Vec<Rc<RefCell<LuaValue>>>) -> LuaResult<Vec<Rc<RefCell<LuaValue>>>>>>>;

#[derive(Clone)]
pub struct LuaFunction {
    // Unique id for every function - allows us to implement Eq
    id: u64,
    handler: HandlerFn
}

impl LuaFunction {
    pub fn new(handler: HandlerFn) -> Self {
        Self {
            id: rand::rng().next_u64(),
            handler
        }
    }

    pub fn invoke(&self, args: &Vec<Rc<RefCell<LuaValue>>>) -> LuaResult<Vec<Rc<RefCell<LuaValue>>>> {
        (self.handler.lock().unwrap())(args)
    }
}

impl std::fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LuaFunction {{ id: {0}, handler: <function> }}", self.id)
    }
}

impl PartialOrd for LuaFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LuaFunction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for LuaFunction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for LuaFunction {}

pub type LuaFunctionArgs = Vec<Rc<RefCell<LuaValue>>>;
pub type LuaFunctionReturn = LuaResult<Vec<Rc<RefCell<LuaValue>>>>;