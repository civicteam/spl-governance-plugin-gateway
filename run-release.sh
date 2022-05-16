#!/usr/bin/env bash

set -euo pipefail

if [[ -z "${PROVIDER_WALLET}" ]]; then
  echo "Please provide path to a provider wallet keypair."
  exit -1
fi

if [[ -z "${VERSION_MANUALLY_BUMPED}" ]]; then
  echo "Please bump versions in package.json and in cargo.toml."
  exit -1
fi

# build program
anchor build

# update on chain program and IDL, atm used for testing/developing
anchor deploy --provider.cluster devnet --provider.wallet ${PROVIDER_WALLET}
anchor idl upgrade --provider.cluster devnet --provider.wallet ${PROVIDER_WALLET}\
 --filepath target/idl/plugin_boilerplate.json 4Q6WW2ouZ6V3iaNm56MTd5n2tnTm4C5fiH8miFHnAFHo

# update types in npm package and publish the npm package
cp ./target/types/plugin_boilerplate.ts src/plugin_boilerplate.ts
yarn clean && yarn build && cp package.json ./dist/ && yarn publish dist

echo
echo Remember to commit and push the version update as well as the changes
echo to src/plugin_boilerplate.ts .
echo
