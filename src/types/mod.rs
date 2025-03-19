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
    ExpectedFunction
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
