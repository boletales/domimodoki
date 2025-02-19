use core::panic;
use std::collections::{vec_deque, VecDeque};

fn main() {
    println!("Hello, world!");
}

// Dominion simulator

#[derive(Clone)]
#[allow(dead_code)]
struct Player<'a> {
    name: String,
    deck: Vec<CardInstance<'a>>,
    hand: Vec<CardInstance<'a>>,
    play: Vec<CardInstance<'a>>,
    pending: Vec<CardInstance<'a>>,
    revealed: Vec<CardInstance<'a>>,
    discard: Vec<CardInstance<'a>>,
    id: PlayerId,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct PlayerId {
    id: usize,
}

#[allow(dead_code)]
struct Card {
    name: String,
    localized_name: String,
    cost: Number,
    vp: Number,
    rules: Vec<(EffectTrigger, CardEffect)>,
    types: Vec<CardType>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct CardInstance<'a> {
    card: &'a Card,
    id: CardId,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct CardId {
    id: usize,
}

#[derive(Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum CardType {
    Action,
    Treasure,
    Victory,
    Reaction,
    Curse,
    Attack,
}

#[derive(Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
enum Number {
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
enum NumberRange<N> {
    Exact(N),
    UpTo(N),
    AtLeast(N),
    Range(N, N),
    AnyNumber,
}

#[derive(Clone)]
#[allow(dead_code)]
enum EffectCond {
    Leq(Number, Number),
    Geq(Number, Number),
    Eq(Number, Number),
    CondAnd(Vec<EffectCond>),
    CondOr(Vec<EffectCond>),
    CondNot(Box<EffectCond>),
}

// カードの働きを記述するためのメタ言語
#[derive(Clone)]
#[allow(dead_code)]
enum CardEffect {
    Noop,
    Sequence(Vec<CardEffect>),
    AtomicSequence(Vec<CardEffect>), // 「不可能な指示は無視」ができない場合（改築の破棄→獲得など）に使う。SkipContinueを伝播
    Optional(AskOptionTag, Box<CardEffect>),

    // Select系：カードを選択し、Focusの選択先を変更した上で、効果を適用する
    Select(
        AskCardTag,
        NumberRange<Number>,
        CardSelector,
        Box<CardEffect>,
    ), // n枚選択

    // Select亜種 該当カードすべてを選択、プレイヤーの選択を必要としない
    FocusAll(CardSelector, Box<CardEffect>),

    TrashSelect(NumberRange<Number>, CardSelector, Box<CardEffect>), // Select亜種 手札から廃棄

    DiscardSelect(NumberRange<Number>, CardSelector, Box<CardEffect>), // Select亜種 手札を捨てるs

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
#[allow(dead_code)]
enum Zone {
    // 実在のゾーン。配置対象としてもよい
    Deck,
    Hand,
    Discard,
    Play,
    Pending,
    Revealed,

