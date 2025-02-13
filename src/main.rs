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
    vp: EffectNumber,
    rules: Vec<(EffectTrigger, CardEffect)>,
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
    Attack,
}

enum EffectTrigger {
    OnAttack,       // Focus: 空, PreventDefaultで攻撃を無効化
    OnPlayAction,   // Focus: 空
    OnPlayTreasure, // Focus: 空
    OnCardPlay,     // Focus: カード
}

enum EffectNumber {
    Constant(i32),
    CountCard(CardSelector),
    CountCost(CardSelector),
    EmptyPiles,
    Plus(Box<EffectNumber>, Box<EffectNumber>),
    Minus(Box<EffectNumber>, Box<EffectNumber>),
    Times(Box<EffectNumber>, Box<EffectNumber>), // 乗算
    Div(Box<EffectNumber>, Box<EffectNumber>),   // 整数除算、切り捨て
    Mod(Box<EffectNumber>, Box<EffectNumber>),   // 剰余
}

enum EffectCond {
    Leq(EffectNumber, EffectNumber),
    Geq(EffectNumber, EffectNumber),
    Eq(EffectNumber, EffectNumber),
    CondAnd(Vec<EffectCond>),
    CondOr(Vec<EffectCond>),
    CondNot(Box<EffectCond>),
}

// カードの働きを記述するためのメタ言語
enum CardEffect {
    Noop,
    Sequence(Vec<CardEffect>),
    AtomicSequence(Vec<CardEffect>), // 「不可能な指示は無視」ができない場合（改築の破棄→獲得など）に使う
    Optional(Box<CardEffect>),

    // Select系：カードを選択し、Focusの選択先を変更した上で、効果を適用する
    SelectExact(String, EffectNumber, CardSelector, Box<CardEffect>), // ちょうどn枚選択
    SelectUpto(String, EffectNumber, CardSelector, Box<CardEffect>),  // n枚以下選択
    SelectAny(String, CardSelector, Box<CardEffect>),                 // 好きなだけ選択

    // Select亜種だけどプレイヤーの選択を必要としない
    FocusAll(CardSelector, Box<CardEffect>),

    // デッキトップ公開・Focus
    RevealTop(EffectNumber, Box<CardEffect>),

    If(EffectCond, Box<CardEffect>),
    While(EffectCond, Box<CardEffect>),
    Until(EffectCond, Box<CardEffect>),

    UseCard(CardSelector),

    PlusDraw(EffectNumber),
    PlusAction(EffectNumber),
    PlusBuy(EffectNumber),
    PlusCoin(EffectNumber),

    TrashCard(CardSelector),
    DiscardCard(CardSelector),
    GainCard(CardNameSelector),
    GainCardToHand(CardNameSelector), // 職人はこっち

    MoveCard(CardSelector, Zone),

    AllOpponents(Box<CardEffect>),
    AttackAllOpponents(Box<CardEffect>),
    PreventDefault, // 「○○する代わりに」の、元の動作を無効化するやつ
}

enum Zone {
    // 実在のゾーン。配置対象としてもよい
    Deck,
    Hand,
    Discard,
    Play,
    Pending,

    // 以下は仮想的なゾーン
    DeckTop, // デッキの一番上。配置対象としてもよい
    AllMyCards,
    Focused,
    Itself,
}

enum CardNameSelector {
    Exact(String),
    NameAnd(Vec<CardNameSelector>),
    NameOr(Vec<CardNameSelector>),
    NameNot(Box<CardNameSelector>),
    HasType(CardType),
    CostLower(Box<EffectNumber>),
    CostHigher(Box<EffectNumber>),
}

enum CardSelector {
    ByName(CardNameSelector),
    ByZone(Zone),
    DeckTop(i32),
    DeckBottom(i32),
    CardAnd(Vec<CardSelector>),
    CardOr(Vec<CardSelector>),
}

mod card_util {

