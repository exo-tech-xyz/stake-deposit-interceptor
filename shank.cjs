const path = require("path");

module.exports = {
  name: "stake-deposit-interceptor",
  version: "0.1.0",
  programId: "5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV",
  sdkDir: path.join(__dirname, "generated", "sdk"),
  sources: [
    {
      directory: path.join(__dirname, "program", "src"),
      patterns: ["**/*.rs"],
    },
  ],
  // Add this types configuration
  types: {
    PodU64: {
      kind: "struct",
      fields: [["value", "u64"]]
    },
    PodU32: {
      kind: "struct",
      fields: [["value", "u32"]]
    }
  }
};