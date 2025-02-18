use std::collections::{vec_deque, VecDeque};

fn main() {
    println!("Hello, world!");
}

// Dominion simulator

#[derive(Clone)]
struct Player<'a> {
    name: String,
    deck: Vec<CardInstance<'a>>,
    hand: Vec<CardInstance<'a>>,
    play: Vec<CardInstance<'a>>,
    pending: Vec<CardInstance<'a>>,
    discard: Vec<CardInstance<'a>>,
    id: PlayerId,
}

#[derive(Clone, Copy)]
struct PlayerId {
    id: i32,
}

struct Card {
    name: String,
    localized_name: String,
    cost: i32,
    vp: Number,
    rules: Vec<(EffectTrigger, CardEffect)>,
    types: Vec<CardType>,
}

#[derive(Clone)]
struct CardInstance<'a> {
    card: &'a Card,
    id: CardId,
}

#[derive(Clone, Copy)]
struct CardId {
    id: i32,
}

#[derive(Clone)]
enum CardType {
    Action,
    Treasure,
    Victory,
    Reaction,
    Curse,
    Attack,
}

#[derive(Clone)]
enum EffectTrigger {
    Attacked,       // Focus: 空, PreventDefaultで攻撃を無効化
    PlayAsAction,   // Focus: 空
    PlayAsTreasure, // Focus: 空
    CardPlayed,     // Focus: カード
    Cleanup,        // Focus: 空, PreventDefaultで場にあっても捨て札にしない
    MyTurnStart,
    MyTurnEnd,
    OncePerTurn(Box<EffectTrigger>),
}

#[derive(Clone)]
enum Number {
    Constant(i32),
    CountCard(CardSelector),
    CountCost(CardSelector),
    CountEmptyPiles,
    UpTo(Box<Number>),
    Plus(Box<Number>, Box<Number>),
    Minus(Box<Number>, Box<Number>),
    Times(Box<Number>, Box<Number>), // 乗算
    Div(Box<Number>, Box<Number>),   // 整数除算、切り捨て
    Mod(Box<Number>, Box<Number>),   // 剰余
}

#[derive(Clone)]
enum EffectCond {
    Leq(Number, Number),
    Geq(Number, Number),
    Eq(Number, Number),
    CondAnd(Vec<EffectCond>),
    CondOr(Vec<EffectCond>),
    CondNot(Box<EffectCond>),
}

// カードの働きを記述するためのメタ言語
enum CardEffect {
    Noop,
    Sequence(Vec<CardEffect>),
    AtomicSequence(Vec<CardEffect>), // 「不可能な指示は無視」ができない場合（改築の破棄→獲得など）に使う。SkipContinueを伝播
    Optional(AskOptionTag, Box<CardEffect>),

    // Select系：カードを選択し、Focusの選択先を変更した上で、効果を適用する
    Select(AskCardTag, Number, CardSelector, Box<CardEffect>), // n枚選択
    SelectAny(AskCardTag, CardSelector, Box<CardEffect>),      // 好きなだけ選択

    // Select亜種 該当カードすべてを選択、プレイヤーの選択を必要としない
    FocusAll(CardSelector, Box<CardEffect>),

    TrashSelect(Number, CardSelector, Box<CardEffect>), // Select亜種 手札から廃棄
    DiscardSelect(Number, CardSelector, Box<CardEffect>), // Select亜種 手札を捨てる

    // デッキトップ公開・Focus
    RevealTop(Number, Box<CardEffect>),

    If(EffectCond, Box<CardEffect>),
    While(EffectCond, Box<CardEffect>),
    Until(EffectCond, Box<CardEffect>),

    UseCard(CardSelector),

    PlusDraw(Number),
    PlusAction(Number),
    PlusBuy(Number),
    PlusCoin(Number),

    TrashCard(CardSelector),
    DiscardCard(CardSelector),
    GainCard(CardNameSelector),
    GainCardToHand(CardNameSelector), // 職人はこっち

    MoveCard(CardSelector, Zone),

    AllOpponents(Box<CardEffect>),
    AttackAllOpponents(Box<CardEffect>),
    PreventDefault, // 「○○する代わりに」の、元の動作を無効化するやつ
}

