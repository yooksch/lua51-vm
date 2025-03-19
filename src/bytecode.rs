use std::{cell::RefCell, rc::Rc};

use async_recursion::async_recursion;
use enum_map::{Enum, enum_map};
use once_cell::sync::Lazy;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::types::LuaValue;

#[derive(Debug)]
pub enum DecodeError {
    InvalidHeaderSignature,
    UnsupportedVersion,
    UnsupportedFormat,
    UnsupportedEndian,
    ReadErr(tokio::io::Error)
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for DecodeError {}

impl From<tokio::io::Error> for DecodeError {
    fn from(value: tokio::io::Error) -> Self {
        Self::ReadErr(value)
    }
}

type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, Enum, Copy, Clone)]
#[allow(nonstandard_style)]
pub enum OpMode {
    iABC,
    iABx,
    iAsBx
}

#[derive(Debug, Enum, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
    Move = 0,
    LoadK,
    LoadBool,
    LoadNil,
    GetUpValue,
    GetGlobal,
    GetTable,
    SetGlobal,
    SetUpValue,
    SetTable,
    NewTable,
    LSelf,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    UnaryMinus,
    Not,
    Len,
    Concat,
    Jmp,
    r#Eq,
    Lt,
    Le,
    Test,
    TestSet,
    Call,
    TailCall,
    Return,
    ForLoop,
    ForPrep,
    TForLoop,
    SetList,
    Close,
    Closure,
    Vararg
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Move,
            1 => Self::LoadK,
            2 => Self::LoadBool,
            3 => Self::LoadNil,
            4 => Self::GetUpValue,
            5 => Self::GetGlobal,
            6 => Self::GetTable,
            7 => Self::SetGlobal,
            8 => Self::SetUpValue,
            9 => Self::SetTable,
            10 => Self::NewTable,
            11 => Self::LSelf,
            12 => Self::Add,
            13 => Self::Sub,
            14 => Self::Mul,
            15 => Self::Div,
            16 => Self::Mod,
            17 => Self::Pow,
            18 => Self::UnaryMinus,
            19 => Self::Not,
            20 => Self::Len,
            21 => Self::Concat,
            22 => Self::Jmp,
            23 => Self::Eq,
            24 => Self::Lt,
            25 => Self::Le,
            26 => Self::Test,
            27 => Self::TestSet,
            28 => Self::Call,
            29 => Self::TailCall,
            30 => Self::Return,
            31 => Self::ForLoop,
            32 => Self::ForPrep,
            33 => Self::TForLoop,
            34 => Self::SetList,
            35 => Self::Close,
            36 => Self::Closure,
            37 => Self::Vararg,
            _ => panic!("Unsupported OP code")
        }
    }
}

static OP_CODE_MODES: Lazy<enum_map::EnumMap<OpCode, OpMode>> = Lazy::new(|| enum_map! {
    OpCode::Move => OpMode::iABC,
    OpCode::LoadK => OpMode::iABx,
    OpCode::LoadBool => OpMode::iABC,
    OpCode::LoadNil => OpMode::iABC,
    OpCode::GetUpValue => OpMode::iABC,
    OpCode::GetGlobal => OpMode::iABx,
    OpCode::GetTable => OpMode::iABC,
    OpCode::SetGlobal => OpMode::iABx,
    OpCode::SetUpValue => OpMode::iABC,
    OpCode::SetTable => OpMode::iABC,
    OpCode::NewTable => OpMode::iABC,
    OpCode::LSelf => OpMode::iABC,
    OpCode::Add => OpMode::iABC,
    OpCode::Sub => OpMode::iABC,
    OpCode::Mul => OpMode::iABC,
    OpCode::Div => OpMode::iABC,
    OpCode::Mod => OpMode::iABC,
    OpCode::Pow => OpMode::iABC,
    OpCode::UnaryMinus => OpMode::iABC,
    OpCode::Not => OpMode::iABC,
    OpCode::Len => OpMode::iABC,
    OpCode::Concat => OpMode::iABC,
    OpCode::Jmp => OpMode::iAsBx,
    OpCode::Eq => OpMode::iABC,
    OpCode::Le => OpMode::iABC,
    OpCode::Lt => OpMode::iABC,
    OpCode::Test => OpMode::iABC,
    OpCode::TestSet => OpMode::iABC,
    OpCode::Call => OpMode::iABC,
    OpCode::TailCall => OpMode::iABC,
    OpCode::Return => OpMode::iABC,
    OpCode::ForLoop => OpMode::iAsBx,
    OpCode::ForPrep => OpMode::iAsBx,
    OpCode::TForLoop => OpMode::iABC,
    OpCode::SetList => OpMode::iABC,
    OpCode::Close => OpMode::iABC,
    OpCode::Closure => OpMode::iABx,
    OpCode::Vararg => OpMode::iABC
});

