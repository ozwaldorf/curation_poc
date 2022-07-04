# curator canister (combined proxy/indexer)

- users can make a jelly transaction request to either the main canister, or to the curation canister.
  - Curation canister -> proxy request to main jelly, and index transaction if successful
  - Jelly canister -> transaction is processed and pushed to the curation canister for indexing
- proxy collects fee if any successful jelly transaction came from the proxy, which the fee is released at time of sale.
  - Maybe can split the protocol fee from jelly? 0.5% each?
- possibly could let each collection maintain their own ?

![Curation and indexing flow](https://upld.is/QmaWQtL5TksCZgK5Dgm1EhUPZnhiXygKTRQm8zMeVLWxYV/curation_flow.png)

## Curation canister

- stores 2 types, filter maps and trait lists
- provides nft index interface, and jelly proxy interface

### Interface

#### Indexer

- insert method for when jelly recieves a transaction, can push the data to the index canister.

  - This method would be guarded by a custodian list
  - this should mimic cap's insert_sync, maybe we can even literally just use the method/interface from the cap-sdk and cap-common?

- Paginated querys for token id lists

#### Proxy

- all transaction methods from jelly (to proxy and insert)

### Sorted Indexes

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

### Filter maps

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

- optional: store user info

(This is really hard to keep up to date without notification from nft contract)

```

{
  [user principal]: [
    1592, 1245, 1292, ...
  ]
}

```

## Proxy insertion

buy, offer, list, cancel events

1. checks if token is already indexed
   -> not found: call nft contract for metadata (maybe also save last fetched metadata timestamp)
   -> update trait map for token
2. proxy the command to the main jelly canister
   -> failure: return error
3. (re)insert and sort to corresponding token index (listed_price, offer_price, offer_count, last_action)
4. update offer count map
5. respond to user with success

## Fee dispersal

If the curation canister facilitated any action (listing, offer, acceptance, or direct purchase), it recieves half of the protocol fee.

- Jelly stores/holds the curation canister balance, and provides a method `withdraw_to`, which sends the callers total held balance at the time to a specified principal id
- Curation canister provides a method `claim_fees`, which is locked to custodians, and calls `withdraw_to` for the callers principal id

## Migration steps for existing (v1) crowns data

1. Create crowns curation canister on mainnet
2. announce on SM and halt jelly transactions
3. Call insert as custodian for all existing jelly transactions (in order, we can build this data from CAP)
4. upgrade jelly canister (still locked) to push new transactions on main interface to curation canister
5. re-enable jelly transactions

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```