    use crate::{CardType::*, EffectNumber::*, EffectTrigger::*, *};

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
            cost,
            vp: Constant(0),
            rules: vec![(OnPlayAction, vanilla_effect(draw, action, buy, coin))],
            types: vec![Action],
        }
    }

    pub fn vanilla_treasure_card(name: &str, localized_name: &str, cost: i32, coin: i32) -> Card {
        Card {
            name: name.to_owned(),
            localized_name: localized_name.to_owned(),
            cost,
            vp: Constant(0),
            rules: vec![(OnPlayTreasure, vanilla_effect(0, 0, 0, coin))],
            types: vec![Treasure],
        }
    }

    pub fn vanilla_vp_card(name: &str, localized_name: &str, cost: i32, vp: i32) -> Card {
        Card {
            name: name.to_owned(),
            localized_name: localized_name.to_owned(),
            cost,
            vp: Constant(vp),
            rules: vec![],
            types: vec![Victory],
        }
    }
    pub fn vanilla_curse_card() -> Card {
        Card {
            name: "Curse".to_owned(),
            localized_name: "呪い".to_owned(),
            cost: 0,
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
            cost,
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
            vec![(OnPlayAction, effect)],
            if has_attack {
                vec![Action, Attack]
            } else {
                vec![Action]
            },
        )
    }

    pub fn focused() -> CardSelector {
        CardSelector::ByZone(Zone::Focused)
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
        use std::vec;

        use crate::{
            card_util::*, CardEffect::*, CardType::*, EffectCond::*, EffectNumber::*,
            EffectTrigger::*, *,
        };

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

        // 地下貯蔵庫 +1アクション、好きな枚数のカードを捨て札にし、同じ枚数のカードを引く。
        pub fn cellar() -> Card {
            simple_action_card(
                "Cellar",
                "地下貯蔵庫",
                2,
                false,
                Sequence(vec![
                    PlusAction(Constant(1)),
                    SelectAny(
                        "捨て札にするカードを選んでください".to_owned(),
                        CardSelector::ByZone(Zone::Hand),
                        Box::new(Sequence(vec![
                            PlusDraw(CountCard(focused())),
                            DiscardCard(focused()),
                        ])),
                    ),
                ]),
            )
        }

        // 礼拝堂 手札から最大4枚まで選んで廃棄する。
        pub fn chapel() -> Card {
            simple_action_card(
                "Chapel",
                "礼拝堂",
                2,
                false,
                Sequence(vec![SelectUpto(
                    "廃棄するカードを選んでください".to_owned(),
                    Constant(4),
                    CardSelector::ByZone(Zone::Hand),
                    Box::new(TrashCard(focused())),
                )]),
            )
        }

        // 堀 +2ドロー。他のプレイヤーがアタックカードをプレイしたとき、手札からこのカードを公開すると、そのアタックカードの効果を受けない。
        pub fn moat() -> Card {
            Card {
                name: "Moat".to_owned(),
                localized_name: "堀".to_owned(),
                cost: 2,
                vp: Constant(0),
                rules: vec![
                    (OnPlayAction, Sequence(vec![PlusDraw(Constant(2))])),
                    (OnAttack, Sequence(vec![PreventDefault])),
                ],
                types: vec![Action, Reaction],
            }
        }

        // 家臣 +2金、デッキの上から1枚を公開して、アクションカードだった場合、そのカードを使用してもよい。
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
                        Box::new(FocusAll(
                            // アクションだったら
                            CardSelector::CardAnd(vec![
                                CardSelector::ByZone(Zone::Focused),
                                CardSelector::ByName(CardNameSelector::HasType(Action)),
                            ]),
                            // 使ってもよい
                            Box::new(Optional(Box::new(Sequence(vec![
                                UseCard(focused()),
                                DiscardCard(focused()),
                            ])))),
                        )),
                    ),
                ]),
            )
        }

        // 工房 コスト4以下のカード1枚を獲得する。
        pub fn workshop() -> Card {
            simple_action_card(
                "Workshop",
                "工房",
                3,
                false,
                GainCard(CardNameSelector::CostLower(Box::new(Constant(4)))),
            )
        }

        // 商人 +1ドロー+1アクション。ターン中銀貨を始めて使った際、+1金。
        pub fn merchant() -> Card {
            simple_rule_card(
                "Merchant",
                "商人",
                3,
                vec![
                    (
                        OnPlayAction,
                        Sequence(vec![PlusDraw(Constant(1)), PlusAction(Constant(1))]),
                    ),
                    (
                        OnCardPlay,
                        Sequence(vec![If(
                            Eq(
                                CountCard(CardSelector::CardAnd(vec![
                                    CardSelector::ByZone(Zone::Play),
                                    CardSelector::ByName(CardNameSelector::Exact(
                                        "Silver".to_owned(),
                                    )),
                                ])),
                                Constant(1),
                            ),
                            Box::new(PlusCoin(Constant(1))),
                        )]),
                    ),
                ],
                vec![Action],
            )
        }

        // 前駆者 +1ドロー+1アクション、捨て札から好きなカード1枚をデッキトップに置く。
        pub fn harbinger() -> Card {
            simple_action_card(
                "Harbinger",
                "前駆者",
                3,
                false,
                Sequence(vec![
                    PlusDraw(Constant(1)),
                    PlusAction(Constant(1)),
                    SelectAny(
                        "デッキトップに置くカードを選んでください".to_owned(),
                        CardSelector::ByZone(Zone::Discard),
                        Box::new(MoveCard(focused(), Zone::DeckTop)),
                    ),
                ]),
            )
        }

        // 村 +1ドロー+2アクション
        pub fn village() -> Card {
            vanilla_action_card("Village", "村", 3, 1, 2, 0, 0)
        }

        // 改築 好きなカード1枚を廃棄し、そのコスト+2までのカード1枚を獲得する。
        pub fn remodel() -> Card {
            simple_action_card(
                "Remodel",
                "改築",
                4,
                false,
                Sequence(vec![SelectExact(
                    "改築するカードを選んでください".to_owned(),
                    Constant(1),
                    CardSelector::ByZone(Zone::Hand),
                    Box::new(AtomicSequence(vec![
                        TrashCard(focused()),
                        GainCard(CardNameSelector::CostLower(Box::new(Plus(
                            Box::new(CountCost(focused())),
                            Box::new(Constant(2)),
                        )))),
                    ])),
                )]),
            )
        }

        // 鍛冶屋 +3ドロー
        pub fn smithy() -> Card {
            vanilla_action_card("Smithy", "鍛冶屋", 4, 3, 0, 0, 0)
        }

        // 金貸し 銅貨を破棄してもよい、破棄した場合+3金。
        pub fn moneylender() -> Card {
            simple_action_card(
                "Moneylender",
                "金貸し",
                4,
                false,
                Optional(Box::new(SelectExact(
                    "破棄するカードを選んでください".to_owned(),
                    Constant(1),
                    CardSelector::CardAnd(vec![
                        CardSelector::ByZone(Zone::Hand),
                        CardSelector::ByName(CardNameSelector::Exact("Copper".to_owned())),
                    ]),
                    Box::new(AtomicSequence(vec![
                        TrashCard(focused()),
                        PlusCoin(Constant(3)),
                    ])),
                ))),
            )
        }

        // 玉座の間 好きなアクションカード1枚を2回使用する。
        pub fn throne_room() -> Card {
            simple_action_card(
                "Throne Room",
                "玉座の間",
                4,
                false,
                SelectExact(
                    "使用するカードを選んでください".to_owned(),
                    Constant(1),
                    CardSelector::CardAnd(vec![
                        CardSelector::ByZone(Zone::Hand),
                        CardSelector::ByName(CardNameSelector::HasType(Action)),
                    ]),
                    Box::new(Sequence(vec![UseCard(focused()), UseCard(focused())])),
                ),
            )
        }

        // 密猟者 +1ドロー+1アクション+1金、空の山の数だけ手札を捨てる。
        pub fn poacher() -> Card {
            simple_action_card(
                "Poacher",
                "密猟者",
                4,
                false,
                Sequence(vec![
                    PlusDraw(Constant(1)),
                    PlusAction(Constant(1)),
                    PlusCoin(Constant(1)),
                    SelectExact(
                        "捨てるカードを選んでください".to_owned(),
                        EmptyPiles,
                        CardSelector::ByZone(Zone::Hand),
                        Box::new(DiscardCard(focused())),
                    ),
                ]),
            )
        }

        // 民兵 +2金、他のプレイヤーは全員、手札が3枚以下になるまで（手札の枚数-3枚）捨て札にする。
        pub fn militia() -> Card {
            simple_action_card(
                "Militia",
                "民兵",
                4,
                true,
                Sequence(vec![
                    PlusCoin(Constant(2)),
                    AttackAllOpponents(Box::new(If(
                        Geq(CountCard(CardSelector::ByZone(Zone::Hand)), Constant(4)),
                        Box::new(SelectExact(
                            "捨てるカードを選んでください".to_owned(),
                            Minus(
                                Box::new(CountCard(CardSelector::ByZone(Zone::Hand))),
                                Box::new(Constant(3)),
                            ),
                            CardSelector::ByZone(Zone::Hand),
                            Box::new(DiscardCard(focused())),
                        )),
                    ))),
                ]),
            )
        }

        // 役人 +2金、対戦相手は手札に勝利点カードがあれば1枚選んでデッキトップに置く。
        pub fn bureaucrat() -> Card {
            simple_action_card(
                "Bureaucrat",
                "役人",
                4,
                true,
                Sequence(vec![
                    PlusCoin(Constant(2)),
                    AttackAllOpponents(Box::new(SelectExact(
                        "デッキトップに置くカードを選んでください".to_owned(),
                        Constant(1),
                        CardSelector::CardAnd(vec![
                            CardSelector::ByZone(Zone::Hand),
                            CardSelector::ByName(CardNameSelector::HasType(Victory)),
                        ]),
                        Box::new(MoveCard(focused(), Zone::DeckTop)),
                    ))),
                ]),
            )
        }

        // 庭園 所有カード10枚につき1VP
        pub fn gardens() -> Card {
            Card {
                name: "Gardens".to_owned(),
                localized_name: "庭園".to_owned(),
                cost: 4,
                vp: Div(
                    Box::new(CountCard(CardSelector::ByZone(Zone::AllMyCards))),
                    Box::new(Constant(10)),
                ),
                rules: vec![],
                types: vec![Victory],
            }
        }

        // 市場 +1ドロー+1アクション+1金+1購入
        pub fn market() -> Card {
            vanilla_action_card("Market", "市場", 5, 1, 1, 1, 1)
        }

        // 衛兵 +1ドロー+1アクション、デッキトップから2枚見て、破棄・捨て・戻すを選ぶ。
        pub fn sentry() -> Card {
            simple_action_card(
                "Sentry",
                "衛兵",
                5,
                false,
                Sequence(vec![
                    PlusDraw(Constant(1)),
                    PlusAction(Constant(1)),
                    RevealTop(
                        Constant(2),
                        Box::new(Sequence(vec![
                            MoveCard(focused(), Zone::Pending),
                            SelectAny(
                                "破棄するカードを選んでください".to_owned(),
                                CardSelector::ByZone(Zone::Pending),
                                Box::new(TrashCard(focused())),
                            ),
                            SelectAny(
                                "捨てるカードを選んでください".to_owned(),
                                CardSelector::ByZone(Zone::Pending),
                                Box::new(DiscardCard(focused())),
                            ),
                            MoveCard(CardSelector::ByZone(Zone::Pending), Zone::DeckTop),
                        ])),
                    ),
                ]),
            )
        }

        // 議事堂 +4ドロー+1アクション+1購入、対戦相手は+1ドロー
        pub fn council_room() -> Card {
            simple_action_card(
                "Council Room",
                "議事堂",
                5,
                true,
                Sequence(vec![
                    PlusDraw(Constant(4)),
                    PlusAction(Constant(1)),
                    PlusBuy(Constant(1)),
                    AllOpponents(Box::new(PlusDraw(Constant(1)))),
                ]),
            )
        }

        // 研究所 +2ドロー+1アクション
        pub fn laboratory() -> Card {
            vanilla_action_card("Laboratory", "研究所", 5, 2, 1, 0, 0)
        }

        // 鉱山 手札の財宝を1枚破棄してもよい、破棄した場合最大3コスト多い財宝を獲得する。
        pub fn mine() -> Card {
            simple_action_card(
                "Mine",
                "鉱山",
                5,
                false,
                Optional(Box::new(SelectExact(
                    "破棄するカードを選んでください".to_owned(),
                    Constant(1),
                    CardSelector::CardAnd(vec![
                        CardSelector::ByZone(Zone::Hand),
                        CardSelector::ByName(CardNameSelector::HasType(Treasure)),
                    ]),
                    Box::new(AtomicSequence(vec![
                        TrashCard(focused()),
                        GainCard(CardNameSelector::NameAnd(vec![
                            CardNameSelector::HasType(Treasure),
                            CardNameSelector::CostLower(Box::new(Plus(
                                Box::new(CountCost(focused())),
                                Box::new(Constant(3)),
                            ))),
                        ])),
                    ])),
                ))),
            )
        }

        // 祝祭 +2アクション+1購入+2金
        pub fn festival() -> Card {
            vanilla_action_card("Festival", "祝祭", 5, 0, 2, 1, 2)
        }

        // 書庫 手札が7枚になるまで、「デッキトップをめくり、アクションでなければ加える、アクションであれば加えるか脇に避けるか選ぶ」を繰り返し、脇に避けたカードを捨て札にする。
        pub fn library() -> Card {
            simple_action_card(
                "Library",
                "書庫",
                5,
                false,
                Sequence(vec![
                    Until(
                        CondOr(vec![
                            // 手札==7 or 手札+捨て札+デッキ <= 6
                            Eq(CountCard(CardSelector::ByZone(Zone::Hand)), Constant(7)),
                            Leq(
                                Plus(
                                    Box::new(CountCard(CardSelector::ByZone(Zone::Hand))),
                                    Box::new(Plus(
                                        Box::new(CountCard(CardSelector::ByZone(Zone::Discard))),
                                        Box::new(CountCard(CardSelector::ByZone(Zone::Deck))),
                                    )),
                                ),
                                Constant(6),
                            ),
                        ]),
                        Box::new(Sequence(vec![RevealTop(
                            Constant(1),
                            Box::new(Sequence(vec![
                                If(
                                    Eq(
                                        CountCard(CardSelector::ByName(CardNameSelector::HasType(
                                            Action,
                                        ))),
                                        Constant(1),
                                    ),
                                    Box::new(SelectAny(
                                        "加えるか脇に避けるか選んでください".to_owned(),
                                        CardSelector::ByZone(Zone::DeckTop),
                                        Box::new(Sequence(vec![MoveCard(
                                            focused(),
                                            Zone::Pending,
                                        )])),
                                    )),
                                ),
                                PlusDraw(Constant(1)),
                            ])),
                        )])),
                    ),
                    DiscardCard(CardSelector::ByZone(Zone::Pending)),
                ]),
            )
        }

        // 山賊 金貨を得る、他のプレイヤーは全員デッキトップ2枚を公開し、財宝を1枚選んで破棄する
        pub fn bandit() -> Card {
            simple_action_card(
                "Bandit",
                "山賊",
                5,
                true,
                Sequence(vec![
                    GainCard(CardNameSelector::Exact("Gold".to_owned())),
                    AttackAllOpponents(Box::new(RevealTop(
                        Constant(2),
                        Box::new(SelectExact(
                            "破棄するカードを選んでください".to_owned(),
                            Constant(1),
                            CardSelector::CardAnd(vec![
                                CardSelector::ByZone(Zone::Focused),
                                CardSelector::ByName(CardNameSelector::HasType(Treasure)),
                            ]),
                            Box::new(TrashCard(focused())),
                        )),
                    ))),
                ]),
            )
        }

        // 魔女 +2ドロー、他のプレイヤーは全員呪いを1枚引く
        pub fn witch() -> Card {
            simple_action_card(
                "Witch",
                "魔女",
                5,
                true,
                Sequence(vec![
                    PlusDraw(Constant(2)),
                    AttackAllOpponents(Box::new(GainCard(CardNameSelector::Exact(
                        "Curse".to_owned(),
                    )))),
                ]),
            )
        }

        // 職人 5コスト以下のカード1枚を手札に獲得し、手札から1枚デッキトップに置く。
        pub fn artisan() -> Card {
            simple_action_card(
                "Artisan",
                "職人",
                6,
                false,
                Sequence(vec![
                    GainCardToHand(CardNameSelector::CostLower(Box::new(Constant(5)))),
                    SelectExact(
                        "デッキトップに置くカードを選んでください".to_owned(),
                        Constant(1),
                        CardSelector::ByZone(Zone::Hand),
                        Box::new(MoveCard(focused(), Zone::DeckTop)),
                    ),
                ]),
            )
        }
    }
}
