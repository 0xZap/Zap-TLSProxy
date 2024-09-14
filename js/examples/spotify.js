const ZapProxy = require("../src/index.js");
const schemas = require("./schemas");

(async () => {
  try {
    const schema = schemas.spotify;

    const zapProxy = new ZapProxy(
      schema.proxyHost,
      schema.proxyPort,
      schema.targetHost,
      schema.targetPort
    );

    await zapProxy.connect();

    const proofResponse = await zapProxy.prove(schema);

    // console.log("Proof:", proofResponse);

    console.log("Signature:", proofResponse.signature);
    console.log("Proof received:", proofResponse.proofData);
  } catch (error) {
    console.error("Error:", error.message);
  }
})();
