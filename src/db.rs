use crate::types::*;
use candid::{CandidType, Nat, Principal};
use ic_cdk::api::time;
use std::collections::{HashMap, HashSet};

#[derive(CandidType, Clone)]
pub struct Database {
    // pre-sorted indexes
    sort_index: TokenIndex,
    // filter key: generic value: index
    trait_maps: HashMap<String, GenericIndex>,
    // token id: token data
    db: HashMap<String, TokenData>,
}

impl Database {
    pub fn new() -> Self {
        Database {
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
            trait_maps: HashMap::new(),
            db: HashMap::new(),
        }
    }

    pub fn get(&self, token_id: &String) -> Option<&TokenData> {
        self.db.get(token_id)
    }

    pub fn query(&self, request: QueryRequest) -> QueryResponse {
        let mut result = vec![];
        let mut size = request.count.unwrap_or(DEFAULT_PAGE_SIZE);
        if size > PAGE_SIZE_LIMIT {
            size = PAGE_SIZE_LIMIT;
        }

        let indexes = &self.sort_index;
        match indexes.get(&request.sort_key) {
            // if sort key is not found, return empty result
            None => QueryResponse {
                total: 0,
                last_index: None,
                data: result,
                error: Some("Sort key not found".to_string()),
            },
            Some(sorted) => {
                // build hashset of accepted token ids from traits, if provided
                let mut accepted_ids: HashSet<String> = HashSet::new();
                if let Some(traits) = request.traits.clone() {
                    for (key, value) in traits {
                        // check if key val exists in trait map
                        if let Some(trait_key) = self.trait_maps.get(&key) {
                            if let Some(tokens) = trait_key.get(&value) {
                                // if value exists, add to accepted_ids
                                for id in tokens {
                                    accepted_ids.insert(id.clone());
                                }
                            }
                        }
                    }

                    // if no accepted_ids, return empty result
                    if accepted_ids.is_empty() {
                        return QueryResponse {
                            total: 0,
                            last_index: None,
                            data: result,
                            error: Some(
                                "No entries found under the specified trait key/vals".to_string(),
                            ),
                        };
                    }
                }

                let max_len = sorted.len();

                match request.reverse.unwrap_or(false) {
                    false => {
                        // descending order, default
                        let last_index = request.last_index.unwrap_or(max_len);

                        if last_index > max_len {
                            // out of bounds, return nothing!
                            return QueryResponse {
                                total: 0,
                                last_index: None,
                                data: result,
                                error: Some("Page out of bounds".to_string()),
                            };
                        }

                        let mut scanned = 0;
                        let mut index = last_index;
                        while scanned < size && index > 0 {
                            index -= 1;
                            let token = &sorted[index];

                            // check if no filters, or if token is in the set of accepted ids
                            // do nothing if token is not in the set of accepted ids
                            if request.traits.is_none() || accepted_ids.contains(token) {
                                match self.db.get(token) {
                                    Some(token) => {
                                        scanned += 1;
                                        result.push(token.clone());
                                    }
                                    None => {
                                        // unreachable
                                        // db entry not found, should we log for removal?
                                    }
                                }
                            }
                        }

                        QueryResponse {
                            total: max_len,
                            last_index: if index > 0 { Some(index) } else { None },
                            data: result,
                            error: None,
                        }
                    }
                    true => {
                        // ascending order
                        let last_index = request.last_index.unwrap_or_default();

                        if last_index > max_len {
                            // out of bounds, return nothing!
                            return QueryResponse {
                                total: max_len,
                                last_index: None,
                                data: result,
                                error: Some("Page out of bounds".to_string()),
                            };
                        }

                        let mut scanned = 0;
                        let mut index = last_index;
                        while scanned < size && index < max_len {
                            let token = &sorted[index];

                            // check if no filters, or if token is in the set of accepted ids
                            if request.traits.is_none() || accepted_ids.contains(token) {
                                match self.get(token) {
                                    Some(token) => {
                                        scanned += 1;
                                        result.push(token.clone());
                                    }
                                    None => {
                                        // db entry not found, should we log here for removal of the index?
                                    }
                                }
                                // do nothing if token is not in the set of accepted ids
                            }

                            index += 1;
                            scanned += 1;
                        }

                        QueryResponse {
                            total: max_len,
                            last_index: if index < max_len { Some(index) } else { None },
                            data: result,
                            error: None,
                        }
                    }
                }
            }
        }
    }

