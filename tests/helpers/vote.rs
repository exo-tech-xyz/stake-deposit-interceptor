use solana_program_test::BanksClient;
use solana_sdk::{hash::Hash, signature::Keypair, signer::Signer, system_instruction, system_program, transaction::Transaction, vote::state::{VoteInit, VoteState}};
use solana_vote_program::vote_instruction;

pub async fn create_vote(
  banks_client: &mut BanksClient,
  payer: &Keypair,
  recent_blockhash: &Hash,
  validator: &Keypair,
  vote: &Keypair,
) {
  let rent = banks_client.get_rent().await.unwrap();
  let rent_voter = rent.minimum_balance(VoteState::size_of());

  let mut instructions = vec![system_instruction::create_account(
      &payer.pubkey(),
      &validator.pubkey(),
      rent.minimum_balance(0),
      0,
      &system_program::id(),
  )];
  instructions.append(&mut vote_instruction::create_account_with_config(
      &payer.pubkey(),
      &vote.pubkey(),
      &VoteInit {
          node_pubkey: validator.pubkey(),
          authorized_voter: validator.pubkey(),
          ..VoteInit::default()
      },
      rent_voter,
      vote_instruction::CreateVoteAccountConfig {
          space: VoteState::size_of() as u64,
          ..Default::default()
      },
  ));

  let transaction = Transaction::new_signed_with_payer(
      &instructions,
      Some(&payer.pubkey()),
      &[validator, vote, payer],
      *recent_blockhash,
  );
  banks_client.process_transaction(transaction).await.unwrap();
}