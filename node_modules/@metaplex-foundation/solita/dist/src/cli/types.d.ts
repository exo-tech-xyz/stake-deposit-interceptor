import { RustbinConfig } from '@metaplex-foundation/rustbin';
import { Idl, Serializers, TypeAliases } from '../types';
export { RustbinConfig };
export type SolitaConfigBase = {
    programName: string;
    idlDir: string;
    sdkDir: string;
    binaryInstallDir: string;
    programDir: string;
    idlHook?: (idl: Idl) => Idl;
    rustbin?: RustbinConfig;
    typeAliases?: TypeAliases;
    serializers?: Serializers;
    removeExistingIdl?: boolean;
    binaryArgs?: string;
};
export type SolitaConfigAnchor = SolitaConfigBase & {
    idlGenerator: 'anchor';
    programId: string;
    anchorRemainingAccounts?: boolean;
};
export type SolitaConfigShank = SolitaConfigBase & {
    idlGenerator: 'shank';
};
export type SolitaConfig = SolitaConfigAnchor | SolitaConfigShank;
export type SolitaHandlerResult = {
    exitCode: number;
    errorMsg?: string;
};
export declare function isSolitaConfigAnchor(config: SolitaConfig): config is SolitaConfigAnchor;
export declare function isSolitaConfigShank(config: SolitaConfig): config is SolitaConfigShank;
export declare function isErrorResult(result: SolitaHandlerResult): result is SolitaHandlerResult & {
    errorMsg: string;
};
