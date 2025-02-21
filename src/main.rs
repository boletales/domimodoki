use core::panic;
use std::collections::{vec_deque, VecDeque};

fn main() {
    println!("Hello, world!");
}

// Dominion simulator

mod player {
    #[derive(Clone)]
    #[allow(dead_code)]
    pub struct PlayerData<'a> {
        name: String,
        deck: Vec<CardInstance<'a>>,
        hand: Vec<CardInstance<'a>>,
        play: Vec<CardInstance<'a>>,
        pending: Vec<CardInstance<'a>>,
        aside: Vec<CardInstance<'a>>,
        revealed: Vec<CardInstance<'a>>,
        discard: Vec<CardInstance<'a>>,
        id: PlayerId,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub struct PlayerId {
        id: usize,
    }
}

mod card_instance {
    #[allow(dead_code)]
    pub struct CardInstance<'a> {
        card: &'a Card,
        id: CardId,
        address: CardAddress,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum CardAddress {
        Supply(usize, usize),
        PlayerOwned(PlayerId, Zone),
        Trash,
    }

    #[derive(Clone, Copy)]
    #[allow(dead_code)]
    pub struct CardId {
        id: usize,
    }
}

mod effect_stack {
    #[derive(Clone)]
    #[allow(dead_code)]
    pub struct EffectStackFrame<'a> {
        player: PlayerId,
        target: PlayerId,
        effect_queue: VecDeque<CardEffect>,
        focus: Vec<&'a CardInstance<'a>>,
        cause: Option<&'a CardInstance<'a>>,
        atomic: bool,
    }

    #[allow(dead_code)]
    pub enum EffectStepResult<'a> {
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
}

mod effect_result {
    #[derive(Clone)]
    #[allow(dead_code)]
    pub struct AskOptionTag {
        tag: String,
        localized_prompt: String,
        default: Option<bool>,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    pub struct AskCardTag {
        tag: String,
        localized_prompt: String,
    }
}

mod card {
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
}

mod number {
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
        const fn contains(&self, n: i32) -> bool {
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
}

mod effect {
    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum EffectCond {
        Leq(Number, Number),
        Geq(Number, Number),
        Eq(Number, Number),
        CondAnd(Vec<EffectCond>),
        CondOr(Vec<EffectCond>),
        CondNot(Box<EffectCond>),
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum EffectTrigger {
        Attacked,       // Focus: 空, PreventDefaultで攻撃を無効化
        PlayAsAction,   // Focus: 空
        PlayAsTreasure, // Focus: 空
        CardPlayed,     // Focus: カード
        Cleanup,        // Focus: 空, PreventDefaultで場にあっても捨て札にしない
        MyTurnStart,
        MyTurnEnd,
        OncePerTurn(Box<EffectTrigger>),
    }

    // カードの働きを記述するためのメタ言語
    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum CardEffect {
        Noop,
        Sequence(Vec<CardEffect>),
        AtomicSequence(Vec<CardEffect>), // 「不可能な指示は無視」ができない場合（改築の破棄→獲得など）に使う。SkipContinueを伝播
        Optional(AskOptionTag, Box<CardEffect>),

        // Select亜種 該当カードすべてを選択、プレイヤーの選択を必要としない
        FocusAll(CardSelector, Box<CardEffect>),

        // Select系：カードを選択し、Focusの選択先を変更した上で、効果を適用する
        Select(
            AskCardTag,
            NumberRange<Number>,
            CardSelector,
            Box<CardEffect>,
        ), // n枚選択

        TrashSelect(NumberRange<Number>, CardSelector, Box<CardEffect>), // Select亜種 手札から廃棄

        DiscardSelect(NumberRange<Number>, CardSelector, Box<CardEffect>), // Select亜種 手札を捨てるs

        // デッキトップ公開・Focus
        RevealTop(Number, Box<CardEffect>),

        LookAtTop(Number, Box<CardEffect>), // デッキトップを見るだけ

        DrawFocus(Number, Box<CardEffect>), // ドローしてFocus

        DrawFrom(CardSelector), // ドロー扱いで手札に加える

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
}

mod zone {
    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum Zone {
        // 実在のゾーン。配置対象としてもよい
        Deck,
        Hand,
        Discard,
        Play,
        Pending,
        Aside,
        Revealed,

        // 以下は仮想的なゾーン
        DeckTop, // デッキの一番上。配置対象としてもよい
        AllMyCards,
        Focused,
        Itself,
    }
}

mod selector {
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
        name: CardNameSelector,
        zone: Vec<Zone>,
    }
}

mod turn_phase {
    #[derive(Clone)]
    #[allow(dead_code)]
    pub enum TurnPhase {
        Action,
        Buy,
        Cleanup,
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct Game<'a> {
    players: Vec<PlayerData<'a>>,
    supply: Vec<Vec<(CardInstance<'a>, i32)>>,
    trash: Vec<CardInstance<'a>>,
    turn: i32,
    stack: Vec<EffectStackFrame<'a>>,
}

mod game {
    use crate::number::{Number::*, NumberRange::*};
    use rand::Rng;
    #[allow(dead_code)]
    impl<'a> Game<'a> {
        fn shuffle(&mut self, player: PlayerId) {
            let playerdata = self.get_player_mut(player).unwrap();
            // Fisher-Yates shuffle
            for i in (1..playerdata.deck.len()).rev() {
                let j = rand::rng().random_range(0..=i);
                playerdata.deck.swap(i, j);
            }
        }

