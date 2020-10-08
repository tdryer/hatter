use {
    crate::{Args, Env, Result, Scope, Stmt, Symbol, Value},
    std::{cell::RefCell, collections::BTreeMap, ops::Deref, rc::Rc},
};

#[derive(Clone)]
pub struct List(Rc<RefCell<Vec<Value>>>);
impl List {
    pub fn new(s: Vec<Value>) -> Self {
        Self(rcell!(s))
    }
}
impl Deref for List {
    type Target = Rc<RefCell<Vec<Value>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq for List {
    fn eq(&self, other: &List) -> bool {
        let me = self.borrow();
        let you = other.borrow();
        me.len() == you.len() && me.iter().enumerate().all(|(i, v)| v == &you[i])
    }
}
impl From<Vec<Value>> for List {
    fn from(v: Vec<Value>) -> Self {
        Self::new(v)
    }
}

#[derive(Clone)]
pub struct Map(Rc<RefCell<BTreeMap<Symbol, Value>>>);
impl Map {
    pub fn new(m: BTreeMap<Symbol, Value>) -> Self {
        Self(rcell!(m))
    }
}
impl Deref for Map {
    type Target = Rc<RefCell<BTreeMap<Symbol, Value>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq for Map {
    fn eq(&self, other: &Map) -> bool {
        let me = self.borrow();
        let you = other.borrow();
        me.len() == you.len() && me.iter().all(|(i, v)| Some(v) == you.get(i))
    }
}
impl From<BTreeMap<Symbol, Value>> for Map {
    fn from(m: BTreeMap<Symbol, Value>) -> Self {
        Self::new(m)
    }
}

#[derive(Clone)]
pub enum Fn {
    Fn(Vec<Symbol>, Vec<Stmt>, Scope),
    Native(Rc<NativeFn>),
    Special(Rc<SpecialFn>),
}

pub type NativeFn = dyn std::ops::Fn(Args) -> Result<Value>;
pub type SpecialFn = dyn std::ops::Fn(&mut Env, &[Stmt]) -> Result<Value>;

#[allow(unused_variables)]
pub trait Object {
    fn get(&self, key: &str) -> Option<Value> {
        Some(Value::None)
    }

    fn set(&self, key: &str, val: Value) {}
}
