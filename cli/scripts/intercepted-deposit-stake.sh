# Path to keypair
staker_keypair=$1
# Pubkey
stake_deposit_authority=$2
# Number
amount=$3

stake_account_keypair="./keys/stake-account.json"
vote_account_keypair="./keys/vote_2.json"

staker_pubkey=$(solana-keygen pubkey "$staker_keypair")
stake_account_pubkey=$(solana-keygen pubkey "$stake_account_keypair")

# airdrop sol to staker
solana airdrop $amount $staker_pubkey

# Create keypair for Stake account
# NOTE: will overwrite existing keypair
solana-keygen new --no-passphrase -o $stake_account_keypair --force

echo "Creating stake account"

# create stake account and stake sol
solana create-stake-account --from $staker_keypair $stake_account_keypair $amount --stake-authority $staker_keypair --withdraw-authority $staker_keypair

echo "Delegating stake"

# delegate the stake to a specific validator
solana delegate-stake $stake_account_keypair $vote_account_keypair --stake-authority $staker_keypair --force

echo "Depositing stake via Interceptor"

# Deposit stake into the stake-pool using the Interceptor
../target/debug/spl-stake-pool-interceptor interceptor deposit-stake $stake_deposit_authority  $stake_account_pubkey --withdraw-authority $staker_keypair