        fn reshuffle(&mut self, player: PlayerId) {
            let playerdata = self.get_player_mut(player).unwrap();
            playerdata.deck.append(&mut playerdata.discard);
            self.shuffle(player);
        }

        /// プレイヤーのデッキからカードをn枚引く。（デッキ残量が不足している場合はリシャッフルしてから引く）
        fn get_from_deck(mut self, player: PlayerId, n: i32) -> Vec<CardInstance<'a>> {
            let playerdata = self.get_player(player).unwrap();
            let mut cards = vec![];

            let n = n.min((playerdata.deck.len() + playerdata.discard.len()) as i32); // 捨て札混ぜても必要数取れないなら、必要数を減らす

            if playerdata.deck.len() < n as usize {
                self.reshuffle(player);
            }

            let playerdata = self.get_player_mut(player).unwrap();
            for _ in 0..n {
                if playerdata.deck.is_empty() {
                    return cards;
                }
                cards.push(playerdata.deck.pop().unwrap());
            }
            cards
        }

        /// デッキトップを見る（リシャッフルしない）
        fn look_at_top(&self, player: PlayerId, n: i32) -> Vec<&'_ CardInstance<'a>> {
            let playerdata = self.get_player(player).unwrap();
            // 後半n枚を見る
            playerdata
                .deck
                .iter()
                .rev()
                .take(n as usize)
                .rev()
                .collect()
        }

        fn get_player(&self, player: PlayerId) -> Option<&'_ PlayerData<'a>> {
            self.players.iter().find(move |p| p.id == player)
        }

        fn get_player_mut(&mut self, player: PlayerId) -> Option<&'_ mut PlayerData<'a>> {
            self.players.iter_mut().find(move |p| p.id == player)
        }

        pub fn resolve_number(&self, player: PlayerId, n: &Number) -> i32 {
            use crate::number::Number::*;
            match n {
                Constant(n) => *n,
                CountCard(selector) => self.resolve_selector(player, selector).len() as i32,
                CountCost(selector) => self
                    .resolve_selector(player, selector)
                    .iter()
                    .map(|c| self.resolve_number(player, &c.card.cost))
                    .sum(),
                CountEmptyPiles => self.supply.iter().filter(|pile| pile.is_empty()).count() as i32,
                Plus(a, b) => self.resolve_number(player, a) + self.resolve_number(player, b),
                Minus(a, b) => self.resolve_number(player, a) - self.resolve_number(player, b),
                Times(a, b) => self.resolve_number(player, a) * self.resolve_number(player, b),
                Div(a, b) => self.resolve_number(player, a) / self.resolve_number(player, b),
                Mod(a, b) => self.resolve_number(player, a) % self.resolve_number(player, b),
            }
        }

