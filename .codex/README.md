# industrial-asset-x PR Plan

`industrial-asset-x` is the generic reusable Rust/WASM-first Asset-X + Industrial-X catalog and store foundation.

It should define generic namespaces:

```text
assetx.*
industrialx.*
storex.*
```

It must not contain customer-specific or Koncar-specific code, names, tags, data, or branding.

## PR sequence

1. Bootstrap generic Rust/WASM industrial asset catalog
2. Add Asset-X core contracts
3. Add Industrial-X MVP contracts
4. Add Store-X package/licensing contracts
5. Add Rust/WASM ABI and analytics crates
6. Add generic WASM components
7. Add GX ops backed by WASM components
8. Add generic flows and views
9. Add generic substation transformer health playbook
10. Add store package validation and promotion checks
11. Add synthetic scenario support later
12. Add advanced model tournament skeleton later
23. Publish supported Industrial-X Rust crate boundaries
24. Publish an executable Greentic Industrial-X operation pack
