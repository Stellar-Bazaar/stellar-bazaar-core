#!/usr/bin/env bash
set -euo pipefail

echo "============================================================"
echo " Running @stellar-bazaar/sdk Validation Suite"
echo "============================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$(dirname "$SCRIPT_DIR")"
SDK_DIR="$CORE_DIR/packages/sdk"
FRONTEND_DIR="$(dirname "$CORE_DIR")/stellar-bazaar-frontend"

if [ ! -d "$SDK_DIR" ]; then
  echo "Error: SDK directory not found at $SDK_DIR" >&2
  exit 1
fi

echo ""
echo "[1/2] Running SDK Vitest Unit Tests..."
cd "$FRONTEND_DIR"
npx vitest run --root "$SDK_DIR"

echo ""
echo "[2/2] Checking TypeScript Compilation..."
cd "$SDK_DIR"
npx tsc --noEmit

echo ""
echo "============================================================"
echo " @stellar-bazaar/sdk Tests & Compilation Passed 100%!"
echo "============================================================"
