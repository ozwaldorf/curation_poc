use candid::{CandidType, Deserialize, Nat, Principal};
use std::collections::HashMap;

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

/// Query Request
///
/// Request for indexed data
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
/// * `reverse_order` - if `true`, returns results in ascending order. If `null` or `false`, returns results in descending order
#[derive(CandidType, Clone, Deserialize)]
pub struct QueryRequest {
    pub sort_key: String,
    pub page: usize,
    pub reverse_order: Option<bool>,
    pub count: Option<usize>,
}

#[derive(CandidType, Clone, Debug)]
pub struct QueryResponse {
    pub total: usize,
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

pub type Index = HashMap<String, Vec<String>>;