    fn push_sort_sale(&mut self, token_id: String, price: Nat) {
        let sorted = self.sort_index.get_mut("sale_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if !sorted.contains(&token_id) {
            // key exists, just sort the array.
            // Uses dmsort which is extremely efficient for mostly sorted vectors
            sorted.push(token_id);
            dmsort::sort_by_key(sorted, |id| {
                match db.entry(id.to_string()).or_default().last_sale.clone() {
                    Some(sale) => sale.price.clone(),
                    None => 0.into(),
                }
            });
            // old sort method
            // sorted.sort_by_cached_key(|id| {
            //     match db.entry(id.to_string()).or_default().last_sale.clone() {
            //         Some(sale) => sale.price,
            //         None => 0.into(),
            //     }
            // });
        } else {
            // not found, partition insert into sorted array
            let index = sorted.partition_point(|id| {
                let sale_price = match db.entry(id.clone()).or_default().last_sale.clone() {
                    Some(sale) => sale.price.clone(),
                    None => 0.into(),
                };

                sale_price < price
            });

            sorted.insert(index, token_id.clone());
        }
    }

    fn push_sort_listing(&mut self, token_id: String, amount: Nat) {
        let sorted = self.sort_index.get_mut("listing_price").unwrap();
        let mut db = self.db.clone();

        // check if key exists already
        if sorted.contains(&token_id) {
            // key exists, just sort the array
            // improvement: use dmsort which is extremely efficient at mostly sorted arrays
            dmsort::sort_by_key(sorted, |id| {
                db.entry(id.to_string())
                    .or_default()
                    .price
                    .clone()
                    .unwrap_or_default()
            });
            // old sort method
            // sorted.sort_by_cached_key(|id| {
            //     db.entry(id.to_string())
            //         .or_default()
            //         .price
            //         .clone()
            //         .unwrap_or_default()
            // });
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
        if sorted.contains(&token_id) {
            // key exists, just sort the array
            // improvement: use dmsort which is extremely efficient at mostly sorted arrays
            dmsort::sort_by_key(sorted, |id| {
                db.entry(id.to_string())
                    .or_default()
                    .best_offer
                    .clone()
                    .unwrap_or_default()
            });
            // old sort method
            // sorted.sort_by_cached_key(|id| {
            //     db.entry(id.to_string())
            //         .or_default()
            //         .best_offer
            //         .clone()
            //         .unwrap_or_default()
            // });
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
        // todo: iter from reverse until found, remove index, and push to end
        sort_index.retain(|token| *token != id);
        sort_index.push(id.clone());
    }

    fn remove(&mut self, key: &str, id: String) {
        // remove; time based indexes
        let sort_index = self.sort_index.entry(key.to_string()).or_default();
        sort_index.retain(|token| *token != id);
    }

    fn push_trait(&mut self, token_id: String, name: String, value: GenericValue) {
        let trait_index = self
            .trait_maps
            .entry(name)
            .or_default()
            .entry(value)
            .or_default();
        if !trait_index.contains(&token_id) {
            trait_index.push(token_id);
        }
    }

    pub fn index_event(&mut self, event: Event) -> Result<(), &'static str> {
        let mut token = self.db.entry(event.token_id.clone()).or_default();
        let time = time();

        match event.operation.as_str() {
            "mint" => {
                // load new metadata into canister
                token.id = event.token_id.clone();
                token.traits = event.traits.clone();

                if let Some(traits) = event.traits.clone() {
                    for (k, v) in traits.clone() {
                        self.push_trait(event.token_id.clone(), k, v);
                    }
                }
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
