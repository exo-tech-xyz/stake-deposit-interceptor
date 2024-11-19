#!/usr/bin/env bash

max_validators=$1
validator_file=$2
sol_amount=$3
transfer_to_wallet=$4
transfer_to_amount=$5
stake_per_validator=$((($sol_amount - ($max_validators * 2))/$max_validators))

keys_dir=keys
mkdir -p $keys_dir

spl_stake_pool=spl-stake-pool

create_keypair () {
  if test ! -f "$1"
  then
    solana-keygen new --no-passphrase -s -o "$1"
  fi
}

create_vote_accounts () {
  max_validators=$1
  validator_file=$2
  for number in $(seq 1 "$max_validators")
  do
    create_keypair "$keys_dir/identity_$number.json"
    create_keypair "$keys_dir/vote_$number.json"
    create_keypair "$keys_dir/withdrawer_$number.json"
    solana create-vote-account "$keys_dir/vote_$number.json" "$keys_dir/identity_$number.json" "$keys_dir/withdrawer_$number.json" --commission 1
    vote_pubkey=$(solana-keygen pubkey "$keys_dir/vote_$number.json")
    echo "$vote_pubkey" >> "$validator_file"
  done
}

add_validator_stakes () {
  stake_pool=$1
  validator_list=$2
  while read -r validator
  do
    $spl_stake_pool add-validator "$stake_pool" "$validator"
  done < "$validator_list"
}

increase_stakes () {
  stake_pool_pubkey=$1
  validator_list=$2
  sol_amount=$3
  while read -r validator
  do
    {
      $spl_stake_pool increase-validator-stake "$stake_pool_pubkey" "$validator" "$sol_amount"
     } || {
      $spl_stake_pool update "$stake_pool_pubkey" && \
      $spl_stake_pool increase-validator-stake "$stake_pool_pubkey" "$validator" "$sol_amount"
     }
  done < "$validator_list"
}


setup_test_validator() {
  solana-test-validator \
   --bpf-program SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy ./deps/stake_pool.so \
   --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ./deps/mpl_metadata.so \
   --slots-per-epoch 32 \
   --quiet --reset &
  pid=$!
  solana config set --url http://127.0.0.1:8899
  solana config set --commitment confirmed
  echo "waiting for solana-test-validator, pid: $pid"
  sleep 15
}

# SETUP LOCAL NET (https://spl.solana.com/stake-pool/quickstart#optional-step-0-setup-a-local-network-for-testing)

echo "Setup keys directory and clear old validator list file if found"
if test -f "$validator_file"
then
  rm "$validator_file"
fi

echo "Setting up local test validator"
setup_test_validator

echo "Creating vote accounts, these accounts be added to the stake pool"
create_vote_accounts "$max_validators" "$validator_file"

echo "Done adding $max_validators validator vote accounts, their pubkeys can be found in $validator_file"

# SETUP Stake Pool (https://spl.solana.com/stake-pool/quickstart#step-1-create-the-stake-pool)

# Script to setup a stake pool from scratch.  Please modify the parameters to
# create a stake pool to your liking!
command_args=()

###################################################
### MODIFY PARAMETERS BELOW THIS LINE FOR YOUR POOL
###################################################

# Epoch fee, assessed as a percentage of rewards earned by the pool every epoch,
# represented as `numerator / denominator`
command_args+=( --epoch-fee-numerator 1 )
command_args+=( --epoch-fee-denominator 100 )

# Withdrawal fee for SOL and stake accounts, represented as `numerator / denominator`
command_args+=( --withdrawal-fee-numerator 2 )
command_args+=( --withdrawal-fee-denominator 100 )

# Deposit fee for SOL and stake accounts, represented as `numerator / denominator`
command_args+=( --deposit-fee-numerator 3 )
command_args+=( --deposit-fee-denominator 100 )

command_args+=( --referral-fee 0 ) # Percentage of deposit fee that goes towards the referrer (a number between 0 and 100, inclusive)

command_args+=( --max-validators 2350 ) # Maximum number of validators in the stake pool, 2350 is the current maximum possible

# (Optional) Deposit authority, required to sign all deposits into the pool.
# Setting this variable makes the pool "private" or "restricted".
# Uncomment and set to a valid keypair if you want the pool to be restricted.
#command_args+=( --deposit-authority keys/authority.json )

###################################################
### MODIFY PARAMETERS ABOVE THIS LINE FOR YOUR POOL
###################################################


echo "Creating pool"
stake_pool_keyfile=$keys_dir/stake-pool.json
validator_list_keyfile=$keys_dir/validator-list.json
mint_keyfile=$keys_dir/mint.json
reserve_keyfile=$keys_dir/reserve.json
create_keypair $stake_pool_keyfile
create_keypair $validator_list_keyfile
create_keypair $mint_keyfile
create_keypair $reserve_keyfile

set -ex
$spl_stake_pool \
  create-pool \
  "${command_args[@]}" \
  --pool-keypair "$stake_pool_keyfile" \
  --validator-list-keypair "$validator_list_keyfile" \
  --mint-keypair "$mint_keyfile" \
  --reserve-keypair "$reserve_keyfile"

set +ex
stake_pool_pubkey=$(solana-keygen pubkey "$stake_pool_keyfile")
set -ex

set +ex
lst_mint_pubkey=$(solana-keygen pubkey "$mint_keyfile")
set -ex

echo "Creating token metadata"
$spl_stake_pool \
  create-token-metadata \
  "$stake_pool_pubkey" \
  NAME \
  SYMBOL \
  URI

echo "Depositing SOL into stake pool"
$spl_stake_pool deposit-sol "$stake_pool_pubkey" "$sol_amount"

echo "Adding validator stake accounts to the pool"
add_validator_stakes "$stake_pool_pubkey" "$validator_file"

echo "Increasing amount delegated to each validator in stake pool"
increase_stakes "$stake_pool_pubkey" "$validator_file" "$stake_per_validator"


## If the user specified transfer_to_wallet argument, then transfer LSTs to them
if [ -z "${transfer_to_wallet-}" ]; then
   echo "transfer_to_wallet was not set"
   exit 0
else
  echo "transfer_to_wallet $transfer_to_wallet"
  solana airdrop 100 $transfer_to_wallet
  spl-token transfer $lst_mint_pubkey $transfer_to_amount $transfer_to_wallet --allow-unfunded-recipient --fund-recipient
fi