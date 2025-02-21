use crate::game::card_instance::CardInstance;

#[derive(Clone)]
#[allow(dead_code)]
pub struct PlayerData<'a> {
    pub name: String,
    pub deck: Vec<CardInstance<'a>>,
    pub hand: Vec<CardInstance<'a>>,
    pub play: Vec<CardInstance<'a>>,
    pub pending: Vec<CardInstance<'a>>,
    pub aside: Vec<CardInstance<'a>>,
    pub revealed: Vec<CardInstance<'a>>,
    pub discard: Vec<CardInstance<'a>>,
    pub id: PlayerId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlayerId {
    id: usize,
}

impl PlayerId {
    pub fn new(id: usize) -> PlayerId {
        PlayerId { id }
    }
}