        pub fn resolve_number_range(
            &self,
            player: PlayerId,
            n: &NumberRange<Number>,
        ) -> NumberRange<i32> {
            match n {
                Exact(n) => Exact(self.resolve_number(player, n)),
                UpTo(n) => UpTo(self.resolve_number(player, n)),
                AtLeast(n) => AtLeast(self.resolve_number(player, n)),
                Range(a, b) => Range(
                    self.resolve_number(player, a),
                    self.resolve_number(player, b),
                ),
                AnyNumber => AnyNumber,
            }
        }

        pub fn resolve_name(
            &self,
            player: PlayerId,
            selector: &CardNameSelector,
            card: &Card,
        ) -> bool {
            use crate::selector::CardNameSelector::*;
            match selector {
                Name(name) => card.name == *name,
                NameAnd(selectors) => selectors.iter().all(|s| self.resolve_name(player, s, card)),
                NameOr(selectors) => selectors.iter().any(|s| self.resolve_name(player, s, card)),
                NameNot(selector) => !self.resolve_name(player, selector, card),
                HasType(t) => card.types.contains(t),
                Cost(n) => self
                    .resolve_number_range(player, n)
                    .contains(self.resolve_number(player, &card.cost)),
                Any => true,
            }
        }

        pub fn calculate_vp(&self, player: PlayerId) -> i32 {
            self.resolve_zone(player, &Zone::AllMyCards)
                .iter()
                .map(|c| self.resolve_number(player, &c.card.vp))
                .sum()
        }

        pub fn resolve_zone(&self, playerid: PlayerId, zone: &Zone) -> Vec<&'_ CardInstance<'a>> {
            let Some(player) = self.get_player(playerid) else {
                return vec![];
            };
            match zone {
                Zone::Deck => player.deck.iter().collect(),
                Zone::Hand => player.hand.iter().collect(),
                Zone::Discard => player.discard.iter().collect(),
                Zone::Play => player.play.iter().collect(),
                Zone::Pending => player.pending.iter().collect(),
                Zone::Aside => player.aside.iter().collect(),
                Zone::Revealed => player.revealed.iter().collect(),
                Zone::DeckTop => player.deck.last().into_iter().collect(),
                Zone::AllMyCards => [
                    &player.deck,
                    &player.hand,
                    &player.discard,
                    &player.play,
                    &player.pending,
                    &player.aside,
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
                _ => vec![],
            }
        }

        fn resolve_selector<'b>(
            &self,
            target: PlayerId,
            selector: &'b CardSelector,
        ) -> Vec<&'_ CardInstance<'a>> {
            selector
                .zone
                .iter()
                .flat_map(|zone| self.resolve_zone(target, zone))
                .filter(|c| self.resolve_name(target, &selector.name, c.card))
                .collect()
        }

        fn step(&mut self) -> EffectStepResult<'_> {
            let mut game = self;

            let Some(mut frame) = game.stack.pop() else {
                return End;
            };

