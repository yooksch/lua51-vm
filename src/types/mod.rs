pub mod value;
pub mod number;
pub mod function;
pub mod macros;

#[derive(Debug)]
pub enum LuaError {
    UnsupportedArithmeticOperation,
    AttemptedNullCall,
    AttemptedTableCall,
    AttemptedBooleanConcatenation,
    AttemptedFunctionConcatenation,
    AttemptedTableConcatenation,
    AttemptedNilConcatenation,
    AttemptedIndexOfNonTable,
    AttemptedNotOperationOnNonBoolean,
    UnsupportedLengthOperation,
    ParseFloatError(std::num::ParseFloatError),
    ConstantNotFound(usize),
    UpValueNotFound(usize),
    AttemptedCallOnUnsupportedType,
    ExpectedArgument,
    ExpectedNumber,
    ExpectedString,
    ExpectedBoolean,
    ExpectedTable,
    ExpectedFunction,
    TriggeredByUser((String, Option<f64>))
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

#[derive(Debug)]
pub struct LuaRuntimeResult<T> {
    pub inner: LuaResult<T>,
    pub source_line: Option<i64>,
    pub source_name: Option<String>
}

impl<T: std::fmt::Debug> std::fmt::Display for LuaRuntimeResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            LuaResult::Ok(r) => write!(f, "{:?}", r),
            LuaResult::Err(e) => write!(f, "{:?} at line {} in {}", e, self.source_line.unwrap_or(-1), self.source_name.clone().unwrap_or("unknown".to_owned()))
        }
    }
}