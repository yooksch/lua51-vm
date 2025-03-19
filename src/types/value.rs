use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::libs;

use super::{LuaResult, LuaError, number::LuaNumber, function::LuaFunction};

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

impl LuaValue {
    pub fn modulo(self, rhs: Self) -> LuaResult<Self> {
        LuaResult::Ok(match (self, rhs) {
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::Number((a.0 % b.0).into()),

            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::Number((a.parse::<f64>()? % b.parse::<f64>()?).into()),
            (LuaValue::String(a), LuaValue::Number(b)) => LuaValue::Number((a.parse::<f64>()? % b.0).into()),
            (LuaValue::Number(a), LuaValue::String(b)) => LuaValue::Number((a.0 % b.parse::<f64>()?).into()),

            _ => return LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        })
    }

    pub fn pow(self, rhs: Self) -> LuaResult<Self> {
        LuaResult::Ok(match (self, rhs) {
            (LuaValue::Number(a), LuaValue::Number(b)) => LuaValue::Number((a.0.powf(b.0)).into()),

            (LuaValue::String(a), LuaValue::String(b)) => LuaValue::Number((a.parse::<f64>()?.powf(b.parse::<f64>()?)).into()),
            (LuaValue::String(a), LuaValue::Number(b)) => LuaValue::Number((a.parse::<f64>()?.powf(b.0)).into()),
            (LuaValue::Number(a), LuaValue::String(b)) => LuaValue::Number((a.0.powf(b.parse::<f64>()?)).into()),

            _ => return LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        })
    }

    pub fn unm(self) -> LuaResult<Self> {
        match self {
            LuaValue::Number(n) => LuaResult::Ok((-n.0).into()),
            _ => LuaResult::Err(LuaError::UnsupportedArithmeticOperation)
        }
    }

    pub fn concat(self, rhs: Self) -> LuaResult<Self> {
        match self {
            LuaValue::String(s) => {
                let mut lhs = s.clone();
                let rhs = match libs::global::tostring(&vec![rhs.into()])?[0].borrow().clone() {
                    LuaValue::String(s) => s,
                    _ => panic!()
                };
                lhs.push_str(&rhs);
                LuaResult::Ok(LuaValue::from(lhs))
            },
            LuaValue::Number(_n) => {
                let lhs = match libs::global::tostring(&vec![self.into()])?[0].borrow().clone() {
                    LuaValue::String(s) => s,
                    _ => panic!()
                };
                let rhs = match libs::global::tostring(&vec![rhs.into()])?[0].borrow().clone() {
                    LuaValue::String(s) => s,
                    _ => panic!()
                };
                LuaResult::Ok(LuaValue::String(format!("{lhs}{rhs}")))
            },
            LuaValue::Boolean(_) => LuaResult::Err(LuaError::AttemptedBooleanConcatenation),
            LuaValue::Function(_) => LuaResult::Err(LuaError::AttemptedFunctionConcatenation),
            LuaValue::Table(_) => LuaResult::Err(LuaError::AttemptedTableConcatenation),
            LuaValue::Nil => LuaResult::Err(LuaError::AttemptedNilConcatenation)
        }
    }

    pub fn call(self, args: Vec<Rc<RefCell<LuaValue>>>) -> LuaResult<Vec<Rc<RefCell<LuaValue>>>> {
        dbg!(&args);
        match self {
            LuaValue::Function(f) => f.invoke(&args),
            LuaValue::Table(_) => LuaResult::Err(LuaError::AttemptedTableCall),
            _ => LuaResult::Err(LuaError::AttemptedCallOnUnsupportedType)
        }
    }

    pub fn as_f64<'a>(&'a self) -> LuaResult<&'a f64> {
        match self {
            LuaValue::Number(n) => LuaResult::Ok(&n.0),
            _ => LuaResult::Err(LuaError::ExpectedNumber)
        }
    }

    pub fn as_string<'a>(&'a self) -> LuaResult<&'a String> {
        match self {
            LuaValue::String(s) => LuaResult::Ok(s),
            _ => LuaResult::Err(LuaError::ExpectedString)
        }
    }

    pub fn as_bool<'a>(&'a self) -> LuaResult<&'a bool> {
        match self {
            LuaValue::Boolean(b) => LuaResult::Ok(b),
            _ => LuaResult::Err(LuaError::ExpectedBoolean)
        }
    }

    pub fn as_table<'a>(&'a self) -> LuaResult<&'a BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>> {
        match self {
            LuaValue::Table(t) => LuaResult::Ok(t),
            _ => LuaResult::Err(LuaError::ExpectedTable)
        }
    }

    pub fn as_table_mut<'a>(&'a mut self) -> LuaResult<&'a mut BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>> {
        match self {
            LuaValue::Table(t) => LuaResult::Ok(t),
            _ => LuaResult::Err(LuaError::ExpectedTable)
        }
    }

    pub fn as_function<'a>(&'a self) -> LuaResult<&'a LuaFunction> {
        match self {
            LuaValue::Function(f) => LuaResult::Ok(f),
            _ => LuaResult::Err(LuaError::ExpectedFunction)
        }
    }
}
