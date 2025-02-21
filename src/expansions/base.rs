use std::collections::HashMap;
use std::vec;

use crate::{
    ask_tag::{AskCardTag, AskOptionTag},
    card::{Card, CardType::*},
    card_util::*,
    effect::{CardEffect::*, EffectCond::*, EffectTrigger::*},
    number::{Number::*, NumberRange::*},
    selector::{
        CardNameSelector::{self, *},
        CardSelector,
    },
    zone::Zone,
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
            Select(
                AskCardTag::new("cellar", "捨て札にするカードを選んでください"),
                AnyNumber,
                hand(),
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
        TrashSelect(UpTo(Constant(4)), hand(), Box::new(Noop)),
    )
}

// 堀 +2ドロー。他のプレイヤーがアタックカードをプレイしたとき、手札からこのカードを公開すると、そのアタックカードの効果を受けない。
pub fn moat() -> Card {
    Card {
        name: "Moat".to_owned(),
        localized_name: "堀".to_owned(),
        cost: Constant(2),
        vp: Constant(0),
        rules: vec![
            (PlayAsAction, Sequence(vec![PlusDraw(Constant(2))])),
            (Attacked, Sequence(vec![PreventDefault])),
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
                    CardSelector {
                        name: CardNameSelector::HasType(Action),
                        zone: vec![Zone::Focused],
                    },
                    // 使ってもよい
                    Box::new(Optional(
                        AskOptionTag::new("chancellor", "このカードを使用しますか？", Some(true)),
                        Box::new(Sequence(vec![UseCard(focused()), DiscardCard(focused())])),
                    )),
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
        GainCard(CardNameSelector::Cost(Box::new(UpTo(Constant(4))))),
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
                PlayAsAction,
                Sequence(vec![PlusDraw(Constant(1)), PlusAction(Constant(1))]),
            ),
            (
                // 直訳：カードがプレイされたとき、今プレイされた銀貨の枚数が場の銀貨の枚数と等しい場合（意訳：このターンはじめて銀貨がプレイされたなら）、+1金
                CardPlayed,
                Sequence(vec![If(
                    Eq(
                        CountCard(CardSelector {
                            name: CardNameSelector::Name("Silver".to_owned()),
                            zone: vec![Zone::Focused],
                        }),
                        CountCard(CardSelector {
                            name: CardNameSelector::Name("Silver".to_owned()),
                            zone: vec![Zone::Play],
                        }),
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
            Select(
                AskCardTag::new("harbinger", "デッキトップに置くカードを選んでください"),
                Exact(Constant(1)),
                discarded(),
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
        TrashSelect(
            Exact(Constant(1)),
            hand(),
            Box::new(GainCard(CardNameSelector::Cost(Box::new(UpTo(Plus(
                Box::new(CountCost(focused())),
                Box::new(Constant(2)),
            )))))),
        ),
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
        Optional(
            AskOptionTag::new("moneylender", "銅貨を破棄しますか？", Some(true)),
            Box::new(TrashSelect(
                Exact(Constant(1)),
                CardSelector {
                    name: CardNameSelector::Name("Copper".to_owned()),
                    zone: vec![Zone::Hand],
                },
                Box::new(AtomicSequence(vec![
                    TrashCard(focused()),
                    PlusCoin(Constant(3)),
                ])),
            )),
        ),
    )
}

// 玉座の間 好きなアクションカード1枚を2回使用する。
pub fn throne_room() -> Card {
    simple_action_card(
        "Throne Room",
        "玉座の間",
        4,
        false,
        Select(
            AskCardTag::new("throne_room", "使用するカードを選んでください"),
            Exact(Constant(1)),
            CardSelector {
                name: CardNameSelector::HasType(Action),
                zone: vec![Zone::Hand],
            },
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
            DiscardSelect(
                Exact(CountEmptyPiles),
                hand(),
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
                Geq(CountCard(hand()), Constant(4)),
                Box::new(DiscardSelect(
                    Exact(Minus(Box::new(CountCard(hand())), Box::new(Constant(3)))),
                    hand(),
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
            AttackAllOpponents(Box::new(Select(
                AskCardTag::new(
                    "bureaucrat",
                    "デッキトップに置く勝利点カードを選んでください",
                ),
                Exact(Constant(1)),
                CardSelector {
                    name: CardNameSelector::HasType(Victory),
                    zone: vec![Zone::Hand],
                },
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
        cost: Constant(4),
        vp: Div(Box::new(CountCard(all_my_cards())), Box::new(Constant(10))),
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
            LookAtTop(
                Constant(2),
                Box::new(Sequence(vec![
                    MoveCard(focused(), Zone::Pending),
                    TrashSelect(AnyNumber, in_zone(Zone::Pending), Box::new(Noop)),
                    DiscardSelect(AnyNumber, in_zone(Zone::Pending), Box::new(Noop)),
                    MoveCard(in_zone(Zone::Pending), Zone::DeckTop),
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
        Optional(
            AskOptionTag::new("mine", "財宝を破棄しますか？", Some(true)),
            Box::new(TrashSelect(
                Exact(Constant(1)),
                CardSelector {
                    name: CardNameSelector::HasType(Treasure),
                    zone: vec![Zone::Hand],
                },
                Box::new(AtomicSequence(vec![
                    TrashCard(focused()),
                    GainCard(CardNameSelector::NameAnd(vec![
                        CardNameSelector::HasType(Treasure),
                        CardNameSelector::Cost(Box::new(UpTo(Plus(
                            Box::new(CountCost(focused())),
                            Box::new(Constant(3)),
                        )))),
                    ])),
                ])),
            )),
        ),
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
                // 手札が7枚以上か、捨て札+デッキが0枚以下になるまで以下を繰り返す
                CondOr(vec![
                    // 手札==7 or (捨て札+デッキ==0)
                    Geq(CountCard(hand()), Constant(7)),
                    Leq(
                        CountCard(CardSelector {
                            name: Any,
                            zone: vec![Zone::Discard, Zone::Deck],
                        }),
                        Constant(0),
                    ),
                ]),
                // デッキトップを1枚見て、
                Box::new(Sequence(vec![LookAtTop(
                    Constant(1),
                    Box::new(Sequence(vec![
                        MoveCard(focused(), Zone::Pending), // 処理中ゾーンに移動する
                        If(
                            // もし処理中ゾーンにアクションカードがあれば、
                            Eq(
                                CountCard(CardSelector {
                                    name: CardNameSelector::HasType(Action),
                                    zone: vec![Zone::Pending],
                                }),
                                Constant(1),
                            ),
                            Box::new(Select(
                                // 好きなだけ脇に避けてもよい
                                AskCardTag::new("library", "このカードを脇に避けますか？"),
                                AnyNumber,
                                focused(),
                                Box::new(Sequence(vec![MoveCard(
                                    in_zone(Zone::Pending),
                                    Zone::Aside,
                                )])),
                            )),
                        ),
                        // その後、処理中ゾーンに残っているカードをドローした扱いで手札に加える
                        DrawFrom(CardSelector {
                            name: Any,
                            zone: vec![Zone::Pending],
                        }),
                    ])),
                )])),
            ),
            // 最後に、脇に避けたカードをすべて捨て札にする
            DiscardCard(in_zone(Zone::Aside)),
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
            GainCard(CardNameSelector::Name("Gold".to_owned())),
            AttackAllOpponents(Box::new(RevealTop(
                Constant(2),
                Box::new(TrashSelect(
                    Exact(Constant(1)),
                    CardSelector {
                        name: CardNameSelector::HasType(Treasure),
                        zone: vec![Zone::Focused],
                    },
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
            AttackAllOpponents(Box::new(GainCard(CardNameSelector::Name(
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
            GainCardToHand(CardNameSelector::Cost(Box::new(UpTo(Constant(5))))),
            Select(
                AskCardTag::new("artisan", "デッキトップに置くカードを選んでください"),
                Exact(Constant(1)),
                hand(),
                Box::new(MoveCard(focused(), Zone::DeckTop)),
            ),
        ]),
    )
}

pub fn base_set() -> HashMap<String, Card> {
    vec![
        cellar(),
        chapel(),
        moat(),
        chancellor(),
        workshop(),
        merchant(),
        harbinger(),
        village(),
        remodel(),
        smithy(),
        moneylender(),
        throne_room(),
        poacher(),
        militia(),
        bureaucrat(),
        gardens(),
        market(),
        sentry(),
        council_room(),
        laboratory(),
        mine(),
        festival(),
        library(),
        bandit(),
        witch(),
        artisan(),
    ]
    .into_iter()
    .map(|c| (c.name.clone(), c))
    .collect()
}
