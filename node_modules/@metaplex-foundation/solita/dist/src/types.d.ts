import { BeetExports, BeetTypeMapKey, NumbersTypeMapKey, SupportedTypeDefinition } from '@metaplex-foundation/beet';
import { BeetSolanaExports, BeetSolanaTypeMapKey } from '@metaplex-foundation/beet-solana';
import { SerdePackage } from './serdes';
export type TypeAliases = Record<string, PrimitiveTypeKey>;
/**
 * Key: account name for which to customize de/serializer
 * Value: path to module from project root providing `serialize` and/or
 *        `deserialize` methods
 */
export type Serializers = Record<string, string>;
export type IdlField = {
    name: string;
    type: IdlType;
    attrs?: string[];
};
export declare const IDL_FIELD_ATTR_PADDING = "padding";
export type IdlInstructionAccount = {
    name: string;
    isMut: boolean;
    isSigner: boolean;
    desc?: string;
    optional?: boolean;
    isOptional?: boolean;
};
export type IdlType = BeetTypeMapKey | 'publicKey' | IdlTypeDefined | IdlTypeOption | IdlTypeVec | IdlTypeArray | IdlTypeEnum | IdlTypeDataEnum | IdlTypeTuple | IdlTypeMap | IdlTypeSet;
export type IdlTypeDefined = {
    defined: string;
};
export type IdlTypeOption = {
    option: IdlType;
};
export type IdlTypeVec = {
    vec: IdlType;
};
export type IdlTypeArray = {
    array: [idlType: IdlType, size: number];
};
export type IdlEnumVariant = {
    name: string;
};
export type IdlDataEnumVariant = IdlDataEnumVariantWithNamedFields | IdlDataEnumVariantWithUnnamedFields | IdlEnumVariant;
export type IdlDataEnumVariantWithNamedFields = {
    name: string;
    fields: IdlField[];
};
export type IdlDataEnumVariantWithUnnamedFields = {
    name: string;
    fields: IdlType[];
};
export type IdlTypeEnum = IdlTypeScalarEnum | IdlTypeDataEnum;
export type IdlTypeScalarEnum = {
    kind: 'enum';
    name?: string;
    variants: IdlEnumVariant[];
};
export type IdlTypeDataEnum = {
    kind: 'enum';
    name?: string;
    variants: IdlDataEnumVariant[];
};
export type IdlTypeTuple = {
    tuple: IdlType[];
};
export type IdlTypeMap = IdlTypeHashMap | IdlTypeBTreeMap;
export type IdlTypeHashMap = {
    hashMap: [IdlType, IdlType];
};
export type IdlTypeBTreeMap = {
    bTreeMap: [IdlType, IdlType];
};
export type IdlTypeSet = IdlTypeHashSet | IdlTypeBTreeSet;
export type IdlTypeHashSet = {
    hashSet: IdlType;
};
export type IdlTypeBTreeSet = {
    bTreeSet: IdlType;
};
export type IdlFieldsType = {
    kind: 'struct' | 'enum';
    fields: IdlField[];
};
export type IdlDefinedTypeDefinition = {
    name: string;
    type: IdlFieldsType | IdlTypeEnum | IdlTypeDataEnum;
};
export type IdlInstructionArg = {
    name: string;
    type: IdlType;
};
export type IdlInstruction = {
    name: string;
    defaultOptionalAccounts?: boolean;
    legacyOptionalAccountsStrategy?: boolean;
    accounts: IdlInstructionAccount[] | IdlAccountsCollection[];
    args: IdlInstructionArg[];
};
export type IdlAccountType = {
    kind: 'struct' | 'enum';
    fields: IdlField[];
};
export type IdlAccount = {
    name: string;
    type: IdlAccountType;
};
export type IdlAccountsCollection = {
    name: string;
    accounts: IdlInstructionAccount[];
};
export type IdlError = {
    code: number;
    name: string;
    msg?: string;
};
export type Idl = {
    version: string;
    name: string;
    instructions: IdlInstruction[];
    accounts?: IdlAccount[];
    errors?: IdlError[];
    types?: IdlDefinedTypeDefinition[];
    metadata: {
        address: string;
        origin?: IdlOrigin;
    };
};
export type IdlOrigin = 'shank' | 'anchor';
export type ShankIdl = Idl & {
    instructions: ShankIdlInstruction[];
    metadata: ShankMetadata;
};
export type ShankIdlInstruction = IdlInstruction & {
    accounts: IdlInstructionAccountWithDesc[];
    discriminant: {
        type: IdlType;
        value: number;
    };
};
export type IdlInstructionAccountWithDesc = IdlInstructionAccount & {
    desc: string;
};
export type ShankMetadata = Idl['metadata'] & {
    origin: 'shank';
};
export type PrimitiveTypeKey = BeetTypeMapKey | BeetSolanaTypeMapKey;
export type PrimaryType = SupportedTypeDefinition & {
    beet: BeetExports | BeetSolanaExports;
};
export type PrimaryTypeMap = Record<PrimitiveTypeKey, PrimaryType>;
export type ProcessedSerde = {
    name: string;
    sourcePack: SerdePackage;
    type: string;
    inner?: ProcessedSerde;
};
export type TypeMappedSerdeField = {
    name: string;
    type: string;
};
export type ResolveFieldType = (typeName: string) => IdlAccountType | IdlTypeEnum | null;
export declare function isIdlTypeOption(ty: IdlType): ty is IdlTypeOption;
export declare function isIdlTypeVec(ty: IdlType): ty is IdlTypeVec;
export declare function isIdlTypeArray(ty: IdlType): ty is IdlTypeArray;
export declare function asIdlTypeArray(ty: IdlType): IdlTypeArray;
export declare function isIdlTypeDefined(ty: IdlType): ty is IdlTypeDefined;
export declare function isIdlTypeEnum(ty: IdlType | IdlFieldsType | IdlTypeEnum): ty is IdlTypeEnum;
export declare function isIdlTypeDataEnum(ty: IdlType | IdlFieldsType | IdlTypeEnum): ty is IdlTypeDataEnum;
export declare function isIdlTypeScalarEnum(ty: IdlType | IdlFieldsType | IdlTypeEnum): ty is IdlTypeScalarEnum;
export declare function isDataEnumVariant(ty: IdlDataEnumVariant): ty is IdlDataEnumVariantWithNamedFields | IdlDataEnumVariantWithUnnamedFields;
export declare function isDataEnumVariantWithNamedFields(ty: IdlDataEnumVariant): ty is IdlDataEnumVariantWithNamedFields;
export declare function isDataEnumVariantWithUnnamedFields(ty: IdlDataEnumVariant): ty is IdlDataEnumVariantWithUnnamedFields;
export declare function isIdlTypeTuple(ty: IdlType): ty is IdlTypeTuple;
export declare function isIdlTypeHashMap(ty: IdlType): ty is IdlTypeHashMap;
export declare function isIdlTypeBTreeMap(ty: IdlType): ty is IdlTypeBTreeMap;
export declare function isIdlTypeMap(ty: IdlType): ty is IdlTypeMap;
export declare function isIdlTypeHashSet(ty: IdlType): ty is IdlTypeHashSet;
export declare function isIdlTypeBTreeSet(ty: IdlType): ty is IdlTypeBTreeSet;
export declare function isIdlTypeSet(ty: IdlType): ty is IdlTypeSet;
export declare function isIdlFieldsType(ty: IdlType | IdlFieldsType): ty is IdlFieldsType;
export declare function isIdlFieldType(ty: IdlType | IdlField): ty is IdlField;
export declare function isFieldsType(ty: IdlFieldsType | IdlTypeEnum | IdlTypeDataEnum): ty is IdlFieldsType;
export declare function isShankIdl(ty: Idl): ty is ShankIdl;
export declare function isAnchorIdl(ty: Idl): ty is ShankIdl;
export declare function isShankIdlInstruction(ty: IdlInstruction): ty is ShankIdlInstruction;
export declare function isIdlInstructionAccountWithDesc(ty: IdlInstructionAccount): ty is IdlInstructionAccountWithDesc;
export declare function isAccountsCollection(account: IdlInstructionAccount | IdlAccountsCollection): account is IdlAccountsCollection;
export declare function hasPaddingAttr(field: IdlField): boolean;
export type PrimitiveType = Exclude<NumbersTypeMapKey, typeof BIGNUM>;
export declare const BIGNUM: readonly ["u64", "u128", "u256", "u512", "i64", "i128", "i256", "i512"];
export type Bignum = (typeof BIGNUM)[number];
export declare function isNumberLikeType(ty: IdlType): ty is NumbersTypeMapKey;
export declare function isPrimitiveType(ty: IdlType): ty is PrimitiveType;
export declare const BEET_PACKAGE = "@metaplex-foundation/beet";
export declare const BEET_SOLANA_PACKAGE = "@metaplex-foundation/beet-solana";
export declare const SOLANA_WEB3_PACKAGE = "@solana/web3.js";
export declare const SOLANA_SPL_TOKEN_PACKAGE = "@solana/spl-token";
export declare const BEET_EXPORT_NAME = "beet";
export declare const BEET_SOLANA_EXPORT_NAME = "beetSolana";
export declare const SOLANA_WEB3_EXPORT_NAME = "web3";
export declare const SOLANA_SPL_TOKEN_EXPORT_NAME = "splToken";
export declare const PROGRAM_ID_PACKAGE = "<program-id>";
export declare const PROGRAM_ID_EXPORT_NAME = "<program-id-export>";
