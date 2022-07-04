use candid::{candid_method, export_service, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;
use std::{cell::RefCell, collections::HashMap};

mod ledger {
    use super::*;

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
                sort_index: HashMap::new(),
                filter_maps: HashMap::new(),
            }
        }

        pub fn remove_and_push(&mut self, key: &str, id: String) {
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

        pub fn insert_and_index(
            &mut self,
            token_id: String,
            event: Event,
        ) -> Result<(), &'static str> {
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

                    self.remove_and_push("recent_listings", token_id.clone());
                }
                "cancelListing" => {
                    self.remove("recent_listings", token_id.clone());
                }

                "makeOffer" => {
                    self.remove_and_push("recent_offers", token_id.clone());
                }
                "cancelOffer" => {
                    self.remove("recent_offers", token_id.clone());
                }

                "directBuy" => {
                    self.remove_and_push("recent_sales", token_id.clone());
                }
                "acceptOffer" => {
                    self.remove_and_push("recent_sales", token_id.clone());
                }
                _ => {
                    return Err("invalid operation");
                }
            }
            self.remove_and_push("all", token_id.clone());

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
}

// insert token transaction
#[update]
#[candid_method(update)]
fn insert(token_id: String, event: ledger::Event) -> Result<(), &'static str> {
    ledger::with_mut(|ledger| ledger.insert_and_index(token_id, event))
}

/// query sorted indexes.
///
/// # Arguments
/// * `sort_key` - sort key. Possible options include:
///    - `recently_listed` - recently listed tokens.
///    - `recently_modified` - recently modified tokens.
///    - `listing_price` - listing price.
/// * `page` - page number. If `null`, returns the last page of results
#[query]
#[candid_method]
fn query(sort_key: String, page: Option<u64>) -> Result<Vec<String>, &'static str> {
    ledger::with(|ledger| {
        let page_size = 64;
        let indexes = &ledger.sort_index;
        match indexes.get(&sort_key) {
            Some(sorted) => {
                let max_len = sorted.len() as u64;
                let page = page.unwrap_or(max_len / page_size);
                let page_start = page * page_size;
                let mut page_end = page_start + page_size;
                if max_len < page_end {
                    page_end = max_len
                }

                return Ok(sorted[page_start as usize..page_end as usize].to_vec());
            }
            None => return Err("invalid sort key"),
        }
    })
}

#[init]
#[candid_method(init)]
fn init(nft_canister_id: Option<Principal>) {
    ledger::with_mut(|ledger| {
        ledger.nft_canister_id = nft_canister_id.unwrap_or(Principal::management_canister());
        ledger.custodians.push(ic_cdk::caller());
    });
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        write(dir.join("curation.did"), export_candid()).expect("Write failed.");
    }
}