use crate::core::{
    card::{
        Card,
        CardType::{self, *},
    },
    effect::{
        CardEffect,
        EffectTrigger::{self, *},
    },
    number::Number::*,
    selector::{CardNameSelector::*, CardSelector},
    zone::Zone,
};

pub fn vanilla_effect(draw: i32, action: i32, buy: i32, coin: i32) -> CardEffect {
    CardEffect::Sequence(vec![
        CardEffect::PlusDraw(Constant(draw)),
        CardEffect::PlusAction(Constant(action)),
        CardEffect::PlusBuy(Constant(buy)),
        CardEffect::PlusCoin(Constant(coin)),
    ])
}

pub fn vanilla_action_card(
    name: &str,
    localized_name: &str,
    cost: i32,
    draw: i32,
    action: i32,
    buy: i32,
    coin: i32,
) -> Card {
    Card {
        name: name.to_owned(),
        localized_name: localized_name.to_owned(),
        cost: Constant(cost),
        vp: Constant(0),
        rules: vec![(PlayAsAction, vanilla_effect(draw, action, buy, coin))],
        types: vec![Action],
    }
}

pub fn vanilla_treasure_card(name: &str, localized_name: &str, cost: i32, coin: i32) -> Card {
    Card {
        name: name.to_owned(),
        localized_name: localized_name.to_owned(),
        cost: Constant(cost),
        vp: Constant(0),
        rules: vec![(PlayAsTreasure, vanilla_effect(0, 0, 0, coin))],
        types: vec![Treasure],
    }
}

pub fn vanilla_vp_card(name: &str, localized_name: &str, cost: i32, vp: i32) -> Card {
    Card {
        name: name.to_owned(),
        localized_name: localized_name.to_owned(),
        cost: Constant(cost),
        vp: Constant(vp),
        rules: vec![],
        types: vec![Victory],
    }
}
pub fn vanilla_curse_card() -> Card {
    Card {
        name: "Curse".to_owned(),
        localized_name: "呪い".to_owned(),
        cost: Constant(0),
        vp: Constant(-1),
        rules: vec![],
        types: vec![Curse],
    }
}
pub fn simple_rule_card(
    name: &str,
    localized_name: &str,
    cost: i32,
    rules: Vec<(EffectTrigger, CardEffect)>,
    types: Vec<CardType>,
) -> Card {
    Card {
        name: name.to_owned(),
        localized_name: localized_name.to_owned(),
        cost: Constant(cost),
        vp: Constant(0),
        rules,
        types,
    }
}

pub fn simple_action_card(
    name: &str,
    localized_name: &str,
    cost: i32,
    has_attack: bool,
    effect: CardEffect,
) -> Card {
    simple_rule_card(
        name,
        localized_name,
        cost,
        vec![(PlayAsAction, effect)],
        if has_attack {
            vec![Action, Attack]
        } else {
            vec![Action]
        },
    )
}

pub fn focused() -> CardSelector {
    CardSelector {
        name: Any,
        zone: vec![Zone::Focused],
    }
}

pub fn hand() -> CardSelector {
    CardSelector {
        name: Any,
        zone: vec![Zone::Hand],
    }
}

pub fn discarded() -> CardSelector {
    CardSelector {
        name: Any,
        zone: vec![Zone::Discard],
    }
}

pub fn all_my_cards() -> CardSelector {
    CardSelector {
        name: Any,
        zone: vec![Zone::AllMyCards],
    }
}

pub fn in_zone(zone: Zone) -> CardSelector {
    CardSelector {
        name: Any,
        zone: vec![zone],
    }
}
