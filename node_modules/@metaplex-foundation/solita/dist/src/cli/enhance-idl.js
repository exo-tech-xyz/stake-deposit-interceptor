"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.enhanceIdl = void 0;
const path_1 = __importDefault(require("path"));
const utils_1 = require("../utils");
const types_1 = require("./types");
const assert_1 = require("assert");
const fs_1 = require("fs");
async function enhanceIdl(config, binaryVersion, libVersion) {
    const { idlDir, programName } = config;
    const idlPath = path_1.default.join(idlDir, `${programName}.json`);
    const idl = require(idlPath);
    if ((0, types_1.isSolitaConfigAnchor)(config)) {
        idl.metadata = {
            ...idl.metadata,
            address: config.programId,
            origin: config.idlGenerator,
            binaryVersion,
            libVersion,
        };
    }
    else if ((0, types_1.isSolitaConfigShank)(config)) {
        idl.metadata = {
            ...idl.metadata,
            binaryVersion,
            libVersion,
        };
    }
    else {
        throw new utils_1.UnreachableCaseError(
        // @ts-ignore this possible is when types were violated via JS
        `Unknown IDL generator ${config.idlGenerator}`);
    }
    // Apply Idl hook if provided
    let finalIdl = idl;
    if (config.idlHook != null) {
        assert_1.strict.equal(typeof config.idlHook, 'function', `idlHook needs to be a function of the type: (idl: Idl) => idl, but is of type ${typeof config.idlHook}`);
        finalIdl = config.idlHook(idl);
    }
    await fs_1.promises.writeFile(idlPath, JSON.stringify(finalIdl, null, 2));
    return finalIdl;
}
exports.enhanceIdl = enhanceIdl;
//# sourceMappingURL=enhance-idl.js.map