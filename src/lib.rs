use std::vec;

use candid::{candid_method, export_service, CandidType, Principal};
use ic_cdk::caller;
use ic_cdk_macros::*;
use ledger::TokenData;

mod ledger;
// mod proxy;

const PAGE_SIZE: usize = 10;

/* QUERY METHODS */

#[derive(Clone, Debug, CandidType)]
pub struct QueryResponse {
    total: usize,
    data: Vec<TokenData>,
}

/// query sorted indexes.
///
/// # Arguments
/// * `sort_key` - sort key. Possible options include:
///   - `listing_price` - listing price.
///   - `offer_price` - offer price.
///   - `last_listing` - recently listed tokens.
///   - `last_offer` - recently modified tokens.
///   - `last_sale` - recently sold tokens.
///   - `all` - all indexed tokens.
/// * `page` - page number. If `null`, returns the last (most recent) page of results. Order is backwards
#[query]
#[candid_method]
fn query(sort_key: String, page: usize) -> QueryResponse {
    ledger::with(|ledger| {
        let mut result = vec![];

        let indexes = &ledger.sort_index;
        match indexes.get(&sort_key) {
            // if sort key is not found, return empty result
            None => QueryResponse {
                total: 0,
                data: result,
            },
            Some(sorted) => {
                let max_len = sorted.len();

                let end = max_len - (PAGE_SIZE * page);
                if end > max_len {
                    // out of bounds, return nothing!
                    return QueryResponse {
                        total: max_len,
                        data: result,
                    };
                }

                let start;
                if end < PAGE_SIZE {
                    // out of bounds, go as far as we can!
                    start = 0;
                } else {
                    start = end - PAGE_SIZE;
                }

                let mut index = end;
                while index > start && index != 0 {
                    index -= 1;

                    match ledger.db.get(&sorted[index].to_string()) {
                        Some(token) => result.push(token.clone()),
                        None => (),
                    }
                }

                QueryResponse {
                    total: max_len,
                    data: result,
                }
            }
        }
    })
}

/* UPDATE METHODS */

/// insert token transaction
#[update]
#[candid_method(update)]
fn insert(event: ledger::Event) -> Result<(), &'static str> {
    ledger::with_mut(|ledger| {
        if event.nft_canister_id != ledger.nft_canister_id {
            return Err("Not accepting data for this canister");
        }

        ledger.index_event(event)
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

// TODO: Upgrade logic

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
