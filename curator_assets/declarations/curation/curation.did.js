export const idlFactory = ({ IDL }) => {
  const GenericValue = IDL.Rec();
  GenericValue.fill(
    IDL.Variant({
      'Nat64Content' : IDL.Nat64,
      'Nat32Content' : IDL.Nat32,
      'BoolContent' : IDL.Bool,
      'Nat8Content' : IDL.Nat8,
      'Int64Content' : IDL.Int64,
      'NatContent' : IDL.Nat,
      'Nat16Content' : IDL.Nat16,
      'Int32Content' : IDL.Int32,
      'Int8Content' : IDL.Int8,
      'Int16Content' : IDL.Int16,
      'BlobContent' : IDL.Vec(IDL.Nat8),
      'NestedContent' : IDL.Vec(IDL.Tuple(IDL.Text, GenericValue)),
      'Principal' : IDL.Principal,
      'TextContent' : IDL.Text,
    })
  );
  const Event = IDL.Record({
    'token_id' : IDL.Text,
    'traits' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, GenericValue))),
    'seller' : IDL.Opt(IDL.Principal),
    'fungible_id' : IDL.Opt(IDL.Principal),
    'operation' : IDL.Text,
    'buyer' : IDL.Opt(IDL.Principal),
    'price' : IDL.Opt(IDL.Nat),
    'nft_canister_id' : IDL.Principal,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const QueryRequest = IDL.Record({
    'reverse' : IDL.Opt(IDL.Bool),
    'traits' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, GenericValue))),
    'count' : IDL.Opt(IDL.Nat64),
    'last_index' : IDL.Opt(IDL.Nat64),
    'sort_key' : IDL.Text,
  });
  const Offer = IDL.Record({
    'fungible' : IDL.Principal,
    'buyer' : IDL.Principal,
    'price' : IDL.Nat,
  });
  const Sale = IDL.Record({
    'time' : IDL.Nat,
    'fungible' : IDL.Principal,
    'buyer' : IDL.Principal,
    'price' : IDL.Nat,
  });
  const TokenData = IDL.Record({
    'id' : IDL.Text,
    'traits' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, GenericValue))),
    'offers' : IDL.Vec(Offer),
    'best_offer' : IDL.Opt(IDL.Nat),
    'last_sale' : IDL.Opt(Sale),
    'last_offer' : IDL.Opt(IDL.Nat),
    'price' : IDL.Opt(IDL.Nat),
    'last_listing' : IDL.Opt(IDL.Nat),
  });
  const QueryResponse = IDL.Record({
    'total' : IDL.Nat64,
    'data' : IDL.Vec(TokenData),
    'last_index' : IDL.Opt(IDL.Nat64),
    'error' : IDL.Opt(IDL.Text),
  });
  return IDL.Service({
    'insert' : IDL.Func([Event], [Result], []),
    'query' : IDL.Func([QueryRequest], [QueryResponse], []),
  });
};
export const init = ({ IDL }) => { return [IDL.Opt(IDL.Principal)]; };
