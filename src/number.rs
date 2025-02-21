use crate::selector::CardSelector;

#[derive(Clone)]
#[allow(dead_code)]
pub enum Number {
    Constant(i32),
    CountCard(CardSelector),
    CountCost(CardSelector),
    CountEmptyPiles,
    Plus(Box<Number>, Box<Number>),
    Minus(Box<Number>, Box<Number>),
    Times(Box<Number>, Box<Number>), // 乗算
    Div(Box<Number>, Box<Number>),   // 整数除算、切り捨て
    Mod(Box<Number>, Box<Number>),   // 剰余
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum NumberRange<N> {
    Exact(N),
    UpTo(N),
    AtLeast(N),
    Range(N, N),
    AnyNumber,
}

impl NumberRange<i32> {
    pub const fn contains(&self, n: i32) -> bool {
        use NumberRange::*;
        match self {
            Exact(m) => n == *m,
            UpTo(m) => n <= *m,
            AtLeast(m) => n >= *m,
            Range(a, b) => n >= *a && n <= *b,
            AnyNumber => true,
        }
    }
}
