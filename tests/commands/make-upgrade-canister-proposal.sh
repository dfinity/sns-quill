PROPOSER_NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069

${CARGO_TARGET_DIR:-../target}/debug/sns-quill \
                              make-upgrade-canister-proposal \
                              --wasm-path=outputs/canister.wasm \
                              --target-canister-id=pycv5-3jbbb-ccccc-ddddd-cai \
                              $PROPOSER_NEURON_ID \
                              --canister-ids-file=./canister_ids.json \
                              --pem-file=- \
    | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill \
                                    send \
                                    --dry-run \
                                    -

