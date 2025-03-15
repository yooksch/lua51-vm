use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::{Arc, Mutex}};

use rand::RngCore;

#[derive(Debug)]
pub enum LuaError {
    UnsupportedArithmeticOperation,
    AttemptedNullCall,
    AttemptedTableCall,
    AttemptedBooleanConcatenation,
    AttemptedFunctionConcatenation,
    AttemptedTableConcatenation,
    AttemptedIndexOfNonTable,
    AttemptedNotOperationOnNonBoolean,
    UnsupportedLengthOperation,
    ParseFloatError(std::num::ParseFloatError),
    ConstantNotFound(usize),
    UpValueNotFound(usize)
}

impl std::fmt::Display for LuaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for LuaError {}

impl From<std::num::ParseFloatError> for LuaError {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}

pub type LuaResult<T> = Result<T, LuaError>;

// Wraps an f64 to provide the Eq trait
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LuaNumber(pub f64);

impl From<f64> for LuaNumber {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl PartialOrd for LuaNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LuaNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Eq for LuaNumber {}

impl std::ops::Add for LuaNumber {
    type Output = LuaNumber;

    fn add(self, rhs: Self) -> Self::Output {
        LuaNumber(self.0 + rhs.0)
    }
}

impl std::ops::Sub for LuaNumber {
    type Output = LuaNumber;

    fn sub(self, rhs: Self) -> Self::Output {
        LuaNumber(self.0 - rhs.0)
    }
}

impl std::ops::Mul for LuaNumber {
    type Output = LuaNumber;

    fn mul(self, rhs: Self) -> Self::Output {
        LuaNumber(self.0 * rhs.0)
    }
}

impl std::ops::Div for LuaNumber {
    type Output = LuaNumber;

    fn div(self, rhs: Self) -> Self::Output {
        LuaNumber(self.0 / rhs.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LuaValue {
    Number(LuaNumber),
    String(String),
    Boolean(bool),
    Table(BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>),
    Function(LuaFunction),
    Nil
}

impl From<bool> for LuaValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for LuaValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for LuaValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<f64> for LuaValue {
    fn from(value: f64) -> Self {
        Self::Number(value.into())
    }
}

impl From<LuaFunction> for LuaValue {
    fn from(value: LuaFunction) -> Self {
        Self::Function(value)
    }
}

impl From<BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>> for LuaValue {
    fn from(value: BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>) -> Self {
        Self::Table(value)
    }
}

impl Into<Rc<RefCell<LuaValue>>> for LuaValue {
    fn into(self) -> Rc<RefCell<LuaValue>> {
        Rc::new(RefCell::new(self))
    }
}

// TODO: Reduice boilerplate
impl std::ops::Add for LuaValue {
    type Output = LuaResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        LuaResult::Ok(match (self, rhs) {
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::Number(a + b),

            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::Number((a.parse::<f64>()? + b.parse::<f64>()?).into()),
            (LuaValue::String(a), LuaValue::Number(b)) => LuaValue::Number((a.parse::<f64>()? + b.0).into()),
            (LuaValue::Number(a), LuaValue::String(b)) => LuaValue::Number((a.0 + b.parse::<f64>()?).into()),

            _ => return LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        })
    }
}

impl std::ops::Sub for LuaValue {
    type Output = LuaResult<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        LuaResult::Ok(match (self, rhs) {
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::Number(a - b),

            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::Number((a.parse::<f64>()? - b.parse::<f64>()?).into()),
            (LuaValue::String(a), LuaValue::Number(b)) => LuaValue::Number((a.parse::<f64>()? - b.0).into()),
            (LuaValue::Number(a), LuaValue::String(b)) => LuaValue::Number((a.0 - b.parse::<f64>()?).into()),

            _ => return LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        })
    }
}

impl std::ops::Mul for LuaValue {
    type Output = LuaResult<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        LuaResult::Ok(match (self, rhs) {
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::Number(a * b),

            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::Number((a.parse::<f64>()? * b.parse::<f64>()?).into()),
            (LuaValue::String(a), LuaValue::Number(b)) => LuaValue::Number((a.parse::<f64>()? * b.0).into()),
            (LuaValue::Number(a), LuaValue::String(b)) => LuaValue::Number((a.0 * b.parse::<f64>()?).into()),

            _ => return LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        })
    }
}

impl std::ops::Div for LuaValue {
    type Output = LuaResult<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        LuaResult::Ok(match (self, rhs) {
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::Number(a / b),

            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::Number((a.parse::<f64>()? / b.parse::<f64>()?).into()),
            (LuaValue::String(a), LuaValue::Number(b)) => LuaValue::Number((a.parse::<f64>()? / b.0).into()),
            (LuaValue::Number(a), LuaValue::String(b)) => LuaValue::Number((a.0 / b.parse::<f64>()?).into()),

            _ => return LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        })
    }
}

#[allow(unused)]
impl LuaValue {
    pub fn modulo(self, rhs: Self) -> LuaResult<Self> {
        todo!()
    }

    pub fn pow(self, rhs: Self) -> LuaResult<Self> {
        todo!()
    }

    pub fn unm(self) -> LuaResult<Self> {
        todo!()
    }

    pub fn concat(self, rhs: Self) -> LuaResult<Self> {
        todo!()
    }

    pub fn call(self, args: Vec<Rc<RefCell<LuaValue>>>) -> LuaResult<Vec<Rc<RefCell<LuaValue>>>> {
        match self {
            LuaValue::Function(f) => f.invoke(&args),
            _ => todo!()
        }
    }
}

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

#[macro_export]
macro_rules! lua_function {
    ( $func:expr ) => {
        crate::types::LuaFunction::new(std::sync::Arc::new(std::sync::Mutex::new(Box::new($func))))
    };
}

#[macro_export]
macro_rules! lua_table {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = std::collections::BTreeMap::new();

        $(
            map.insert(Rc::new(RefCell::new($key)), Rc::new(RefCell::new($value)));
        )*

        map
    }};
}

#[macro_export]
macro_rules! lua_string {
    ( $string:expr ) => {
        LuaValue::String($string.into())
    }
}

#[macro_export]
macro_rules! lua_number {
    ( $number:expr ) => {
        LuaValue::Number(($number).into())
    };
}

#[macro_export]
macro_rules! lua_return {
    ( $( $value:expr ),* $(,)? ) => {
        return LuaResult::Ok(vec![$($value, )*])
    };
}

pub type LuaFunctionArgs = Vec<Rc<RefCell<LuaValue>>>;
pub type LuaFunctionReturn = LuaResult<Vec<Rc<RefCell<LuaValue>>>>;
