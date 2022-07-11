use crate::types::*;
use candid::{candid_method, export_service, Principal};
use ic_cdk::caller;
use ic_cdk_macros::*;
use std::{collections::HashSet, vec};

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
        let res = match indexes.get(&request.sort_key) {
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
                        if let Some(trait_key) = ledger.trait_maps.get(&key) {
                            if let Some(tokens) = trait_key.get(&value) {
                                // if value exists, add to accepted_ids
                                for id in tokens {
                                    accepted_ids.insert(id.clone());
                                }
                            }
                        }
                    }

                    ic_cdk::println!("accepted_ids: {:?}", accepted_ids);

                    // if no accepted_ids, return empty result
                    if accepted_ids.is_empty() {
                        return QueryResponse {
                            total: 0,
                            last_index: None,
                            data: result,
                            error: Some(
                                "No tokens found under the specified trait key/vals".to_string(),
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
                            if request.traits.is_none() || accepted_ids.contains(token) {
                                match ledger.db.get(token) {
                                    Some(token) => {
                                        scanned += 1;
                                        result.push(token.clone());
                                    }
                                    None => {
                                        // db entry not found, should we log for removal?
                                    }
                                }
                                // do nothing if token is not in the set of accepted ids
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
                                match ledger.db.get(token) {
                                    Some(token) => {
                                        scanned += 1;
                                        result.push(token.clone());
                                    }
                                    None => {
                                        // db entry not found, should we log here for removal?
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
        };

        res
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
