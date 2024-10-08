const ZapProxy = require("../src/index.js");
const schemas = require("./schemas");

(async () => {
  try {
    const schema = schemas.nodeguardiansSchema;

    const zapProxy = new ZapProxy(
      schema.proxyHost,
      schema.proxyPort,
      schema.targetHost,
      schema.targetPort
    );

    await zapProxy.connect();

    const proofResponse = await zapProxy.prove(schema);

    console.log("Signature:", proofResponse.signature);
    console.log("Proof received:", proofResponse.proof);
  } catch (error) {
    console.error("Error:", error.message);
  }
})();
