use candid::{CandidType, Deserialize, Nat, Principal};
use std::collections::HashMap;

pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const PAGE_SIZE_LIMIT: usize = 64;

#[derive(CandidType, Clone, Deserialize, Debug, Hash, Eq, PartialEq)]
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
    NestedContent(Vec<(String, GenericValue)>),
}

/// Query Request
///
/// ### Required Arguments
///
/// * `sort_key` - sort key. Possible options include:
///   - `listing_price` - listing price.
///   - `offer_price` - offer price.
///   - `sale_price` - last sale price
///   - `last_listing` - recently listed tokens.
///   - `last_offer` - recently modified tokens.
///   - `last_sale` - recently sold tokens.
///   - `all` - all indexed tokens.
/// * `page` - page number. If `null`, returns the last (most recent) page of results. Order is backwards
///
/// ### Optional Arguments
///
/// * `count` - number of results to return. Default is 10, max 64
/// * `offset` - For complicated filter queries past the first page (0), specify this parameter to hint the previous request left off at a specific point in the index. Default is 0.
/// * `traits` - filter results by traits. Passed as a vec of (key, value) tuples.
/// * `reverse` - Default: false. If true, returns results in reverse (ascending) order
#[derive(CandidType, Clone, Deserialize)]
pub struct QueryRequest {
    pub sort_key: String,
    pub last_index: Option<usize>,
    pub count: Option<usize>,
    pub traits: Option<Vec<(String, GenericValue)>>,
    pub reverse: Option<bool>,
}

#[derive(CandidType, Clone, Debug)]
pub struct QueryResponse {
    pub total: usize,
    pub last_index: Option<usize>,
    pub data: Vec<TokenData>,
    pub error: Option<String>,
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Event {
    pub nft_canister_id: Principal,
    pub fungible_id: Option<Principal>,
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
#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct Sale {
    pub buyer: Principal,
    pub fungible: Principal,
    pub price: Nat,
    pub time: Nat,
}

#[derive(Default, Clone, Deserialize, Debug, CandidType)]
pub struct TokenData {
    pub id: String,
    pub traits: Option<HashMap<String, GenericValue>>,

    pub offers: Vec<Offer>,
    pub best_offer: Option<Nat>,
    pub price: Option<Nat>,
    pub last_sale: Option<Sale>,

    pub last_listing: Option<Nat>,
    pub last_offer: Option<Nat>,
}

pub type TokenIndex = HashMap<String, Vec<String>>;
pub type GenericIndex = HashMap<GenericValue, Vec<String>>;