pub static FIELDS_PER_FLUSH: usize = 50;

#[derive(Debug, Clone)]
#[allow(nonstandard_style)]
pub struct Instruction {
    pub mode: OpMode,
    pub code: OpCode,
    // We use larger than required types to do less conversion in the VM
    pub A: usize,
    pub B: usize,
    pub C: usize,
    pub Bx: usize,
    pub sBx: i64
}

impl From<u32> for Instruction {
    fn from(value: u32) -> Self {
        let op: u8 = (value & 0b111111) as u8;
        let code: OpCode = op.into();
        let mode = &OP_CODE_MODES[code];

        let mut instruction = Instruction {
            mode: mode.clone(),
            code,
            A: (value >> 6 & 0b1111_1111) as usize,
            B: 0,
            C: 0,
            Bx: 0,
            sBx: 0
        };

        match mode {
            OpMode::iABC => {
                instruction.C = ((value >> 14) & 0b0001_1111_1111) as usize;
                instruction.B = ((value >> 23) & 0b0001_1111_1111) as usize;
            },
            OpMode::iABx => {
                instruction.Bx = ((value >> 14) & 0b0011_1111_1111_1111_1111) as usize;
            },
            OpMode::iAsBx => {
                instruction.sBx = ((value >> 14) & 0b0011_1111_1111_1111_1111) as i64 - 131071;
            }
        };

        instruction
    }
}

#[derive(Debug, Clone)]
pub struct LuaLocal {
    pub name: String,
    pub start_pc: i64,
    pub end_pc: i64
}

#[derive(Debug, Clone)]
pub struct LuaPrototype {
    pub source_name: Option<String>,
    pub line_defined: i64,
    pub last_line_defined: i64,
    pub upvalue_count: u8,
    pub param_count: u8,
    pub vararg_flags: u8,
    pub max_stack_size: u8,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Rc<RefCell<LuaValue>>>,
    pub prototypes: Vec<LuaPrototype>,
    pub source_line_positions: Vec<i64>,
    pub locals: Vec<LuaLocal>,
    pub upvalues: Vec<String>
}

impl LuaPrototype {
    pub fn new() -> LuaPrototype {
        LuaPrototype {
            source_name: None,
            line_defined: 0,
            last_line_defined: 0,
            upvalue_count: 0,
            param_count: 0,
            vararg_flags: 0,
            max_stack_size: 255,
            instructions: Vec::new(),
            constants: Vec::new(),
            prototypes: Vec::new(),
            source_line_positions: Vec::new(),
            locals: Vec::new(),
            upvalues: Vec::new() 
        }
    }
}

#[derive(Debug)]
pub struct LuaHeader {
    pub little_endian: bool,
    pub int_size: u8,
    pub size_t_size: u8,
    pub instruction_size: u8,
    pub lua_number_size: u8,
    pub integral_flag: u8
}

async fn read_u64<R: AsyncRead + Unpin>(header: &LuaHeader, size: u8, reader: &mut BufReader<R>) -> DecodeResult<u64> {
    DecodeResult::Ok(if header.little_endian {
        if size == 4 { reader.read_u32_le().await?.into() } else { reader.read_u64_le().await? }
    } else {
        if size == 4 { reader.read_u32().await?.into() } else { reader.read_u64().await? }
    })
}

async fn read_i64<R: AsyncRead + Unpin>(header: &LuaHeader, size: u8, reader: &mut BufReader<R>) -> DecodeResult<i64> {
    DecodeResult::Ok(if header.little_endian {
        if size == 4 { reader.read_i32_le().await?.into() } else { reader.read_i64_le().await? }
    } else {
        if size == 4 { reader.read_i32().await?.into() } else { reader.read_i64().await? }
    })
}

async fn read_string<R: AsyncRead + Unpin>(length: usize, reader: &mut BufReader<R>) -> DecodeResult<String> {
    let mut s = vec![0u8; length];
    reader.read_exact(&mut s).await?;
    s.remove(length - 1);
    DecodeResult::Ok(String::from_utf8_lossy(&s).to_string())
}

async fn read_lua_number<R: AsyncRead + Unpin>(header: &LuaHeader, reader: &mut BufReader<R>) -> DecodeResult<f64> {
    DecodeResult::Ok(if header.little_endian {
        if header.lua_number_size == 4 { reader.read_f32_le().await?.into() } else { reader.read_f64_le().await? }
    } else {
        if header.lua_number_size == 4 { reader.read_f32().await?.into() } else { reader.read_f64().await? }
    })
}

