{
  stdenv,
  writeShellApplication,
  curl,
  # --
  self,
}:

let
  inherit (self.lib) cloudflareAccountId cloudflareWorkerName r2BucketName;
  inherit (self.packages.${stdenv.hostPlatform.system}) prod;

  workerCompatibilityDate = "2026-03-11";

  metadata = builtins.toJSON {
    main_module = "index.js";
    compatibility_date = workerCompatibilityDate;
    annotations = {
      "workers/tag" = self.shortRev;
    };
    bindings = [
      {
        type = "r2_bucket";
        name = prod.buildEnv.R2_BUCKET_BINDING_NAME;
        bucket_name = r2BucketName;
      }
    ];
  };
in
(writeShellApplication {
  name = "deploy";
  runtimeInputs = [ curl ];
  text = ''
    : "''${CACHE_DEPLOY_TOKEN:?must be set}"

    curl -sSf -X PUT \
      "https://api.cloudflare.com/client/v4/accounts/${cloudflareAccountId}/workers/scripts/${cloudflareWorkerName}" \
      -H "Authorization: Bearer $CACHE_DEPLOY_TOKEN" \
      -F 'metadata=${metadata};type=application/json' \
      -F 'index.js=@${prod}/index.js;type=application/javascript+module' \
      -F 'index_bg.wasm=@${prod}/index_bg.wasm;type=application/wasm'
  '';
}).overrideAttrs
  {
    passthru = { inherit workerCompatibilityDate; };
  }
