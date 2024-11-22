/// <reference types="node" />
import { IdlField, IdlInstructionArg, IdlType, PrimaryTypeMap, PrimitiveTypeKey, TypeMappedSerdeField } from './types';
import { BeetTypeMapKey } from '@metaplex-foundation/beet';
import { BeetSolanaTypeMapKey } from '@metaplex-foundation/beet-solana';
import { SerdePackage } from './serdes';
import { PathLike } from 'fs';
export declare function resolveSerdeAlias(ty: string): string;
export type ForceFixable = (ty: IdlType) => boolean;
export declare const FORCE_FIXABLE_NEVER: ForceFixable;
export declare class TypeMapper {
    /** Account types mapped { typeName: fullPath } */
    private readonly accountTypesPaths;
    /** Custom types mapped { typeName: fullPath } */
    private readonly customTypesPaths;
    /** Aliases mapped { alias: actualType } */
    private readonly typeAliases;
    private readonly forceFixable;
    private readonly primaryTypeMap;
    readonly serdePackagesUsed: Set<SerdePackage>;
    readonly localImportsByPath: Map<string, Set<string>>;
    readonly scalarEnumsUsed: Map<string, string[]>;
    usedFixableSerde: boolean;
    constructor(
    /** Account types mapped { typeName: fullPath } */
    accountTypesPaths?: Map<string, string>, 
    /** Custom types mapped { typeName: fullPath } */
    customTypesPaths?: Map<string, string>, 
    /** Aliases mapped { alias: actualType } */
    typeAliases?: Map<string, PrimitiveTypeKey>, forceFixable?: ForceFixable, primaryTypeMap?: PrimaryTypeMap);
    clearUsages(): void;
    clone(): TypeMapper;
    /**
     * When using a cloned typemapper temporarily in order to track usages for a
     * subset of mappings we need to sync the main mapper to include the updates
     * captured by the sub mapper. This is what this method does.
     */
    syncUp(tm: TypeMapper): void;
    private updateUsedFixableSerde;
    private updateScalarEnumsUsed;
    private mapPrimitiveType;
    private mapOptionType;
    private mapVecType;
    private mapArrayType;
    private mapTupleType;
    private mapBTreeMapType;
    private mapHashMapType;
    private mapMapType;
    private mapBTreeSetType;
    private mapHashSetType;
    private mapSetType;
    private mapDefinedType;
    private mapEnumType;
    map(ty: IdlType, name?: string): string;
    private mapPrimitiveSerde;
    private mapStringSerde;
    private mapOptionSerde;
    private mapVecSerde;
    private mapArraySerde;
    private mapDefinedSerde;
    private mapEnumSerde;
    private mapTupleSerde;
    private mapBTreeMapSerde;
    private mapHashMapSerde;
    private mapMapSerde;
    private mapBTreeSetSerde;
    private mapHashSetSerde;
    private mapSetSerde;
    mapSerde(ty: IdlType, name?: string): string;
    mapSerdeField: (field: IdlField | IdlInstructionArg) => TypeMappedSerdeField;
    mapSerdeFields(fields: (IdlField | IdlInstructionArg)[]): TypeMappedSerdeField[];
    importsUsed(fileDir: PathLike, forcePackages?: Set<SerdePackage>): string[];
    private _importsForSerdePackages;
    private _importsForLocalPackages;
    assertBeetSupported(serde: IdlType, context: string): asserts serde is BeetTypeMapKey | BeetSolanaTypeMapKey;
    private definedTypesImport;
    static defaultPrimaryTypeMap: PrimaryTypeMap;
}