            let Some(effect) = frame.effect_queue.clone().pop_front() else {
                game.stack.pop();
                return Continue;
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
                    return Continue;
                }
                Optional(prompt, effect) => {
                    let target = frame.target;
                    let mut newframe = frame.clone();
                    newframe.effect_queue = VecDeque::from(vec![*effect]);
                    game.stack.push(frame);
                    game.stack.push(newframe);
                    return AskOptional(target, prompt);
                }
                FocusAll(selector, effect) => {
                    let target = frame.target;
                    let mut newframe = frame.clone();
                    newframe.effect_queue = VecDeque::from(vec![*effect]);
                    newframe.focus = self.resolve_selector(target, &selector);
                    game.stack.push(frame);
                    game.stack.push(newframe);
                    return Continue;
                }
                Select(prompt, n, selector, effect) => {
                    let target = frame.target;
                    let mut newframe = frame.clone();
                    newframe.effect_queue = VecDeque::from(vec![*effect]);
                    newframe.focus = vec![];
                    game.stack.push(frame);
                    game.stack.push(newframe);
                    return AskCard(
                        target,
                        prompt,
                        self.resolve_number_range(target, &n),
                        self.resolve_selector(target, &selector),
                    );
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
                    return AskTrash(
                        target,
                        self.resolve_number_range(target, &n),
                        self.resolve_selector(target, &selector),
                    );
                }
                DiscardSelect(n, selector, effect) => {
                    let target = frame.target;
                    let mut newframe = frame.clone();
                    newframe.effect_queue = VecDeque::from(vec![
                        DiscardCard(CardSelector {
                            name: CardNameSelector::Any,
                            zone: vec![Focused],
                        }),
                        *effect,
                    ]);
                    newframe.focus = vec![];
                    game.stack.push(frame);
                    game.stack.push(newframe);
                    return AskDiscard(
                        target,
                        self.resolve_number_range(target, &n),
                        self.resolve_selector(target, &selector),
                    );
                }
                RevealTop(n, effect) => {
                    let target = frame.target;
                    let topn = game.get_from_deck(target, self.resolve_number(target, &n));
                    let mut newframe = frame.clone();
                    newframe.effect_queue = VecDeque::from(vec![*effect]);
                    newframe.focus = vec![];
                    game.stack.push(frame);
                    game.stack.push(newframe);
                    return Continue;
                }
                _ => SkipContinue,
            };

            game.stack.push(frame);
            result
        }
    }
}

#[allow(dead_code)]
mod card_util {

