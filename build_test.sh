#!/bin/bash

input=$1

if [[ $input = "devnet" ]]
then
    solana program dump -u https://api.devnet.solana.com metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ./tests/fixtures/mpl_token_metadata.so
elif [[ $input = "mainnet" ]]
then
    solana program dump -u https://api.mainnet-beta.solana.com metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ./tests/fixtures/mpl_token_metadata.so
else
    echo "Invalid input"
fi