#[async_recursion(?Send)]
async fn read_function<R: AsyncRead + Unpin>(header: &LuaHeader, reader: &mut BufReader<R>) -> DecodeResult<LuaPrototype> {
    let mut function = LuaPrototype::new();

    function.source_name = match read_u64(header, header.size_t_size, reader).await? as usize {
        n if n > 0 => {
            Some(read_string(n, reader).await?)
        },
        _ => None
    };

    function.line_defined = read_i64(header, header.int_size, reader).await?;
    function.last_line_defined = read_i64(header, header.int_size, reader).await?;
    function.upvalue_count = reader.read_u8().await?;
    function.param_count = reader.read_u8().await?;
    function.vararg_flags = reader.read_u8().await?;
    function.max_stack_size = reader.read_u8().await?;

    // read instructions
    let instruction_count = read_i64(header, header.int_size, reader).await?;
    for _i in 0..instruction_count {
        let raw_instruction = read_u64(header, header.instruction_size, reader).await? as u32;
        function.instructions.push(raw_instruction.into());
    }

    // read constants
    let constants_count = read_i64(header, header.int_size, reader).await?;
    for _i in 0..constants_count {
        let constant_type = reader.read_u8().await?;

        match constant_type {
            0 => function.constants.push(LuaValue::Nil.into()),
            1 => function.constants.push(LuaValue::from(reader.read_u8().await? == 1).into()),
            3 => function.constants.push(LuaValue::from(read_lua_number(header, reader).await?).into()),
            4 => {
                let length = read_u64(header, header.size_t_size, reader).await? as usize;
                function.constants.push(LuaValue::from(read_string(length, reader).await?).into());
            },
            _ => {}
        };
    }

    // read function prototypes
    let function_count = read_i64(header, header.int_size, reader).await?;
    for _i in 0..function_count {
        function.prototypes.push(read_function(header, reader).await?);
    }

    // read source line positions
    let slp_count = read_i64(header, header.int_size, reader).await?;
    for _i in 0..slp_count {
        function.source_line_positions.push(read_i64(header, header.int_size, reader).await?);
    }

    // read locals
    let local_count = read_i64(header, header.int_size, reader).await?;
    for _i in 0..local_count {
        let string_len = read_u64(header, header.size_t_size, reader).await? as usize;
        let name = read_string(string_len, reader).await?;
        let start_pc = read_i64(header, header.int_size, reader).await?;
        let end_pc = read_i64(header, header.int_size, reader).await?;

        function.locals.push(LuaLocal { name, start_pc, end_pc });
    }

    // read upvalues
    let upvalue_count = read_i64(header, header.int_size, reader).await?;
    for _i in 0..upvalue_count {
        let string_len = read_u64(header, header.size_t_size, reader).await? as usize;
        function.upvalues.push(read_string(string_len, reader).await?);
    }

    DecodeResult::Ok(function)
}

pub async fn read_bytecode<R: AsyncRead + Unpin>(reader: &mut BufReader<R>) -> DecodeResult<LuaPrototype> {
    let mut header: [u8; 4] = [0; 4];
    reader.read_exact(&mut header).await?;

    if header != [0x1B, 0x4C, 0x75, 0x61] {
        return DecodeResult::Err(DecodeError::InvalidHeaderSignature);
    }

    // Verify if version == 0x51 
    match reader.read_u8().await {
        Ok(0x51) => {},
        Ok(_) => return DecodeResult::Err(DecodeError::UnsupportedVersion),
        Err(e) => return DecodeResult::Err(DecodeError::ReadErr(e))
    };

    // Verify if format version == 0
    match reader.read_u8().await {
        Ok(0) => {},
        Ok(_) => return DecodeResult::Err(DecodeError::UnsupportedFormat),
        Err(e) => return DecodeResult::Err(DecodeError::ReadErr(e))
    };

    // Init header with defaults
    let mut header = LuaHeader {
        little_endian: true,
        int_size: 4,
        size_t_size: 4,
        instruction_size: 4,
        lua_number_size: 8,
        integral_flag: 0
    };

    header.little_endian = reader.read_u8().await? == 1;
    header.int_size = reader.read_u8().await?;
    header.size_t_size = reader.read_u8().await?;
    header.instruction_size = reader.read_u8().await?;
    header.lua_number_size = reader.read_u8().await?;
    header.integral_flag = reader.read_u8().await?;

    read_function(&header, reader).await
}
