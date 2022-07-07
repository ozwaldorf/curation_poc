use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::time;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub static LEDGER: RefCell<Ledger>  = RefCell::new(Ledger::new());
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub enum GenericValue {
    BoolContent(bool),
    TextContent(String),
    BlobContent(Vec<u8>),
    Principal(Principal),
    Nat8Content(u8),
    Nat16Content(u16),
    Nat32Content(u32),
    Nat64Content(u64),
    NatContent(Nat),
    Int8Content(i8),
    Int16Content(i16),
    Int32Content(i32),
    Int64Content(i64),
    FloatContent(f64), // motoko only support f64
    NestedContent(Vec<(String, GenericValue)>),
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Event {
    pub nft_canister_id: Principal,
    pub token_id: String,
    pub operation: String,

    pub traits: Option<HashMap<String, GenericValue>>,
    pub price: Option<Nat>,
    pub buyer: Option<Principal>,
    pub seller: Option<Principal>,
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Offer {
    pub buyer: Principal,
    pub fungible: Principal,
    pub price: Nat,
}

#[derive(Default, Clone, Deserialize, Debug, CandidType)]
pub struct TokenData {
    pub id: String,
    pub traits: Option<HashMap<String, GenericValue>>,
    // pub events: Vec<Event>, // do we want to store txn history at all??
    pub offers: Vec<Offer>,
    pub price: Option<Nat>,
    pub best_offer: Option<Nat>,
    pub last_listing: Option<Nat>,
    pub last_offer: Option<Nat>,
    pub last_sale: Option<Nat>,
}

pub type Index = HashMap<String, Vec<String>>;

pub struct Ledger {
    pub nft_canister_id: Principal,
    pub custodians: Vec<Principal>,
    pub sort_index: Index,
    pub filter_maps: HashMap<String, Index>,
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
                ("last_listing".to_string(), vec![]),
                ("last_offer".to_string(), vec![]),
                ("last_sale".to_string(), vec![]),
                ("all".to_string(), vec![]),
            ]),
            filter_maps: HashMap::new(),
            db: HashMap::new(),
        }
    }

    fn push_sort_listing(&mut self, token_id: String) {
        let sorted = self.sort_index.get_mut("listing_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if !sorted.contains(&token_id) {
            sorted.push(token_id);
        }

        sorted.sort_by_cached_key(
            |id| match db.entry(id.to_string()).or_default().price.clone() {
                Some(price) => price,
                None => Nat::from(0),
            },
        );
    }

    fn push_sort_offer(&mut self, token_id: String) {
        let sorted = self.sort_index.get_mut("offer_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if !sorted.contains(&token_id) {
            sorted.push(token_id);
        }

        sorted.sort_by_cached_key(|id| {
            match db.entry(id.to_string()).or_default().best_offer.clone() {
                Some(price) => price,
                None => Nat::from(0),
            }
        });
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
                token.price = event.price;
                token.last_listing = Some(time.into());

                // index listing price
                self.push_sort_listing(event.token_id.clone());
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

                token.offers.push(Offer {
                    buyer: event.buyer.unwrap_or(Principal::management_canister()),
                    fungible: event.nft_canister_id.clone(),
                    price: event.price.unwrap_or(Nat::from(0)),
                });

                // index offer price
                self.push_sort_offer(event.token_id.clone());

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
                    token.best_offer = best_offer;

                    // sort offer price index
                    self.push_sort_offer(event.token_id.clone());
                }
            }

            "directBuy" => {
                // update db entry
                token.price = None;
                token.last_sale = Some(time.into());

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
                    token.best_offer = best_offer;

                    // sort offer price index
                    self.push_sort_offer(event.token_id.clone());
                }

                // remove from listing index
                self.remove("listing_price", event.token_id.clone());
                // update last sale index
                self.shift_or_push("last_sale", event.token_id.clone());
            }
            "acceptOffer" => {
                // update db entry
                token.price = None;
                token.last_sale = Some(time.into());

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
                    token.best_offer = best_offer;

                    // sort offer price index
                    self.push_sort_offer(event.token_id.clone());
                }

                // remove from listing index
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
