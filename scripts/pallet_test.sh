# Read environment from .env file
set -o allexport; source .env; set +o allexport;
WASM_BUILD_TOOLCHAIN=nightly-2020-10-06-x86_64-apple-darwin
cd pallets
cd polkaswap
cargo +nightly-2020-10-06 test
