#!/usr/bin/env bash
set -euo pipefail

package_dir="${1:-}"
if [ -z "$package_dir" ]; then
  echo "usage: scripts/validate-store-package.sh <package-dir>" >&2
  exit 2
fi

cargo run --quiet -- store validate "$package_dir"
