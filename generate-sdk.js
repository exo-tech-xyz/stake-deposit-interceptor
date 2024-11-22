const { generateIdl } = require('@metaplex-foundation/solita');

const config = {
  program: {
    name: 'stake_deposit_interceptor',
    idl: './program/idl/stake_deposit_interceptor.json',
  },
  output: {
    directory: './js/src/generated',
    clientName: 'StakeDepositInterceptorClient',
  },
};

(async () => {
  await generateIdl(config);
})();
