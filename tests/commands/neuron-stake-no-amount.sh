${CARGO_TARGET_DIR:-../target}/debug/sns-quill stake-neuron --memo 777 --canister-ids-file ./canister_ids.json --pem-file - | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
