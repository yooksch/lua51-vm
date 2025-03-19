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