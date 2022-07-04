# curator canister (combined proxy/indexer)

- users can make a jelly transaction request to either the main canister, or to the curation canister.
  - Curation canister -> proxy request to main jelly, and index transaction if successful
  - Jelly canister -> transaction is processed and pushed to the curation canister for indexing
- proxy collects fee if any successful jelly transaction came from the proxy, which the fee is released at time of sale.
  - Maybe can split the protocol fee from jelly? 0.5% each?
- possibly could let each collection maintain their own ?

![Curation and indexing flow](https://upld.is/QmaWQtL5TksCZgK5Dgm1EhUPZnhiXygKTRQm8zMeVLWxYV/curation_flow.png)

### Pros

- completely on chain
- developers can very easily run the entire project

### Cons

- added inter canister call to txn time

## Curation canister

- stores 2 types, filter maps and trait lists
- provides nft index interface, and jelly proxy interface

### Interface

- all transaction methods from jelly (to proxy and insert)
- insert method for when jelly recieves a transaction, can push the data to the index canister.

  - This method would be guarded by a custodian list
  - this should mimic cap's insert_sync, maybe we can even literally just use the method/interface from the cap-sdk and cap-common?

- Paginated querys for token id lists

### Sorted Indexes

- canister builds sorted indexes of token ids as the token entries are modified/created

```
{
  // insert and sort
  listing_price: [],
  highest_offer_price: [],
  listing_price: [],

  // remove and insert to end
  last_listing_time: [],
  last_offer_time: [],
  last_modified: []
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

## IC Quickstart

Welcome to your new curator project and to the internet computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with curator, see the following documentation available online:

- [Quick Start](https://smartcontracts.org/docs/quickstart/quickstart-intro.html)
- [SDK Developer Tools](https://smartcontracts.org/docs/developers-guide/sdk-guide.html)
- [Rust Canister Devlopment Guide](https://smartcontracts.org/docs/rust-guide/rust-intro.html)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://smartcontracts.org/docs/candid-guide/candid-intro.html)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd curator/
dfx help
dfx config --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:8000?canisterId={asset_canister_id}`.
