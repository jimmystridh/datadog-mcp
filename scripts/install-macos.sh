#!/usr/bin/env bash

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "This installer is only for macOS." >&2
    exit 1
fi

signing_identity="${1:-${DATADOG_MCP_CODESIGN_IDENTITY:-}}"
if [[ -z "$signing_identity" ]]; then
    echo "Usage: $0 '<code-signing identity>'" >&2
    echo "Available identities:" >&2
    security find-identity -v -p codesigning >&2
    exit 2
fi

available_identities="$(security find-identity -v -p codesigning)"
if [[ "$available_identities" != *"\"$signing_identity\""* ]]; then
    echo "Code-signing identity not found: $signing_identity" >&2
    echo "$available_identities" >&2
    exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_dir="$(cd "$script_dir/.." && pwd)"
install_root="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-${HOME}/.cargo}}"
binary_path="$install_root/bin/datadog-mcp"

cargo install \
    --path "$repository_dir/datadog-mcp" \
    --root "$install_root" \
    --locked \
    --force

codesign \
    --force \
    --identifier "com.jimmystridh.datadog-mcp" \
    --sign "$signing_identity" \
    --timestamp=none \
    "$binary_path"

codesign --verify --strict --verbose=2 "$binary_path"
echo "Installed and signed $binary_path"
