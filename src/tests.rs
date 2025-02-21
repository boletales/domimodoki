use std::collections::HashMap;

use crate::{
    card::Card,
    card_instance::{CardAddress::*, CardInstance, CardInstanceId},
    expansions::{base::*, basic_supply::*},
    game::Game,
    player::{PlayerData, PlayerId},
    zone::Zone::*,
};

pub fn setup<'a>() -> Game<'a> {
    let p0 = PlayerData {
        id: PlayerId::new(0),
        name: "Alice".to_owned(),
        deck: vec![],
        hand: vec![],
        play: vec![],
        pending: vec![],
        aside: vec![],
        revealed: vec![],
        discard: vec![],
    };

    let p1 = PlayerData {
        id: PlayerId::new(1),
        name: "Bob".to_owned(),
        deck: vec![],
        hand: vec![],
        play: vec![],
        pending: vec![],
        aside: vec![],
        revealed: vec![],
        discard: vec![],
    };

    Game {
        players: vec![p0, p1],
        supply: vec![],
        trash: vec![],
        turn: 0,
        stack: vec![],
    }
}

fn supply() -> HashMap<String, Card> {
    let mut supply = basic_supply();
    supply.extend(base_set());
    supply
}

/*
手札：銅貨、銀貨、金貨、堀、山賊
山札：屋敷、民兵、役人、庭園、市場
捨て札：礼拝堂、前駆者、魔女、職人、鉱山
場札：地下貯蔵庫、密猟者、衛兵
処理中：商人、玉座の間
 */
fn setup2(supply: &HashMap<String, Card>) -> Game<'_> {
    let mut game = setup();
    let alice = &mut game.players[0];
    let mut cid = 0;
    let hand = ["Copper", "Silver", "Gold", "Moat", "Bandit"];
    let deck = ["Estate", "Militia", "Bureaucrat", "Gardens", "Market"];
    let discard = ["Chapel", "Harbinger", "Witch", "Artisan", "Mine"];
    let play = ["Cellar", "Poacher", "Sentry"];
    let pending = ["Merchant", "Throne Room"];
    for card in hand.iter() {
        alice.hand.push(CardInstance {
            id: CardInstanceId::new(cid),
            card: &supply[*card],
            address: PlayerOwned(alice.id, Hand),
        });
        cid += 1;
    }
    for card in deck.iter() {
        alice.deck.push(CardInstance {
            id: CardInstanceId::new(cid),
            card: &supply[*card],
            address: PlayerOwned(alice.id, Deck),
        });
        cid += 1;
    }
    for card in discard.iter() {
        alice.discard.push(CardInstance {
            id: CardInstanceId::new(cid),
            card: &supply[*card],
            address: PlayerOwned(alice.id, Discard),
        });
        cid += 1;
    }
    for card in play.iter() {
        alice.play.push(CardInstance {
            id: CardInstanceId::new(cid),
            card: &supply[*card],
            address: PlayerOwned(alice.id, Play),
        });
        cid += 1;
    }
    for card in pending.iter() {
        alice.pending.push(CardInstance {
            id: CardInstanceId::new(cid),
            card: &supply[*card],
            address: PlayerOwned(alice.id, Pending),
        });
        cid += 1;
    }
    game
}
mod resolvers {
    mod cardname {
        use crate::{
            card::CardType::*,
            card_instance::{CardAddress::*, CardInstance, CardInstanceId},
            expansions::base::*,
            expansions::basic_supply::*,
            number::{Number::*, NumberRange::*},
            selector::{CardNameSelector, CardSelector},
            tests::setup,
            zone::Zone::*,
        };
        #[test]
        fn cardname_exact() {
            let mut game = setup();
            let copper = copper();
            let alice = &mut game.players[0];
            let hand = [&copper, &copper, &copper];
            for (i, card) in hand.iter().enumerate() {
                alice.hand.push(CardInstance {
                    id: CardInstanceId::new(i),
                    card,
                    address: PlayerOwned(alice.id, Hand),
                });
            }
            let selector = CardSelector {
                name: CardNameSelector::Name("Copper".to_owned()),
                zone: vec![Hand],
            };
            let alice = &game.players[0];
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn cardname_cost() {
            let mut game = setup();
            let copper = copper();
            let silver = silver();
            let gold = gold();
            let alice = &mut game.players[0];
            let hand = [&copper, &silver, &silver, &gold];
            for (i, card) in hand.iter().enumerate() {
                alice.hand.push(CardInstance {
                    id: CardInstanceId::new(i),
                    card,
                    address: PlayerOwned(alice.id, Hand),
                });
            }
            let selector = CardSelector {
                name: CardNameSelector::Cost(Box::new(Exact(Constant(3)))),
                zone: vec![Hand],
            };
            let alice = &game.players[0];
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn cardname_costupto() {
            let mut game = setup();
            let copper = copper();
            let silver = silver();
            let gold = gold();
            let alice = &mut game.players[0];
            let hand = [&copper, &silver, &silver, &gold];
            for (i, card) in hand.iter().enumerate() {
                alice.hand.push(CardInstance {
                    id: CardInstanceId::new(i),
                    card,
                    address: PlayerOwned(alice.id, Hand),
                });
            }
            let selector = CardSelector {
                name: CardNameSelector::Cost(Box::new(UpTo(Constant(3)))),
                zone: vec![Hand],
            };
            let alice = &game.players[0];
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 3);
        }

        #[test]
        fn cardname_or() {
            let mut game = setup();
            let copper = copper();
            let silver = silver();
            let gold = gold();
            let alice = &mut game.players[0];
            let hand = [&copper, &silver, &gold];
            for (i, card) in hand.iter().enumerate() {
                alice.hand.push(CardInstance {
                    id: CardInstanceId::new(i),
                    card,
                    address: PlayerOwned(alice.id, Hand),
                });
            }
            let selector = CardSelector {
                name: CardNameSelector::NameOr(vec![
                    CardNameSelector::Name("Copper".to_owned()),
                    CardNameSelector::Name("Silver".to_owned()),
                ]),
                zone: vec![Hand],
            };
            let alice = &game.players[0];
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn cardname_type() {
            let mut game = setup();
            let copper = copper();
            let silver = silver();
            let gold = gold();
            let moat = moat();
            let bandit = bandit();

            let alice = &mut game.players[0];
            let hand = [&copper, &silver, &gold, &moat, &bandit];
            for (i, card) in hand.iter().enumerate() {
                alice.hand.push(CardInstance {
                    id: CardInstanceId::new(i),
                    card,
                    address: PlayerOwned(alice.id, Hand),
                });
            }
            let selector_t = CardSelector {
                name: CardNameSelector::HasType(Treasure),
                zone: vec![Hand],
            };
            let selector_a = CardSelector {
                name: CardNameSelector::HasType(Action),
                zone: vec![Hand],
            };
            let selector_r = CardSelector {
                name: CardNameSelector::HasType(Reaction),
                zone: vec![Hand],
            };
            let alice = &game.players[0];
            let result_t = game.resolve_selector(alice.id, &selector_t);
            let result_a = game.resolve_selector(alice.id, &selector_a);
            let result_r = game.resolve_selector(alice.id, &selector_r);
            assert_eq!(result_t.len(), 3);
            assert_eq!(result_a.len(), 2);
            assert_eq!(result_r.len(), 1);
        }

        #[test]
        fn cardname_any() {
            let mut game = setup();
            let copper = copper();
            let silver = silver();
            let gold = gold();
            let moat = moat();
            let bandit = bandit();

            let alice = &mut game.players[0];
            let hand = [&copper, &silver, &gold, &moat, &bandit];
            for (i, card) in hand.iter().enumerate() {
                alice.hand.push(CardInstance {
                    id: CardInstanceId::new(i),
                    card,
                    address: PlayerOwned(alice.id, Hand),
                });
            }
            let selector = CardSelector {
                name: CardNameSelector::Any,
                zone: vec![Hand],
            };
            let alice = &game.players[0];
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 5);
        }
    }
    mod cardselector {
        use std::vec;

