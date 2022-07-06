echo "-> Checking local replica..."
dfx ping || dfx start --clean --background

echo "-> Checking curation canister..."
dfx canister id curation || dfx deploy curation --argument '(null)'

# Random traits
traits=("Bronze" "Silver" "Gold" "Platinum" "Diamond")
nft_canister_id="aaaaa-aa"

echo "-> insert 'mint' events (tokens 0-9)..."
for i in {0..9}; do
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
  echo "makeListing $i"
  dfx canister call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"makeListing\";
      price=opt($i);
    }
  )"
done

echo "-> query for tokens by 'last_listing'"
for page in {0..1}; do
  printf "  $page: "
  dfx canister call curation query "(\"last_listing\", $page)"
done

echo "-> insert 'makeOffer' events (tokens 2-3)"
for i in {2..3}; do
  echo "makeOffer $i"
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