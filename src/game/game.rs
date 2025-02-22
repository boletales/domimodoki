use crate::{
    core::{
        card::Card,
        effect::CardEffect::{self, *},
        number::{
            Number::{self, *},
            NumberRange::{self, *},
        },
        selector::{
            CardNameSelector::{self, *},
            CardSelector,
        },
        zone::Zone::{self, *},
    },
    game::{
        card_instance::{CardInstance, CardInstanceId},
        effect_stack::{
            EffectStackFrame,
            EffectStepResult::{self, *},
        },
        player::{PlayerData, PlayerId},
    },
};
use rand::Rng;
use std::{
    clone,
    collections::{HashMap, VecDeque},
    vec,
};

#[derive(Clone)]
#[allow(dead_code)]
pub struct Game<'a> {
    pub players: Vec<PlayerData>,
    pub supply: Vec<Vec<CardInstanceId>>,
    pub trash: Vec<CardInstanceId>,
    pub turn: i32,
    pub stack: Vec<EffectStackFrame>,
    card_instances: HashMap<CardInstanceId, CardInstance<'a>>,
}

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
    fn get_from_deck(&mut self, player: PlayerId, n: i32) -> Vec<CardInstanceId> {
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
    fn look_at_top(&self, player: PlayerId, n: i32) -> Vec<CardInstanceId> {
        let playerdata = self.get_player(player).unwrap();
        // 後半n枚を見る
        playerdata
            .deck
            .iter()
            .rev()
            .take(n as usize)
            .rev()
            .copied()
            .collect()
    }

    fn get_player(&self, player: PlayerId) -> Option<&'_ PlayerData> {
        self.players.iter().find(move |p| p.id == player)
    }

    fn get_player_mut(&mut self, player: PlayerId) -> Option<&'_ mut PlayerData> {
        self.players.iter_mut().find(move |p| p.id == player)
    }

    fn get_card_instance(&self, id: CardInstanceId) -> Option<&'_ CardInstance<'a>> {
        self.card_instances.get(&id)
    }

    fn get_card_instances(&self, ids: &Vec<CardInstanceId>) -> Vec<&'_ CardInstance<'a>> {
        ids.iter()
            .map(|id| self.get_card_instance(*id).unwrap())
            .collect()
    }

    pub fn resolve_number(&self, player: PlayerId, n: &Number) -> i32 {
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

    pub fn resolve_name(&self, player: PlayerId, selector: &CardNameSelector, card: &Card) -> bool {
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
            Zone::Deck => self.get_card_instances(&player.deck),
            Zone::Hand => self.get_card_instances(&player.hand),
            Zone::Discard => self.get_card_instances(&player.discard),
            Zone::Play => self.get_card_instances(&player.play),
            Zone::Pending => self.get_card_instances(&player.pending),
            Zone::Aside => self.get_card_instances(&player.aside),
            Zone::Revealed => self.get_card_instances(&player.revealed),
            Zone::DeckTop => {
                self.get_card_instances(&player.deck.last().into_iter().copied().collect())
            }
            Zone::AllMyCards => self.get_card_instances(
                &[
                    &player.deck,
                    &player.hand,
                    &player.discard,
                    &player.play,
                    &player.pending,
                    &player.aside,
                    &player.revealed,
                ]
                .iter()
                .flat_map(|v| v.iter().copied())
                .collect(),
            ),
            Zone::Focused => self
                .stack
                .last()
                .map_or(vec![], |frame| self.get_card_instances(&frame.focus)),
            Zone::Itself => self.stack.last().map_or(vec![], |frame| {
                self.get_card_instances(&frame.cause.into_iter().collect())
            }),
            _ => vec![],
        }
    }

    pub fn resolve_selector<'b>(
        &self,
        target: PlayerId,
        selector: &'b CardSelector,
    ) -> Vec<&'_ CardInstance<'a>> {
        selector
            .zone
            .iter()
            .flat_map(|zone| self.resolve_zone(target, zone))
            .filter(|c: &&CardInstance<'a>| self.resolve_name(target, &selector.name, c.card))
            .collect()
    }

    fn pop_and_step(&mut self) -> EffectStepResult {
        let Some(mut frame) = self.stack.pop() else {
            return End;
        };

        let Some(effect) = frame.effect_queue.clone().pop_front() else {
            self.stack.pop();
            return Continue;
        };

        let clone = frame.clone();
        self.stack.push(frame);
        self.exec_effect_one(clone, effect)
    }

    fn extend_frame(&mut self, effects: &[CardEffect]) {
        let Some(frame) = self.stack.last_mut() else {
            return;
        };

        frame.effect_queue.extend(effects.to_owned());
    }

    fn exec_effect_one(&mut self, frame: EffectStackFrame, effect: CardEffect) -> EffectStepResult {
        let result = match effect {
            Noop => Continue,
            Sequence(effects) => {
                self.extend_frame(&effects);
                Continue
            }
            AtomicSequence(effects) => {
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(effects);
                newframe.atomic = true;
                self.stack.push(newframe);
                Continue
            }
            Optional(prompt, effect) => {
                let target = frame.target;
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                self.stack.push(newframe);
                return AskOptional(target, prompt);
            }
            FocusAll(selector, effect) => {
                let target = frame.target;
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                self.stack.push(newframe);
                self.stack.last_mut().unwrap().focus = self
                    .resolve_selector(target, &selector)
                    .iter()
                    .map(|c| c.id)
                    .collect();
                return Continue;
            }
            Select(prompt, n, selector, effect) => {
                let target = frame.target;
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                newframe.focus = vec![];
                self.stack.push(newframe);
                return AskCard(
                    target,
                    prompt,
                    self.resolve_number_range(target, &n),
                    self.resolve_selector(target, &selector)
                        .iter()
                        .map(|c| c.info())
                        .collect(),
                );
            }
            TrashSelect(n, selector, effect) => {
                let target = frame.target;
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(vec![
                    TrashCard(CardSelector {
                        name: CardNameSelector::Any,
                        zone: vec![Focused],
                    }),
                    *effect,
                ]);
                newframe.focus = vec![];
                self.stack.push(newframe);
                return AskTrash(
                    target,
                    self.resolve_number_range(target, &n),
                    self.resolve_selector(target, &selector)
                        .iter()
                        .map(|c| c.info())
                        .collect(),
                );
            }
            DiscardSelect(n, selector, effect) => {
                let target = frame.target;
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(vec![
                    DiscardCard(CardSelector {
                        name: CardNameSelector::Any,
                        zone: vec![Focused],
                    }),
                    *effect,
                ]);
                newframe.focus = vec![];
                self.stack.push(newframe);
                return AskDiscard(
                    target,
                    self.resolve_number_range(target, &n),
                    self.resolve_selector(target, &selector)
                        .iter()
                        .map(|c| c.info())
                        .collect(),
                );
            }
            RevealTop(n, effect) => {
                let target = frame.target;
                let topn = self.get_from_deck(target, self.resolve_number(target, &n));
                let mut newframe = frame;
                newframe.effect_queue = VecDeque::from(vec![*effect]);
                newframe.focus = vec![];
                self.stack.push(newframe);
                return Continue;
            }
            _ => SkipContinue,
        };
        result
    }
}