        use crate::{
            number::{Number::*, NumberRange::*},
            selector::{CardNameSelector::*, CardSelector},
            tests::{setup2, supply},
            zone::Zone::*,
        };

        #[test]
        fn cardselector_hand() {
            let supply = supply();
            let game = setup2(&supply);
            let alice = &game.players[0];
            let selector = CardSelector {
                name: Any,
                zone: vec![Hand],
            };
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 5);
        }

        #[test]
        fn cardselector_all() {
            let supply = supply();
            let game = setup2(&supply);
            let alice = &game.players[0];
            let selector = CardSelector {
                name: Cost(Box::new(Exact(Constant(2)))),
                zone: vec![AllMyCards],
            };
            let result = game.resolve_selector(alice.id, &selector);
            assert_eq!(result.len(), 4); // 屋敷、地下貯蔵庫、礼拝堂、堀
        }
    }
}

mod base {
    use crate::{
        card_instance::{CardAddress::*, CardInstance, CardInstanceId},
        tests::{setup2, supply},
        zone::Zone::*,
    };

    #[test]
    fn gardens_vp() {
        let supply = supply();
        let mut game = setup2(&supply);
        let alice = &game.players[0];
        let vp = game.calculate_vp(alice.id);
        assert_eq!(vp, 3); // 屋敷(1VP)x1 + 庭園(20枚: 2VP)x1

        let alice = &mut game.players[0];
        for i in 20..28 {
            alice.deck.push(CardInstance {
                id: CardInstanceId::new(i),
                card: &supply["Copper"],
                address: PlayerOwned(alice.id, Deck),
            });
        }

        let alice = &game.players[0];
        let vp = game.calculate_vp(alice.id);
        assert_eq!(vp, 3); // 屋敷(1VP)x1 + 庭園(28枚: 2VP)x1

        let alice = &mut game.players[0];
        for i in 28..30 {
            alice.deck.push(CardInstance {
                id: CardInstanceId::new(i),
                card: &supply["Copper"],
                address: PlayerOwned(alice.id, Deck),
            });
        }

        let alice = &game.players[0];
        let vp = game.calculate_vp(alice.id);
        assert_eq!(vp, 4); // 屋敷(1VP)x1 + 庭園(30枚: 3VP)x1
    }
}
