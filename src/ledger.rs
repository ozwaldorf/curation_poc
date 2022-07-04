use candid::{CandidType, Deserialize, Nat, Principal};
use std::{cell::RefCell, collections::HashMap};

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
    pub operation: String,
    pub details: HashMap<String, GenericValue>,
}

pub type SortIndex = HashMap<String, Vec<String>>;

pub struct Ledger {
    pub nft_canister_id: Principal,
    pub custodians: Vec<Principal>,
    pub sort_index: SortIndex,
    pub filter_maps: HashMap<String, HashMap<String, Vec<String>>>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            nft_canister_id: Principal::management_canister(),
            custodians: vec![],
            sort_index: HashMap::from([
                ("last_listing".to_string(), vec![]),
                ("last_offer".to_string(), vec![]),
                ("last_sale".to_string(), vec![]),
                ("all".to_string(), vec![]),
            ]),
            filter_maps: HashMap::new(),
        }
    }

    pub fn push(&mut self, key: &str, id: String) {
        // remove and push; time based indexes
        let sort_index = self.sort_index.entry(key.to_string()).or_default();
        if let Some(index) = sort_index.iter().position(|token| *token == id.clone()) {
            sort_index.remove(index);
        }
        sort_index.push(id.clone());
    }

    fn remove(&mut self, key: &str, id: String) {
        // remove; time based indexes
        let sort_index = self.sort_index.entry(key.to_string()).or_default();
        if let Some(index) = sort_index.iter().position(|token| *token == id.clone()) {
            sort_index.remove(index);
        }
    }

    pub fn insert_and_index(&mut self, token_id: String, event: Event) -> Result<(), &'static str> {
        /*
        details:
            - token id
            - nft_canister_id
            - price
        */
        match event.operation.as_str() {
            // load new metadata into canister
            "mint" => {
                // TODO: grab metadata from nft contract, insert to trait filter map
            }

            "makeListing" => {
                // TODO: price index

                self.push("last_listing", token_id.clone());
            }
            "cancelListing" => {
                self.remove("last_listing", token_id.clone());
            }

            "makeOffer" => {
                self.push("last_offer", token_id.clone());
            }
            "cancelOffer" => {
                self.remove("last_offer", token_id.clone());
            }

            "directBuy" => {
                self.push("last_sale", token_id.clone());
            }
            "acceptOffer" => {
                self.push("last_sale", token_id.clone());
            }
            _ => {
                return Err("invalid operation");
            }
        }
        self.push("all", token_id.clone());

        Ok(())
    }
}

thread_local! {
    pub static LEDGER: RefCell<Ledger>  = RefCell::new(Ledger::new());
}

pub fn with<T, F: FnOnce(&Ledger) -> T>(f: F) -> T {
    LEDGER.with(|ledger| f(&ledger.borrow()))
}

pub fn with_mut<T, F: FnOnce(&mut Ledger) -> T>(f: F) -> T {
    LEDGER.with(|ledger| f(&mut ledger.borrow_mut()))
}
