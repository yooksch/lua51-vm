#[macro_export]
macro_rules! lua_function {
    ( $func:expr ) => {
        crate::types::function::LuaFunction::new(std::sync::Arc::new(std::sync::Mutex::new(Box::new($func))))
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