#[derive(Clone)]
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

#[derive(Clone)]
enum CardNameSelector {
    Exact(String),
    NameAnd(Vec<CardNameSelector>),
    NameOr(Vec<CardNameSelector>),
    NameNot(Box<CardNameSelector>),
    HasType(CardType),
    Cost(Box<Number>),
    Any,
}

#[derive(Clone)]
struct CardSelector {
    name: CardNameSelector,
    zone: Vec<Zone>,
}

#[derive(Clone)]
enum TurnPhase {
    Action,
    Buy,
    Cleanup,
}

#[derive(Clone)]
struct Game<'a> {
    players: Vec<Player<'a>>,
    supply: Vec<(&'a Card, i32)>,
    trash: Vec<CardInstance<'a>>,
    turn: i32,
    stack: Vec<EffectStackFrame<'a>>,
}

#[derive(Clone)]
struct EffectStackFrame<'a> {
    player: &'a Player<'a>,
    target: &'a Player<'a>,
    effect_queue: VecDeque<&'a CardEffect>,
    focus: Vec<&'a CardInstance<'a>>,
    cause: Option<&'a CardInstance<'a>>,
    atomic: bool,
}

enum EffectStepResult<'a> {
    Continue,
    Error(String),
    AskCard(PlayerId, &'a AskCardTag, Vec<&'a CardInstance<'a>>), // 次のStepはFocusした状態で
    AskOptional(PlayerId, &'a AskOptionTag), // 答えがNoだったらそのスタックフレームをスキップ
    SkipContinue,                            // 不可能な指示なので飛ばす
    End,
}

#[derive(Clone)]
struct AskOptionTag {
    tag: String,
    localized_prompt: String,
    default: Option<bool>,
}

#[derive(Clone)]
struct AskCardTag {
    tag: String,
    localized_prompt: String,
}

mod player_util {
    use rand::Rng;

    use crate::{CardInstance, Player};

    fn shuffle(player: &mut Player) {
        // Fisher-Yates shuffle
        for i in (1..player.deck.len()).rev() {
            let j = rand::rng().random_range(0..=i);
            player.deck.swap(i, j);
        }
    }

    fn reshuffle(player: &mut Player) {
        player.deck.extend(player.discard.drain(..));
        shuffle(player);
    }

    fn draw(player: &mut Player, n: i32) {
        for _ in 0..n {
            if player.deck.is_empty() {
                reshuffle(player);
            }
            if player.deck.is_empty() {
                return;
            }
            player.hand.push(player.deck.pop().unwrap());
        }
    }
}
impl Game<'_> {
    fn resolve_selector<'a>(
        &self,
        target: &'a Player,
        selector: &'a CardSelector,
    ) -> Vec<&'a CardInstance<'a>> {
        use {CardSelector, Zone::*};
        panic!()
    }

    fn step(&self) -> (Game, EffectStepResult) {
        use {CardEffect::*, EffectStepResult::*, Zone::*};

        let mut game = self.clone();

        let Some(mut frame) = game.stack.pop() else {
            return (game, EffectStepResult::End);
        };

        let Some(effect) = frame.effect_queue.clone().pop_front() else {
            game.stack.pop();
            return (game, EffectStepResult::Continue);
        };

        let result = match effect {
            Noop => Continue,
            Sequence(effects) => {
                frame.effect_queue.extend(effects.into_iter());
                Continue
            }
            AtomicSequence(effects) => {
                let newframe = EffectStackFrame {
                    player: frame.player,
                    target: frame.target,
                    effect_queue: effects.into_iter().collect(),
                    focus: frame.focus.clone(),
                    cause: frame.cause,
                    atomic: true,
                };
                game.stack.push(frame);
                game.stack.push(newframe);
                return (game, Continue);
            }
            Optional(prompt, effect) => {
                let targetid = frame.target.id;
                let newframe = EffectStackFrame {
                    player: frame.player,
                    target: frame.target,
                    effect_queue: VecDeque::from(vec![&**effect]),
                    focus: frame.focus.clone(),
                    cause: frame.cause,
                    atomic: true,
                };
                game.stack.push(frame);
                game.stack.push(newframe);
                return (game, AskOptional(targetid, prompt));
            }
            Select(prompt, n, selector, effect) => {
                let targetid = frame.target.id;
                let newframe = EffectStackFrame {
                    player: frame.player,
                    target: frame.target,
                    effect_queue: VecDeque::from(vec![&**effect]),
                    focus: frame.focus.clone(),
                    cause: frame.cause,
                    atomic: true,
                };
                game.stack.push(frame);
                game.stack.push(newframe);
                return (game, AskCard(targetid, prompt, vec![]));
            }
            _ => SkipContinue,
        };

        game.stack.push(frame);
        (game, result)
    }
}

