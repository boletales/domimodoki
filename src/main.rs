fn main() {
    println!("Hello, world!");
}

// Dominion simulator

struct Player<'a> {
    name: String,
    deck: Vec<CardInstance<'a>>,
    hand: Vec<CardInstance<'a>>,
    discard: Vec<CardInstance<'a>>,
    id: PlayerId,
}

struct PlayerId {
    id: i32,
}

struct Card {
    name: String,
    localized_name: String,
    cost: i32,
    vp: i32,
    action: CardEffect,
    reaction: CardEffect,
    treasure: CardEffect,
    types: Vec<CardType>,
}

struct CardInstance<'a> {
    card: &'a Card,
    id: CardId,
}

struct CardId {
    id: i32,
}

struct Game<'a> {
    players: Vec<Player<'a>>,
    supply: Vec<(Card, i32)>,
    trash: Vec<CardInstance<'a>>,
    turn: i32,
}

enum CardType {
    Action,
    Treasure,
    Victory,
    Reaction,
    Curse,
}

/*
市場（+1アクション、+1ドロー、+1購入、+1金）
Sequence([
    PlusCoin(1),
    PlusAction(1),
    PlusBuy(1),
    PlusDraw(1),
])

山賊（金貨を得る。他のプレイヤーは全員、山札の上から2枚を公開し、銅貨以外の財宝カードがあれば1枚を廃棄し、残りを捨て札にする。）
Sequence([
    PlusCoin(2),
    AllOpponents(Atack(Sequence([
        RevealTop(2,
            Sequence([
                SelectExact(
                    1,
                    CardSelector::CardAnd(vec![
                        CardSelector::ByZone(Zones::Selected),
                        CardSelector::ByName(
                            CardNameSelector::And(vec![
                                CardNameSelector::Type(CardType::Treasure),
                                CardNameSelector::Not(Box::new(CardNameSelector::Exact("Copper".to_string()))),
                            ])
                        ),
                    ]),

                    TrashCard(CardSelector::Selected),
                ),
                FocusAll(
                    CardSelector::Selected,
                    DiscardCard(CardSelector::Selected),
                ),
            ])
        ),
    ]))),
])

魔女（+2ドロー。他のプレイヤーは全員、呪いカードを1枚ずつ獲得する。）
Sequence([
    PlusDraw(2),
    AllOpponents(GainCard(Card::Curse)),
])

*/

enum EffectNumber {
    Constant(i32),
    CountCard(CardSelector),
    CountCost(CardSelector),
}

// カードの働きを記述するためのメタ言語
enum CardEffect {
    Noop,
    Sequence(Vec<CardEffect>),
    AtomicSequence(Box<CardEffect>),
    Optional(Box<CardEffect>),

    // Select系：カードを選択し、Focusの選択先を変更した上で、効果を適用する
    SelectExact(EffectNumber, CardSelector, Box<CardEffect>), // ちょうどn枚選択
    SelectFewer(EffectNumber, CardSelector, Box<CardEffect>), // n枚以下選択
    SelectAny(CardSelector, Box<CardEffect>),                 // 好きなだけ選択

    // Select亜種だけどプレイヤーの選択を必要としない
    FocusAll(CardSelector, Box<CardEffect>),

    // デッキトップ公開・Focus
    RevealTop(EffectNumber, Box<CardEffects>),

    UseCard(CardSelector),

    PlusDraw(EffectNumber),
    PlusAction(EffectNumber),
    PlusBuy(EffectNumber),
    PlusCoin(EffectNumber),

    TrashCard(CardSelector),
    DiscardCard(CardSelector),
    GainCard(CardNameSelector),

    AllOpponents(Box<CardEffect>),
    Atack(Box<CardEffect>),
    PreventAttack,
}

enum Zones {
    Deck,
    Hand,
    Discard,
    Play,
    Focused,
}

enum CardNameSelector {
    Exact(String),
    NameAnd(Vec<CardNameSelector>),
    NameOr(Vec<CardNameSelector>),
    NameNot(Box<CardNameSelector>),
    HasType(CardType),
    CostLower(i32),
    CostHigher(i32),
}

enum CardSelector {
    ByName(CardNameSelector),
    ByZone(Zones),
    DeckTop(i32),
    DeckBottom(i32),
    CardAnd(Vec<CardSelector>),
    CardOr(Vec<CardSelector>),
}

