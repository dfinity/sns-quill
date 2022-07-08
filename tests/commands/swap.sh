${CARGO_TARGET_DIR:-../target}/debug/sns-quill --canister-ids-file ./canister_ids.json --pem-file - swap --amount 500 --memo 4 | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
