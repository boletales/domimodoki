use crate::{
    core::{
        ask_tag::{AskCardTag, AskOptionTag},
        effect::CardEffect,
        number::NumberRange,
    },
    game::{
        card_instance::{CardInstanceId, CardInstanceInfo},
        player::PlayerId,
    },
};
use std::collections::VecDeque;

#[derive(Clone)]
#[allow(dead_code)]
pub struct EffectStackFrame {
    pub player: PlayerId,
    pub target: PlayerId,
    pub effect_queue: VecDeque<CardEffect>,
    pub focus: Vec<CardInstanceId>,
    pub cause: Option<CardInstanceId>,
    pub atomic: bool,
}

#[allow(dead_code)]
pub enum EffectStepResult {
    Continue,
    Error(String),
    AskCard(
        PlayerId,
        AskCardTag,
        NumberRange<i32>,
        Vec<CardInstanceInfo>,
    ), // 次のStepはFocusした状態で
    AskTrash(PlayerId, NumberRange<i32>, Vec<CardInstanceInfo>), // 次のStepはFocusした状態で
    AskDiscard(PlayerId, NumberRange<i32>, Vec<CardInstanceInfo>), // 次のStepはFocusした状態で
    AskOptional(PlayerId, AskOptionTag), // 答えがNoだったらそのスタックフレームをスキップ
    SkipContinue,                        // 不可能な指示なので飛ばす
    End,
}
