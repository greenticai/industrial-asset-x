# Industrial-X operation pack

Versioned Greentic pack exposing `validate-industrial-data`,
`build-asset-health-baseline`, and `run-shadow-health-check`. The embedded
`industrialx-analytics` component uses the standard Greentic 0.6 component world and
requires no network, filesystem mounts, secrets, or state provider.

`build-answer.json` and `flow.answers.json` are the source of truth; pack manifests,
flows, bindings, locks, and copied components are generated in a temporary directory
using the same deterministic replay pattern as `greentic-demo`.

Build and validate with `bash scripts/build-industrialx-pack.sh`. The resulting
`.gtpack`, component hash, pack lock, and provenance form the immutable release
artifact. Inputs and outputs follow the schemas under the component and operation
directories. Operations are deterministic, bounded to 10,000 samples, 128 MiB, and a
two-second host timeout.

Relevant pushes to `main` automatically publish the generated pack as an OCI artifact
at `ghcr.io/greentic-biz/industrialx-operations`, tagged with `sha-<commit>` and the
moving `latest` tag. Tags matching `industrialx-pack-v*` additionally publish an
immutable semantic version, move the `release` tag, and attach the checksummed pack to
a GitHub Release. For example:

```sh
oras pull ghcr.io/greentic-biz/industrialx-operations:0.1.0
oras pull ghcr.io/greentic-biz/industrialx-operations:latest
```

Pack and component versions follow Semantic Versioning. Schema, WIT world, operation
name, equation, error diagnostic, or evidence-contract changes require compatibility
review. Consumers should pin the released pack digest as well as version.
