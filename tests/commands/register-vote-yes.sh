NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
${CARGO_TARGET_DIR:-../target}/debug/sns-quill register-vote $NEURON_ID --proposal-id 1 --vote y --canister-ids-file ./canister_ids.json --pem-file - | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -

