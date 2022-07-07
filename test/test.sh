#!/bin/bash

# Setup the environment
traits=("Bronze" "Silver" "Gold" "Platinum" "Diamond")
prices=(1 2 3 5 8 13 21 34 55 89 144) # lets get fibby!
nft_canister_id="aaaaa-aa"
user_a="ffuck-kxghi-gyvia-r5htr-246cy-acq5u-2tdgd-avtvf-jyqbt-xtmf7-cae"
user_b="3crrz-quea6-mdmy3-3btit-f2mgf-esqo6-ybiz7-i6s4z-xrf7g-izcxw-zae"

echo "-> Checking local replica..."
dfx ping || dfx start --clean --background

echo "-> Deploying curation canister..."
dfx deploy curation --argument '(null)'



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

echo "-> query for tokens by 'last_listing' (page 0)"
dfx canister call curation query "(\"last_listing\", 0)"
echo "-> query for tokens by 'listing_price' (page 0)"
dfx canister call curation query "(\"listing_price\", 0)"



echo "-> insert 'makeOffer' events (tokens 2-4)"
for i in {2..4}; do
  price=${prices[$((RANDOM % ${#prices[@]}))]}
  echo "make offer for token $i (price: $price)"

  dfx canister call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"makeOffer\";
      buyer=opt principal\"$nft_canister_id\";
      price=opt($price);
    }
  )"
done

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister call curation query "(\"last_offer\", 0)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister call curation query "(\"offer_price\", 0)"



echo "-> 'cancelOffer' for token 2"
dfx canister call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"2\";
    operation=\"cancelOffer\";
    buyer=opt principal\"$nft_canister_id\";
  }
)"

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister call curation query "(\"last_offer\", 0)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister call curation query "(\"offer_price\", 0)"



echo "-> make additional offer to token 3"
dfx canister call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"3\";
    operation=\"makeOffer\";
    buyer=opt principal\"$nft_canister_id\";
    price=opt(200);
  }
)"

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister call curation query "(\"last_offer\", 0)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister call curation query "(\"offer_price\", 0)"



echo "-> query for all tokens"
for page in {0..0}; do
  printf "  $page: "
  dfx canister call curation query "(\"all\", $page)"
done