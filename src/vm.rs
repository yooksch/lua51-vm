use std::{cell::RefCell, collections::BTreeMap, ops::Sub, rc::Rc, sync::{Arc, Mutex}};

use crate::{bytecode::{LuaPrototype, OpCode, FIELDS_PER_FLUSH}, libs, lua_function, lua_string, lua_table, types::{LuaError, LuaFunction, LuaResult, LuaValue}};

macro_rules! get_rk {
    ($idx:expr, $constants:ident, $stack:ident) => {
        if $idx >= 256 {
            match $constants.get($idx - 256) {
                Some(c) => c.clone(),
                None => return LuaResult::Err(LuaError::ConstantNotFound($idx - 256))
            }
        } else {
            $stack[$idx].clone()
        }
    };
}

pub struct VirtualMachine {
    pub environment: Rc<RefCell<BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>>>
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(BTreeMap::new()))
        }
    }

    pub fn load_std_libraries(&mut self) {
        let insert = |t: BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>>| {
            for (k, v) in t.iter() {
                self.environment.borrow_mut().insert(k.clone(), v.clone());
            }
        };

        insert(libs::global::make());
    }

    pub fn execute(&mut self, function: LuaPrototype, args: Option<Vec<Rc<RefCell<LuaValue>>>>, upvalues: Option<Vec<Rc<RefCell<LuaValue>>>>, vararg: Option<Vec<Rc<RefCell<LuaValue>>>>) -> LuaResult<Vec<Rc<RefCell<LuaValue>>>> {
        let mut upvalues = match upvalues {
            Some(v) => v,
            None => Vec::new()
        };
        let vararg = match vararg {
            Some(v) => v,
            None => Vec::new()
        };

        let mut pc = 0i64;
        let mut stack: Vec<Rc<RefCell<LuaValue>>> = vec![Rc::new(RefCell::new(LuaValue::Nil)); function.max_stack_size.into()];
        let mut stack_top = 0usize;

        // push args onto the stack
        if let Some(args) = args {
            for i in 0..args.len() {
                stack[i] = args[i].clone();
            }
        }

        let instructions = function.instructions;
        let constants = function.constants;
        while pc < instructions.len() as i64 {
            let inst = &instructions[pc as usize];
            match inst.code {
                OpCode::Move => {
                    stack[inst.A] = stack[inst.B].clone();
                },
                OpCode::LoadNil => {
                    for i in inst.A..inst.B {
                        stack[i] = LuaValue::Nil.into();
                    }
                },
                OpCode::LoadK => {
                    stack[inst.A] = match constants.get(inst.Bx) {
                        Some(k) => k.clone(),
                        None => return LuaResult::Err(LuaError::ConstantNotFound(inst.Bx))
                    };
                },
                OpCode::LoadBool => {
                    stack[inst.A] = LuaValue::Boolean(inst.B > 0).into();
                    if inst.C != 0 {
                        pc += 1;
                    }
                },
                OpCode::GetGlobal => {
                    let name = match constants.get(inst.Bx) {
                        Some(n) => n,
                        None => return LuaResult::Err(LuaError::ConstantNotFound(inst.Bx))
                    };
                    stack[inst.A] = match self.environment.borrow().get(name) {
                        Some(v) => v.clone(),
                        None => LuaValue::Nil.into()
                    };
                },
                OpCode::SetGlobal => {
                    let name = match constants.get(inst.Bx) {
                        Some(n) => n,
                        None => return LuaResult::Err(LuaError::ConstantNotFound(inst.Bx))
                    };
                    self.environment.borrow_mut().insert(name.clone(), stack[inst.A].clone());
                },
                OpCode::GetUpValue => {
                    stack[inst.A] = upvalues[inst.Bx].clone();
                },
                OpCode::SetUpValue => {
                    upvalues[inst.B] = stack[inst.A].clone();
                },
                OpCode::GetTable => {
                    let v = match &*stack[inst.B].borrow() {
                        LuaValue::Table(t) => {
                            t.get(&get_rk!(inst.C, constants, stack)).or(Some(&LuaValue::Nil.into())).unwrap().clone()
                        },
                        _ => LuaValue::Nil.into()
                    };
                    stack[inst.A] = v;
                },
                OpCode::SetTable => {
                    match &mut *stack[inst.A].borrow_mut() {
                        LuaValue::Table(t) => {
                            let index = get_rk!(inst.B, constants, stack);
                            t.insert(index, get_rk!(inst.C, constants, stack));
                        },
                        _ => return LuaResult::Err(LuaError::AttemptedIndexOfNonTable)
                    }
                },
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Pow | OpCode::Mod => {
                    let lhs = get_rk!(inst.B, constants, stack).borrow().clone();
                    let rhs = get_rk!(inst.C, constants, stack).borrow().clone();
                    let res = match inst.code {
                        OpCode::Add => lhs + rhs,
                        OpCode::Sub => lhs - rhs,
                        OpCode::Mul => lhs * rhs,
                        OpCode::Div => lhs / rhs,
                        OpCode::Pow => lhs.pow(rhs),
                        OpCode::Mod => lhs.modulo(rhs),
                        _ => panic!()
                    };
                    stack[inst.A] = res?.into();
                },
                OpCode::UnaryMinus => {
                    let v = stack[inst.B].borrow().clone().unm()?;
                    stack[inst.A] = v.into();
                },
                OpCode::Not => {
                    let v = match *stack[inst.B].borrow() {
                        LuaValue::Boolean(b) => LuaValue::Boolean(!b),
                        _ => return LuaResult::Err(LuaError::AttemptedNotOperationOnNonBoolean)
                    };
                    stack[inst.A] = v.into();
                },
                OpCode::Len => {
                    let v = match stack[inst.B].borrow().clone() {
                        LuaValue::String(s) => LuaValue::Number((s.len() as f64).into()),
                        LuaValue::Table(t) => LuaValue::Number((t.keys().len() as f64).into()),
                        _ => return LuaResult::Err(LuaError::UnsupportedLengthOperation)
                    };
                    stack[inst.A] = v.into();
                },
                OpCode::Concat => {
                    let v = stack[inst.B].borrow().clone().concat(stack[inst.C].borrow().clone())?.into();
                    stack[inst.A] = v;
                },
                OpCode::Jmp => {
                    pc += inst.sBx;
                },
                OpCode::Call => {
                    let mut args = Vec::new();
                    let last_arg_idx = if inst.B == 0 {
                        stack_top
                    } else {
                        inst.A + inst.B
                    };
                    for i in inst.A + 1..last_arg_idx {
                        args.push(stack[i].clone());
                    }

                    let results = stack[inst.A].borrow().clone().call(args)?;
                    
                    if inst.C == 0 {
                        stack_top = inst.A + results.len() - 1;
                    }

                    for i in 0.. if inst.C != 0 { inst.C - 1 } else { results.len() } {
                        stack[inst.A + i] = results[i].clone();
                    }
                },
                OpCode::Return => {
                    let mut values = Vec::new();

                    let last_value_idx = if inst.B == 0 {
                        stack_top
                    } else {
                        inst.A + inst.B - 1
                    };

                    for i in inst.A..last_value_idx {
                        values.push(stack[i].clone());
                    }

                    return LuaResult::Ok(values);
                },
                OpCode::TailCall => {
                    let mut args = Vec::new();
                    
                    for i in inst.A + 1..inst.A + inst.B - 1 {
                        args.push(stack[i].clone());
                    }

                    return LuaResult::Ok(stack[inst.A].borrow().clone().call(args)?);
                },
                OpCode::Vararg => {
                    let len = if inst.B == 0 {
                        stack_top = inst.A + vararg.len() - 1;
                        vararg.len()
                    } else {
                        inst.B
                    };

                    for i in 0..len {
                        let v = match vararg.get(i) {
                            Some(v) => v.clone(),
                            None => LuaValue::Nil.into()
                        };
                        stack[inst.A + i] = v;
                    }
                },
                OpCode::LSelf => {
                    stack[inst.A + 1] = stack[inst.B].clone();
                    let v = match &*stack[inst.B].borrow() {
                        LuaValue::Table(t) => {
                            let key = get_rk!(inst.C, constants, stack);
                            match t.get(&key) {
                                Some(v) => v.clone(),
                                None => LuaValue::Nil.into()
                            }
                        },
                        _ => return LuaResult::Err(LuaError::AttemptedIndexOfNonTable)
                    };
                    stack[inst.A] = v;
                },
                OpCode::Eq | OpCode::Lt | OpCode::Le => {
                    let lhs = get_rk!(inst.B, constants, stack);
                    let rhs = get_rk!(inst.C, constants, stack);
                    let res = match inst.code {
                        OpCode::Eq => lhs.eq(&rhs),
                        OpCode::Lt => lhs.lt(&rhs),
                        OpCode::Le => lhs.le(&rhs),
                        _ => panic!()
                    };

                    if res != (inst.A == 1) {
                        pc += 1;
                    }
                },
                OpCode::Test => {
                    if let LuaValue::Boolean(b) = &*stack[inst.A].borrow() {
                        if *b != (inst.C == 1) {
                            pc += 1;
                        }
                    }
                },
                OpCode::TestSet => {
                    let v = match &*stack[inst.B].borrow() {
                        LuaValue::Boolean(b) => *b,
                        _ => panic!()
                    };

                    if v == (inst.C == 1) {
                        stack[inst.A] = stack[inst.B].clone();
                    } else {
                        pc += 1;
                    }
                },
                OpCode::ForPrep => {
                    let lhs = stack[inst.A].borrow().clone();
                    let rhs = stack[inst.A + 2].borrow().clone();
                    stack[inst.A] = lhs.sub(rhs)?.into();
                    pc += inst.sBx;
                },
                OpCode::ForLoop => {
                    let index = stack[inst.A].borrow().clone();
                    let limit = stack[inst.A + 1].borrow().clone();
                    let step = stack[inst.A + 2].borrow().clone();

                    let do_loop = if step >= 0f64.into() {
                        index <= limit
                    } else {
                        index >= limit
                    };

                    if do_loop {
                        stack[inst.A] = index.clone().into();
                        stack[inst.A + 3] = index.into();
                        pc += inst.sBx;
                    }
                },
                OpCode::TForLoop => {
                    let results = stack[inst.A].borrow().clone().call(vec![
                        stack[inst.A + 1].clone(),
                        stack[inst.A + 2].clone()
                    ])?;

                    for i in inst.A + 3..inst.A + 2 + inst.C {
                        stack[i] = results[i - inst.A - 3].clone();
                    }

                    if !matches!(*stack[inst.A + 3].borrow(), LuaValue::Nil) {
                        stack[inst.A + 2] = stack[inst.A + 3].clone();
                        pc += inst.sBx;
                    }

                    pc += 1;
                },
                OpCode::NewTable => {
                    let mut table: BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>> = BTreeMap::new();

                    for i in 1..inst.B + 1 {
                        table.insert(LuaValue::Number((i as f64).into()).into(), LuaValue::Nil.into());
                    }

                    stack[inst.A] = LuaValue::Table(table).into();
                },
                OpCode::SetList => {
                    match &mut *stack[inst.A].borrow_mut() {
                        LuaValue::Table(t) => {
                            for i in 1..inst.B {
                                let key = (((inst.C - 1) * FIELDS_PER_FLUSH + i) as f64).into();
                                t.insert(LuaValue::Number(key).into(), stack[inst.A + i].clone());
                            }
                        },
                        _ => return LuaResult::Err(LuaError::AttemptedIndexOfNonTable)
                    }
                },
                OpCode::Closure => {
                    let sub_func = function.prototypes[inst.Bx].clone();
                    let sub_upvalues = if sub_func.upvalue_count > 0 {
                        let mut sub_upvalues: Vec<Rc<RefCell<LuaValue>>> = Vec::new();

                        for i in 0..sub_func.upvalue_count as usize {
                            let pseudo = &instructions[(pc as usize) + i];

                            if matches!(pseudo.code, OpCode::Move) {
                                sub_upvalues[i] = stack[pseudo.B].clone();
                            } else if matches!(pseudo.code, OpCode::GetUpValue) {
                                sub_upvalues[i] = upvalues[pseudo.B].clone();
                            }
                        }

                        pc += sub_func.upvalue_count as i64;

                        Some(sub_upvalues)
                    } else {
                        None
                    };


                    // Create new virtual machine and clone a reference to the environment
                    let mut new_vm = VirtualMachine::new();
                    new_vm.environment = self.environment.clone();
                    let func = lua_function!(move |args| {
                        new_vm.execute(sub_func.clone(), Some(args.to_vec()), sub_upvalues.clone(), None)
                    });
                    stack[inst.A] = LuaValue::Function(func).into();
                },
                OpCode::Close => {
                    for i in 0..inst.A {
                        upvalues[i] = LuaValue::Nil.into();
                    }
                }
            };

            pc += 1;
        }

        Ok(vec![])
    }
}
