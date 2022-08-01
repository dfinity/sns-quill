PRINCIPAL=fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae
${CARGO_TARGET_DIR:-../target}/debug/sns-quill --canister-ids-file ./canister_ids.json --pem-file - transfer $PRINCIPAL --amount 1.23456 | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
