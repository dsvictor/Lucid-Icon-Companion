# Lucide Companion — Native FigCraft Plugin

**Author:** Jayanta  
**Status:** Preview and original-source improvements implemented; compile and in-app verification pending  
**Target:** FigCraft plugin API 6

Lucide Companion is a native WebAssembly plugin for FigCraft. It is separate from the existing Rust/egui macOS companion; the desktop app and its existing code are not changed by this plugin.

## Features in this prototype

- Bundles the official `lucide-static` SVG icon set into the WebAssembly module.
- Searches icon names locally; no network access is used by the plugin.
- Searches the complete bundled Lucide catalogue and exposes all matches through a compact native selection menu, paged in groups of 64 to respect FigCraft's per-menu option limit.
- Provides icon size, stroke width and stroke colour controls.
- Shows a selected-icon PNG preview in the floating panel.
- Includes Insert vector, Refresh list (clears the search and returns to page 1), and Export original SVG actions.
- Provides page navigation when a search returns more than 64 icons.
- Creates a dedicated square frame named after each selected icon, centred in the current view.
- Inserts the icon into a dedicated 24 × 24 frame using the original Lucide SVG viewBox coordinates.
- Retains the untouched upstream SVG source in the generated catalogue and exports that original source without rebuilding it.
- Converts Lucide primitive elements to path geometry using exact arc commands for circles, ellipses and rounded rectangles; source path data is kept verbatim.
- Keeps each vector’s own source-coordinate bounds on insertion rather than forcing every path to (0, 0), avoiding unintended translation of independent elements.
- Exports the selected icon as an SVG file through FigCraft's `save_file` capability.

The panel uses FigCraft's native controls. It is a floating, draggable panel, not a dockable sidebar or a custom thumbnail grid.

## SDK and compatibility

The plugin targets API 6. The official FigCraft SDK lives in a nested Cargo workspace at `plugins/sdk`; Cargo cannot select that nested workspace package directly as a Git dependency. To keep this project buildable without a separate FigCraft checkout, the SDK source is vendored under `vendor/figcraft-plugin-sdk`, with its upstream provenance and MIT/Apache license files.

The vendored SDK snapshot is documented in `vendor/figcraft-plugin-sdk/README.md`. Compare it with the upstream SDK when updating FigCraft compatibility.

The plugin uses documented FigCraft commands:
- `view.get` to find the current view centre.
- `node.create` to create a dedicated square frame for the icon.
- `vector.create` with the frame ID as `parent` to create editable child vectors from the source SVG elements.
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

The script downloads `lucide-static` and the Resvg renderer, generates a catalogue containing untouched SVG source plus editable path geometry, renders compact PNG previews, runs Python and Rust tests, installs the `wasm32-unknown-unknown` target and builds the plugin.

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
5. Search for an icon, open the selection menu and scroll through the current group of up to 64 names, choose one, use the page selector to browse further groups, adjust its appearance, then insert or export it. Use **Refresh list** to clear the search and return to the full catalogue.

Keep the manifest, WASM module and `assets` folder together.

## Permissions and sandbox

The manifest requests document write access for `node.create`, `vector.create` and `node.set`, read access for `view.get` and `node.get`, and `saveFile` for SVG export. The plugin has no filesystem or network access of its own. The icon data is bundled at build time.

## Known prototype limitations

- Search matches icon names; Lucide tags and aliases are not included yet.
- The native panel uses FigCraft's native selection menu, which is limited to 64 options per field; larger result sets are paged in groups of 64 rather than shown in one custom scrollable list.
- The panel displays one selected-icon preview, not a thumbnail grid; native panel image controls are limited to 256 KB each.
- No favourites, recent icons or Ink Icon collection yet.
- Final insertion, visual styling and export still need to be checked in the installed FigCraft desktop app.

## Licences

- Plugin code: MIT.
- Vendored FigCraft SDK: MIT OR Apache-2.0; see `vendor/figcraft-plugin-sdk/`.
- Lucide icon data: ISC, with the upstream licence included in the built bundle at `assets/LUCIDE-LICENSE.txt`.
