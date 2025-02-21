use crate::core::{card::Card, zone::Zone};
use crate::game::player::PlayerId;

#[derive(Clone)]
#[allow(dead_code)]
pub struct CardInstance<'a> {
    pub card: &'a Card,
    pub id: CardInstanceId,
    pub address: CardAddress,
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
pub struct CardInstanceId {
    id: usize,
}

impl CardInstanceId {
    pub fn new(id: usize) -> CardInstanceId {
        CardInstanceId { id }
    }
}
