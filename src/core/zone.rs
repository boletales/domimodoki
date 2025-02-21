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
