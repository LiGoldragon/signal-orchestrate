set -eu
if grep -R -n -E '^(pub(\([^)]*\))? )?fn ' "$src/src"; then
  echo "production Rust must not use module-level free functions" >&2
  exit 1
fi
touch "$out"