mod card_util {
    use crate::CardEffect::*;
    use crate::CardType::*;
    use crate::EffectNumber::*;
    use crate::*;
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
            name: name.to_string(),
            localized_name: localized_name.to_string(),
            cost,
            vp: 0,
            action: vanilla_effect(draw, action, buy, coin),
            reaction: Noop,
            treasure: Noop,
            types: vec![Action],
        }
    }

    pub fn vanilla_treasure_card(name: &str, localized_name: &str, cost: i32, coin: i32) -> Card {
        Card {
            name: name.to_string(),
            localized_name: localized_name.to_string(),
            cost,
            vp: 0,
            action: Noop,
            reaction: Noop,
            treasure: PlusCoin(Constant(coin)),
            types: vec![Treasure],
        }
    }

    pub fn vanilla_vp_card(name: &str, localized_name: &str, cost: i32, vp: i32) -> Card {
        Card {
            name: name.to_string(),
            localized_name: localized_name.to_string(),
            cost,
            vp,
            action: Noop,
            reaction: Noop,
            treasure: Noop,
            types: vec![Victory],
        }
    }
    pub fn vanilla_curse_card() -> Card {
        Card {
            name: "Curse".to_string(),
            localized_name: "呪い".to_string(),
            cost: 0,
            vp: -1,
            action: Noop,
            reaction: Noop,
            treasure: Noop,
            types: vec![Curse],
        }
    }

    pub fn simple_action_card(
        name: &str,
        localized_name: &str,
        cost: i32,
        has_attack: bool,
        effect: CardEffect,
    ) -> Card {
        Card {
            name: name.to_string(),
            localized_name: localized_name.to_string(),
            cost,
            vp: 0,
            action: effect,
            reaction: Noop,
            treasure: Noop,
            types: if has_attack {
                vec![Action, Reaction]
            } else {
                vec![Action]
            },
        }
    }

    pub fn focused() -> CardSelector {
        CardSelector::ByZone(Zones::Focused)
    }
}

mod expansions {
    pub mod basic_supply {
        use crate::*;
        use card_util::*;
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
    }
    pub mod base {
        use crate::{card_util::*, CardEffect::*, CardType::*, EffectNumber::*, *};

        /* ドミニオン 基本セット（第2版）
        カードリスト
        - 地下貯蔵庫
        - 礼拝堂
        - 堀
        - 家臣
        - 工房
        - 商人
        - 前駆者
        - 村
        - 改築
        - 鍛冶屋
        - 金貸し
        - 玉座の間
        - 密猟者
        - 民兵
        - 役人
        - 庭園
        - 市場
        - 衛兵
        - 議事堂
        - 研究所
        - 鉱山
        - 祝祭
        - 書庫
        - 山賊
        - 魔女
        - 職人
        */

        pub fn cellar() -> Card {
            simple_action_card(
                "Cellar",
                "地下貯蔵庫",
                2,
                false,
                Sequence(vec![
                    PlusAction(Constant(1)),
                    SelectAny(
                        CardSelector::ByZone(Zones::Hand),
                        Box::new(Sequence(vec![
                            PlusDraw(CountCard(focused())),
                            DiscardCard(focused()),
                        ])),
                    ),
                ]),
            )
        }

        pub fn chapel() -> Card {
            simple_action_card(
                "Chapel",
                "礼拝堂",
                2,
                false,
                Sequence(vec![SelectFewer(
                    Constant(4),
                    CardSelector::ByZone(Zones::Hand),
                    Box::new(TrashCard(focused())),
                )]),
            )
        }

        pub fn moat() -> Card {
            Card {
                name: "Moat".to_string(),
                localized_name: "堀".to_string(),
                cost: 2,
                vp: 0,
                action: vanilla_effect(2, 0, 0, 0),
                reaction: PreventAttack,
                treasure: Noop,
                types: vec![Action, Reaction],
            }
        }

        pub fn chancellor() -> Card {
            simple_action_card(
                "Chancellor",
                "家臣",
                3,
                false,
                Sequence(vec![
                    PlusCoin(Constant(2)), // +2金
                    RevealTop(
                        // デッキトップ1枚公開
                        Constant(1),
                        FocusAll(
                            // アクションだったら
                            CardSelector::CardAnd(vec![
                                CardSelector::ByZone(Zones::Focused),
                                CardSelector::ByName(CardNameSelector::HasType(Action)),
                            ]),
                            // 使ってもよい
                            Box::new(Optional(Box::new(Sequence(vec![
                                UseCard(focused()),
                                DiscardCard(focused()),
                            ])))),
                        ),
                    ),
                ]),
            )
        }

        pub fn workshop() -> Card {
            simple_action_card(
                "Workshop",
                "工房",
                3,
                false,
                GainCard(CardNameSelector::CostLower(4)),
            )
        }
    }
}
