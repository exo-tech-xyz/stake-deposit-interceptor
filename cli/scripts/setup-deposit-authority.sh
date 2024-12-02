#!/bin/bash
set -e

echo "Building spl-stake-pool-interceptor..."
cargo build

solana-keygen new -o ./keys/fee-wallet.json  

STAKE_POOL_KEYPAIR_PATH="./keys/stake-pool.json"
FEE_WALLET_KEYPAIR_PATH="./keys/fee-wallet.json"
AUTHORITY_KEYPAIR_PATH=$(solana config get | grep "Keypair Path" | awk '{print $3}')

# Verify keypair files exist
if [ ! -f "$STAKE_POOL_KEYPAIR_PATH" ]; then
    echo "Error: Stake pool keypair not found at $STAKE_POOL_KEYPAIR_PATH"
    exit 1
fi

if [ ! -f "$FEE_WALLET_KEYPAIR_PATH" ]; then
    echo "Error: Fee wallet keypair not found at $FEE_WALLET_KEYPAIR_PATH"
    exit 1
fi

# Get public keys
STAKE_POOL=$(solana-keygen pubkey "$STAKE_POOL_KEYPAIR_PATH")
FEE_WALLET=$(solana-keygen pubkey "$FEE_WALLET_KEYPAIR_PATH")
AUTHORITY=$(solana-keygen pubkey "$AUTHORITY_KEYPAIR_PATH")

echo "STAKE_POOL: $STAKE_POOL"
echo "FEE_WALLET: $FEE_WALLET"
echo "AUTHORITY: $AUTHORITY"

# Fund fee wallet if needed
echo "Funding fee wallet..."
solana transfer --keypair "$AUTHORITY_KEYPAIR_PATH" $FEE_WALLET 1 --allow-unfunded-recipient
sleep 5

# Get pool token mint
echo "Getting pool token mint..."
POOL_TOKEN_MINT=$(spl-stake-pool list $STAKE_POOL | grep "Pool Token Mint:" | awk '{print $4}')
echo "Pool Token Mint: $POOL_TOKEN_MINT"

# Create token account for fee wallet
echo "Creating token account for fee wallet..."
FEE_WALLET_TOKEN_ACCOUNT=$(spl-token create-account $POOL_TOKEN_MINT \
    --owner $FEE_WALLET \
    --fee-payer "$AUTHORITY_KEYPAIR_PATH" \
    | grep "Creating account" | awk '{print $3}' || echo "Account already exists")

if [[ "$FEE_WALLET_TOKEN_ACCOUNT" == "Account already exists" ]]; then
    echo "Finding existing token account..."
    FEE_WALLET_TOKEN_ACCOUNT=$(spl-token accounts --owner $FEE_WALLET | grep $POOL_TOKEN_MINT | awk '{print $1}')
fi

echo "Fee Wallet Token Account: $FEE_WALLET_TOKEN_ACCOUNT"

# Verify the token account
echo "Verifying token account..."
spl-token account-info --address $FEE_WALLET_TOKEN_ACCOUNT

# Create stake deposit authority
echo "Creating stake deposit authority..."
STAKE_DEPOSIT_AUTHORITY=$(../target/debug/spl-stake-pool-interceptor interceptor create-stake-deposit-authority \
    --pool $STAKE_POOL \
    --fee-wallet $FEE_WALLET_TOKEN_ACCOUNT \
    --authority $AUTHORITY \
    --cool-down-seconds 10 \
    --initial-fee-bps 100 | tail -1)

echo "STAKE_DEPOSIT_AUTHORITY: $STAKE_DEPOSIT_AUTHORITY"

# Update stake pool's funding authority
echo "Updating stake pool's stake_deposit_authority..."
spl-stake-pool set-funding-authority $STAKE_POOL \
    stake-deposit \
    $STAKE_DEPOSIT_AUTHORITY \
    --funding-authority "$AUTHORITY_KEYPAIR_PATH"

if [ $? -ne 0 ]; then
    echo "Error: Failed to update stake_deposit_authority."
    exit 1
fi

echo "Setup completed successfully"

# Print important information
echo ""
echo "Summary:"
echo "Pool: $STAKE_POOL"
echo "Pool Token Mint: $POOL_TOKEN_MINT"
echo "Fee Wallet: $FEE_WALLET"
echo "Fee Wallet Token Account: $FEE_WALLET_TOKEN_ACCOUNT"
echo "Stake Deposit Authority: $STAKE_DEPOSIT_AUTHORITY"