use crate::core::{
    ask_tag::{AskCardTag, AskOptionTag},
    number::{Number, NumberRange},
    selector::{CardNameSelector, CardSelector},
    zone::Zone,
};

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