    // 以下は仮想的なゾーン
    DeckTop, // デッキの一番上。配置対象としてもよい
    AllMyCards,
    Focused,
    Itself,
}

#[derive(Clone)]
#[allow(dead_code)]
enum CardNameSelector {
    Name(String),
    NameAnd(Vec<CardNameSelector>),
    NameOr(Vec<CardNameSelector>),
    NameNot(Box<CardNameSelector>),
    HasType(CardType),
    Cost(Box<Number>),
    CostUpTo(Box<Number>),
    Any,
}

#[derive(Clone)]
#[allow(dead_code)]
struct CardSelector {
    name: CardNameSelector,
    zone: Vec<Zone>,
}

#[derive(Clone)]
#[allow(dead_code)]
enum TurnPhase {
    Action,
    Buy,
    Cleanup,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Game<'a> {
    players: Vec<Player<'a>>,
    supply: Vec<Vec<(CardInstance<'a>, i32)>>,
    trash: Vec<CardInstance<'a>>,
    turn: i32,
    stack: Vec<EffectStackFrame<'a>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct EffectStackFrame<'a> {
    player: &'a Player<'a>,
    target: &'a Player<'a>,
    effect_queue: VecDeque<CardEffect>,
    focus: Vec<&'a CardInstance<'a>>,
    cause: Option<&'a CardInstance<'a>>,
    atomic: bool,
}

#[allow(dead_code)]
enum EffectStepResult<'a> {
    Continue,
    Error(String),
    AskCard(
        PlayerId,
        AskCardTag,
        NumberRange<i32>,
        Vec<&'a CardInstance<'a>>,
    ), // 次のStepはFocusした状態で
    AskTrash(PlayerId, NumberRange<i32>, Vec<&'a CardInstance<'a>>), // 次のStepはFocusした状態で
    AskDiscard(PlayerId, NumberRange<i32>, Vec<&'a CardInstance<'a>>), // 次のStepはFocusした状態で
    AskOptional(PlayerId, AskOptionTag), // 答えがNoだったらそのスタックフレームをスキップ
    SkipContinue,                        // 不可能な指示なので飛ばす
    End,
}

#[derive(Clone)]
#[allow(dead_code)]
struct AskOptionTag {
    tag: String,
    localized_prompt: String,
    default: Option<bool>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct AskCardTag {
    tag: String,
    localized_prompt: String,
}

#[allow(dead_code)]
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
        player.deck.append(&mut player.discard);
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
#[allow(dead_code)]
impl<'a> Game<'a> {
    fn resolve_number(&self, n: &Number) -> i32 {
        use Number::*;
        match n {
            Constant(n) => *n,
            CountCard(selector) => self.resolve_selector(&self.players[0], selector).len() as i32,
            CountCost(selector) => self
                .resolve_selector(&self.players[0], selector)
                .iter()
                .map(|c| self.resolve_number(&c.card.cost))
                .sum(),
            CountEmptyPiles => self.supply.iter().filter(|pile| pile.is_empty()).count() as i32,
            Plus(a, b) => self.resolve_number(a) + self.resolve_number(b),
            Minus(a, b) => self.resolve_number(a) - self.resolve_number(b),
            Times(a, b) => self.resolve_number(a) * self.resolve_number(b),
            Div(a, b) => self.resolve_number(a) / self.resolve_number(b),
            Mod(a, b) => self.resolve_number(a) % self.resolve_number(b),
        }
    }

    fn resolve_number_range(&self, n: &NumberRange<Number>) -> NumberRange<i32> {
        use NumberRange::*;
        match n {
            Exact(n) => Exact(self.resolve_number(n)),
            UpTo(n) => UpTo(self.resolve_number(n)),
            AtLeast(n) => AtLeast(self.resolve_number(n)),
            Range(a, b) => Range(self.resolve_number(a), self.resolve_number(b)),
            AnyNumber => AnyNumber,
        }
    }

    fn resolve_name(&self, selector: &CardNameSelector, card: &Card) -> bool {
        use CardNameSelector::*;
        match selector {
            Name(name) => card.name == *name,
            NameAnd(selectors) => selectors.iter().all(|s| self.resolve_name(s, card)),
            NameOr(selectors) => selectors.iter().any(|s| self.resolve_name(s, card)),
            NameNot(selector) => !self.resolve_name(selector, card),
            HasType(t) => card.types.contains(t),
            Cost(n) => self.resolve_number(n) == self.resolve_number(&card.cost),
            CostUpTo(n) => self.resolve_number(n) >= self.resolve_number(&card.cost),
            Any => true,
        }
    }

    fn resolve_zone(&self, player: &'a Player, zone: &Zone) -> Vec<&'a CardInstance<'a>> {
        match zone {
            Zone::Deck => player.deck.iter().collect(),
            Zone::Hand => player.hand.iter().collect(),
            Zone::Discard => player.discard.iter().collect(),
            Zone::Play => player.play.iter().collect(),
            Zone::Pending => player.pending.iter().collect(),
            Zone::Revealed => player.revealed.iter().collect(),
            Zone::DeckTop => player.deck.last().into_iter().collect(),
            Zone::AllMyCards => [
                &player.deck,
                &player.hand,
                &player.discard,
                &player.play,
                &player.pending,
                &player.revealed,
            ]
            .iter()
            .flat_map(|v| v.iter())
            .collect(),
            Zone::Focused => self
                .stack
                .last()
                .map_or(vec![], |frame| frame.focus.clone()),
            Zone::Itself => self
                .stack
                .last()
                .map_or(vec![], |frame| frame.cause.into_iter().collect()),
        }
    }

