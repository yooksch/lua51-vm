use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{lua_function, lua_return, lua_string, lua_table, types::{LuaFunctionArgs, LuaFunctionReturn, LuaResult, LuaValue}};

pub fn print(args: &LuaFunctionArgs) -> LuaFunctionReturn {
    println!("{:?}", args);
    LuaResult::Ok(vec![])
}

pub fn tostring(args: &LuaFunctionArgs) -> LuaFunctionReturn {
    if args.len() == 0 {
        lua_return!(LuaValue::Nil.into());
    }

    lua_return!(match &*args[0].borrow() {
        LuaValue::Number(n) => lua_string!(format!("{}", n.0)).into(),
        a => lua_string!(format!("{:?}", a)).into()
    });
}

pub fn make() -> BTreeMap<Rc<RefCell<LuaValue>>, Rc<RefCell<LuaValue>>> {
    lua_table! {
        lua_string!("print") => lua_function!(print).into(),
        lua_string!("tostring") => lua_function!(tostring).into()
    }
}
