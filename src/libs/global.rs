use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{lua_function, lua_return, lua_string, lua_table, types::{LuaError, function::{LuaFunctionArgs, LuaFunctionReturn}, LuaResult, value::LuaValue}};

pub fn print(args: &LuaFunctionArgs) -> LuaFunctionReturn {
    if args.len() > 0 {
        let mut s = "".to_owned();
        for arg in args {
            let x = tostring(&vec![arg.clone()])?[0].borrow().as_string()?.to_owned();
            s.push_str(&x);
            s.push_str("\t");
        }
        println!("{}", s);
    }

    LuaResult::Ok(vec![])
}

pub fn error(args: &LuaFunctionArgs) -> LuaFunctionReturn {
    if args.len() == 0 {
        lua_return!(); // Follow Lua's behavior
    }

    let msg = tostring(&vec![args[0].clone()])?[0].borrow().as_string()?.to_owned();
    let level = match args.get(1) {
        Some(l) => Some(*l.borrow().as_f64()?),
        None => None
    };
    LuaResult::Err(LuaError::TriggeredByUser((msg, level)))
}

pub fn tostring(args: &LuaFunctionArgs) -> LuaFunctionReturn {
    if args.len() == 0 {
        return LuaResult::Err(LuaError::ExpectedArgument);
    }

    lua_return!(match &*args[0].borrow() {
        LuaValue::String(s) => lua_string!(s).into(),
        LuaValue::Number(n) => lua_string!(format!("{}", n.0)).into(),
        LuaValue::Boolean(b) => lua_string!(if *b { "true" } else { "false" }).into(),
        LuaValue::Nil => lua_string!("nil").into(),
        LuaValue::Table(_t) => lua_string!(format!("table:{:?}", args[0].as_ptr())).into(),
        LuaValue::Function(_f) => lua_string!(format!("function:{:?}", args[0].as_ptr())).into()
    });
}

pub fn make() -> BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>> {
    lua_table! {
        lua_string!("print") => lua_function!(print).into(),
        lua_string!("error") => lua_function!(error).into(),
        lua_string!("tostring") => lua_function!(tostring).into()
    }
}
