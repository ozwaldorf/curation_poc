# curation canister (combined proxy/indexer)

![Curation and indexing flow](https://upld.is/QmaWQtL5TksCZgK5Dgm1EhUPZnhiXygKTRQm8zMeVLWxYV/curation_flow.png)

## POC Demo

Frontend: https://j2brg-liaaa-aaaah-abmta-cai.ic0.app/

canister id: `jtc22-5aaaa-aaaah-abmsq-cai`

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy

# Run the test file to generate some data
./test/test.sh
```

## Curation canister

- users can make a jelly transaction request to either the main canister, or to the curation canister.
  - Curation canister -> proxy request to main jelly, and index transaction if successful
  - Jelly canister -> transaction is processed and pushed to the curation canister for indexing
- proxy collects fee if any successful jelly transaction came from the proxy, which the fee is released at time of sale.

  - Maybe can split the protocol fee from jelly? 0.5% each?

- stores 2 types, filter maps and sorted vectors

### Indexer

- insert method for when jelly recieves a transaction, can push the data to the index canister.

  - This method would be guarded by a custodian list
  - this should mimic cap's insert_sync, maybe we can even literally just use the method/interface from the cap-sdk and cap-common?

- Paginated querys for token id lists

#### Sorted Indexes

- canister builds sorted indexes of token ids as the token entries are modified/created

```
{
  // insert and sort
  listing_price: [],
  best_offer_price: [],

  // remove and insert to end
  last_listing: [],
  last_offer: [],
  last_sale: [],
  all: []

}
```

#### Trait filters

ideas:

- 1. easy: iterate through sort key until we reach the final desired count (most likely for poc)

  - build list of token ids for selected traits, just push to a single array
  - iterate through sort key and only push results that are included in the accepted tokens array
  - con: inefficient scaling, multi page results require recomputing previous pages
  - con: would need to scan entire sort key for a total result

- 2. medium: seperate update call `filter_query` that implements #1, but caches the request/last result index to resume filtering to achieve pagination

  - iteration of #1
  - need to store db as a btreemap, to get a root hash
  - request cache stored for the duration of the dbs hash, any change would wipe cache
  - request_cache: ([...sorted_traits_request], db_hash) -> last_scanned_index

- 3. hard: preflight query + opt update + query

  - iteration of #2, cache stored
  - CHECK_CACHE query for request - this could also be a query cache
  - PRELOAD_CACHE update if request is falsey
  - QUERY_CACHE query for paginated data from cache
  - pros:
    - can compute once per db state for unlimited users, scales really well
    - have total number of items from the getgo

- 4. hard: precompute sort indexes for each of the tokens traits on insertion

  - iteration of #2, could use this instead of #3
  - combine precomputed sort indexes for multiple trait selections during query, need to dedupe

  - pros:
    - fastest for users querying for data
    - single trait requests nearly instant
    - minimal computation (merging pre sorted arrays excluding duplicates)
  - cons:
    - heavy computation on insert, for existing insert computation x and number of traits y, XY computation time

---

- trait map

```

{
  [trait name]: {
    [trait data]: [
      1592, 1245, 1292, ...
    ]
  }
}

```

- optional: store number of offers to token ids

```

{
  [offer_count]: [
    1592, 1245, 1292, ...
  ]
}

```

### Proxy (ideas)

- all transaction methods from jelly (to proxy and insert)

#### Proxy insertion

buy, offer, list, cancel events

1. checks if token is already indexed
   -> not found: call nft contract for metadata (maybe also save last fetched metadata timestamp)
   -> update trait map for token
2. proxy the command to the main jelly canister
   -> failure: return error
3. (re)insert and sort to corresponding token index (listed_price, offer_price, offer_count, last_action)
4. update offer count map
5. respond to user with success

#### Fee dispersal

If the curation canister facilitated any action (listing, offer, acceptance, or direct purchase), it recieves half of the protocol fee.

- Jelly stores/holds the curation canister balance, and provides a method `withdraw_to`, which sends the callers total held balance at the time to a specified principal id
- Curation canister provides a method `claim_fees`, which is locked to custodians, and calls `withdraw_to` for the callers principal id

## Migration steps for existing (v1) crowns data

1. Create crowns curation canister on mainnet
2. announce on SM and halt jelly transactions
3. Call insert as custodian for all existing jelly transactions (in order, we can build this data from CAP)
4. upgrade jelly canister (still locked) to push new transactions on main interface to curation canister
5. re-enable jelly transactions

## Canister creation/registration (ideas)

- jelly canister creates canister id on behalf of a user (`jelly_canister_create` call sent with cycles - 2-4T)
- jelly sets itself and the user as the controller/custodian (to insert events)
- init argument performs handshake to jelly, which allows

```
1. plug call using XTC or thru cycles canister, to create a canister id. Jelly spawns a canister id and assigns it to the user
2. users then deploy to their canister id the curation canister, or we do.
  - canister is initialized with the jelly set as the source of truth
3. at first, we manually configure the collection on jelly using addCollection and a fungible token
  - this could be switched to a 2 part registration, one for the deployment of the curation canister, another from dab on submission
  - can happen in either order, once both are submitted/handshaked, collection is in
```

## Tasks

- [x] basic implementation that indexes tokens by most recent actions
- [x] basic query method for token id results

- [x] index price and best offer
- [x] build an internal db of token data, and respond with the full data instead of key names

- [x] request type def - predecessor for trait filters
- [x] ascending/descending option
- [x] sale price index
- [x] sale db type
- [x] handle direct buy and accept offer events properly
- [x] track fungible id with offers

- [x] basic trait filter implementation (iterate results, optimization is passing an optional offset to skip over a certain number of already indexed tokens under the sort key)
- [x] refactor pagination to use a last index instead of pages - this is a strat that works good for filtering.
- [x] basic UI preview to test and showcase
- [ ] expand test dataset size

Post POC:

- [ ] jelly proxy
- [ ] batch insertion
- [ ] scale tests (load 10k tokens and perform 100s of actions)
- [ ] move POC indexer/filter logic into a more generically defined common-lib
- [ ] (future) hook up to jelly and further optimizations!