    use crate::{
        card::{Card, CardType::*},
        effect::EffectTrigger::*,
        number::Number::*,
        selector::{CardNameSelector::*, CardSelector},
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
                                        AskCardTag {
                                            tag: "library".to_owned(),
                                            localized_prompt: "このカードを脇に避けますか？"
                                                .to_owned(),
                                        },
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
    use std::{collections::HashMap, hash::Hash};

    use crate::{
        expansions::{
            base::*,
            basic_supply::{self, *},
        },
        Card,
        CardAddress::*,
        CardId, CardInstance, Game, PlayerData, PlayerId,
        Zone::*,
    };

    pub fn setup<'a>() -> Game<'a> {
        let p0 = PlayerData {
            id: PlayerId { id: 0 },
            name: "Alice".to_owned(),
            deck: vec![],
            hand: vec![],
            play: vec![],
            pending: vec![],
            aside: vec![],
            revealed: vec![],
            discard: vec![],
        };

        let p1 = PlayerData {
            id: PlayerId { id: 1 },
            name: "Bob".to_owned(),
            deck: vec![],
            hand: vec![],
            play: vec![],
            pending: vec![],
            aside: vec![],
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
                address: PlayerOwned(alice.id, Hand),
            });
            cid += 1;
        }
        for card in deck.iter() {
            alice.deck.push(CardInstance {
                id: CardId { id: cid },
                card: &supply[*card],
                address: PlayerOwned(alice.id, Deck),
            });
            cid += 1;
        }
        for card in discard.iter() {
            alice.discard.push(CardInstance {
                id: CardId { id: cid },
                card: &supply[*card],
                address: PlayerOwned(alice.id, Discard),
            });
            cid += 1;
        }
        for card in play.iter() {
            alice.play.push(CardInstance {
                id: CardId { id: cid },
                card: &supply[*card],
                address: PlayerOwned(alice.id, Play),
            });
            cid += 1;
        }
        for card in pending.iter() {
            alice.pending.push(CardInstance {
                id: CardId { id: cid },
                card: &supply[*card],
                address: PlayerOwned(alice.id, Pending),
            });
            cid += 1;
        }
        game
    }
    mod resolvers {
        mod cardname {
            use crate::{
                card::CardType::*,
                card_instance::{CardAddress::*, CardId, CardInstance},
                expansions::base::*,
                expansions::basic_supply::*,
                number::{Number::*, NumberRange::*},
                player::{PlayerData, PlayerId},
                selector::{CardNameSelector, CardSelector},
                tests::setup,
                zone::Zone::*,
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
                        address: PlayerOwned(alice.id, Hand),
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Name("Copper".to_owned()),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice.id, &selector);
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
                        address: PlayerOwned(alice.id, Hand),
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Cost(Box::new(Exact(Constant(3)))),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice.id, &selector);
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
                        address: PlayerOwned(alice.id, Hand),
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Cost(Box::new(UpTo(Constant(3)))),
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice.id, &selector);
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
                        address: PlayerOwned(alice.id, Hand),
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
                let result = game.resolve_selector(alice.id, &selector);
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
                        address: PlayerOwned(alice.id, Hand),
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
                let result_t = game.resolve_selector(alice.id, &selector_t);
                let result_a = game.resolve_selector(alice.id, &selector_a);
                let result_r = game.resolve_selector(alice.id, &selector_r);
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
                        address: PlayerOwned(alice.id, Hand),
                    });
                }
                let selector = CardSelector {
                    name: CardNameSelector::Any,
                    zone: vec![Hand],
                };
                let alice = &game.players[0];
                let result = game.resolve_selector(alice.id, &selector);
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
                tests::{setup2, supply},
                CardNameSelector::*,
                CardSelector,
                Number::*,
                NumberRange::*,
                Zone::*,
            };

            #[test]
            fn cardselector_hand() {
                let supply = supply();
                let game = setup2(&supply);
                let alice = &game.players[0];
                let selector = CardSelector {
                    name: Any,
                    zone: vec![Hand],
                };
                let result = game.resolve_selector(alice.id, &selector);
                assert_eq!(result.len(), 5);
            }

            #[test]
            fn cardselector_all() {
                let supply = supply();
                let game = setup2(&supply);
                let alice = &game.players[0];
                let selector = CardSelector {
                    name: Cost(Box::new(Exact(Constant(2)))),
                    zone: vec![AllMyCards],
                };
                let result = game.resolve_selector(alice.id, &selector);
                assert_eq!(result.len(), 4); // 屋敷、地下貯蔵庫、礼拝堂、堀
            }
        }
    }

    mod base {
        use std::vec;
        use std::{collections::HashMap, hash::Hash};

        use crate::{
            expansions::{
                base::*,
                basic_supply::{self, *},
            },
            tests::{setup2, supply},
            CardAddress::*,
            CardNameSelector::*,
            CardSelector,
            Number::*,
            NumberRange::*,
            Zone::*,
        };

        #[test]
        fn gardens_vp() {
            let supply = supply();
            let mut game = setup2(&supply);
            let alice = &game.players[0];
            let vp = game.calculate_vp(alice.id);
            assert_eq!(vp, 3); // 屋敷(1VP)x1 + 庭園(20枚: 2VP)x1

            let alice = &mut game.players[0];
            for i in 20..28 {
                alice.deck.push(crate::CardInstance {
                    id: crate::CardId { id: i },
                    card: &supply["Copper"],
                    address: PlayerOwned(alice.id, Deck),
                });
            }

            let alice = &game.players[0];
            let vp = game.calculate_vp(alice.id);
            assert_eq!(vp, 3); // 屋敷(1VP)x1 + 庭園(28枚: 2VP)x1

            let alice = &mut game.players[0];
            for i in 28..30 {
                alice.deck.push(crate::CardInstance {
                    id: crate::CardId { id: i },
                    card: &supply["Copper"],
                    address: PlayerOwned(alice.id, Deck),
                });
            }

            let alice = &game.players[0];
            let vp = game.calculate_vp(alice.id);
            assert_eq!(vp, 4); // 屋敷(1VP)x1 + 庭園(30枚: 3VP)x1
        }
    }
}
