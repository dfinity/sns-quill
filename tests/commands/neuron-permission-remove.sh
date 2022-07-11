NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
${CARGO_TARGET_DIR:-../target}/debug/sns-quill --canister-ids-file ./canister_ids.json --pem-file - neuron-permission remove $NEURON_ID --principal fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae --permissions merge-maturity disburse-maturity | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
