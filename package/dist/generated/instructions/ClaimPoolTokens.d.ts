/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
/**
 * @category Instructions
 * @category ClaimPoolTokens
 * @category generated
 */
export declare const ClaimPoolTokensStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number;
}>;
/**
 * Accounts required by the _ClaimPoolTokens_ instruction
 *
 * @property [_writable_] depositReceipt
 * @property [_writable_, **signer**] owner
 * @property [_writable_] vault
 * @property [_writable_] destination
 * @property [_writable_] feeWallet
 * @property [] depositAuthority
 * @property [] poolMint
 * @category Instructions
 * @category ClaimPoolTokens
 * @category generated
 */
export type ClaimPoolTokensInstructionAccounts = {
    depositReceipt: web3.PublicKey;
    owner: web3.PublicKey;
    vault: web3.PublicKey;
    destination: web3.PublicKey;
    feeWallet: web3.PublicKey;
    depositAuthority: web3.PublicKey;
    poolMint: web3.PublicKey;
    tokenProgram?: web3.PublicKey;
    systemProgram?: web3.PublicKey;
};
export declare const claimPoolTokensInstructionDiscriminator = 5;
/**
 * Creates a _ClaimPoolTokens_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category ClaimPoolTokens
 * @category generated
 */
export declare function createClaimPoolTokensInstruction(accounts: ClaimPoolTokensInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;