mod card_util {

    use crate::{CardNameSelector::Any, CardType::*, EffectTrigger::*, Number::*, *};

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
            rules: vec![(PlayAsAction, vanilla_effect(draw, action, buy, coin))],
            types: vec![Action],
        }
    }

    pub fn vanilla_treasure_card(name: &str, localized_name: &str, cost: i32, coin: i32) -> Card {
        Card {
            name: name.to_owned(),
            localized_name: localized_name.to_owned(),
            cost,
            vp: Constant(0),
            rules: vec![(PlayAsTreasure, vanilla_effect(0, 0, 0, coin))],
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

    pub fn discard() -> CardSelector {
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

    pub fn upto(n: i32) -> Number {
        Number::UpTo(Box::new(Constant(n)))
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
            card_util::*, CardEffect::*, CardNameSelector::Any, CardType::*, EffectCond::*,
            EffectTrigger::*, Number::*, *,
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
                        AskCardTag {
                            tag: "cellar".to_owned(),
                            localized_prompt: "捨て札にするカードを選んでください".to_owned(),
                        },
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
                TrashSelect(upto(4), hand(), Box::new(Noop)),
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
                                AskOptionTag {
                                    tag: "chancellor".to_owned(),
                                    localized_prompt: "このカードを使用しますか？".to_owned(),
                                    default: Some(true),
                                },
                                Box::new(Sequence(vec![
                                    UseCard(focused()),
                                    DiscardCard(focused()),
                                ])),
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
                GainCard(CardNameSelector::Cost(Box::new(upto(4)))),
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
                                    name: CardNameSelector::Exact("Silver".to_owned()),
                                    zone: vec![Zone::Focused],
                                }),
                                CountCard(CardSelector {
                                    name: CardNameSelector::Exact("Silver".to_owned()),
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
                    SelectAny(
                        AskCardTag {
                            tag: "harbinger".to_owned(),
                            localized_prompt: "デッキトップに置くカードを選んでください".to_owned(),
                        },
                        discard(),
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
                    Constant(1),
                    hand(),
                    Box::new(GainCard(CardNameSelector::Cost(Box::new(Plus(
                        Box::new(CountCost(focused())),
                        Box::new(Constant(2)),
                    ))))),
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
                    AskOptionTag {
                        tag: "moneylender".to_owned(),
                        localized_prompt: "銅貨を破棄しますか？".to_owned(),
                        default: Some(true),
                    },
                    Box::new(TrashSelect(
                        Constant(1),
                        CardSelector {
                            name: CardNameSelector::Exact("Copper".to_owned()),
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
                    AskCardTag {
                        tag: "throne_room".to_owned(),
                        localized_prompt: "使用するカードを選んでください".to_owned(),
                    },
                    Constant(1),
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
                    DiscardSelect(CountEmptyPiles, hand(), Box::new(DiscardCard(focused()))),
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
                            Minus(Box::new(CountCard(hand())), Box::new(Constant(3))),
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
                        AskCardTag {
                            tag: "bureaucrat".to_owned(),
                            localized_prompt: "デッキトップに置く勝利点カードを選んでください"
                                .to_owned(),
                        },
                        Constant(1),
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
                cost: 4,
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
                    RevealTop(
                        Constant(2),
                        Box::new(Sequence(vec![
                            MoveCard(focused(), Zone::Pending),
                            SelectAny(
                                AskCardTag {
                                    tag: "sentry_trash".to_owned(),
                                    localized_prompt: "破棄するカードを選択してください".to_owned(),
                                },
                                in_zone(Zone::Pending),
                                Box::new(TrashCard(focused())),
                            ),
                            SelectAny(
                                AskCardTag {
                                    tag: "sentry_discard".to_owned(),
                                    localized_prompt: "捨て札にするカードを選択してください"
                                        .to_owned(),
                                },
                                in_zone(Zone::Pending),
                                Box::new(DiscardCard(focused())),
                            ),
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
                    AskOptionTag {
                        tag: "mine".to_owned(),
                        localized_prompt: "財宝を破棄しますか？".to_owned(),
                        default: Some(true),
                    },
                    Box::new(TrashSelect(
                        Constant(1),
                        CardSelector {
                            name: CardNameSelector::HasType(Treasure),
                            zone: vec![Zone::Hand],
                        },
                        Box::new(AtomicSequence(vec![
                            TrashCard(focused()),
                            GainCard(CardNameSelector::NameAnd(vec![
                                CardNameSelector::HasType(Treasure),
                                CardNameSelector::Cost(Box::new(UpTo(Box::new(Plus(
                                    Box::new(CountCost(focused())),
                                    Box::new(Constant(3)),
                                ))))),
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
                        CondOr(vec![
                            // 手札==7 or 手札+捨て札+デッキ <= 6
                            Eq(CountCard(hand()), Constant(7)),
                            Leq(
                                CountCard(CardSelector {
                                    name: Any,
                                    zone: vec![Zone::Discard, Zone::Deck, Zone::Hand],
                                }),
                                Constant(6),
                            ),
                        ]),
                        Box::new(Sequence(vec![RevealTop(
                            Constant(1),
                            Box::new(Sequence(vec![
                                If(
                                    Eq(
                                        CountCard(CardSelector {
                                            name: CardNameSelector::HasType(Action),
                                            zone: vec![Zone::Focused],
                                        }),
                                        Constant(1),
                                    ),
                                    Box::new(SelectAny(
                                        AskCardTag {
                                            tag: "library".to_owned(),
                                            localized_prompt: "このカードを脇に避けますか？"
                                                .to_owned(),
                                        },
                                        focused(),
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
                    DiscardCard(in_zone(Zone::Pending)),
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
                        Box::new(TrashSelect(
                            Constant(1),
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
                    GainCardToHand(CardNameSelector::Cost(Box::new(upto(5)))),
                    Select(
                        AskCardTag {
                            tag: "artisan".to_owned(),
                            localized_prompt: "デッキトップに置くカードを選んでください".to_owned(),
                        },
                        Constant(1),
                        hand(),
                        Box::new(MoveCard(focused(), Zone::DeckTop)),
                    ),
                ]),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    mod selector_resolver {
        use crate::{
            card_util::*, expansions::basic_supply::*, CardId, CardNameSelector, CardSelector,
            CardSelector, Game, Player, PlayerId, Zone::*,
        };

        fn setup<'a>() -> Game<'a> {
            let p0 = Player {
                id: PlayerId { id: 0 },
                name: "Alice".to_owned(),
                deck: vec![],
                hand: vec![],
                play: vec![],
                pending: vec![],
                discard: vec![],
            };

            let p1 = Player {
                id: PlayerId { id: 1 },
                name: "Bob".to_owned(),
                deck: vec![],
                hand: vec![],
                play: vec![],
                pending: vec![],
                discard: vec![],
            };

            Game {
                players: vec![p0, p1],
                supply: vec![],
                trash: vec![],
                turn: 0,
                stack: vec![],
            }
        }

        #[test]
        fn byname_exact() {
            let mut game = setup();
            let mut alice = &mut game.players[0];
            alice.hand.push(CardInstance {
                id: CardId { id: 0 },
                card: copper(),
            });
            alice.hand.push(CardInstance {
                id: CardId { id: 1 },
                card: copper(),
            });
            alice.deck.push(CardInstance {
                id: CardId { id: 2 },
                card: copper(),
            });
            let selector = CardSelector {
                name: CardNameSelector::Exact("Copper".to_owned()),
                zone: vec![Hand],
            };
            let result = game.resolve_selector(alice, &selector);
            assert_eq!(result.len(), 2);
        }
    }
}
