const path = require('path');

module.exports = {
  idl: path.resolve(__dirname, "./program/idl/stake_deposit_interceptor.json"),
  sdk: path.resolve(__dirname, "./js/src/generated"),
  program: {
    name: "stake_deposit_interceptor",
    id: "5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV",
    path: path.resolve(__dirname, "./program")
  }
};