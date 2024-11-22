"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.logInfo = exports.logWarn = exports.logError = exports.logTrace = exports.logDebug = exports.logInfoDebug = exports.logWarnDebug = exports.logErrorDebug = void 0;
const debug_1 = __importDefault(require("debug"));
exports.logErrorDebug = (0, debug_1.default)('solita:error');
exports.logWarnDebug = (0, debug_1.default)('solita:warn');
exports.logInfoDebug = (0, debug_1.default)('solita:info');
exports.logDebug = (0, debug_1.default)('solita:debug');
exports.logTrace = (0, debug_1.default)('solita:trace');
exports.logError = exports.logErrorDebug.enabled
    ? exports.logErrorDebug
    : console.error.bind(console);
exports.logWarn = exports.logErrorDebug.enabled
    ? exports.logWarnDebug
    : console.warn.bind(console);
exports.logInfo = exports.logInfoDebug.enabled
    ? exports.logInfoDebug
    : console.log.bind(console);
//# sourceMappingURL=logs.js.map