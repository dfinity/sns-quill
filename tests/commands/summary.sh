${CARGO_TARGET_DIR:-../target}/debug/sns-quill --canister-ids-file ./canister_ids.json --pem-file - summary | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
