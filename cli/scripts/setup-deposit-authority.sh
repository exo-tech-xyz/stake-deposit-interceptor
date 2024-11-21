STAKE_POOL_KEYPAIR_PATH="./keys/stake-pool.json"
FEE_WALLET_KEYPAIR="./keys/identity_1.json"

# TODO update authority keypair to be that of the cli config manager?
AUTHORITY_KEYPAIR=$(solana config get | grep "Keypair Path" | awk '{print $3}')

if [ ! -f "$STAKE_POOL_KEYPAIR_PATH" ]; then
  echo "Error: Keypair file not found at $STAKE_POOL_KEYPAIR_PATH"
  exit 1
fi

if [ ! -f "$FEE_WALLET_KEYPAIR" ]; then
  echo "Error: Keypair file not found at $FEE_WALLET_KEYPAIR"
  exit 1
fi
if [ ! -f "$AUTHORITY_KEYPAIR" ]; then
  echo "Error: Keypair file not found at $AUTHORITY_KEYPAIR"
  exit 1
fi

STAKE_POOL=$(solana-keygen pubkey "$STAKE_POOL_KEYPAIR_PATH")
FEE_WALLET=$(solana-keygen pubkey "$FEE_WALLET_KEYPAIR")
AUTHORITY=$(solana-keygen pubkey "$AUTHORITY_KEYPAIR")
FUNDING_TYPE="stake-deposit"  # or the appropriate funding type

echo "creating deposit authority"

# create stake_deposit_authority and set pubkey to var
STAKE_DEPOSIT_AUTHORITY=$(../target/debug/spl-stake-pool-interceptor interceptor create-stake-deposit-authority --pool $STAKE_POOL --fee-wallet $FEE_WALLET --authority $AUTHORITY --cool-down-seconds 200 --initial-fee-bps 100 | tail -1)

# Update the stake pool's stake_deposit_authority
echo "Updating stake pool's stake_deposit_authority to $STAKE_DEPOSIT_AUTHORITY"
spl-stake-pool set-funding-authority $STAKE_POOL $FUNDING_TYPE $STAKE_DEPOSIT_AUTHORITY

# Check if the update command was successful
if [ $? -ne 0 ]; then
  echo "Error: Failed to update stake_deposit_authority."
  exit 1
fi

echo "Stake pool's stake_deposit_authority updated successfully."