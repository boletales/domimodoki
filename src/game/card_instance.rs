use crate::core::{card::Card, zone::Zone};
use crate::game::player::PlayerId;

#[derive(Clone)]
#[allow(dead_code)]
pub struct CardInstance<'a> {
    pub card: &'a Card,
    pub id: CardInstanceId,
    pub address: CardAddress,
}

impl CardInstance<'_> {
    pub fn info(&self) -> CardInstanceInfo {
        CardInstanceInfo {
            name: self.card.name.clone(),
            localized_name: self.card.localized_name.clone(),
            instance_id: self.id,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum CardAddress {
    Supply(usize, usize),
    PlayerOwned(PlayerId, Zone),
    Trash,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct CardInstanceId {
    id: usize,
}

impl CardInstanceId {
    pub fn new(id: usize) -> CardInstanceId {
        CardInstanceId { id }
    }
}

pub struct CardInstanceInfo {
    pub name: String,
    pub localized_name: String,
    pub instance_id: CardInstanceId,
}