    fn resolve_selector<'b>(
        &self,
        target: &'a Player,
        selector: &'b CardSelector,
    ) -> Vec<&'a CardInstance<'a>> {
        use {CardSelector, Zone::*};

        selector
            .zone
            .iter()
            .flat_map(|zone| self.resolve_zone(target, zone))
            .filter(|c| self.resolve_name(&selector.name, c.card))
            .collect()
    }

    fn step(&self) -> (Game<'a>, EffectStepResult) {
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
                frame.effect_queue.extend(effects);
                Continue
            }
            AtomicSequence(effects) => {
                let mut newframe = frame.clone();
                newframe.effect_queue = effects.into_iter().collect();
                newframe.atomic = true;
                game.stack.push(frame);
                game.stack.push(newframe);
                return (game, Continue);
            }
            Optional(prompt, effect) => {
                let target = frame.target;
                let mut newframe = frame.clone();
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                game.stack.push(frame);
                game.stack.push(newframe);
                return (game, AskOptional(target.id, prompt));
            }
            Select(prompt, n, selector, effect) => {
                let target = frame.target;
                let mut newframe = frame.clone();
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                newframe.focus = vec![];
                game.stack.push(frame);
                game.stack.push(newframe);
                return (
                    game,
                    AskCard(
                        target.id,
                        prompt,
                        self.resolve_number_range(&n),
                        self.resolve_selector(target, &selector),
                    ),
                );
            }
            FocusAll(selector, effect) => {
                let target = frame.target;
                let mut newframe = frame.clone();
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                newframe.focus = self.resolve_selector(target, &selector);
                game.stack.push(frame);
                game.stack.push(newframe);
                return (game, Continue);
            }
            TrashSelect(n, selector, effect) => {
                let target = frame.target;
                let mut newframe = frame.clone();
                newframe.effect_queue = VecDeque::from(vec![
                    TrashCard(CardSelector {
                        name: CardNameSelector::Any,
                        zone: vec![Focused],
                    }),
                    *effect,
                ]);
                newframe.focus = vec![];
                game.stack.push(frame);
                game.stack.push(newframe);
                let candidate = self.resolve_selector(target, &selector);
                return (
                    game,
                    AskTrash(
                        target.id,
                        self.resolve_number_range(&n),
                        self.resolve_selector(target, &selector),
                    ),
                );
            }
            _ => SkipContinue,
        };

        game.stack.push(frame);
        (game, result)
    }
}

#[allow(dead_code)]
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
}

#[allow(dead_code)]
mod expansions {
    pub mod basic_supply {
        use crate::*;
        use card_util::*;
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
    }
    pub mod base {
        use std::collections::HashMap;
        use std::vec;

