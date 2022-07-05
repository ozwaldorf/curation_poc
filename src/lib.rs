use candid::{candid_method, export_service, CandidType, Deserialize, Principal};
use ic_cdk::caller;
use ic_cdk_macros::*;
use std::collections::HashMap;

mod ledger;
// mod proxy;

/* UPDATE METHODS */

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Event {
    pub operation: String,
    pub details: HashMap<String, ledger::GenericValue>,
}

// insert token transaction
#[update]
#[candid_method(update)]
fn insert(token_id: String, event: Event) -> Result<(), &'static str> {
    ledger::with_mut(|ledger| ledger.insert_and_index(token_id, event))
}

/* QUERY METHODS */

/// query sorted indexes.
///
/// # Arguments
/// * `sort_key` - sort key. Possible options include:
///    - `last_listing` - recently listed tokens.
///    - `last_offer` - recently modified tokens.
///    - `last_sale` - recently sold tokens.
///    - `all` - all indexed tokens.
/// * `page` - page number. If `null`, returns the last (most recent) page of results. Order is backwards
#[query]
#[candid_method]
fn query(sort_key: String, page: Option<usize>) -> Result<Vec<String>, &'static str> {
    ledger::with(|ledger| {
        let size = 64 as usize;
        let indexes = &ledger.sort_index;
        match indexes.get(&sort_key) {
            None => Err("invalid sort key"),
            Some(sorted) => {
                let max_len = sorted.len();
                let page = page.unwrap_or(max_len / size);

                let start = page * size;
                if start > max_len {
                    return Ok(vec![]); // return early if requesting data past whats available
                }

                let mut end = start + size;
                if max_len < end {
                    end = max_len
                }

                Ok(sorted[start..end].to_vec())
            }
        }
    })
}

/* CANISTER METHODS */

#[init]
#[candid_method(init)]
fn init(nft_canister_id: Option<Principal>) {
    ledger::with_mut(|ledger| {
        ledger.nft_canister_id = nft_canister_id.unwrap_or(Principal::management_canister());
        ledger.custodians.push(caller());
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
        write(dir.join("candid/").join("curation.did"), export_candid()).expect("Write failed.");
    }
}
