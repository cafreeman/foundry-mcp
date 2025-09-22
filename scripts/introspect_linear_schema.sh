#!/usr/bin/env bash
set -euo pipefail

# Introspect Linear's GraphQL schema using the official Cynic CLI and write
# the SDL to the vendored path used by cynic::schema!(...).
#
# Requirements:
# - cynic CLI installed: `cargo install cynic-cli`
# - LINEAR_API_TOKEN (or LINEAR_API_KEY) exported in your environment
#
# Usage:
#   ./scripts/introspect_linear_schema.sh
#
# Notes:
# - This script only runs networked introspection on demand; the generated SDL
#   is committed to the repo so normal builds/tests stay offline.

PROJECT_ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
SCHEMA_OUT="${PROJECT_ROOT_DIR}/src/core/backends/linear/graphql/schema.graphql"
ENDPOINT="${LINEAR_GRAPHQL_ENDPOINT:-https://api.linear.app/graphql}"

if ! command -v cynic >/dev/null 2>&1; then
  echo "Error: 'cynic' CLI not found. Install with: cargo install cynic-cli" >&2
  exit 1;
fi

TOKEN="${LINEAR_API_TOKEN:-${LINEAR_API_KEY:-}}"
if [[ -z "${TOKEN}" ]]; then
  echo "Error: set LINEAR_API_TOKEN or LINEAR_API_KEY in your environment." >&2
  exit 1
fi

mkdir -p "$(dirname "${SCHEMA_OUT}")"

# Run introspection using official CLI, passing Authorization header.
echo "Introspecting schema from ${ENDPOINT} → ${SCHEMA_OUT}" >&2
cynic introspect "${ENDPOINT}" \
  --header "Authorization: Bearer ${TOKEN}" \
  --output "${SCHEMA_OUT}"

echo "Schema written to ${SCHEMA_OUT}" >&2