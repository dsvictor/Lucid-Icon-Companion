# Lucide Companion for FigCraft

A native desktop companion for browsing the complete Lucide icon library, previewing and customizing SVGs, and inserting editable SVG geometry into FigCraft.

**Author:** Jayanta  
**Initial target:** macOS Apple Silicon  
**Implementation:** Rust + egui

## Features

- Downloads the current published `lucide-static` package at runtime.
- Caches every upstream SVG and its upstream `tags.json` search metadata.
- Searches icon names and multiple metadata tags.
- All, Favorites and Recent library views.
- SVG preview with adjustable size, stroke width, stroke colour and fill.
- Export SVG, copy SVG source, and preview the raw SVG markup.
- Saves favourites, recents, appearance settings and FigCraft connection preferences.
- Launches FigCraft with the loopback control channel enabled.
- Tests the control connection and imports SVG as editable geometry.
- Supports viewport-centred insertion or explicit canvas X/Y placement.

## Install the macOS build

Download the latest **Lucide-Companion-macOS-arm64** artifact from [GitHub Actions](https://github.com/dsvictor/Lucid-Icon-Companion/actions). Extract the artifact ZIP, open the DMG, and drag Lucide Companion into Applications. This initial distribution is unsigned; macOS may require you to approve opening it.

## Build from source

Install Rust using rustup, then run `cargo run`.

Package an Apple Silicon app and DMG on macOS with `bash scripts/package-macos.sh`.

## FigCraft control channel

The app connects to `127.0.0.1:7979` using newline-delimited JSON. It can launch FigCraft with `--control 7979`, test the connection, and call `engine.execute` with `file.import`, passing SVG bytes as base64 data. The import supports viewport-centred or explicit X/Y placement.

## Known limitations

- Search uses Lucide's published tags metadata; icon categories are not included in the static package's tag index.
- The target-machine behaviour must still be tested with the user's installed FigCraft 0.15.0.
- The initial macOS package is unsigned. Developer ID signing and notarization require the owner's Apple Developer ID credentials.
- Windows and Linux builds are not yet included in the release pipeline.

## Licensing

The companion source is MIT-licensed. Lucide SVG assets remain under the Lucide project's ISC license: https://github.com/lucide-icons/lucide.
