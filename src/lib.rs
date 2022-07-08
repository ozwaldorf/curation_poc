use crate::types::*;
use candid::{candid_method, export_service, Principal};
use ic_cdk::caller;
use ic_cdk_macros::*;
use std::vec;

mod ledger;
mod types;
// mod proxy;

const DEFAULT_PAGE_SIZE: usize = 10;
const PAGE_SIZE_LIMIT: usize = 64;

/* QUERY METHODS */

/// query sorted indexes.
///
/// # Arguments
/// * `request` - query request.
#[query]
#[candid_method]
fn query(request: QueryRequest) -> QueryResponse {
    ledger::with(|ledger| {
        let mut result = vec![];
        let mut size = request.count.unwrap_or(DEFAULT_PAGE_SIZE);
        if size > PAGE_SIZE_LIMIT {
            size = PAGE_SIZE_LIMIT;
        }

        let indexes = &ledger.sort_index;
        match indexes.get(&request.sort_key) {
            // if sort key is not found, return empty result
            None => QueryResponse {
                total: 0,
                data: result,
                error: Some("Sort key not found".to_string()),
            },
            Some(sorted) => {
                let max_len = sorted.len();

                match request.reverse_order.unwrap_or(false) {
                    false => {
                        // descending order, default

                        let end = max_len - (size * request.page);
                        if end > max_len {
                            // out of bounds, return nothing!
                            return QueryResponse {
                                total: max_len,
                                data: result,
                                error: Some("Page out of bounds".to_string()),
                            };
                        }

                        let start;
                        if end < size {
                            // out of bounds, go as far as we can!
                            start = 0;
                        } else {
                            start = end - size;
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
                            error: None,
                        }
                    }
                    true => {
                        // ascending order

                        let start = size * request.page;
                        if start > max_len {
                            // out of bounds, return nothing!
                            return QueryResponse {
                                total: max_len,
                                data: result,
                                error: Some("Page out of bounds".to_string()),
                            };
                        }

                        let end;
                        if start + size > max_len {
                            // out of bounds, go as far as we can!
                            end = max_len;
                        } else {
                            end = start + size;
                        }

                        let mut index = start;
                        while index < end {
                            match ledger.db.get(&sorted[index].to_string()) {
                                Some(token) => result.push(token.clone()),
                                None => (),
                            }
                            index += 1;
                        }

                        QueryResponse {
                            total: max_len,
                            data: result,
                            error: None,
                        }
                    }
                }
            }
        }
    })
}

/* UPDATE METHODS */

/// insert token transaction
#[update]
#[candid_method(update)]
fn insert(event: Event) -> Result<(), &'static str> {
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

/// Run `cargo test` to generate the updated candid file in `PROJECT_ROOT/candid/curation.did`
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
