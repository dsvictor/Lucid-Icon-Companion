# Lucide Companion for FigCraft

A native desktop companion for browsing the complete Lucide icon library, previewing and customizing SVGs, and inserting editable SVG geometry into FigCraft.

**Author:** Jayanta  
**Initial target:** macOS Apple Silicon  
**Implementation:** Rust + egui

## Capabilities in this development build

- Resolve the latest published `lucide-static` package at runtime and download its SVG catalogue (not a hard-coded curated subset).
- Cache SVG assets locally and search icon names.
- Render SVG previews and customize stroke width, stroke colour and fill.
- Export SVG files and copy SVG source to the clipboard.
- Launch FigCraft with its loopback control channel enabled.
- Insert SVG as editable geometry using FigCraft's documented `engine.execute` / `file.import` control command.

## Build from source

Install Rust using rustup, then:

```sh
cargo run
```

To package an Apple Silicon app and DMG on macOS:

```sh
bash scripts/package-macos.sh
```

## FigCraft control channel

The app connects to `127.0.0.1:7979` using newline-delimited JSON. It can launch FigCraft with `--control 7979`, then call `engine.execute` with `file.import`, passing the SVG as base64 data. FigCraft places and selects the imported editable frame.

## Current development scope

The first build focuses on the main browse, preview, customize, export and insert path. Search currently matches icon names. Upstream tags/categories, favourites, recents, drag-and-drop, richer placement controls, and polish remain follow-on work.

The macOS workflow builds an Apple Silicon app bundle and uploads a DMG artifact. The initial package is unsigned; code signing and notarization require an Apple Developer ID certificate and credentials.

## Licensing

The companion source is MIT-licensed. Lucide SVGs remain upstream assets under the Lucide project's ISC license: https://github.com/lucide-icons/lucide.
