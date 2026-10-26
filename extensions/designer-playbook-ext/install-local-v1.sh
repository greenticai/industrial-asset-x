#!/usr/bin/env bash
# Install this extension into the local Greentic Designer 1.1.x runtime.
# Designer 1.1.x only accepts apiVersion greentic.ai/v1, while the source
# describe.json is v2 for the newer SDK/store flow. This script installs the
# built pack, then writes a v1-compatible local manifest that points at the
# same extension.wasm.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PACK="$HERE/greentic.industrial-x-designer-0.1.0.gtxpack"
DEST="$HOME/.greentic/extensions/design/greentic.industrial-x-designer-0.1.0"

if [ ! -f "$PACK" ]; then
  "$HERE/build.sh"
fi

mkdir -p "$DEST"
unzip -o "$PACK" -d "$DEST" >/dev/null
cat > "$DEST/describe.json" <<'JSON'
{
  "apiVersion": "greentic.ai/v1",
  "kind": "DesignExtension",
  "metadata": {
    "id": "greentic.industrial-x-designer",
    "name": "Industrial-X Designer",
    "version": "0.1.0",
    "summary": "Authoring tools for Industrial-X playbooks in Greentic Designer.",
    "description": "Design extension exposing agentic-worker tools that help Designer list Industrial-X authoring primitives, generate starter playbook JSON, and validate a playbook against the dialect schema, reference checks and the advisory-only rule.",
    "author": { "name": "Greentic", "email": "team@greentic.ai" },
    "license": "MIT",
    "repository": "https://github.com/greentic-biz/industrial-x",
    "keywords": ["industrial-x", "playbook", "designer", "authoring", "agentic-worker", "design-extension"]
  },
  "engine": { "greenticDesigner": ">=1.1.0", "extRuntime": "^1.2.0" },
  "capabilities": {
    "offered": [{ "id": "greentic:industrial-x-designer/playbook-authoring", "version": "0.1.0" }],
    "required": []
  },
  "runtime": {
    "component": "extension.wasm",
    "memoryLimitMB": 64,
    "permissions": { "network": [], "secrets": [], "callExtensionKinds": [] }
  },
  "contributions": {
    "tools": [
      { "name": "ix_list_authoring_catalog", "export": "greentic:extension-design/tools.invoke-tool" },
      { "name": "ix_generate_playbook", "export": "greentic:extension-design/tools.invoke-tool" },
      { "name": "ix_validate_playbook", "export": "greentic:extension-design/tools.invoke-tool" }
    ]
  }
}
JSON

echo "Installed greentic.industrial-x-designer@0.1.0 to $DEST"
