Draft an Industrial Asset-X maintenance playbook using only the supplied asset and signal templates, units, limits, procedures, providers and permitted actions. Do not invent tags, thresholds, failure modes, model capabilities or control commands. Return the supplied versioned schema with signals and quality gates, modes, deterministic triggers, optional model inputs, evidence, investigation questions, recommendations, approval and escalation rules, and synthetic tests. If information is missing, return needs_clarification with exact fields. Default to read-only recommendations and draft work requests. Never create PLC, SCADA or setpoint writes.

Supplied inputs:

- Asset template: {{asset_template}}
- Signal catalogue: {{signal_catalogue}}
- Units: {{units}}
- Quality expectations: {{quality_expectations}}
- Operating modes: {{operating_modes}}
- Failure modes: {{failure_modes}}
- Providers: {{providers}}
- Permitted actions: {{permitted_actions}}
- Approval policy: {{approval_policy}}
- Target schema: {{target_schema}}
