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
  dfx canister call curation query "(
    record {
      sort_key=\"all\";
      page=$page;
    }
  )"
done


echo "-> trait filter query for random traits"
for i in {0..1}; do
  trait=${traits[$((RANDOM % ${#traits[@]}))]}
  printf "page 0 (Base: $trait)"
  dfx canister call curation query "(
    record {
      sort_key=\"all\";
      page=0;
      traits=opt vec {
        record {
          \"Base\";
          variant {
            \"TextContent\" = \"$trait\"
          };
        }
      };
    }
  )"
done

echo "-> trait filter query for multiple random traits"
for i in {0..1}; do
  trait1=${traits[$((RANDOM % ${#traits[@]}))]}
  trait2=${traits[$((RANDOM % ${#traits[@]}))]}
  printf "page 0 (Base: $trait)"
  dfx canister call curation query "(
    record {
      sort_key=\"all\";
      page=0;
      traits=opt vec {
        record {
          \"Base\";
          variant {
            \"TextContent\" = \"$trait\"
          };
        };
        record {
          \"Base\";
          variant {
            \"TextContent\" = \"$trait1\"
          };
        };
      };
    }
  )"
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
dfx canister call curation query "(
  record {
    sort_key=\"last_listing\";
    page=0;
  }
)"
echo "-> query for tokens by 'listing_price' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"listing_price\";
    page=0;
  }
)"



echo "-> insert 'makeOffer' events (tokens 1-3)"
for i in {2..4}; do
  price=${prices[$((RANDOM % ${#prices[@]}))]}
  echo "make offer for token $i (price: $price)"

  dfx canister call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"makeOffer\";
      buyer=opt principal\"$user_a\";
      price=opt($price);
    }
  )"
done

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"last_offer\";
    page=0;
  }
)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"offer_price\";
    page=0;
  }
)"



echo "-> 'cancelOffer' for token 2"
dfx canister call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"2\";
    operation=\"cancelOffer\";
    buyer=opt principal\"$user_a\";
  }
)"

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"last_offer\";
    page=0;
  }
)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"offer_price\";
    page=0;
  }
)"



echo "-> make additional offer to token 3"
dfx canister call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"3\";
    operation=\"makeOffer\";
    buyer=opt principal\"$user_a\";
    price=opt(200);
  }
)"

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"last_offer\";
    page=0;
  }
)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"offer_price\";
    page=0;
  }
)"



price=${prices[$((RANDOM % ${#prices[@]}))]}
echo "-> directBuy for token 3 (price: $price)"
dfx canister call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"3\";
    operation=\"directBuy\";
    buyer=opt principal\"$user_a\";
    price=opt($price);
  }
)"

price=${prices[$((RANDOM % ${#prices[@]}))]}
echo "-> acceptOffer for token 2 (price: $price)"
dfx canister call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"1\";
    operation=\"acceptOffer\";
    buyer=opt principal\"$user_a\";
    price=opt($price);
  }
)"


echo "-> query for tokens by 'last_sale' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"last_sale\";
    page=0;
  }
)"
echo "-> query for tokens by 'sale_price' (page 0)"
dfx canister call curation query "(
  record {
    sort_key=\"sale_price\";
    page=0;
  }
)"


echo "-> query for all tokens"
for page in {0..0}; do
  printf "  $page: "
  dfx canister call curation query "(
    record {
      sort_key=\"all\";
      page=0;
    }
  )"
done