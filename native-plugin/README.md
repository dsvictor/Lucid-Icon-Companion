# Lucide Companion — Native FigCraft Plugin

**Author:** Jayanta  
**Status:** Prototype / first native-plugin implementation  
**Target:** FigCraft desktop, plugin API 4 (compatible with the current API 6 runtime)

This is a native WebAssembly plugin, separate from the existing Rust/egui macOS companion. The desktop companion remains untouched on the main branch.

## Current prototype features

- Bundles the official `lucide-static` icon set into the WebAssembly module at build time.
- Searches icon names locally; the sandbox makes no network requests.
- Shows up to 48 matching names in the native floating plugin panel.
- Lets you choose icon size, stroke width and stroke colour.
- Inserts the selected icon using `vector.create`, producing a normal editable vector layer rather than a locked widget.
- Exports the selected icon as SVG through FigCraft's own Save dialog.
- Uses FigCraft's native panel controls and persistent plugin installation.

## Why API 4?

The current plugin only needs the panel/settings ABI and `save_file` (API 4). FigCraft's API 6 adds SVG and line drawing to canvas widgets, but widgets are not needed for this first implementation: `vector.create` creates editable document vectors directly. The plugin can run on API 6 while declaring the lowest API it actually uses.

## Build on macOS

Requirements:
- Rust toolchain via [rustup](https://rustup.rs)
- Node.js and npm (only to download the upstream icon package during the build)
- Python 3

From this folder:

```sh
chmod +x build-wasm.sh
./build-wasm.sh
```

The build creates `dist/figcraft-plugin.json`, `dist/lucide_companion.wasm` and the generated catalogue.

## Install in FigCraft

1. Build the plugin.
2. In FigCraft, choose **Plugins → Development → Import plugin from manifest…**
3. Select `native-plugin/dist/figcraft-plugin.json`.
4. Open **Plugins → Lucide Companion → Lucide icon browser…**
5. Search, select an icon and use **Insert editable vector** or **Export SVG**.

FigCraft reads the WASM module beside the manifest. Keep the two files together.

## Sandbox and permissions

The manifest requests document write access only for `view.get`, `vector.create` and `select.set`, plus the API 4 `saveFile` capability. The plugin has no filesystem or network access. Icon data is bundled at build time.

## Known prototype limitations

- Search currently matches icon names, not upstream tags.
- The panel uses a native list selector rather than a custom thumbnail grid; plugin panels do not support arbitrary custom layouts.
- No favourites/recent list or Ink Icon collection yet.
- This repository's CI build produces the WASM artifact; final verification of insertion and styling must be done inside the user's installed FigCraft desktop app.

## License

Plugin source: MIT. Bundled Lucide icon data: ISC, as published by the Lucide project.
