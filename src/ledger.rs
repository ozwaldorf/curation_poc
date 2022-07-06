use candid::{CandidType, Deserialize, Nat, Principal};
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

pub type SortIndex = HashMap<String, Vec<String>>;

#[derive(Default)]
pub struct TokenData {
    pub traits: Option<HashMap<String, GenericValue>>,
    // pub events: Vec<Event>, // do we want to store txn history at all??
    pub price: Option<Nat>,
    pub best_offer: Option<Nat>,
    pub last_listing: Option<Nat>,
    pub last_offer: Option<Nat>,
    pub last_sale: Option<Nat>,
}

pub type TokenCache = HashMap<String, TokenData>;

pub struct Ledger {
    pub nft_canister_id: Principal,
    pub custodians: Vec<Principal>,
    pub sort_index: SortIndex,
    pub filter_maps: HashMap<String, HashMap<String, Vec<String>>>,
    pub token_cache: TokenCache,
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
            token_cache: HashMap::new(),
        }
    }

    fn push(&mut self, key: &str, id: String) {
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
        /*
        details:
            - token id: string
            - nft_canister_id: principal
            - price: opt nat
            - buyer: opt principal
            - seller: opt principal
        */
        let mut token = self.token_cache.entry(event.token_id.clone()).or_default();

        match event.operation.as_str() {
            // load new metadata into canister
            "mint" => {
                // if let Some(traits) = event.traits.clone() {
                //     for (_k, _v) in traits.clone() {
                //         // TODO: insert to trait map
                //     }
                // }

                token.traits = event.traits.clone();
                // TODO: grab metadata from nft contract, insert to trait filter map
            }

            // update indexes
            "makeListing" => {
                // TODO: price index

                token.price = event.price;

                self.push("last_listing", event.token_id.clone());
            }
            "cancelListing" => {
                token.price = None;

                // TODO: price index
                self.remove("last_listing", event.token_id.clone());
            }

            "makeOffer" => {
                // TODO: offer price index
                self.push("last_offer", event.token_id.clone());
            }
            "cancelOffer" => {
                self.remove("last_offer", event.token_id.clone());
            }

            "directBuy" => {
                self.push("last_sale", event.token_id.clone());
            }
            "acceptOffer" => {
                self.push("last_sale", event.token_id.clone());
            }
            _ => {
                return Err("invalid operation");
            }
        }
        self.push("all", event.token_id.clone());

        Ok(())
    }
}

pub fn with<T, F: FnOnce(&Ledger) -> T>(f: F) -> T {
    LEDGER.with(|ledger| f(&ledger.borrow()))
}

pub fn with_mut<T, F: FnOnce(&mut Ledger) -> T>(f: F) -> T {
    LEDGER.with(|ledger| f(&mut ledger.borrow_mut()))
}
