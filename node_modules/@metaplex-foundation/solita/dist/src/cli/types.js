"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isErrorResult = exports.isSolitaConfigShank = exports.isSolitaConfigAnchor = void 0;
// -----------------
// Guards
// -----------------
function isSolitaConfigAnchor(config) {
    return config.idlGenerator === 'anchor';
}
exports.isSolitaConfigAnchor = isSolitaConfigAnchor;
function isSolitaConfigShank(config) {
    return config.idlGenerator === 'shank';
}
exports.isSolitaConfigShank = isSolitaConfigShank;
function isErrorResult(result) {
    return result.errorMsg != null;
}
exports.isErrorResult = isErrorResult;
//# sourceMappingURL=types.js.map