#!/bin/bash

if [ -z "$1" ]; then
    NETWORK=local
else
    NETWORK=$1
fi

# Setup the environment
traits=("Bronze" "Silver" "Gold" "Platinum" "Diamond")
traits2=("Diamond" "Psychedelic" "Emerald" "Sapphire" "Ruby")
prices=(1 2 3 5 8 13 21 34 55 89 144) # lets get fibby!
nft_canister_id="aaaaa-aa"
user_a="ffuck-kxghi-gyvia-r5htr-246cy-acq5u-2tdgd-avtvf-jyqbt-xtmf7-cae"
user_b="3crrz-quea6-mdmy3-3btit-f2mgf-esqo6-ybiz7-i6s4z-xrf7g-izcxw-zae"

if [ "$NETWORK" == "local" ]; then
    echo "-> Checking local replica..."
    dfx ping || dfx start --clean --background

    echo "-> Deploying curation canister..."
    dfx deploy --argument '(null)'
fi



echo "-> insert 'mint' events (tokens 0-99)..."
for i in {0..99}; do
  echo "mint $i"

  # Randomly select a trait
  # fetch example token meta
  token_meta=$(jq ".[$i]" test/data.json)
  trait1=$(echo $token_meta | jq '.properties[0][1].TextContent')
  trait2=$(echo $token_meta | jq '.properties[1][1].TextContent')
  trait3=$(echo $token_meta | jq '.properties[2][1].TextContent')
  trait4=$(echo $token_meta | jq '.properties[3][1].TextContent')
  trait5=$(echo $token_meta | jq '.properties[5][1].TextContent')

  dfx canister --network $NETWORK call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"mint\";
      traits=opt vec {
        record {
          \"smallgem\";
          variant {
            \"TextContent\" = $trait1
          }
        };
        record {
          \"biggem\";
          variant {
            \"TextContent\" = $trait2
          }
        };
        record {
          \"base\";
          variant {
            \"TextContent\" = $trait3
          }
        };
        record {
          \"rim\";
          variant {
            \"TextContent\" = $trait4
          }
        };
        record {
          \"location\";
          variant {
            \"TextContent\" = $trait5
          }
        };
      };
    }
  )"
done

echo "-> query for all tokens"
printf "  $page: "
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"all\";
  }
)"

trait=${traits[$((RANDOM % ${#traits[@]}))]}
echo "-> trait filter query for random single trait page 0 (Base: $trait), page 0"
printf 
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"all\";
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

trait1=${traits[$((RANDOM % ${#traits[@]}))]}
trait2=${traits[$((RANDOM % ${#traits[@]}))]}
echo "-> trait filter query for multiple random traits (Base: $trait1 | $trait2), page 0"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"all\";
    traits=opt vec {
      record {
        \"Base\";
        variant {
          \"TextContent\" = \"$trait1\"
        };
      };
      record {
        \"Base\";
        variant {
          \"TextContent\" = \"$trait2\"
        };
      };
    };
  }
)"



echo "-> insert 'makeListing' events (tokens 0-1)"
for i in {0..4}; do
  price=${prices[$((RANDOM % ${#prices[@]}))]}
  echo "make listing for token $i (price: $price)"

  dfx canister --network $NETWORK call curation insert "(
    record {
      nft_canister_id=principal\"$nft_canister_id\";
      token_id=\"$i\";
      operation=\"makeListing\";
      price=opt($price);
    }
  )"
done

echo "-> query for tokens by 'last_listing' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"last_listing\";
  }
)"
echo "-> query for tokens by 'listing_price' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"listing_price\";
  }
)"



echo "-> insert 'makeOffer' events (tokens 5-9)"
for i in {5..9}; do
  price=${prices[$((RANDOM % ${#prices[@]}))]}
  echo "make offer for token $i (price: $price)"

  dfx canister --network $NETWORK call curation insert "(
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
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"last_offer\";
  }
)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"offer_price\";
  }
)"



echo "-> 'cancelOffer' for token 5"
dfx canister --network $NETWORK call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"5\";
    operation=\"cancelOffer\";
    buyer=opt principal\"$user_a\";
  }
)"

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"last_offer\";
  }
)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"offer_price\";
  }
)"



echo "-> make additional offer to token 6"
dfx canister --network $NETWORK call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"6\";
    operation=\"makeOffer\";
    buyer=opt principal\"$user_b\";
    price=opt(200);
  }
)"

echo "-> query for tokens by 'last_offer' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"last_offer\";
  }
)"
echo "-> query for tokens by 'offer_price' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"offer_price\";
  }
)"



price=${prices[$((RANDOM % ${#prices[@]}))]}
echo "-> directBuy for token 4 (price: $price)"
dfx canister --network $NETWORK call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"4\";
    operation=\"directBuy\";
    buyer=opt principal\"$user_a\";
    price=opt($price);
  }
)"

echo "-> acceptOffer for token 6 (price: 200)"
dfx canister --network $NETWORK call curation insert "(
  record {
    nft_canister_id=principal\"$nft_canister_id\";
    token_id=\"5\";
    operation=\"acceptOffer\";
    buyer=opt principal\"$user_b\";
    price=opt(200);
  }
)"


echo "-> query for tokens by 'last_sale' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"last_sale\";
  }
)"
echo "-> query for tokens by 'sale_price' (page 0)"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"sale_price\";
  }
)"


echo "-> query for all tokens"
printf "  $page: "
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"all\";
  }
)"

echo "-> ascending trait filter query for multiple traits, sk=last_sale (Base: ${traits[1]} | ${traits[2]} | ${traits[3]}), page 0"
dfx canister --network $NETWORK call curation query "(
  record {
    sort_key=\"last_sale\";
    reverse=opt(true);
    traits=opt vec {
      record {
        \"Base\";
        variant {
          \"TextContent\" = \"${traits[1]}\"
        };
      };
      record {
        \"Base\";
        variant {
          \"TextContent\" = \"${traits[2]}\"
        };
      };
      record {
        \"Base\";
        variant {
          \"TextContent\" = \"${traits[3]}\"
        };
      };
    };
  }
)"