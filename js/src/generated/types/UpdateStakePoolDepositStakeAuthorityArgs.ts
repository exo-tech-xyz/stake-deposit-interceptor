/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js'
import * as beet from '@metaplex-foundation/beet'
import * as beetSolana from '@metaplex-foundation/beet-solana'
export type UpdateStakePoolDepositStakeAuthorityArgs = {
  feeWallet: beet.COption<web3.PublicKey>
  coolDownSeconds: beet.COption<beet.bignum>
  initialFeeBps: beet.COption<number>
}

/**
 * @category userTypes
 * @category generated
 */
export const updateStakePoolDepositStakeAuthorityArgsBeet =
  new beet.FixableBeetArgsStruct<UpdateStakePoolDepositStakeAuthorityArgs>(
    [
      ['feeWallet', beet.coption(beetSolana.publicKey)],
      ['coolDownSeconds', beet.coption(beet.u64)],
      ['initialFeeBps', beet.coption(beet.u32)],
    ],
    'UpdateStakePoolDepositStakeAuthorityArgs'
  )
