# PR 19 — Replace transformer playbook skeleton with validated reusable package

## Goal

Turn the generic substation transformer health playbook into a production-ready reusable package with strict assets, sensors, features, scoring, recommendations, fixtures, and documentation.

## Placeholder removal targets

```text
playbooks/substation-transformer-health-basic/playbook.json broad lists only
store/examples/substation-transformer-health-basic minimal fixture and reports
baseline scoring config without executable validation
recommendation rules as plain text only
```

## Scope

- Define full asset/component templates.
- Define sensor requirements with units, frequency expectations, quality requirements, and optional/required flags.
- Add feature definitions linked to analytics crate implementations.
- Add baseline scoring configuration with thresholds, risk levels, and evidence mapping.
- Add recommendation rules with machine-readable conditions/actions.
- Add synthetic fixture sets for healthy, degraded cooling, overheating, sensor quality issue, and tap changer abnormality cases.

## Acceptance criteria

- Store package validation passes using the production validator.
- Fixture outputs are deterministic and checked in tests.
- Recommendation rules are machine-readable and evidence-linked.
- The playbook remains customer-neutral and demo-safe.

## Out of scope

- Real customer data.
- Customer presentation wrappers.
