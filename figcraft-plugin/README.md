# Lucide Library — FigCraft plugin

**Author:** Jayanta  
**Version:** 0.1.0  
**Target:** FigCraft plugin API 2 (WASM)

This is the native FigCraft-plugin experiment, kept separate from the Rust/egui desktop companion.

## Current scope

- Opens a FigCraft-managed Lucide Library panel.
- Searches the bundled starter icon set by name.
- Lets the user choose an icon, stroke colour and stroke width.
- Creates editable vector geometry through the permitted `node.create` command. The plugin does not call restricted `file.*` commands.
- Uses FigCraft's own panel fields, settings and callbacks; no external network or filesystem access.

## Important panel behaviour

FigCraft API 2 panels are **floating windows**, initially placed beside the properties panel. Their title bars can be dragged. The current public plugin API documentation does not describe a permanent dock/side-bar registration API. This plugin therefore uses the closest supported native panel rather than pretending it is dockable.

## Starter icon set

The first implementation bundles a complete upstream Lucide SVG catalogue (2,000+ icons) in the WASM module. The full Lucide catalogue is compiled into the WASM module at build time from pinned source revision `53fab88ff9d1b48f4704b5b5b560d6b4d560bf4c`; no runtime network or filesystem access is needed. The import ZIP includes the upstream license.

## Build

Requirements: Rust toolchain and the `wasm32-unknown-unknown` target.

```sh
rustup target add wasm32-unknown-unknown
cargo build --manifest-path figcraft-plugin/Cargo.toml --release --target wasm32-unknown-unknown
```

The module is written to:

```
target/wasm32-unknown-unknown/release/lucide_library_plugin.wasm
```

Copy that file beside `figcraft-plugin.json` in a folder, then in FigCraft choose:

**Plugins → Development → Import plugin from manifest…**

Select the manifest. Run **Plugins → Lucide Library** to open the panel.

## Permissions

The manifest requests write access only for `node.create`. It does not request network, filesystem, clock or other document-command permissions.

## Compatibility

The SDK dependency is pinned to FigCraft source commit `f45b261687880e56d1bad11ec82b45131e57ef8b`. The plugin requires API 2 for panels and settings.
