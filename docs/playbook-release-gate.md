# Playbook release gate

Implements the release gate of the Industrial Intelligence design (section 8.3). A playbook, including one drafted from a prompt, is data an engineer reviews. The gate refuses what a JSON Schema cannot express. Code: `src/playbook_gate.rs`; it runs inside `playbook_ref_diagnostics` and `validate_catalog`, so the executor `POST /validate` returns its findings as ordinary `{code, field, message}` errors (response shape unchanged).

Defaults: a playbook with no `actions` is advice only (read-only). The model-facing design never writes PLC, SCADA or setpoints.

## Schema 1.1.0 (minor, backwards compatible)

`playbooks/playbook.schema.json` carries `x-schema-version: 1.1.0`. New top-level fields, all optional: `actions`, `historian_queries`, `providers`, `evidence_requirements`. Existing playbooks stay valid unchanged. Unknown fields (top level or inside the new objects) are still refused (`additionalProperties: false`). Removing or tightening an existing field requires a major version.

| field | item shape |
|---|---|
| `actions[]` | `{id, kind, rule?, component?, failure_mode?}` |
| `historian_queries[]` | `{id, signal, provider?, window, max_points}` (`window`, `max_points` enforced by the gate) |
| `providers[]` | `{id, permissions?[]}` |
| `evidence_requirements[]` | evidence name strings |

## Rules and diagnostic codes

| code | refused when |
|---|---|
| `playbook.action_control_forbidden` | action kind is `write_setpoint`, `plc_write`, `scada_write`, `control_command`, `acknowledge_alarm`, `suppress_alarm` or `execute_command` |
| `playbook.action_kind_unsupported` | any other kind than `recommendation`, `draft_work_request` |
| `playbook.historian_query_unbounded` | `window` or `max_points` missing |
| `playbook.historian_window_invalid` | window is not `<integer><s\|m\|h\|d>` or is zero |
| `playbook.historian_window_too_large` | window longer than 30 days |
| `playbook.historian_max_points_invalid` | `max_points` not an integer in 1..=100000 |
| `playbook.provider_not_approved` | provider id not in `playbooks/provider-catalog.json` |
| `playbook.permission_not_allowed` | permission not granted to that provider by the catalog |
| `playbook.gate_catalog_unavailable` | the embedded catalog cannot be read (fail closed) |
| `playbook.evidence_not_declared` | `evidence_requirements` is present and a rule's `evidence_required` or `baseline_scoring.evidence_mapping` item is not declared |
| `playbook.invented_signal` | query signal absent from `sensor_requirements` |
| `playbook.invented_provider` | query provider absent from the playbook's `providers` |
| `playbook.invented_rule` | action `rule` not in `recommendation_rules` |
| `playbook.invented_failure_mode` | action `failure_mode` not in `failure_modes` |
| `playbook.invented_component` | action `component` not in `components` / `asset_template.components` |

`field` points at the location, e.g. `historian_queries[0].window`.

Evidence declaration is opt-in: playbooks without `evidence_requirements` keep today's behaviour (the executor still reports `evidence_not_in_baseline_mapping` as a warning).

## Provider catalog

`playbooks/provider-catalog.json` lists approved provider ids and the read-only or draft-only permissions each may request. Extending it is a reviewed change.
