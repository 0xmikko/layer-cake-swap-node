# Read environment from .env file
ulimit -m
ulimit -v
ulimit -u
set -o allexport; source ./.env; set +o allexport;
WASM_BUILD_TOOLCHAIN=nightly-2020-10-06
cargo +nightly-2020-10-06 run -- --dev --tmp  --rpc-cors all
