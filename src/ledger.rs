use crate::types::*;
use candid::{Nat, Principal};
use ic_cdk::api::time;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub static LEDGER: RefCell<Ledger>  = RefCell::new(Ledger::new());
}

pub struct Ledger {
    pub nft_canister_id: Principal,
    pub custodians: Vec<Principal>,
    // pre-sorted indexes
    pub sort_index: Index,
    // filter items: index
    pub filter_maps: HashMap<String, Index>,
    // token id: token data
    pub db: HashMap<String, TokenData>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            nft_canister_id: Principal::management_canister(),
            custodians: vec![],
            // define sort indexes to unwrap get_mut safely
            sort_index: HashMap::from([
                ("listing_price".to_string(), vec![]),
                ("offer_price".to_string(), vec![]),
                ("sale_price".to_string(), vec![]),
                ("last_listing".to_string(), vec![]),
                ("last_offer".to_string(), vec![]),
                ("last_sale".to_string(), vec![]),
                ("all".to_string(), vec![]),
            ]),
            filter_maps: HashMap::new(),
            db: HashMap::new(),
        }
    }

    fn push_sort_sale(&mut self, token_id: String, price: Nat) {
        let sorted = self.sort_index.get_mut("sale_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if !sorted.contains(&token_id) {
            // key exists, just sort the array
            // improvement: use dmsort which is extremely efficient at mostly sorted arrays
            sorted.push(token_id);
            sorted.sort_by_cached_key(|id| {
                match db.entry(id.to_string()).or_default().last_sale.clone() {
                    Some(sale) => sale.price,
                    None => 0.into(),
                }
            });
        } else {
            // not found, partition insert into sorted array
            let index = sorted.partition_point(|id| {
                db.entry(id.clone())
                    .or_default()
                    .price
                    .clone()
                    .unwrap_or_default()
                    < price
            });

            sorted.insert(index, token_id.clone());
        }
    }

    fn push_sort_listing(&mut self, token_id: String, amount: Nat) {
        let sorted = self.sort_index.get_mut("listing_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if !sorted.contains(&token_id) {
            // key exists, just sort the array
            // improvement: use dmsort which is extremely efficient at mostly sorted arrays
            sorted.push(token_id);
            sorted.sort_by_cached_key(|id| {
                db.entry(id.to_string())
                    .or_default()
                    .price
                    .clone()
                    .unwrap_or_default()
            });
        } else {
            // not found, partition insert into sorted array
            let index = sorted.partition_point(|id| {
                db.entry(id.clone())
                    .or_default()
                    .price
                    .clone()
                    .unwrap_or_default()
                    < amount
            });

            sorted.insert(index, token_id.clone());
        }
    }

    fn push_sort_offer(&mut self, token_id: String, amount: Nat) {
        let sorted = self.sort_index.get_mut("offer_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if !sorted.contains(&token_id) {
            // key exists, just sort the array
            // improvement: use dmsort which is extremely efficient at mostly sorted arrays
            sorted.sort_by_cached_key(|id| {
                db.entry(id.to_string())
                    .or_default()
                    .price
                    .clone()
                    .unwrap_or_default()
            });
        } else {
            // not found, partition insert into sorted array
            let index = sorted.partition_point(|id| {
                // research: should we use an Option here? would it be optimized for the null value to be 0 ?
                db.entry(id.clone())
                    .or_default()
                    .best_offer
                    .clone()
                    .unwrap_or_default()
                    < amount
            });

            sorted.insert(index, token_id.clone());
        }
    }

    fn shift_or_push(&mut self, key: &str, id: String) {
        // remove and push; time based indexes
        let sort_index = self.sort_index.entry(key.to_string()).or_default();
        sort_index.retain(|token| *token != id);
        sort_index.push(id.clone());
    }

    fn remove(&mut self, key: &str, id: String) {
        // remove; time based indexes
        let sort_index = self.sort_index.entry(key.to_string()).or_default();
        sort_index.retain(|token| *token != id);
    }

    pub fn index_event(&mut self, event: Event) -> Result<(), &'static str> {
        let mut token = self.db.entry(event.token_id.clone()).or_default();
        let time = time();

        match event.operation.as_str() {
            "mint" => {
                // load new metadata into canister

                // if let Some(traits) = event.traits.clone() {
                //     for (_k, _v) in traits.clone() {
                //         // TODO: insert to trait map
                //     }
                // }
                token.id = event.token_id.clone();
                token.traits = event.traits.clone();
            }

            "makeListing" => {
                // update db entry
                token.price = event.price.clone();
                token.last_listing = Some(time.into());

                let price = event.price.unwrap_or(Nat::from(0));

                // index listing price
                self.push_sort_listing(event.token_id.clone(), price);
                // update last listing index
                self.shift_or_push("last_listing", event.token_id.clone());
            }
            "cancelListing" => {
                // update db entry
                token.price = None;

                // remove from listing index
                self.remove("listing_price", event.token_id.clone());
                // remove from last listing index
                self.remove("last_listing", event.token_id.clone());
            }

            "makeOffer" => {
                // update db entry
                token.last_offer = Some(time.into());
                token.best_offer = event.price.clone();
                let price = event.price.unwrap_or(Nat::from(0));

                token.offers.push(Offer {
                    buyer: event.buyer.unwrap_or(Principal::anonymous()),
                    fungible: event
                        .fungible_id
                        .unwrap_or(Principal::management_canister()),
                    price: price.clone(),
                });

                // index offer price
                self.push_sort_offer(event.token_id.clone(), price);

                // TODO: offer price index
                self.shift_or_push("last_offer", event.token_id.clone());
            }
            "cancelOffer" => {
                // TODO:
                // remove from last offer index and offer price index if its the only one left (cancelled only offer)
                // If not, leave it in the index, and re-sort the offer price index (cancelled offer but others remain)
                match event.buyer {
                    Some(buyer) => {
                        token.offers.retain(|o| o.buyer != buyer);

                        match &token.best_offer {
                            Some(best_offer) => match token.offers.last() {
                                Some(offer) => {
                                    if offer.price.clone() > best_offer.clone() {
                                        token.best_offer = Some(offer.price.clone());
                                    }
                                }
                                None => {
                                    token.best_offer = None;
                                }
                            },
                            None => unreachable!(),
                        }
                    }
                    None => {}
                }

                if token.offers.is_empty() {
                    // remove from last offer and offer price indexes if no more offers on the token
                    self.remove("last_offer", event.token_id.clone());
                    self.remove("offer_price", event.token_id.clone());
                } else {
                    // re-sort offer price index

                    // find best offer
                    let mut best_offer = None;
                    for offer in token.offers.iter() {
                        if best_offer.is_none() {
                            best_offer = Some(offer.price.clone());
                        } else if offer.price.clone() > best_offer.clone().unwrap() {
                            best_offer = Some(offer.price.clone());
                        }
                    }

                    // update best offer
                    token.best_offer = best_offer.clone();

                    // sort offer price index
                    self.push_sort_offer(event.token_id.clone(), best_offer.unwrap());
                }
            }

            "directBuy" => {
                // update db entry
                token.last_sale = Some(Sale {
                    buyer: event.buyer.unwrap_or(Principal::anonymous()),
                    fungible: event
                        .fungible_id
                        .unwrap_or(Principal::management_canister()),
                    price: event.price.clone().unwrap_or_default(),
                    time: time.into(),
                });
                token.price = None;

                match event.buyer {
                    Some(buyer) => {
                        if !token.offers.is_empty() {
                            token.offers.retain(|o| o.buyer != buyer);

                            match &token.best_offer {
                                Some(best_offer) => match token.offers.last() {
                                    Some(offer) => {
                                        if offer.price.clone() > best_offer.clone() {
                                            token.best_offer = Some(offer.price.clone());
                                        }
                                    }
                                    None => {
                                        token.best_offer = None;
                                    }
                                },
                                None => {}
                            }

                            if token.offers.is_empty() {
                                // remove from last offer and offer price indexes if no more offers on the token
                                self.remove("last_offer", event.token_id.clone());
                                self.remove("offer_price", event.token_id.clone());
                            } else {
                                // re-sort offer price index

                                // find best offer
                                let mut best_offer = None;
                                for offer in token.offers.iter() {
                                    if best_offer.is_none() {
                                        best_offer = Some(offer.price.clone());
                                    } else if offer.price.clone() > best_offer.clone().unwrap() {
                                        best_offer = Some(offer.price.clone());
                                    }
                                }

                                // update best offer
                                token.best_offer = best_offer.clone();

                                // sort offer price index
                                self.push_sort_offer(event.token_id.clone(), best_offer.unwrap());
                            }
                        }
                    }
                    None => {}
                }

                // update sale price index
                self.push_sort_sale(event.token_id.clone(), event.price.unwrap());
                // remove from listing index
                self.remove("listing_price", event.token_id.clone());
                // update last sale index
                self.shift_or_push("last_sale", event.token_id.clone());
            }
            "acceptOffer" => {
                // update db entry
                token.last_sale = Some(Sale {
                    buyer: event.buyer.unwrap_or(Principal::anonymous()),
                    fungible: event
                        .fungible_id
                        .unwrap_or(Principal::management_canister()),
                    price: event.price.clone().unwrap_or_default(),
                    time: time.into(),
                });
                token.price = None;

                // TODO: mirror offer removal logic
                match event.buyer {
                    Some(buyer) => {
                        token.offers.retain(|o| o.buyer != buyer);

                        match &token.best_offer {
                            Some(best_offer) => match token.offers.last() {
                                Some(offer) => {
                                    if offer.price.clone() > best_offer.clone() {
                                        token.best_offer = Some(offer.price.clone());
                                    }
                                }
                                None => {
                                    token.best_offer = None;
                                }
                            },
                            None => {}
                        }

                        if token.offers.is_empty() {
                            // remove from last offer and offer price indexes if no more offers on the token
                            self.remove("last_offer", event.token_id.clone());
                            self.remove("offer_price", event.token_id.clone());
                        } else {
                            // re-sort offer price index

                            // find best offer
                            let mut best_offer = None;
                            for offer in token.offers.iter() {
                                if best_offer.is_none() {
                                    best_offer = Some(offer.price.clone());
                                } else if offer.price.clone() > best_offer.clone().unwrap() {
                                    best_offer = Some(offer.price.clone());
                                }
                            }

                            // update best offer
                            token.best_offer = best_offer.clone();

                            // sort offer price index
                            self.push_sort_offer(event.token_id.clone(), best_offer.unwrap());
                        }
                    }
                    None => {}
                }

                // update sale price index
                self.push_sort_sale(event.token_id.clone(), event.price.unwrap());
                // remove from listing price index
                self.remove("listing_price", event.token_id.clone());
                // update last sale index
                self.shift_or_push("last_sale", event.token_id.clone());
            }
            _ => {
                return Err("invalid operation");
            }
        }
        self.shift_or_push("all", event.token_id.clone());

        Ok(())
    }
}

pub fn with<T, F: FnOnce(&Ledger) -> T>(f: F) -> T {
    LEDGER.with(|ledger| f(&ledger.borrow()))
}

pub fn with_mut<T, F: FnOnce(&mut Ledger) -> T>(f: F) -> T {
    LEDGER.with(|ledger| f(&mut ledger.borrow_mut()))
}
