/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js'
import * as beet from '@metaplex-foundation/beet'
import * as beetSolana from '@metaplex-foundation/beet-solana'

/**
 * Arguments used to create {@link DepositReceipt}
 * @category Accounts
 * @category generated
 */
export type DepositReceiptArgs = {
  base: web3.PublicKey
  owner: web3.PublicKey
  stakePool: web3.PublicKey
  stakePoolDepositStakeAuthority: web3.PublicKey
  depositTime: beet.bignum
  lstAmount: beet.bignum
  coolDownSeconds: beet.bignum
  initialFeeBps: number
  bumpSeed: number
  reserved: number[] /* size: 256 */
}
/**
 * Holds the data for the {@link DepositReceipt} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export class DepositReceipt implements DepositReceiptArgs {
  private constructor(
    readonly base: web3.PublicKey,
    readonly owner: web3.PublicKey,
    readonly stakePool: web3.PublicKey,
    readonly stakePoolDepositStakeAuthority: web3.PublicKey,
    readonly depositTime: beet.bignum,
    readonly lstAmount: beet.bignum,
    readonly coolDownSeconds: beet.bignum,
    readonly initialFeeBps: number,
    readonly bumpSeed: number,
    readonly reserved: number[] /* size: 256 */
  ) {}

  /**
   * Creates a {@link DepositReceipt} instance from the provided args.
   */
  static fromArgs(args: DepositReceiptArgs) {
    return new DepositReceipt(
      args.base,
      args.owner,
      args.stakePool,
      args.stakePoolDepositStakeAuthority,
      args.depositTime,
      args.lstAmount,
      args.coolDownSeconds,
      args.initialFeeBps,
      args.bumpSeed,
      args.reserved
    )
  }

  /**
   * Deserializes the {@link DepositReceipt} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0
  ): [DepositReceipt, number] {
    return DepositReceipt.deserialize(accountInfo.data, offset)
  }

  /**
   * Retrieves the account info from the provided address and deserializes
   * the {@link DepositReceipt} from its data.
   *
   * @throws Error if no account info is found at the address or if deserialization fails
   */
  static async fromAccountAddress(
    connection: web3.Connection,
    address: web3.PublicKey,
    commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig
  ): Promise<DepositReceipt> {
    const accountInfo = await connection.getAccountInfo(
      address,
      commitmentOrConfig
    )
    if (accountInfo == null) {
      throw new Error(`Unable to find DepositReceipt account at ${address}`)
    }
    return DepositReceipt.fromAccountInfo(accountInfo, 0)[0]
  }

  /**
   * Provides a {@link web3.Connection.getProgramAccounts} config builder,
   * to fetch accounts matching filters that can be specified via that builder.
   *
   * @param programId - the program that owns the accounts we are filtering
   */
  static gpaBuilder(
    programId: web3.PublicKey = new web3.PublicKey(
      '5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV'
    )
  ) {
    return beetSolana.GpaBuilder.fromStruct(programId, depositReceiptBeet)
  }

  /**
   * Deserializes the {@link DepositReceipt} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [DepositReceipt, number] {
    return depositReceiptBeet.deserialize(buf, offset)
  }

  /**
   * Serializes the {@link DepositReceipt} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return depositReceiptBeet.serialize(this)
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link DepositReceipt}
   */
  static get byteSize() {
    return depositReceiptBeet.byteSize
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link DepositReceipt} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      DepositReceipt.byteSize,
      commitment
    )
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link DepositReceipt} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === DepositReceipt.byteSize
  }

  /**
   * Returns a readable version of {@link DepositReceipt} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      base: this.base.toBase58(),
      owner: this.owner.toBase58(),
      stakePool: this.stakePool.toBase58(),
      stakePoolDepositStakeAuthority:
        this.stakePoolDepositStakeAuthority.toBase58(),
      depositTime: (() => {
        const x = <{ toNumber: () => number }>this.depositTime
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
      lstAmount: (() => {
        const x = <{ toNumber: () => number }>this.lstAmount
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
      coolDownSeconds: (() => {
        const x = <{ toNumber: () => number }>this.coolDownSeconds
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
      initialFeeBps: this.initialFeeBps,
      bumpSeed: this.bumpSeed,
      reserved: this.reserved,
    }
  }
}

/**
 * @category Accounts
 * @category generated
 */
export const depositReceiptBeet = new beet.BeetStruct<
  DepositReceipt,
  DepositReceiptArgs
>(
  [
    ['base', beetSolana.publicKey],
    ['owner', beetSolana.publicKey],
    ['stakePool', beetSolana.publicKey],
    ['stakePoolDepositStakeAuthority', beetSolana.publicKey],
    ['depositTime', beet.u64],
    ['lstAmount', beet.u64],
    ['coolDownSeconds', beet.u64],
    ['initialFeeBps', beet.u32],
    ['bumpSeed', beet.u8],
    ['reserved', beet.uniformFixedSizeArray(beet.u8, 256)],
  ],
  DepositReceipt.fromArgs,
  'DepositReceipt'
)