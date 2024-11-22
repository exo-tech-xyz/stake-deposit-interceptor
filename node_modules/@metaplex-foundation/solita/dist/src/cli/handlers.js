"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.handleShank = exports.handleAnchor = void 0;
const rustbin_1 = require("@metaplex-foundation/rustbin");
const child_process_1 = require("child_process");
const path_1 = __importDefault(require("path"));
const enhance_idl_1 = require("./enhance-idl");
const gen_typescript_1 = require("./gen-typescript");
const utils_1 = require("../utils");
const ansi_colors_1 = require("ansi-colors");
const handlerErrorRx = /^Error\:/;
function handleAnchor(config, prettierConfig) {
    var _a, _b;
    const { idlDir, binaryInstallDir, programDir } = config;
    const binaryArgs = (_b = (_a = config.binaryArgs) === null || _a === void 0 ? void 0 : _a.split(' ')) !== null && _b !== void 0 ? _b : [];
    const spawnArgs = ['build', '--idl', idlDir, ...binaryArgs];
    const spawnOpts = {
        cwd: programDir,
    };
    const rustbinConfig = {
        rootDir: binaryInstallDir,
        binaryName: 'anchor',
        binaryCrateName: 'anchor-cli',
        libName: 'anchor-lang',
        cargoToml: path_1.default.join(programDir, 'Cargo.toml'),
        dryRun: false,
        ...config.rustbin,
    };
    return handle(config, rustbinConfig, spawnArgs, spawnOpts, prettierConfig, config.anchorRemainingAccounts);
}
exports.handleAnchor = handleAnchor;
function handleShank(config, prettierConfig) {
    const { idlDir, binaryInstallDir, programDir } = config;
    const spawnArgs = ['idl', '--out-dir', idlDir, '--crate-root', programDir];
    const spawnOpts = {
        cwd: programDir,
    };
    const rustbinConfig = {
        rootDir: binaryInstallDir,
        binaryName: 'shank',
        binaryCrateName: 'shank-cli',
        libName: 'shank',
        cargoToml: path_1.default.join(programDir, 'Cargo.toml'),
        dryRun: false,
        ...config.rustbin,
    };
    return handle(config, rustbinConfig, spawnArgs, spawnOpts, prettierConfig, false);
}
exports.handleShank = handleShank;
async function handle(config, rustbinConfig, spawnArgs, spawnOpts, prettierConfig, anchorRemainingAccounts) {
    const { programName, idlDir, sdkDir } = config;
    const { fullPathToBinary, binVersion, libVersion } = await (0, rustbin_1.rustbinMatch)(rustbinConfig, confirmAutoMessageLog);
    if (binVersion == null) {
        throw new Error(`rustbin was unable to determine installed version ${rustbinConfig.binaryName}, it may ` +
            `not have been installed correctly.`);
    }
    return new Promise((resolve, reject) => {
        const tool = path_1.default.basename(fullPathToBinary);
        const idlGenerator = (0, child_process_1.spawn)(fullPathToBinary, spawnArgs, spawnOpts)
            .on('error', (err) => {
            (0, utils_1.logError)(`${programName} idl generation failed`);
            reject(err);
        })
            .on('exit', async (exitCode) => {
            exitCode !== null && exitCode !== void 0 ? exitCode : (exitCode = 0);
            (0, utils_1.logDebug)(`${tool} completed with code ${exitCode}`);
            if (exitCode == 0) {
                (0, utils_1.logInfo)('IDL written to: %s', path_1.default.join(idlDir, `${programName}.json`));
                const idl = await (0, enhance_idl_1.enhanceIdl)(config, binVersion, libVersion);
                await (0, gen_typescript_1.generateTypeScriptSDK)(idl, sdkDir, prettierConfig, config.typeAliases, config.serializers, anchorRemainingAccounts);
                resolve({ exitCode });
            }
            else {
                const errorMsg = (0, ansi_colors_1.red)(`${tool} returned with non-zero exit code. Please review the output above to diagnose the issue.`);
                resolve({ exitCode, errorMsg });
            }
        });
        idlGenerator.stdout.on('data', (buf) => process.stdout.write(buf));
        idlGenerator.stderr.on('data', (buf) => {
            const dataStr = buf.toString();
            if (handlerErrorRx.test(dataStr)) {
                (0, utils_1.logError)((0, ansi_colors_1.red)(dataStr));
            }
            else {
                process.stderr.write(buf);
            }
        });
    });
}
function confirmAutoMessageLog({ binaryName, libVersion, libName, binVersion, fullPathToBinary, }) {
    if (binVersion == null) {
        (0, utils_1.logInfo)(`No existing version found for ${binaryName}.`);
    }
    else {
        (0, utils_1.logInfo)(`Version for ${binaryName}: ${binVersion}`);
    }
    (0, utils_1.logInfo)(`Will install version matching "${libName}: '${libVersion}'" to ${fullPathToBinary}`);
    return Promise.resolve(true);
}
//# sourceMappingURL=handlers.js.map