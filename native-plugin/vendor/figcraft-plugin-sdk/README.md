# Vendored FigCraft plugin SDK

This directory vendors the official Rust SDK from [storytold/figcraft](https://github.com/storytold/figcraft), file `plugins/sdk/src/lib.rs`.

- Upstream snapshot: FigCraft main branch commit `0d6ce1124c03c8efc3807db5689fee7f8ec94756`
- Upstream crate: `figcraft-plugin-sdk` 0.1.0
- License: MIT OR Apache-2.0; see the accompanying license files.

The SDK lives in FigCraft's nested `plugins/` Cargo workspace. Cargo Git dependencies cannot select a package from that nested workspace directly, so the source is vendored here to make this plugin build reproducibly without requiring a separate FigCraft checkout.

When updating FigCraft compatibility, compare this snapshot with the upstream SDK and update the snapshot and this note together.