        use crate::{
            card_util::*, CardEffect::*, CardNameSelector::Any, CardType::*, EffectCond::*,
            EffectTrigger::*, Number::*, NumberRange::*, *,
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
                        AskCardTag {
                            tag: "cellar".to_owned(),
                            localized_prompt: "捨て札にするカードを選んでください".to_owned(),
                        },
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
                GainCard(CardNameSelector::CostUpTo(Box::new(Constant(4)))),
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
                        AskCardTag {
                            tag: "harbinger".to_owned(),
                            localized_prompt: "デッキトップに置くカードを選んでください".to_owned(),
                        },
                        AnyNumber,
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
                    AskCardTag {
                        tag: "throne_room".to_owned(),
                        localized_prompt: "使用するカードを選んでください".to_owned(),
                    },
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
                        AskCardTag {
                            tag: "bureaucrat".to_owned(),
                            localized_prompt: "デッキトップに置く勝利点カードを選んでください"
                                .to_owned(),
                        },
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
                    RevealTop(
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
                    AskOptionTag {
                        tag: "mine".to_owned(),
                        localized_prompt: "財宝を破棄しますか？".to_owned(),
                        default: Some(true),
                    },
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
                                CardNameSelector::CostUpTo(Box::new(Plus(
                                    Box::new(CountCost(focused())),
                                    Box::new(Constant(3)),
                                ))),
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
                                    Box::new(Select(
                                        AskCardTag {
                                            tag: "library".to_owned(),
                                            localized_prompt: "このカードを脇に避けますか？"
                                                .to_owned(),
                                        },
                                        AnyNumber,
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
                    GainCardToHand(CardNameSelector::CostUpTo(Box::new(Constant(5)))),
                    Select(
                        AskCardTag {
                            tag: "artisan".to_owned(),
                            localized_prompt: "デッキトップに置くカードを選んでください".to_owned(),
                        },
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
    }
}

#[cfg(test)]
mod tests {
    mod resolvers {
        use crate::{Game, Player, PlayerId};

        pub fn setup<'a>() -> Game<'a> {
            let p0 = Player {
                id: PlayerId { id: 0 },
                name: "Alice".to_owned(),
                deck: vec![],
                hand: vec![],
                play: vec![],
                pending: vec![],
                revealed: vec![],
                discard: vec![],
            };

            let p1 = Player {
                id: PlayerId { id: 1 },
                name: "Bob".to_owned(),
                deck: vec![],
                hand: vec![],
                play: vec![],
                pending: vec![],
                revealed: vec![],
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
        mod cardname {
            use crate::{
                expansions::base::*, expansions::basic_supply::*, tests::resolvers::setup, CardId,
                CardInstance, CardNameSelector, CardSelector, CardType::*, Game, Number::*, Player,
                PlayerId, Zone::*,
            };
            #[test]
            fn cardname_exact() {
                let mut game = setup();
                let copper = copper();
                let alice = &mut game.players[0];
                let hand = [&copper, &copper, &copper];
                for (i, card) in hand.iter().enumerate() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: i },
                        card,
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Name("Copper".to_owned()),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 2);
            }

            #[test]
            fn cardname_cost() {
                let mut game = setup();
                let copper = copper();
                let silver = silver();
                let gold = gold();
                let alice = &mut game.players[0];
                let hand = [&copper, &silver, &silver, &gold];
                for (i, card) in hand.iter().enumerate() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: i },
                        card,
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Cost(Box::new(Constant(3))),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 2);
            }

            #[test]
            fn cardname_costupto() {
                let mut game = setup();
                let copper = copper();
                let silver = silver();
                let gold = gold();
                let alice = &mut game.players[0];
                let hand = [&copper, &silver, &silver, &gold];
                for (i, card) in hand.iter().enumerate() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: i },
                        card,
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::CostUpTo(Box::new(Constant(3))),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 3);
            }

            #[test]
            fn cardname_or() {
                let mut game = setup();
                let copper = copper();
                let silver = silver();
                let gold = gold();
                let alice = &mut game.players[0];
                let hand = [&copper, &silver, &gold];
                for (i, card) in hand.iter().enumerate() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: i },
                        card,
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::NameOr(vec![
                        CardNameSelector::Name("Copper".to_owned()),
                        CardNameSelector::Name("Silver".to_owned()),
                    ]),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 2);
            }

