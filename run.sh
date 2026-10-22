#!/bin/sh
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
exec cargo run --release -- "$@"
