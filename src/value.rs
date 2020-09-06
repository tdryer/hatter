use {crate::Env, std::collections::HashMap};

pub type HatterFn = fn(&mut Env, &[Value]) -> Value;

#[derive(Clone)]
pub enum Value {
    None,
    Bool(bool),
    Number(f64),
    String(String),
    Fn(HatterFn),
    List(Vec<Value>),
    Map(HashMap<Value, Value>),
}

impl Value {
    pub fn typename(&self) -> &str {
        use Value::*;
        match self {
            None => "None",
            Bool(..) => "Bool",
            Number(..) => "Number",
            String(..) => "String",
            Fn(..) => "Fn",
            List(..) => "List",
            Map(..) => "Map",
        }
    }
}