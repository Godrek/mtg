use std::collections::HashMap;

use super::{CardDef, CardId};

/// A simple card database that maps CardId -> CardDef.
#[derive(Debug, Clone, Default)]
pub struct CardDatabase {
    pub cards: HashMap<CardId, CardDef>,
}

impl CardDatabase {
    pub fn new() -> Self {
        CardDatabase {
            cards: HashMap::new(),
        }
    }

    pub fn insert(&mut self, card: CardDef) {
        self.cards.insert(card.id, card);
    }

    pub fn get(&self, id: CardId) -> Option<&CardDef> {
        self.cards.get(&id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<CardId> {
        let target = name.trim();
        self.cards
            .values()
            .find(|card| card.name.eq_ignore_ascii_case(target))
            .map(|card| card.id)
    }
}
