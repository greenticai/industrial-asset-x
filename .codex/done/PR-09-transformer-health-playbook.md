# PR 9 — Add generic substation transformer health playbook

## Goal

Provide the first reusable industrial playbook package: generic substation transformer health.

## Scope

Add a generic playbook and store example for transformer thermal/cooling health.

## Adds

```text
playbooks/substation-transformer-health-basic/
store/examples/substation-transformer-health-basic/
```

## Contents

```text
asset template
component template
sensor requirements
failure modes
feature definitions
baseline scoring config
operator feedback questions
recommendation rules
store package manifest
validation fixtures
```

## Generic sensor requirements

```text
oil_temperature
winding_temperature
load
current
voltage
cooling_fan_status
cooling_pump_status
tap_position
```

## Generic failure modes

```text
load-related overheating
cooling system degradation
abnormal winding temperature
tap changer abnormality
sensor quality issue
```

## Acceptance criteria

- Playbook is generic and reusable.
- Store metadata exists.
- Fixture data is synthetic/demo-safe and customer-neutral.
- No Koncar-specific content.

## Out of scope

- Koncar demo presentation.
- Real customer data.
