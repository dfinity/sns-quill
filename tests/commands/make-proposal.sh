NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
PROPOSAL='( record { title="SNS Launch"; url="https://dfinity.org"; summary="A motion to start the SNS"; action=opt variant { Motion=record { motion_text="I hereby raise the motion that the use of the SNS shall commence"; } }; } )'
${CARGO_TARGET_DIR:-../target}/debug/sns-quill make-proposal $NEURON_ID --proposal "${PROPOSAL}" --canister-ids-file ./canister_ids.json --pem-file - | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -

