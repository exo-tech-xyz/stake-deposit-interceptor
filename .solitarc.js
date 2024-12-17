const path = require('path');
const programDir = path.join(__dirname, 'program');
const idlDir = path.join(__dirname, 'package', 'idl');
const sdkDir = path.join(__dirname, 'package', 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
  idlGenerator: 'shank',
  programName: 'stake_deposit_interceptor',
  programId: '5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV',
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
  removeExistingIdl: true,
  typeAliases: {
    PodU64: "u64",
    PodU32: "u32",
  }
};