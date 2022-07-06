echo "-> Checking local replica..."
dfx ping || dfx start --clean --background

echo "-> Deploying curation canister..."
dfx deploy curation --argument '(null)'

# Random traits
traits=("Bronze" "Silver" "Gold" "Platinum" "Diamond")
prices=(1 2 3 4 5)
nft_canister_id="aaaaa-aa"

echo "-> insert 'mint' events (tokens 0-14)..."
for i in {0..14}; do
  # Randomly select a trait
  trait=${traits[$((RANDOM % ${#traits[@]}))]}
  echo "mint $i (base: $trait)"

  dfx canister call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"mint\";
      traits=opt vec {
        record {
          \"Base\";
          variant {
            \"TextContent\" = \"$trait\"
          }
        }
      };
    }
  )"
done

echo "-> query for all tokens"
for page in {0..1}; do
  printf "  $page: "
  dfx canister call curation query "(\"all\", $page)"
done

echo "-> insert 'makeListing' events (tokens 0-1)"
for i in {0..1}; do
  price=${prices[$((RANDOM % ${#prices[@]}))]}
  echo "make listing for token $i (price: $price)"

  dfx canister call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"makeListing\";
      price=opt($price);
    }
  )"
done

echo "-> query for tokens by 'last_listing'"
for page in {0..1}; do
  printf "  $page: "
  dfx canister call curation query "(\"last_listing\", $page)"
done

echo "-> query for tokens by 'listing_price'"
for page in {0..1}; do
  printf "  $page: "
  dfx canister call curation query "(\"listing_price\", $page)"
done

echo "-> insert 'makeOffer' events (tokens 2-3)"
for i in {2..3}; do
  price=${prices[$((RANDOM % ${#prices[@]}))]}
  echo "make offer for token $i (price: $price)"

  dfx canister call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"makeOffer\";
      price=opt($i);
    }
  )"
done

echo "-> query for tokens by 'last_offer'"
for page in {0..1}; do
  printf "  $page: "
  dfx canister call curation query "(\"last_offer\", $page)"
done

echo "-> query for all tokens"
for page in {0..1}; do
  printf "  $page: "
  dfx canister call curation query "(\"all\", $page)"
done