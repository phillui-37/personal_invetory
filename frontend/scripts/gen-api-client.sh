#!/bin/sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
FRONTEND_DIR="$(dirname -- "$SCRIPT_DIR")"

openapi-generator-cli generate \
  -i "${OPENAPI_SPEC_URL:-http://localhost:8080/api/v1/system/openapi}" \
  -g dart \
  -o "$FRONTEND_DIR/lib/api"
