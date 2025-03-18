use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{lua_function, lua_return, lua_string, lua_table, types::{LuaError, LuaFunctionArgs, LuaFunctionReturn, LuaResult, LuaValue}};

pub fn print(args: &LuaFunctionArgs) -> LuaFunctionReturn {
    if args.len() > 0 {
        let mut s = "".to_owned();
        for arg in args {
            let x = match tostring(&vec![arg.clone()])?[0].borrow().clone() {
                LuaValue::String(s) => s,
                _ => panic!()
            };
            s.push_str(&x);
            s.push_str("\t");
        }
        println!("{}", s);
    }

    LuaResult::Ok(vec![])
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
        a => lua_string!(format!("{:?}", a)).into()
    });
}

pub fn make() -> BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>> {
    lua_table! {
        lua_string!("print") => lua_function!(print).into(),
        lua_string!("tostring") => lua_function!(tostring).into()
    }
}
