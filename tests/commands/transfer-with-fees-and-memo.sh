PRINCIPAL=fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae
${CARGO_TARGET_DIR:-../target}/debug/sns-quill --canister-ids-file ./canister_ids.json --pem-file - transfer $PRINCIPAL --amount 123.0456 --fee 0.0023 --memo 777 | ${CARGO_TARGET_DIR:-../target}/debug/sns-quill send --dry-run -