            #[test]
            fn cardname_type() {
                let mut game = setup();
                let copper = copper();
                let silver = silver();
                let gold = gold();
                let moat = moat();
                let bandit = bandit();

                let alice = &mut game.players[0];
                let hand = [&copper, &silver, &gold, &moat, &bandit];
                for (i, card) in hand.iter().enumerate() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: i },
                        card,
                    });
                }
                let selector_t = CardSelector {
                    name: CardNameSelector::HasType(Treasure),
                    zone: vec![Hand],
                };
                let selector_a = CardSelector {
                    name: CardNameSelector::HasType(Action),
                    zone: vec![Hand],
                };
                let selector_r = CardSelector {
                    name: CardNameSelector::HasType(Reaction),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result_t = game.resolve_selector(alice, &selector_t);
                let result_a = game.resolve_selector(alice, &selector_a);
                let result_r = game.resolve_selector(alice, &selector_r);
                assert_eq!(result_t.len(), 3);
                assert_eq!(result_a.len(), 2);
                assert_eq!(result_r.len(), 1);
            }

            #[test]
            fn cardname_any() {
                let mut game = setup();
                let copper = copper();
                let silver = silver();
                let gold = gold();
                let moat = moat();
                let bandit = bandit();

                let alice = &mut game.players[0];
                let hand = [&copper, &silver, &gold, &moat, &bandit];
                for (i, card) in hand.iter().enumerate() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: i },
                        card,
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Any,
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 5);
            }
        }
        mod cardselector {
            use std::vec;
            use std::{collections::HashMap, hash::Hash};

            use crate::{
                expansions::{
                    base::*,
                    basic_supply::{self, *},
                },
                tests::resolvers::setup,
                Card, CardId, CardInstance, CardNameSelector, CardSelector,
                CardType::*,
                Game,
                Number::*,
                Player, PlayerId,
                Zone::*,
            };

            fn supply() -> HashMap<String, Card> {
                let mut supply = basic_supply();
                supply.extend(base_set());
                supply
            }

            /*
            手札：銅貨、銀貨、金貨、堀、山賊
            山札：屋敷、民兵、役人、庭園、市場
            捨て札：礼拝堂、前駆者、魔女、職人、鉱山
            場札：地下貯蔵庫、密猟者、衛兵
            処理中：商人、玉座の間
             */
            fn setup2(supply: &HashMap<String, Card>) -> Game<'_> {
                let mut game = setup();
                let alice = &mut game.players[0];
                let mut cid = 0;
                let hand = ["Copper", "Silver", "Gold", "Moat", "Bandit"];
                let deck = ["Estate", "Militia", "Bureaucrat", "Gardens", "Market"];
                let discard = ["Chapel", "Harbinger", "Witch", "Artisan", "Mine"];
                let play = ["Cellar", "Poacher", "Sentry"];
                let pending = ["Merchant", "Throne Room"];
                for card in hand.iter() {
                    alice.hand.push(CardInstance {
                        id: CardId { id: cid },
                        card: &supply[*card],
                    });
                    cid += 1;
                }
                for card in deck.iter() {
                    alice.deck.push(CardInstance {
                        id: CardId { id: cid },
                        card: &supply[*card],
                    });
                    cid += 1;
                }
                for card in discard.iter() {
                    alice.discard.push(CardInstance {
                        id: CardId { id: cid },
                        card: &supply[*card],
                    });
                    cid += 1;
                }
                for card in play.iter() {
                    alice.play.push(CardInstance {
                        id: CardId { id: cid },
                        card: &supply[*card],
                    });
                    cid += 1;
                }
                for card in pending.iter() {
                    alice.pending.push(CardInstance {
                        id: CardId { id: cid },
                        card: &supply[*card],
                    });
                    cid += 1;
                }
                game
            }

            #[test]
            fn cardselector_hand() {
                let supply = supply();
                let game = setup2(&supply);
                let alice = &game.players[0];
                let selector = CardSelector {
                    name: CardNameSelector::Any,
                    zone: vec![Hand],
                };
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 5);
            }

            #[test]
            fn cardselector_all() {
                let supply = supply();
                let game = setup2(&supply);
                let alice = &game.players[0];
                let selector = CardSelector {
                    name: CardNameSelector::Cost(Box::new(Constant(2))),
                    zone: vec![AllMyCards],
                };
                let result = game.resolve_selector(alice, &selector);
                assert_eq!(result.len(), 4); // 屋敷、地下貯蔵庫、礼拝堂、堀
            }
        }
    }
}
