use crate::types::*;
use candid::{candid_method, export_service, Principal};
use ic_cdk::caller;
use ic_cdk_macros::*;
use std::vec;

mod db;
mod ledger;
mod types;
// mod proxy;

/* QUERY METHODS */

/// query sorted indexes.
///
/// # Arguments
/// * `request` - query request.
#[query]
#[candid_method(query)]
fn query(request: QueryRequest) -> QueryResponse {
    ledger::with(|ledger| ledger.db.query(request))
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

        ledger.db.index_event(event)
    })
}

/// batch insert token transactions
#[update]
#[candid_method(update)]
fn batch_insert(events: Vec<Event>) -> Result<(), &'static str> {
    ledger::with_mut(|ledger| {
        for event in events {
            if event.nft_canister_id != ledger.nft_canister_id {
                return Err("Not accepting data for this canister");
            }
            match ledger.db.index_event(event) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
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
