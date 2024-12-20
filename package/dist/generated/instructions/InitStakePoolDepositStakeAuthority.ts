/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import {
  InitStakePoolDepositStakeAuthorityArgs,
  initStakePoolDepositStakeAuthorityArgsBeet,
} from '../types/InitStakePoolDepositStakeAuthorityArgs'

/**
 * @category Instructions
 * @category InitStakePoolDepositStakeAuthority
 * @category generated
 */
export type InitStakePoolDepositStakeAuthorityInstructionArgs = {
  initStakePoolDepositStakeAuthorityArgs: InitStakePoolDepositStakeAuthorityArgs
}
/**
 * @category Instructions
 * @category InitStakePoolDepositStakeAuthority
 * @category generated
 */
export const InitStakePoolDepositStakeAuthorityStruct = new beet.BeetArgsStruct<
  InitStakePoolDepositStakeAuthorityInstructionArgs & {
    instructionDiscriminator: number
  }
>(
  [
    ['instructionDiscriminator', beet.u8],
    [
      'initStakePoolDepositStakeAuthorityArgs',
      initStakePoolDepositStakeAuthorityArgsBeet,
    ],
  ],
  'InitStakePoolDepositStakeAuthorityInstructionArgs'
)
/**
 * Accounts required by the _InitStakePoolDepositStakeAuthority_ instruction
 *
 * @property [_writable_, **signer**] payer
 * @property [_writable_] depositStakeAuthority
 * @property [_writable_] vaultAta
 * @property [] authority
 * @property [**signer**] base
 * @property [] stakePool
 * @property [] stakePoolMint
 * @property [] stakePoolProgram
 * @property [] associatedTokenProgram
 * @category Instructions
 * @category InitStakePoolDepositStakeAuthority
 * @category generated
 */
export type InitStakePoolDepositStakeAuthorityInstructionAccounts = {
  payer: web3.PublicKey
  depositStakeAuthority: web3.PublicKey
  vaultAta: web3.PublicKey
  authority: web3.PublicKey
  base: web3.PublicKey
  stakePool: web3.PublicKey
  stakePoolMint: web3.PublicKey
  stakePoolProgram: web3.PublicKey
  tokenProgram?: web3.PublicKey
  associatedTokenProgram: web3.PublicKey
  systemProgram?: web3.PublicKey
}

export const initStakePoolDepositStakeAuthorityInstructionDiscriminator = 0

/**
 * Creates a _InitStakePoolDepositStakeAuthority_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category InitStakePoolDepositStakeAuthority
 * @category generated
 */
export function createInitStakePoolDepositStakeAuthorityInstruction(
  accounts: InitStakePoolDepositStakeAuthorityInstructionAccounts,
  args: InitStakePoolDepositStakeAuthorityInstructionArgs,
  programId = new web3.PublicKey('5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV')
) {
  const [data] = InitStakePoolDepositStakeAuthorityStruct.serialize({
    instructionDiscriminator:
      initStakePoolDepositStakeAuthorityInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.depositStakeAuthority,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.vaultAta,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.authority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.base,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.stakePool,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.stakePoolMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.stakePoolProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.associatedTokenProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
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