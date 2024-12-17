const fs = require("fs");
const path = require("path");
const idlFile = path.join(
  __dirname,
  "package",
  "idl",
  "stake_deposit_interceptor.json"
);

const replaceInFields = (obj) => {
  if (!obj.fields) return;
  obj.fields.forEach((field) => {
    if (typeof field.type === "object") {
      if (field.type.defined === "PodU64") {
        field.type = "u64";
      }
      if (field.type.defined === "PodU32") {
        field.type = "u32";
      }
    }
  });
};

const overwriteTypes = (idl) => {
  if (idl.types) {
    idl.types.forEach((type) => {
      replaceInFields(type);
    });
  }

  if (idl.accounts) {
    idl.accounts.forEach((account) => {
      replaceInFields(account.type);
    });
  }

  if (idl.instructions) {
    idl.instructions.forEach((instruction) => {
      instruction.accounts?.forEach(replaceInFields);
      instruction.args?.forEach(replaceInFields);
    });
  }
};

const main = () => {
  let idl = JSON.parse(fs.readFileSync(idlFile, "utf-8"));
  overwriteTypes(idl);
  fs.writeFileSync(idlFile, JSON.stringify(idl, null, 2));
};

main();
