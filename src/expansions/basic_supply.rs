use crate::{card::Card, card_util::*};
use std::collections::HashMap;
// 基本カード
pub fn copper() -> Card {
    vanilla_treasure_card("Copper", "銅貨", 0, 1)
}
pub fn silver() -> Card {
    vanilla_treasure_card("Silver", "銀貨", 3, 2)
}
pub fn gold() -> Card {
    vanilla_treasure_card("Gold", "金貨", 6, 3)
}
pub fn estate() -> Card {
    vanilla_vp_card("Estate", "屋敷", 2, 1)
}
pub fn duchy() -> Card {
    vanilla_vp_card("Duchy", "公領", 5, 3)
}
pub fn province() -> Card {
    vanilla_vp_card("Province", "属州", 8, 6)
}
pub fn curse() -> Card {
    vanilla_curse_card()
}

pub fn basic_supply() -> HashMap<String, Card> {
    vec![
        copper(),
        silver(),
        gold(),
        estate(),
        duchy(),
        province(),
        curse(),
    ]
    .into_iter()
    .map(|c| (c.name.clone(), c))
    .collect()
}
