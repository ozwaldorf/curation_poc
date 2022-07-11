import type { Principal } from '@dfinity/principal';
export interface Event {
  'token_id' : string,
  'traits' : [] | [Array<[string, GenericValue]>],
  'seller' : [] | [Principal],
  'fungible_id' : [] | [Principal],
  'operation' : string,
  'buyer' : [] | [Principal],
  'price' : [] | [bigint],
  'nft_canister_id' : Principal,
}
export type GenericValue = { 'Nat64Content' : bigint } |
  { 'Nat32Content' : number } |
  { 'BoolContent' : boolean } |
  { 'Nat8Content' : number } |
  { 'Int64Content' : bigint } |
  { 'NatContent' : bigint } |
  { 'Nat16Content' : number } |
  { 'Int32Content' : number } |
  { 'Int8Content' : number } |
  { 'Int16Content' : number } |
  { 'BlobContent' : Array<number> } |
  { 'NestedContent' : Array<[string, GenericValue]> } |
  { 'Principal' : Principal } |
  { 'TextContent' : string };
export interface Offer {
  'fungible' : Principal,
  'buyer' : Principal,
  'price' : bigint,
}
export interface QueryRequest {
  'reverse' : [] | [boolean],
  'traits' : [] | [Array<[string, GenericValue]>],
  'count' : [] | [bigint],
  'last_index' : [] | [bigint],
  'sort_key' : string,
}
export interface QueryResponse {
  'total' : bigint,
  'data' : Array<TokenData>,
  'last_index' : [] | [bigint],
  'error' : [] | [string],
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export interface Sale {
  'time' : bigint,
  'fungible' : Principal,
  'buyer' : Principal,
  'price' : bigint,
}
export interface TokenData {
  'id' : string,
  'traits' : [] | [Array<[string, GenericValue]>],
  'offers' : Array<Offer>,
  'best_offer' : [] | [bigint],
  'last_sale' : [] | [Sale],
  'last_offer' : [] | [bigint],
  'price' : [] | [bigint],
  'last_listing' : [] | [bigint],
}
export interface _SERVICE {
  'insert' : (arg_0: Event) => Promise<Result>,
  'query' : (arg_0: QueryRequest) => Promise<QueryResponse>,
}
