const path = require('path');
const programDir = path.join(__dirname, 'program');
const idlDir = path.join(__dirname, 'program', 'idl');
const sdkDir = path.join(__dirname, 'js', 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
  idlGenerator: 'shank',
  programName: 'stake_deposit_interceptor',
  programId: '5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV',
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
  removeExistingIdl: false,
};