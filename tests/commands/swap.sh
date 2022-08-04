${CARGO_TARGET_DIR:-../target}/debug/sns-quill swap --amount 500 --memo 4 --canister-ids-file ./canister_ids.json --pem-file - | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
