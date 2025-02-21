use crate::{
    effect::{CardEffect, EffectTrigger},
    number::Number,
};

#[allow(dead_code)]
pub struct Card {
    pub name: String,
    pub localized_name: String,
    pub cost: Number,
    pub vp: Number,
    pub rules: Vec<(EffectTrigger, CardEffect)>,
    pub types: Vec<CardType>,
}

#[derive(Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CardType {
    Action,
    Treasure,
    Victory,
    Reaction,
    Curse,
    Attack,
}
