LEDGER_ACCOUNT_ID=345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752
${CARGO_TARGET_DIR:-../target}/debug/sns-quill transfer $LEDGER_ACCOUNT_ID --amount 123.0456 --canister-ids-file ./canister_ids.json --pem-file - | gzip --best --to-stdout | zcat | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
