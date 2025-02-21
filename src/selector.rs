use crate::{
    card::CardType,
    number::{Number, NumberRange},
    zone::Zone,
};

#[derive(Clone)]
#[allow(dead_code)]
pub enum CardNameSelector {
    Name(String),
    NameAnd(Vec<CardNameSelector>),
    NameOr(Vec<CardNameSelector>),
    NameNot(Box<CardNameSelector>),
    HasType(CardType),
    Cost(Box<NumberRange<Number>>),
    Any,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct CardSelector {
    pub name: CardNameSelector,
    pub zone: Vec<Zone>,
}
