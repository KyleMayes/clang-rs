set -e

RUST_BACKTRACE=1 cargo test --verbose --features $CLANG_VERSION;

if [ "${CLANG_VERSION}" \< "clang_3_7" ]; then
    cargo clean
    RUST_BACKTRACE=1 cargo test --verbose --features "$CLANG_VERSION static"
fi
