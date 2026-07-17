# PR 2 — Add Asset-X core contracts

## Goal

Define the generic asset domain resources that downstream industrial solutions can reuse.

## Scope

Add GX contract packages and JSON schemas for core Asset-X resources.

## Adds contracts

```text
contracts/assetx/site/
contracts/assetx/asset/
contracts/assetx/component/
contracts/assetx/sensor/
contracts/assetx/failure-mode/
contracts/assetx/maintenance-action/
contracts/assetx/health-state/
contracts/assetx/maintenance-outcome/
```

## Resource concepts

### `assetx.site`

Represents an industrial site, plant, substation, facility, or location.

### `assetx.asset`

Represents a maintainable asset such as a transformer, motor, pump, generator, switchgear bay, cooling system, or compressor.

### `assetx.component`

Represents sub-parts of an asset such as windings, bearings, cooling fans, pumps, main tank, tap changer, or protection devices.

### `assetx.sensor`

Represents an observed signal/tag mapped to an asset/component.

### `assetx.failure-mode`

Represents a generic failure mode, linked to asset types and components.

### `assetx.maintenance-action`

Represents an action such as inspect, monitor, derate, repair, replace, lubricate, or shut down.

### `assetx.health-state`

Represents current or historical health state, score, risk, and supporting evidence.

### `assetx.maintenance-outcome`

Represents the result of an action or inspection.

## Acceptance criteria

- Each contract includes manifest, schema, examples, lifecycle state if applicable, and mutation rules.
- Contracts are asset-domain generic.
- Contracts can support substation transformer, rotating equipment, cooling system, and power-station equipment without hardcoding any customer.
- No Koncar-specific names, tags, branding, or examples.

## Out of scope

- Industrial-X model/evaluation resources.
- Store-X package contracts.
- WASM components.
