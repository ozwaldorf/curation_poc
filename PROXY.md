# Curation and Proxy Canisters

![Untitled](Curation%20and%20Proxy%20Canisters%2008851206cb384107a6cb28ce745687d4/Untitled.png)

# Next Steps for Curation Canisters

- move the indexed db datatype under its own library for use by any project. Options:
  - under ic-kit (`ic_kit::db`, `ic_kit::indexed_map`)
  - package as its own name (`sproutdb`, `psychedb`, `indexed_map`, etc)
  - create lib under jelly as `jelly-db`
- define jelly datatypes under a jelly-common lib (like cap-common)
- make a `v2` branch (or move to new repo `jelly` branch develop)
- add dab submodule and auto deployment if local
- Implement the following:

---

# Jelly run/deployed curation canisters

- pros:
  - jelly controls and guarantees uptime
  - can ensure same subnet for fast inserts (inter canister call speeds up when same subnet)
  - easier process for collections joining jelly
- cons:
  - we pay cycles for continued use (proto fee more than covers it though)

### Minting

- minting and initial sale handled externally
- could also provide the initial sale:
  - mint to the collection canister id directly
  - call an added `list_token` method on the dip721 canister that would list a given nft on jelly automatically
- New collection, push meta as they mint
  - collections would add a small snippit of code (patch file?) to their mint function, which would push the metadata
- Existing collection,
  - manually push meta
  - OR pull meta as users list/offer/etc

---

## Jelly Hub (root marketplace canister)

### Ledger

- for each collection, store a list of approved proxy canisters
  - `ledger.collections[C].proxies: Vec<Principal>`
- also store total list of all curation canisters
  - `ledger.curation_canisters: Vec<Principal>`

### Interface

---

### add_collection

```rust
struct AddCollectionArgs {
	// main token to trade with.
	fungible_canister_id: Principal,
	// fee to for the collection fee, and the proxy fee if used
	owner: Principal,
	// collection fee up to 10% (1000:nat)
	fee: u64,

	// optional standard (we're dip721 only for now tho)
	fungible_canister_standard: Option<FungibleStandard>,
}

add_collection(args: AddCollectionArgs) -> Result<(), MAPIErr> {}
```

register a collection and spawn the curation canisters

if not a controller, require message with cycles

- requirements
  - call is made with 2-4T cycles (create canister id + 1-3T to fund curation canister)
    OR
  - an XTC approval of 2-4T cycles
- flow
  - collection deploys canister and registers via dab (no mint yet)
  - checks dab registration status and standard - locked to dip721v2
  - jelly hub spawns a curation canister initialized for the collection if checks pass
  - commits the curation canister as a jelly proxy
  - collection mints nfts, pushing mint transactions to the curation canister (provide a dip721 patch file?)
  - for existing collections, can manually push these mint events.

---

### update_collection

```rust
update_collection(args: AddCollectionArgs) -> Result<(), MAPIErr> {}
```

update a collections data - jelly controllers and collection owners only

---

### existing interactions (directBuy, acceptOffer, make/cancel offer/listing)

- new v2 endpoints snake case version namespace
  ```rust
  pub fn direct_buy
  pub fn accept_offer
  ...
  ```
- define common argument struct for each interface
  ```rust
  // OPTION A
  // can add new fields and remain interface compatable
  // proxies would need to be replaced or upgradable
  // Can also version typed arguments in enum

  /// Versioned argument types
  /// dfx input would look like:
  /// record {
  ///   "V1";
  ///   record {...};
  /// }
  pub enum VersionedArgs {
  	V1(TransactionArgs)
  }

  /// Catch all argument type
  pub struct TransactionArgs {
  	/// nft canister id to trade with
  	collection: Principal,
  	/// token id to offer for. Deserialized based on collection standard
  	token_id: String,

  	// Use with `make_listing` and `make_offer`
  	/// fungible canister id.
  	/// Defaults to the collection fungible canister
  	fungible: Option<Principal>,
  	/// Specified amount for the transaction
  	price: Option<Nat>,

  	// Use with `direct_buy` and `accept_offer`, AND for proxy events to specify the caller
  	/// buyer field
  	buyer: Option<Principal>,
  	/// seller field.
  	/// Example: to directBuy, would specify
  	seller: Option<Principal>
  }

  // OPTION B

  /// hashmap argument example
  pub enum ArgumentValue {
  	StringVal(String),
  	PrincipalVal(Principal),
  	NatVal(Nat),
  	U64(u64),
  }
  pub type TransactionArgs: HashMap<String, ArgumentValue>;

  pub fn make_listing(args: TransactionArgs) -> TransactionResponse {}
  ```
- flow for transactions
  1. checks if caller is a registered curator for the given collection
     - false => caller is a user, non curated interaction
     - true => caller is a proxy, buyer/seller is specified, error if None
       - if proxy is collection run, need some sort of proof from the user. A signed blob?
  2. proceed for ownership/approval checks, initiate transfers, commit to state
  3. if event is non curated, push to corresponding curation canister

## Curation Canister

### controllers

- completely blackholed:
  - `move_cycles` to pull avaliable cycles back in the event of replacing the canister
  - `migrate` to push all events to a new curation canister
- partial blackhole:
  - jelly is sole controller and can upgrade the canisters

### proxy interactions

- provides identical interface for all existing interactions under snake case
- call jelly with request, include caller as buyer or seller param
  - index transaction if success
  - return error

=======================================================

## Further ideas:

### curated categories

- registered curation canisters + collections also assigned a category in the jelly hub, default would be unverified, and we can manually assign verified status through this
- This could also be expanded to create categories of collections, ie a collection has multiple sets of nfts, a trusted service creates collections (looking at btcflower)
- would be a `get_curated_collections` endpoint that accepts a category, and returns a list of collections and their data (fungible, curation id, volume, name, desc, etc)

============================================

# Alternative options:

- collection run:
  - pros:
    - we dont need to fund jelly computations for the indexing
    - more decentralized
  - cons:
    - risk of bad actors, harder to secure
    - need to implement some form of multi sig (like cover, users submit a "proof", ie signing a timestamp)
    - less control, would need to provide support if a collection miss-manages the canister
    - more load on projects/dao to maintain and track
  - User signature verification Flow
    - user signs arguments with id - encode to blob or u8 arr
    - proxy calls jelly hub with encoded signed arguments
    - jelly hub verifies authenticity of args, returning error if invalid signature
    - jelly hub performs action

![Untitled](Curation%20and%20Proxy%20Canisters%2008851206cb384107a6cb28ce745687d4/Untitled%201.png)
