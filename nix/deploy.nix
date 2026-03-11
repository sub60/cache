{
  writeShellApplication,
  curl,
  prod,
  workerName,
  workerCompatibilityDate,
}:

let
  metadata = builtins.toJSON {
    main_module = "index.js";
    compatibility_date = workerCompatibilityDate;
  };
in
writeShellApplication {
  name = "deploy";
  runtimeInputs = [ curl ];
  text = ''
    : "''${CLOUDFLARE_API_TOKEN:?must be set}"
    : "''${CLOUDFLARE_ACCOUNT_ID:?must be set}"

    curl -sSf -X PUT \
      "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/workers/scripts/${workerName}" \
      -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
      -F 'metadata=${metadata};type=application/json' \
      -F 'index.js=@${prod}/index.js;type=application/javascript+module' \
      -F 'index_bg.wasm=@${prod}/index_bg.wasm;type=application/wasm'
  '';
}
