"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TypeMapper = exports.FORCE_FIXABLE_NEVER = exports.resolveSerdeAlias = void 0;
const types_1 = require("./types");
const utils_1 = require("./utils");
const assert_1 = require("assert");
const beet_1 = require("@metaplex-foundation/beet");
const beet_solana_1 = require("@metaplex-foundation/beet-solana");
const serdes_1 = require("./serdes");
const render_type_1 = require("./render-type");
const path_1 = __importDefault(require("path"));
function resolveSerdeAlias(ty) {
    switch (ty) {
        case 'option':
            return 'coption';
        default:
            return ty;
    }
}
exports.resolveSerdeAlias = resolveSerdeAlias;
const FORCE_FIXABLE_NEVER = () => false;
exports.FORCE_FIXABLE_NEVER = FORCE_FIXABLE_NEVER;
const NO_NAME_PROVIDED = '<no name provided>';
class TypeMapper {
    constructor(
    /** Account types mapped { typeName: fullPath } */
    accountTypesPaths = new Map(), 
    /** Custom types mapped { typeName: fullPath } */
    customTypesPaths = new Map(), 
    /** Aliases mapped { alias: actualType } */
    typeAliases = new Map(), forceFixable = exports.FORCE_FIXABLE_NEVER, primaryTypeMap = TypeMapper.defaultPrimaryTypeMap) {
        this.accountTypesPaths = accountTypesPaths;
        this.customTypesPaths = customTypesPaths;
        this.typeAliases = typeAliases;
        this.forceFixable = forceFixable;
        this.primaryTypeMap = primaryTypeMap;
        this.serdePackagesUsed = new Set();
        this.localImportsByPath = new Map();
        this.scalarEnumsUsed = new Map();
        this.usedFixableSerde = false;
        this.mapSerdeField = (field) => {
            const ty = this.mapSerde(field.type, field.name);
            return { name: field.name, type: ty };
        };
    }
    clearUsages() {
        this.serdePackagesUsed.clear();
        this.localImportsByPath.clear();
        this.scalarEnumsUsed.clear();
        this.usedFixableSerde = false;
    }
    clone() {
        return new TypeMapper(this.accountTypesPaths, this.customTypesPaths, this.typeAliases, this.forceFixable, this.primaryTypeMap);
    }
    /**
     * When using a cloned typemapper temporarily in order to track usages for a
     * subset of mappings we need to sync the main mapper to include the updates
     * captured by the sub mapper. This is what this method does.
     */
    syncUp(tm) {
        var _a, _b;
        for (const used of tm.serdePackagesUsed) {
            this.serdePackagesUsed.add(used);
        }
        for (const [key, val] of tm.localImportsByPath) {
            const thisVal = (_a = this.localImportsByPath.get(key)) !== null && _a !== void 0 ? _a : new Set();
            this.localImportsByPath.set(key, new Set([...thisVal, ...val]));
        }
        for (const [key, val] of tm.scalarEnumsUsed) {
            const thisVal = (_b = this.scalarEnumsUsed.get(key)) !== null && _b !== void 0 ? _b : [];
            this.scalarEnumsUsed.set(key, Array.from(new Set([...thisVal, ...val])));
        }
        this.usedFixableSerde = this.usedFixableSerde || tm.usedFixableSerde;
    }
    updateUsedFixableSerde(ty) {
        this.usedFixableSerde = this.usedFixableSerde || ty.isFixable;
    }
    updateScalarEnumsUsed(name, ty) {
        const variants = ty.variants.map((x) => x.name);
        const currentUsed = this.scalarEnumsUsed.get(name);
        if (currentUsed != null) {
            assert_1.strict.deepStrictEqual(variants, currentUsed, `Found two enum variant specs for ${name}, ${variants} and ${currentUsed}`);
        }
        else {
            this.scalarEnumsUsed.set(name, variants);
        }
    }
    // -----------------
    // Map TypeScript Type
    // -----------------
    mapPrimitiveType(ty, name) {
        this.assertBeetSupported(ty, 'map primitive type');
        const mapped = this.primaryTypeMap[ty];
        let typescriptType = mapped.ts;
        if (typescriptType == null) {
            (0, utils_1.logDebug)(`No mapped type found for ${name}: ${ty}, using any`);
            typescriptType = 'any';
        }
        if (mapped.pack != null) {
            (0, serdes_1.assertKnownSerdePackage)(mapped.pack);
            const exp = (0, serdes_1.serdePackageExportName)(mapped.pack);
            typescriptType = `${exp}.${typescriptType}`;
            this.serdePackagesUsed.add(mapped.pack);
        }
        return typescriptType;
    }
    mapOptionType(ty, name) {
        const inner = this.map(ty.option, name);
        const optionPackage = beet_1.BEET_PACKAGE;
        this.serdePackagesUsed.add(optionPackage);
        const exp = (0, serdes_1.serdePackageExportName)(optionPackage);
        return `${exp}.COption<${inner}>`;
    }
    mapVecType(ty, name) {
        const inner = this.map(ty.vec, name);
        return `${inner}[]`;
    }
    mapArrayType(ty, name) {
        const inner = this.map(ty.array[0], name);
        const size = ty.array[1];
        return `${inner}[] /* size: ${size} */`;
    }
    mapTupleType(ty, name) {
        const innerTypes = [];
        for (const inner of ty.tuple) {
            innerTypes.push(this.map(inner, name));
        }
        const inners = innerTypes.join(', ');
        return `[${inners}]`;
    }
    mapBTreeMapType(ty, name) {
        return this.mapMapType(ty.bTreeMap, name);
    }
    mapHashMapType(ty, name) {
        return this.mapMapType(ty.hashMap, name);
    }
    mapMapType(inners, name) {
        const innerTypes = [this.map(inners[0], name), this.map(inners[1], name)];
        // Overcoming TypeScript issues related to `toFixedFromValue` which considers `bignum`
        // incompat with `Partial<bignum>`.
        // If this can be fixed in beet instead we won't need this overspecification anymore.
        const innerTy1 = !(0, types_1.isNumberLikeType)(inners[1]) || (0, types_1.isPrimitiveType)(inners[1])
            ? innerTypes[1]
            : `Partial<${innerTypes[1]}>`;
        return `Map<${innerTypes[0]}, ${innerTy1}>`;
    }
    mapBTreeSetType(ty, name) {
        return this.mapSetType(ty.bTreeSet, name);
    }
    mapHashSetType(ty, name) {
        return this.mapSetType(ty.hashSet, name);
    }
    mapSetType(inner, name) {
        const innerType = this.map(inner, name);
        return `Set<${innerType}>`;
    }
    mapDefinedType(ty) {
        const fullFileDir = this.definedTypesImport(ty);
        const imports = (0, utils_1.getOrCreate)(this.localImportsByPath, fullFileDir, new Set());
        imports.add(ty.defined);
        return ty.defined;
    }
    mapEnumType(ty, name) {
        if (name === NO_NAME_PROVIDED && ty.name != null) {
            name = ty.name;
        }
        assert_1.strict.notEqual(name, NO_NAME_PROVIDED, 'Need to provide name for enum types');
        this.updateScalarEnumsUsed(name, ty);
        return name;
    }
    map(ty, name = NO_NAME_PROVIDED) {
        (0, assert_1.strict)(ty != null, `Type for ${name} needs to be defined`);
        if (typeof ty === 'string') {
            return this.mapPrimitiveType(ty, name);
        }
        if ((0, types_1.isIdlTypeOption)(ty)) {
            return this.mapOptionType(ty, name);
        }
        if ((0, types_1.isIdlTypeVec)(ty)) {
            return this.mapVecType(ty, name);
        }
        if ((0, types_1.isIdlTypeArray)(ty)) {
            return this.mapArrayType(ty, name);
        }
        if ((0, types_1.isIdlTypeDefined)(ty)) {
            const alias = this.typeAliases.get(ty.defined);
            return alias == null
                ? this.mapDefinedType(ty)
                : this.mapPrimitiveType(alias, name);
        }
        if ((0, types_1.isIdlTypeEnum)(ty)) {
            return this.mapEnumType(ty, name);
        }
        if ((0, types_1.isIdlTypeTuple)(ty)) {
            return this.mapTupleType(ty, name);
        }
        if ((0, types_1.isIdlTypeHashMap)(ty)) {
            return this.mapHashMapType(ty, name);
        }
        if ((0, types_1.isIdlTypeBTreeMap)(ty)) {
            return this.mapBTreeMapType(ty, name);
        }
        if ((0, types_1.isIdlTypeHashSet)(ty)) {
            return this.mapHashSetType(ty, name);
        }
        if ((0, types_1.isIdlTypeBTreeSet)(ty)) {
            return this.mapBTreeSetType(ty, name);
        }
        console.log(ty);
        throw new Error(`Type ${ty} required for ${name} is not yet supported`);
    }
    // -----------------
    // Map Serde
    // -----------------
    mapPrimitiveSerde(ty, name) {
        this.assertBeetSupported(ty, `account field ${name}`);
        if (ty === 'string')
            return this.mapStringSerde(ty);
        const mapped = this.primaryTypeMap[ty];
        (0, serdes_1.assertKnownSerdePackage)(mapped.sourcePack);
        const packExportName = (0, serdes_1.serdePackageExportName)(mapped.sourcePack);
        this.serdePackagesUsed.add(mapped.sourcePack);
        this.updateUsedFixableSerde(mapped);
        return `${packExportName}.${ty}`;
    }
    mapStringSerde(ty) {
        const mapped = this.primaryTypeMap[ty];
        (0, serdes_1.assertKnownSerdePackage)(mapped.sourcePack);
        const packExportName = (0, serdes_1.serdePackageExportName)(mapped.sourcePack);
        this.serdePackagesUsed.add(mapped.sourcePack);
        this.updateUsedFixableSerde(mapped);
        return `${packExportName}.${mapped.beet}`;
    }
    mapOptionSerde(ty, name) {
        const inner = this.mapSerde(ty.option, name);
        const optionPackage = beet_1.BEET_PACKAGE;
        this.serdePackagesUsed.add(optionPackage);
        this.usedFixableSerde = true;
        const exp = (0, serdes_1.serdePackageExportName)(optionPackage);
        return `${exp}.coption(${inner})`;
    }
    mapVecSerde(ty, name) {
        const inner = this.mapSerde(ty.vec, name);
        const arrayPackage = beet_1.BEET_PACKAGE;
        this.serdePackagesUsed.add(arrayPackage);
        this.usedFixableSerde = true;
        const exp = (0, serdes_1.serdePackageExportName)(arrayPackage);
        return `${exp}.array(${inner})`;
    }
    mapArraySerde(ty, name) {
        const inner = this.mapSerde(ty.array[0], name);
        const size = ty.array[1];
        const mapped = this.primaryTypeMap['UniformFixedSizeArray'];
        const arrayPackage = mapped.sourcePack;
        (0, serdes_1.assertKnownSerdePackage)(arrayPackage);
        this.serdePackagesUsed.add(arrayPackage);
        this.updateUsedFixableSerde(mapped);
        const exp = (0, serdes_1.serdePackageExportName)(arrayPackage);
        return `${exp}.${mapped.beet}(${inner}, ${size})`;
    }
    mapDefinedSerde(ty) {
        const fullFileDir = this.definedTypesImport(ty);
        const imports = (0, utils_1.getOrCreate)(this.localImportsByPath, fullFileDir, new Set());
        const varName = (0, render_type_1.beetVarNameFromTypeName)(ty.defined);
        imports.add(varName);
        return varName;
    }
    mapEnumSerde(ty, name) {
        if (name === NO_NAME_PROVIDED && ty.name != null) {
            name = ty.name;
        }
        assert_1.strict.notEqual(name, NO_NAME_PROVIDED, 'Need to provide name for enum types');
        const scalarEnumPackage = beet_1.BEET_PACKAGE;
        const exp = (0, serdes_1.serdePackageExportName)(beet_1.BEET_PACKAGE);
        this.serdePackagesUsed.add(scalarEnumPackage);
        this.updateScalarEnumsUsed(name, ty);
        return `${exp}.fixedScalarEnum(${name})`;
    }
    mapTupleSerde(ty, name) {
        const tuplePackage = beet_1.BEET_PACKAGE;
        const exp = (0, serdes_1.serdePackageExportName)(beet_1.BEET_PACKAGE);
        this.serdePackagesUsed.add(tuplePackage);
        const innerSerdes = [];
        const innerMapper = this.clone();
        for (const inner of ty.tuple) {
            innerSerdes.push(innerMapper.mapSerde(inner, name));
        }
        this.syncUp(innerMapper);
        const inners = innerSerdes.join(', ');
        if (innerMapper.usedFixableSerde) {
            const tuple = this.primaryTypeMap.Tuple;
            return `${exp}.${tuple.beet}([${inners}])`;
        }
        else {
            const fixedTuple = this.primaryTypeMap.FixedSizeTuple;
            return `${exp}.${fixedTuple.beet}([${inners}])`;
        }
    }
    mapBTreeMapSerde(ty, name) {
        return this.mapMapSerde(ty.bTreeMap, name);
    }
    mapHashMapSerde(ty, name) {
        return this.mapMapSerde(ty.hashMap, name);
    }
    mapMapSerde(inners, name) {
        const mapPackage = beet_1.BEET_PACKAGE;
        const exp = (0, serdes_1.serdePackageExportName)(beet_1.BEET_PACKAGE);
        this.serdePackagesUsed.add(mapPackage);
        this.usedFixableSerde = true;
        const [key, val] = [
            this.mapSerde(inners[0], name),
            this.mapSerde(inners[1], name),
        ];
        const map = this.primaryTypeMap.Map;
        return `${exp}.${map.beet}(${key}, ${val})`;
    }
    mapBTreeSetSerde(ty, name) {
        return this.mapSetSerde(ty.bTreeSet, name);
    }
    mapHashSetSerde(ty, name) {
        return this.mapSetSerde(ty.hashSet, name);
    }
    mapSetSerde(inner, name) {
        const mapPackage = beet_1.BEET_PACKAGE;
        const exp = (0, serdes_1.serdePackageExportName)(beet_1.BEET_PACKAGE);
        this.serdePackagesUsed.add(mapPackage);
        this.usedFixableSerde = true;
        const key = this.mapSerde(inner, name);
        const set = this.primaryTypeMap.Set;
        return `${exp}.${set.beet}(${key})`;
    }
    mapSerde(ty, name = NO_NAME_PROVIDED) {
        (0, assert_1.strict)(ty != null, `Type for ${name} needs to be defined`);
        if (this.forceFixable(ty)) {
            this.usedFixableSerde = true;
        }
        if (typeof ty === 'string') {
            return this.mapPrimitiveSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeOption)(ty)) {
            return this.mapOptionSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeVec)(ty)) {
            return this.mapVecSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeArray)(ty)) {
            return this.mapArraySerde(ty, name);
        }
        if ((0, types_1.isIdlTypeEnum)(ty)) {
            return this.mapEnumSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeDefined)(ty)) {
            const alias = this.typeAliases.get(ty.defined);
            return alias == null
                ? this.mapDefinedSerde(ty)
                : this.mapPrimitiveSerde(alias, name);
        }
        if ((0, types_1.isIdlTypeTuple)(ty)) {
            return this.mapTupleSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeHashMap)(ty)) {
            return this.mapHashMapSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeBTreeMap)(ty)) {
            return this.mapBTreeMapSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeHashSet)(ty)) {
            return this.mapHashSetSerde(ty, name);
        }
        if ((0, types_1.isIdlTypeBTreeSet)(ty)) {
            return this.mapBTreeSetSerde(ty, name);
        }
        throw new Error(`Type ${ty} required for ${name} is not yet supported`);
    }
    mapSerdeFields(fields) {
        return fields.map(this.mapSerdeField);
    }
    // -----------------
    // Imports Generator
    // -----------------
    importsUsed(fileDir, forcePackages) {
        return [
            ...this._importsForSerdePackages(forcePackages),
            ...this._importsForLocalPackages(fileDir.toString()),
        ];
    }
    _importsForSerdePackages(forcePackages) {
        const packagesToInclude = forcePackages == null
            ? this.serdePackagesUsed
            : new Set([
                ...Array.from(this.serdePackagesUsed),
                ...Array.from(forcePackages),
            ]);
        const imports = [];
        for (const pack of packagesToInclude) {
            const exp = (0, serdes_1.serdePackageExportName)(pack);
            imports.push(`import * as ${exp} from '${pack}';`);
        }
        return imports;
    }
    _importsForLocalPackages(fileDir) {
        const renderedImports = [];
        for (const [originPath, imports] of this.localImportsByPath) {
            let relPath = path_1.default.relative(fileDir, originPath);
            if (!relPath.startsWith('.')) {
                relPath = `./${relPath}`;
            }
            const importPath = (0, utils_1.withoutTsExtension)(relPath);
            renderedImports.push(`import { ${Array.from(imports).join(', ')} }  from '${importPath}';`);
        }
        return renderedImports;
    }
    assertBeetSupported(serde, context) {
        (0, assert_1.strict)(this.primaryTypeMap[serde] != null, `Types to ${context} need to be supported by Beet, ${serde} is not`);
    }
    definedTypesImport(ty) {
        var _a, _b;
        return ((_b = (_a = this.accountTypesPaths.get(ty.defined)) !== null && _a !== void 0 ? _a : this.customTypesPaths.get(ty.defined)) !== null && _b !== void 0 ? _b : assert_1.strict.fail(`Unknown type ${ty.defined} is neither found in types nor an Account`));
    }
}
exports.TypeMapper = TypeMapper;
TypeMapper.defaultPrimaryTypeMap = {
    ...beet_1.supportedTypeMap,
    ...beet_solana_1.supportedTypeMap,
};
//# sourceMappingURL=type-mapper.js.map