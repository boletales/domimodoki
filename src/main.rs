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

// カードの働きを記述するためのメタ言語
enum CardEffect {
    Noop,
    Sequence(Vec<CardEffect>),
    AtomicSequence(Box<CardEffect>),

    // Select系：カードを選択し、Focusの選択先を変更した上で、効果を適用する
    SelectExact(i32, CardSelector, Box<CardEffect>), // ちょうどn枚選択
    SelectFewer(i32, CardSelector, Box<CardEffect>), // n枚以下選択

    // Select亜種だけどプレイヤーの選択を必要としない
    FocusAll(CardSelector, Box<CardEffect>),

    PlusDraw(i32),
    PlusAction(i32),
    PlusBuy(i32),
    PlusCoin(i32),
    TrashCard(CardSelector),
    DiscardCard(CardSelector),
    GainCard(CardNameSelector),
    Optional(Box<CardEffect>),
    AllOpponents(Box<CardEffect>),
    Atack(Box<CardEffect>),
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
    Type(CardType),
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

fn vanillaEffect(draw: i32, action: i32, buy: i32, coin: i32) -> CardEffect {
    CardEffect::Sequence(vec![
        CardEffect::PlusDraw(draw),
        CardEffect::PlusAction(action),
        CardEffect::PlusBuy(buy),
        CardEffect::PlusCoin(coin),
    ])
}

fn vanillaActionCard(name: &str, cost: i32, draw: i32, action: i32, buy: i32, coin: i32) -> Card {
    Card {
        name: name.to_string(),
        cost,
        vp: 0,
        action: vanillaEffect(draw, action, buy, coin),
        reaction: CardEffect::Noop,
        treasure: CardEffect::Noop,
        types: vec![],
    }
}

fn vanillaTreasureCard(name: &str, cost: i32, coin: i32) -> Card {
    Card {
        name: name.to_string(),
        cost,
        vp: 0,
        action: CardEffect::Noop,
        reaction: CardEffect::Noop,
        treasure: CardEffect::PlusCoin(coin),
        types: vec![],
    }
}

fn vanillaVPCard(name: &str, cost: i32, vp: i32) -> Card {
    Card {
        name: name.to_string(),
        cost,
        vp,
        action: CardEffect::Noop,
        reaction: CardEffect::Noop,
        treasure: CardEffect::Noop,
        types: vec![],
    }
}
