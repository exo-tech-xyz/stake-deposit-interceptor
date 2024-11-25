/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category ChangeDepositReceiptOwner
 * @category generated
 */
export const ChangeDepositReceiptOwnerStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number
}>(
  [['instructionDiscriminator', beet.u8]],
  'ChangeDepositReceiptOwnerInstructionArgs'
)
/**
 * Accounts required by the _ChangeDepositReceiptOwner_ instruction
 *
 * @property [_writable_] depositReceipt
 * @property [**signer**] currentOwner
 * @property [] newOwner
 * @category Instructions
 * @category ChangeDepositReceiptOwner
 * @category generated
 */
export type ChangeDepositReceiptOwnerInstructionAccounts = {
  depositReceipt: web3.PublicKey
  currentOwner: web3.PublicKey
  newOwner: web3.PublicKey
}

export const changeDepositReceiptOwnerInstructionDiscriminator = 4

/**
 * Creates a _ChangeDepositReceiptOwner_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category ChangeDepositReceiptOwner
 * @category generated
 */
export function createChangeDepositReceiptOwnerInstruction(
  accounts: ChangeDepositReceiptOwnerInstructionAccounts,
  programId = new web3.PublicKey('5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV')
) {
  const [data] = ChangeDepositReceiptOwnerStruct.serialize({
    instructionDiscriminator: changeDepositReceiptOwnerInstructionDiscriminator,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.depositReceipt,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.currentOwner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.newOwner,
      isWritable: false,
      isSigner: false,
    },
  ]

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
