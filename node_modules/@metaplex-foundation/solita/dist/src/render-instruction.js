"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.renderInstruction = void 0;
const types_1 = require("./types");
const type_mapper_1 = require("./type-mapper");
const serdes_1 = require("./serdes");
const known_pubkeys_1 = require("./known-pubkeys");
const beet_1 = require("@metaplex-foundation/beet");
const render_enums_1 = require("./render-enums");
const instruction_discriminator_1 = require("./instruction-discriminator");
class InstructionRenderer {
    constructor(ix, fullFileDir, programId, typeMapper, renderAnchorRemainingAccounts) {
        this.ix = ix;
        this.fullFileDir = fullFileDir;
        this.programId = programId;
        this.typeMapper = typeMapper;
        this.renderAnchorRemainingAccounts = renderAnchorRemainingAccounts;
        // -----------------
        // Instruction Args Type
        // -----------------
        this.renderIxArgField = (arg) => {
            const typescriptType = this.typeMapper.map(arg.type, arg.name);
            return `${arg.name}: ${typescriptType}`;
        };
        this.upperCamelIxName = ix.name
            .charAt(0)
            .toUpperCase()
            .concat(ix.name.slice(1));
        this.camelIxName = ix.name.charAt(0).toLowerCase().concat(ix.name.slice(1));
        this.argsTypename = `${this.upperCamelIxName}InstructionArgs`;
        this.accountsTypename = `${this.upperCamelIxName}InstructionAccounts`;
        this.instructionDiscriminatorName = `${this.camelIxName}InstructionDiscriminator`;
        this.structArgName = `${ix.name}Struct`;
        this.instructionDiscriminator = new instruction_discriminator_1.InstructionDiscriminator(ix, 'instructionDiscriminator', typeMapper);
        this.programIdPubkey = `new ${types_1.SOLANA_WEB3_EXPORT_NAME}.PublicKey('${this.programId}')`;
        this.defaultOptionalAccounts = !ix.legacyOptionalAccountsStrategy;
    }
    renderIxArgsType() {
        if (this.ix.args.length === 0)
            return '';
        const fields = this.ix.args
            .map((field) => this.renderIxArgField(field))
            .join(',\n  ');
        const code = `
/**
 * @category Instructions
 * @category ${this.upperCamelIxName}
 * @category generated
 */
export type ${this.argsTypename} = {
  ${fields}
}`.trim();
        return code;
    }
    // -----------------
    // Imports
    // -----------------
    renderImports(processedKeys) {
        const typeMapperImports = this.typeMapper.importsUsed(this.fullFileDir.toString(), new Set([types_1.SOLANA_WEB3_PACKAGE, beet_1.BEET_PACKAGE]));
        const needsSplToken = processedKeys.some((x) => { var _a; return ((_a = x.knownPubkey) === null || _a === void 0 ? void 0 : _a.pack) === types_1.SOLANA_SPL_TOKEN_PACKAGE; });
        const splToken = needsSplToken
            ? `\nimport * as ${types_1.SOLANA_SPL_TOKEN_EXPORT_NAME} from '${types_1.SOLANA_SPL_TOKEN_PACKAGE}';`
            : '';
        return `
${splToken}
${typeMapperImports.join('\n')}`.trim();
    }
    // -----------------
    // Accounts
    // -----------------
    processIxAccounts() {
        var _a, _b, _c, _d;
        let processedAccountsKey = [];
        for (const acc of this.ix.accounts) {
            if ((0, types_1.isAccountsCollection)(acc)) {
                for (const ac of acc.accounts) {
                    // Make collection items easy to identify and avoid name clashes
                    ac.name = deriveCollectionAccountsName(ac.name, acc.name);
                    const knownPubkey = (0, known_pubkeys_1.resolveKnownPubkey)(ac.name);
                    const optional = (_b = (_a = ac.optional) !== null && _a !== void 0 ? _a : ac.isOptional) !== null && _b !== void 0 ? _b : false;
                    if (knownPubkey == null) {
                        processedAccountsKey.push({ ...ac, optional });
                    }
                    else {
                        processedAccountsKey.push({ ...ac, knownPubkey, optional });
                    }
                }
            }
            else {
                const knownPubkey = (0, known_pubkeys_1.resolveKnownPubkey)(acc.name);
                const optional = (_d = (_c = acc.optional) !== null && _c !== void 0 ? _c : acc.isOptional) !== null && _d !== void 0 ? _d : false;
                if (knownPubkey == null) {
                    processedAccountsKey.push({ ...acc, optional });
                }
                else {
                    processedAccountsKey.push({ ...acc, knownPubkey, optional });
                }
            }
        }
        return processedAccountsKey;
    }
    // -----------------
    // AccountKeys
    // -----------------
    /*
     * Main entry to render account metadata for provided account keys.
     * The `defaultOptionalAccounts` strategy determines how optional accounts
     * are rendered.
     *
     * a) If the defaultOptionalAccounts strategy is set all accounts will be
     *    added to the accounts array, but default to the program id when they weren't
     *    provided by the user.
     *
     * b) If the defaultOptionalAccounts strategy is not enabled optional accounts
     *    that are not provided will be omitted from the accounts array.
     *
     * @private
     */
    renderIxAccountKeys(processedKeys) {
        const fixedAccountKeys = this.defaultOptionalAccounts
            ? this.renderAccountKeysDefaultingOptionals(processedKeys)
            : this.renderAccountKeysNotDefaultingOptionals(processedKeys);
        const anchorRemainingAccounts = this.renderAnchorRemainingAccounts && processedKeys.length > 0
            ? `
  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc)
    }
  }
`
            : '';
        return `${fixedAccountKeys}\n${anchorRemainingAccounts}\n`;
    }
    // -----------------
    // AccountKeys: with strategy to not defaultOptionalAccounts
    // -----------------
    renderAccountKeysNotDefaultingOptionals(processedKeys) {
        const indexOfFirstOptional = processedKeys.findIndex((x) => x.optional);
        if (indexOfFirstOptional === -1) {
            return this.renderAccountKeysInsideArray(processedKeys) + '\n';
        }
        const accountsInsideArray = this.renderAccountKeysInsideArray(processedKeys.slice(0, indexOfFirstOptional));
        const accountsToPush = this.renderAccountKeysToPush(processedKeys.slice(indexOfFirstOptional));
        return `${accountsInsideArray}\n${accountsToPush}`;
    }
    renderAccountKeysInsideArray(processedKeys) {
        const metaElements = processedKeys
            .map((processedKey) => renderRequiredAccountMeta(processedKey, this.programIdPubkey))
            .join(',\n    ');
        return `[\n    ${metaElements}\n  ]`;
    }
    renderAccountKeysToPush(processedKeys) {
        if (processedKeys.length === 0) {
            return '';
        }
        const statements = processedKeys
            .map((processedKey, idx) => {
            if (!processedKey.optional) {
                const accountMeta = renderRequiredAccountMeta(processedKey, this.programIdPubkey);
                return `keys.push(${accountMeta})`;
            }
            const requiredOptionals = processedKeys
                .slice(0, idx)
                .filter((x) => x.optional);
            const requiredChecks = requiredOptionals
                .map((x) => `accounts.${x.name} == null`)
                .join(' || ');
            const checkRequireds = requiredChecks.length > 0
                ? `if (${requiredChecks}) { throw new Error('When providing \\'${processedKey.name}\\' then ` +
                    `${requiredOptionals
                        .map((x) => `\\'accounts.${x.name}\\'`)
                        .join(', ')} need(s) to be provided as well.') }`
                : '';
            const pubkey = `accounts.${processedKey.name}`;
            const accountMeta = renderAccountMeta(pubkey, processedKey.isMut.toString(), processedKey.isSigner.toString());
            // renderRequiredAccountMeta
            // NOTE: we purposely don't add the default resolution here since the intent is to
            // only pass that account when it is provided
            return `
if (accounts.${processedKey.name} != null) {
  ${checkRequireds}
  keys.push(${accountMeta})
}`.trim();
        })
            .join('\n');
        return `\n${statements}\n`;
    }
    // -----------------
    // AccountKeys: with strategy to defaultOptionalAccounts
    // -----------------
    /*
     * This renders optional accounts when the defaultOptionalAccounts strategy is
     * enabled.
     * This means that all accounts will be added to the accounts array, but default
     * to the program id when they weren't provided by the user.
     * @category private
     */
    renderAccountKeysDefaultingOptionals(processedKeys) {
        const metaElements = processedKeys
            .map((processedKey) => {
            return processedKey.optional
                ? renderOptionalAccountMetaDefaultingToProgramId(processedKey)
                : renderRequiredAccountMeta(processedKey, this.programIdPubkey);
        })
            .join(',\n    ');
        return `[\n    ${metaElements}\n  ]`;
    }
    // -----------------
    // AccountsType
    // -----------------
    renderAccountsType(processedKeys) {
        if (processedKeys.length === 0)
            return '';
        const web3 = types_1.SOLANA_WEB3_EXPORT_NAME;
        const fields = processedKeys
            .map((x) => {
            if (x.knownPubkey != null) {
                return `${x.name}?: ${web3}.PublicKey`;
            }
            const optional = x.optional ? '?' : '';
            return `${x.name}${optional}: ${web3}.PublicKey`;
        })
            .join('\n  ');
        const anchorRemainingAccounts = this.renderAnchorRemainingAccounts
            ? 'anchorRemainingAccounts?: web3.AccountMeta[]'
            : '';
        const propertyComments = processedKeys
            // known pubkeys are not provided by the user and thus aren't part of the type
            .filter((x) => !(0, known_pubkeys_1.isKnownPubkey)(x.name))
            .map((x) => {
            const attrs = [];
            if (x.isMut)
                attrs.push('_writable_');
            if (x.isSigner)
                attrs.push('**signer**');
            const optional = x.optional ? ' (optional) ' : ' ';
            const desc = (0, types_1.isIdlInstructionAccountWithDesc)(x) ? x.desc : '';
            return (`* @property [${attrs.join(', ')}] ` + `${x.name}${optional}${desc} `);
        });
        const properties = propertyComments.length > 0
            ? `\n *\n  ${propertyComments.join('\n')} `
            : '';
        const docs = `
/**
  * Accounts required by the _${this.ix.name}_ instruction${properties}
  * @category Instructions
  * @category ${this.upperCamelIxName}
  * @category generated
  */
`.trim();
        return `${docs}
          export type ${this.accountsTypename} = {
  ${fields}
  ${anchorRemainingAccounts}
        }
        `;
    }
    renderAccountsParamDoc(processedKeys) {
        if (processedKeys.length === 0)
            return '  *';
        return `  *
  * @param accounts that will be accessed while the instruction is processed`;
    }
    renderAccountsArg(processedKeys) {
        if (processedKeys.length === 0)
            return '';
        return `accounts: ${this.accountsTypename}, \n`;
    }
    // -----------------
    // Data Struct
    // -----------------
    serdeProcess() {
        return this.typeMapper.mapSerdeFields(this.ix.args);
    }
    renderDataStruct(args) {
        const discriminatorField = this.typeMapper.mapSerdeField(this.instructionDiscriminator.getField());
        const discriminatorType = this.instructionDiscriminator.renderType();
        const struct = (0, serdes_1.renderDataStruct)({
            fields: args,
            discriminatorName: 'instructionDiscriminator',
            discriminatorField,
            discriminatorType,
            structVarName: this.structArgName,
            argsTypename: this.argsTypename,
            isFixable: this.typeMapper.usedFixableSerde,
        });
        return `
/**
 * @category Instructions
 * @category ${this.upperCamelIxName}
 * @category generated
 */
${struct} `.trim();
    }
    render() {
        this.typeMapper.clearUsages();
        const ixArgType = this.renderIxArgsType();
        const processedKeys = this.processIxAccounts();
        const accountsType = this.renderAccountsType(processedKeys);
        const processedArgs = this.serdeProcess();
        const argsStructType = this.renderDataStruct(processedArgs);
        const keys = this.renderIxAccountKeys(processedKeys);
        const accountsParamDoc = this.renderAccountsParamDoc(processedKeys);
        const accountsArg = this.renderAccountsArg(processedKeys);
        const instructionDisc = this.instructionDiscriminator.renderValue();
        const enums = (0, render_enums_1.renderScalarEnums)(this.typeMapper.scalarEnumsUsed).join('\n');
        const web3 = types_1.SOLANA_WEB3_EXPORT_NAME;
        const imports = this.renderImports(processedKeys);
        const [createInstructionArgsComment, createInstructionArgs, createInstructionArgsSpread, comma,] = this.ix.args.length === 0
            ? ['', '', '', '']
            : [
                `\n * @param args to provide as instruction data to the program\n * `,
                `args: ${this.argsTypename} `,
                '...args',
                ', ',
            ];
        const programIdArg = `${comma}programId = ${this.programIdPubkey}`;
        const optionalAccountsComment = optionalAccountsStrategyDocComment(this.defaultOptionalAccounts, processedKeys.some((x) => x.optional));
        return `${imports}

${enums}
${ixArgType}
${argsStructType}
${accountsType}
    export const ${this.instructionDiscriminatorName} = ${instructionDisc};

    /**
     * Creates a _${this.upperCamelIxName}_ instruction.
    ${optionalAccountsComment}${accountsParamDoc}${createInstructionArgsComment}
     * @category Instructions
     * @category ${this.upperCamelIxName}
     * @category generated
     */
    export function create${this.upperCamelIxName}Instruction(
      ${accountsArg}${createInstructionArgs}${programIdArg}
    ) {
      const [data] = ${this.structArgName}.serialize({
        instructionDiscriminator: ${this.instructionDiscriminatorName},
    ${createInstructionArgsSpread}
    });
    const keys: ${web3}.AccountMeta[] = ${keys}
    const ix = new ${web3}.TransactionInstruction({
      programId,
      keys,
      data
  });
  return ix; 
}
`;
    }
}
function renderInstruction(ix, fullFileDir, programId, accountFilesByType, customFilesByType, typeAliases, forceFixable, renderAnchorRemainingAccounts) {
    const typeMapper = new type_mapper_1.TypeMapper(accountFilesByType, customFilesByType, typeAliases, forceFixable);
    const renderer = new InstructionRenderer(ix, fullFileDir, programId, typeMapper, renderAnchorRemainingAccounts);
    return renderer.render();
}
exports.renderInstruction = renderInstruction;
// -----------------
// Utility Functions
// -----------------
function renderAccountMeta(pubkey, isWritable, isSigner) {
    return `{
      pubkey: ${pubkey},
      isWritable: ${isWritable},
      isSigner: ${isSigner},
    }`;
}
function deriveCollectionAccountsName(accountName, collectionName) {
    const camelAccount = accountName
        .charAt(0)
        .toUpperCase()
        .concat(accountName.slice(1));
    return `${collectionName}Item${camelAccount}`;
}
function renderOptionalAccountMetaDefaultingToProgramId(processedKey) {
    const { name, isMut, isSigner } = processedKey;
    const pubkey = `accounts.${name} ?? programId`;
    const mut = isMut ? `accounts.${name} != null` : 'false';
    const signer = isSigner ? `accounts.${name} != null` : 'false';
    return renderAccountMeta(pubkey, mut, signer);
}
function renderRequiredAccountMeta(processedKey, programIdPubkey) {
    const { name, isMut, isSigner, knownPubkey } = processedKey;
    const pubkey = knownPubkey == null
        ? `accounts.${name}`
        : `accounts.${name} ?? ${(0, known_pubkeys_1.renderKnownPubkeyAccess)(knownPubkey, programIdPubkey)}`;
    return renderAccountMeta(pubkey, isMut.toString(), isSigner.toString());
}
function optionalAccountsStrategyDocComment(defaultOptionalAccounts, someAccountIsOptional) {
    if (!someAccountIsOptional)
        return '';
    if (defaultOptionalAccounts) {
        return ` * 
 * Optional accounts that are not provided default to the program ID since 
 * this was indicated in the IDL from which this instruction was generated.
`;
    }
    return ` * 
 * Optional accounts that are not provided will be omitted from the accounts
 * array passed with the instruction.
 * An optional account that is set cannot follow an optional account that is unset.
 * Otherwise an Error is raised.
`;
}
//# sourceMappingURL=render-instruction.js.map