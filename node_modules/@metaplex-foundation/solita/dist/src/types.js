"use strict";
// Currently a limited version of the types found inside https://github.com/project-serum/anchor/blob/master/ts/src/idl.ts
// Will be extended to include the full spec eventually. At this point only cases actually encountered in contracts were
// addressed
Object.defineProperty(exports, "__esModule", { value: true });
exports.PROGRAM_ID_EXPORT_NAME = exports.PROGRAM_ID_PACKAGE = exports.SOLANA_SPL_TOKEN_EXPORT_NAME = exports.SOLANA_WEB3_EXPORT_NAME = exports.BEET_SOLANA_EXPORT_NAME = exports.BEET_EXPORT_NAME = exports.SOLANA_SPL_TOKEN_PACKAGE = exports.SOLANA_WEB3_PACKAGE = exports.BEET_SOLANA_PACKAGE = exports.BEET_PACKAGE = exports.isPrimitiveType = exports.isNumberLikeType = exports.BIGNUM = exports.hasPaddingAttr = exports.isAccountsCollection = exports.isIdlInstructionAccountWithDesc = exports.isShankIdlInstruction = exports.isAnchorIdl = exports.isShankIdl = exports.isFieldsType = exports.isIdlFieldType = exports.isIdlFieldsType = exports.isIdlTypeSet = exports.isIdlTypeBTreeSet = exports.isIdlTypeHashSet = exports.isIdlTypeMap = exports.isIdlTypeBTreeMap = exports.isIdlTypeHashMap = exports.isIdlTypeTuple = exports.isDataEnumVariantWithUnnamedFields = exports.isDataEnumVariantWithNamedFields = exports.isDataEnumVariant = exports.isIdlTypeScalarEnum = exports.isIdlTypeDataEnum = exports.isIdlTypeEnum = exports.isIdlTypeDefined = exports.asIdlTypeArray = exports.isIdlTypeArray = exports.isIdlTypeVec = exports.isIdlTypeOption = exports.IDL_FIELD_ATTR_PADDING = void 0;
const beet_1 = require("@metaplex-foundation/beet");
const assert_1 = require("assert");
exports.IDL_FIELD_ATTR_PADDING = 'padding';
// -----------------
// Guards
// -----------------
function isIdlTypeOption(ty) {
    return ty.option != null;
}
exports.isIdlTypeOption = isIdlTypeOption;
function isIdlTypeVec(ty) {
    return ty.vec != null;
}
exports.isIdlTypeVec = isIdlTypeVec;
function isIdlTypeArray(ty) {
    return ty.array != null;
}
exports.isIdlTypeArray = isIdlTypeArray;
function asIdlTypeArray(ty) {
    (0, assert_1.strict)(isIdlTypeArray(ty));
    return ty;
}
exports.asIdlTypeArray = asIdlTypeArray;
function isIdlTypeDefined(ty) {
    return ty.defined != null;
}
exports.isIdlTypeDefined = isIdlTypeDefined;
function isIdlTypeEnum(ty) {
    return ty.variants != null;
}
exports.isIdlTypeEnum = isIdlTypeEnum;
// -----------------
// Enums
// -----------------
function isIdlTypeDataEnum(ty) {
    const dataEnum = ty;
    return (dataEnum.variants != null &&
        dataEnum.variants.length > 0 &&
        // if only one variant has data then we have to treat the entire enum as a data enum
        // since we can no longer represent it as a TypeScript enum
        dataEnum.variants.some(isDataEnumVariant));
}
exports.isIdlTypeDataEnum = isIdlTypeDataEnum;
function isIdlTypeScalarEnum(ty) {
    return isIdlTypeEnum(ty) && !isIdlTypeDataEnum(ty);
}
exports.isIdlTypeScalarEnum = isIdlTypeScalarEnum;
function isDataEnumVariant(ty) {
    return (ty.fields != null);
}
exports.isDataEnumVariant = isDataEnumVariant;
function isDataEnumVariantWithNamedFields(ty) {
    return (isDataEnumVariant(ty) &&
        ty.fields[0].name != null);
}
exports.isDataEnumVariantWithNamedFields = isDataEnumVariantWithNamedFields;
function isDataEnumVariantWithUnnamedFields(ty) {
    return !isDataEnumVariantWithNamedFields(ty);
}
exports.isDataEnumVariantWithUnnamedFields = isDataEnumVariantWithUnnamedFields;
// -----------------
// Tuple
// -----------------
function isIdlTypeTuple(ty) {
    return ty.tuple != null;
}
exports.isIdlTypeTuple = isIdlTypeTuple;
// -----------------
// Maps
// -----------------
function isIdlTypeHashMap(ty) {
    return ty.hashMap != null;
}
exports.isIdlTypeHashMap = isIdlTypeHashMap;
function isIdlTypeBTreeMap(ty) {
    return ty.bTreeMap != null;
}
exports.isIdlTypeBTreeMap = isIdlTypeBTreeMap;
function isIdlTypeMap(ty) {
    return isIdlTypeHashMap(ty) || isIdlTypeBTreeMap(ty);
}
exports.isIdlTypeMap = isIdlTypeMap;
// -----------------
// Sets
// -----------------
function isIdlTypeHashSet(ty) {
    return ty.hashSet != null;
}
exports.isIdlTypeHashSet = isIdlTypeHashSet;
function isIdlTypeBTreeSet(ty) {
    return ty.bTreeSet != null;
}
exports.isIdlTypeBTreeSet = isIdlTypeBTreeSet;
function isIdlTypeSet(ty) {
    return isIdlTypeHashSet(ty) || isIdlTypeBTreeSet(ty);
}
exports.isIdlTypeSet = isIdlTypeSet;
// -----------------
// Fields
// -----------------
function isIdlFieldsType(ty) {
    return ty.fields != null;
}
exports.isIdlFieldsType = isIdlFieldsType;
function isIdlFieldType(ty) {
    const fieldTy = ty;
    return fieldTy.type != null && fieldTy.name != null;
}
exports.isIdlFieldType = isIdlFieldType;
// -----------------
// Struct/Enum
// -----------------
function isFieldsType(ty) {
    const dety = ty;
    return ((dety.kind === 'enum' || dety.kind === 'struct') &&
        Array.isArray(dety.fields));
}
exports.isFieldsType = isFieldsType;
// -----------------
// Idl
// -----------------
function isShankIdl(ty) {
    var _a;
    return ((_a = ty.metadata) === null || _a === void 0 ? void 0 : _a.origin) === 'shank';
}
exports.isShankIdl = isShankIdl;
function isAnchorIdl(ty) {
    // This needs to change once we support more than two IDL generators
    return !isShankIdl(ty);
}
exports.isAnchorIdl = isAnchorIdl;
function isShankIdlInstruction(ty) {
    return typeof ty.discriminant === 'object';
}
exports.isShankIdlInstruction = isShankIdlInstruction;
function isIdlInstructionAccountWithDesc(ty) {
    return typeof ty.desc === 'string';
}
exports.isIdlInstructionAccountWithDesc = isIdlInstructionAccountWithDesc;
// -----------------
// Accounts
// -----------------
function isAccountsCollection(account) {
    return account.accounts != null;
}
exports.isAccountsCollection = isAccountsCollection;
// -----------------
// Padding
// -----------------
function hasPaddingAttr(field) {
    return field.attrs != null && field.attrs.includes(exports.IDL_FIELD_ATTR_PADDING);
}
exports.hasPaddingAttr = hasPaddingAttr;
exports.BIGNUM = [
    'u64',
    'u128',
    'u256',
    'u512',
    'i64',
    'i128',
    'i256',
    'i512',
];
function isNumberLikeType(ty) {
    return (typeof ty === 'string' && beet_1.numbersTypeMap[ty] != null);
}
exports.isNumberLikeType = isNumberLikeType;
function isPrimitiveType(ty) {
    return isNumberLikeType(ty) && !exports.BIGNUM.includes(ty);
}
exports.isPrimitiveType = isPrimitiveType;
// -----------------
// Packages
// -----------------
exports.BEET_PACKAGE = '@metaplex-foundation/beet';
exports.BEET_SOLANA_PACKAGE = '@metaplex-foundation/beet-solana';
exports.SOLANA_WEB3_PACKAGE = '@solana/web3.js';
exports.SOLANA_SPL_TOKEN_PACKAGE = '@solana/spl-token';
exports.BEET_EXPORT_NAME = 'beet';
exports.BEET_SOLANA_EXPORT_NAME = 'beetSolana';
exports.SOLANA_WEB3_EXPORT_NAME = 'web3';
exports.SOLANA_SPL_TOKEN_EXPORT_NAME = 'splToken';
exports.PROGRAM_ID_PACKAGE = '<program-id>';
exports.PROGRAM_ID_EXPORT_NAME = '<program-id-export>';
//# sourceMappingURL=types.js.map