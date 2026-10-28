# Industrial-X Designer extension

## Prompt templates

`prompts/authoring.md` (playbook authoring, with named input placeholders) and
`prompts/investigation.md` (runtime anomaly investigation). Their prohibitions
are pinned by `tests/prompts_and_service_asset.rs`; do not weaken them.

## Dialect service address

`assets/dialect-service.json` is `{"url": "<base url>"}` and nothing else. The
designer reads it from the installed extension directory and uses it only when
the URL matches a pattern in `runtime.permissions.network` of `describe.json`.
`build.sh` packs it into the `.gtxpack` as `assets/dialect-service.json`.

**The production address is not decided yet.** Until the service is hosted, the
asset and the network permission hold only the local development address of the
executor service (`127.0.0.1:8787`, the default `--port`). When a production host
exists, change both files together (and extend the permission test).
