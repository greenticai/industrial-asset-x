# industrialx-executor

Deterministic, advisory-only native executor for Industrial-X playbooks. A loopback
HTTP service (`127.0.0.1` only, `--port`, default 8787) with:

- `GET /healthz`
- `POST /validate`: `{"playbook": {...}}` (or the bare playbook) returns
  `{valid, errors, playbook_id, warnings}` with HTTP 200 even for invalid input.
- `POST /execute`: request fields `playbook`, `mode` (only `"advisory"`), `readings`,
  `features` (precomputed), `bindings` (`{feature: {inputs: [sensor, ...]}}`),
  `scoring` (`{feature, band: {lower, upper}}`), `failure_modes`
  (`{mode: {active, score?}}`), `evidence` (`{name: any non-null}`).

Typed errors are `{"error": {"code", "message"}}`: `body_too_large` (413),
`invalid_json`/`invalid_request`/`unknown_field` (400), `control_refused`,
`secret_refused`, `playbook_invalid`, `unknown_feature`, `unknown_failure_mode` (422).

Nothing the playbook plus the request do not fully determine is computed: it is
returned under `unresolved` with a reason. Run it with
`cargo run -p industrialx-executor`. It is a workspace member only; the WASM
component build script names its components explicitly and is unaffected.
