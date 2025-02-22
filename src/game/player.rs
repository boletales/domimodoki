use crate::game::card_instance::CardInstanceId;

#[derive(Clone)]
#[allow(dead_code)]
pub struct PlayerData {
    pub name: String,
    pub deck: Vec<CardInstanceId>,
    pub hand: Vec<CardInstanceId>,
    pub play: Vec<CardInstanceId>,
    pub pending: Vec<CardInstanceId>,
    pub aside: Vec<CardInstanceId>,
    pub revealed: Vec<CardInstanceId>,
    pub discard: Vec<CardInstanceId>,
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
