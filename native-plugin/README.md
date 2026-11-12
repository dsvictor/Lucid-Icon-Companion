# Lucide Companion — Native FigCraft Plugin

**Author:** Jayanta  
**Status:** Native plugin prototype; compile and in-app verification are still required  
**Target:** FigCraft plugin API 6

Lucide Companion is a native WebAssembly plugin for FigCraft. It is separate from the existing Rust/egui macOS companion; the desktop app and its existing code are not changed by this plugin.

## Features in this prototype

- Bundles the official `lucide-static` SVG icon set into the WebAssembly module.
- Searches icon names locally; no network access is used by the plugin.
- Shows up to 48 matching icon names in FigCraft's native floating panel and reports the total match count.
- Provides icon size, stroke width and stroke colour controls.
- Inserts the selected icon as an editable FigCraft vector layer using `vector.create`.
- Preserves the vector's aspect ratio and centres the created layer in the current view.
- Exports the selected icon as an SVG file through FigCraft's `save_file` capability.

The panel uses FigCraft's native controls. It is a floating, draggable panel, not a dockable sidebar or a custom thumbnail grid.

## SDK and compatibility

The plugin targets API 6. The official FigCraft SDK lives in a nested Cargo workspace at `plugins/sdk`; Cargo cannot select that nested workspace package directly as a Git dependency. To keep this project buildable without a separate FigCraft checkout, the SDK source is vendored under `vendor/figcraft-plugin-sdk`, with its upstream provenance and MIT/Apache license files.

The vendored SDK snapshot is documented in `vendor/figcraft-plugin-sdk/README.md`. Compare it with the upstream SDK when updating FigCraft compatibility.

The plugin uses documented FigCraft commands:
- `view.get` to find the current view centre.
- `vector.create` to create an editable vector from SVG path data.
- `node.get` and `node.set` to read the new vector bounds and centre it precisely.
- `save_file` to offer the SVG export to the user.

## Requirements

- Rust toolchain via [rustup](https://rustup.rs)
- Node.js and npm
- Python 3

The build downloads the pinned `lucide-static@1.72.0` package. To intentionally use another compatible release, set `LUCIDE_VERSION` when running the build.

## Build on macOS

From the repository root:

```sh
cd native-plugin
bash build-wasm.sh
```

The script generates `catalogue.json`, runs Python and Rust tests, installs the `wasm32-unknown-unknown` target and builds the plugin.

The output is:

```text
native-plugin/dist/
├── figcraft-plugin.json
├── lucide_companion.wasm
└── assets/
    └── LUCIDE-LICENSE.txt
```

## Install in FigCraft

1. Build the plugin.
2. In FigCraft, choose **Plugins → Development → Import plugin from manifest…**
3. Select `native-plugin/dist/figcraft-plugin.json`.
4. Open **Plugins → Lucide Companion → Lucide icon browser…**
5. Search for an icon, choose it, adjust its appearance, then insert or export it.

Keep the manifest, WASM module and `assets` folder together.

## Permissions and sandbox

The manifest requests document write access for `vector.create` and `node.set`, read access for `view.get` and `node.get`, and `saveFile` for SVG export. The plugin has no filesystem or network access of its own. The icon data is bundled at build time.

## Known prototype limitations

- Search matches icon names; Lucide tags and aliases are not included yet.
- The native panel uses a list selector rather than a custom thumbnail grid.
- No favourites, recent icons or Ink Icon collection yet.
- Final insertion, visual styling and export still need to be checked in the installed FigCraft desktop app.

## Licences

- Plugin code: MIT.
- Vendored FigCraft SDK: MIT OR Apache-2.0; see `vendor/figcraft-plugin-sdk/`.
- Lucide icon data: ISC, with the upstream licence included in the built bundle at `assets/LUCIDE-LICENSE.txt`.
