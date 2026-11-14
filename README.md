# Lucide Companion for FigCraft

A native macOS companion for browsing the full Lucide icon set, styling SVGs, and inserting editable vector geometry into FigCraft.

**Author:** Jayanta  
**Version:** 0.2.0  
**First platform:** macOS Apple Silicon  
**Architecture:** Rust + egui, with a portable application core.

## Features

### Icon library
- Resolves the latest published `lucide-static` package at runtime.
- Downloads the complete SVG set and upstream `tags.json` metadata.
- Search across icon names and metadata tags; all search terms must match.
- Filter by upstream tag groups.
- All, Favorites and Recent views.
- Persistent favourites and recent-icon history.

### Preview and SVG styling
- SVG preview rendered locally.
- Supplied red-and-blue companion mark is packaged as the native macOS app icon.
- Adjustable preview size and stroke width.
- Custom stroke and fill colours (hex values); fill can remain `none`.
- Export SVG to a chosen location.
- Copy SVG source to the clipboard and inspect raw markup.

### FigCraft integration
- Launch FigCraft with its loopback control listener.
- Test the control connection using the documented `window.list` method.
- Import SVG as editable geometry with `engine.execute` → `file.import`; the selected Lucide icon name is passed as the import name.
- Clipboard Copy remains a generic SVG paste operation and FigCraft may label that object “Pasted SVG”; use Insert for the named import path.
- Insert centred in the current viewport or at explicit canvas X/Y coordinates.
- Catalogue download and FigCraft actions run in background threads so the UI stays responsive.

### Keyboard
- `⌘F`: focus icon search.
- `↑` / `↓`: move through the filtered icon list when search is not focused.
- `Enter`: select the first result when no result is selected.
- `Esc`: clear the search query.

## Download

<https://github.com/dsvictor/Lucid-Icon-Companion/releases/latest>

Download **Lucide-Companion-macOS-arm64.dmg** from the latest release.

### Install on macOS

1. Download the DMG from the [latest GitHub release](https://github.com/dsvictor/Lucid-Icon-Companion/releases/latest).
2. Open the DMG.
3. Drag **Lucide Companion.app** into your Applications folder.
4. Eject the mounted DMG, then launch the app from Applications.

**Compatibility:** macOS 13 or later, Apple Silicon (ARM64).

**Security note:** The current release is not code-signed or notarized. macOS may block the first launch; if so, use System Settings → Privacy & Security to approve the app after confirming that you trust the download.

**SHA-256 (v0.2.0):**
`fc486b60e54b1f03e656b8526d7b2dd1d3e8cccc12b54b94c80074b4e4f1f464`

For development builds and CI checks, see [GitHub Actions](https://github.com/dsvictor/Lucid-Icon-Companion/actions).

## Build from source

Install Rust using rustup, then run:

```sh
cargo run
```

Create an Apple Silicon app bundle and DMG on macOS:

```sh
bash scripts/package-macos.sh
```

## FigCraft control channel

The companion connects to `127.0.0.1:7979` using newline-delimited JSON. It launches FigCraft with `--control 7979`, tests the connection with `window.list`, and sends SVG data as base64 through `engine.execute` / `file.import`.

The import command accepts X/Y coordinates and a `center` flag. When centred mode is enabled, no explicit coordinates are sent and FigCraft uses the current viewport centre.

## Known limitations

- Lucide's static package publishes tags and tag groups, not a separate category taxonomy; the app therefore exposes upstream tag groups rather than inventing categories.
- Runtime insertion still needs a hands-on test against the user's installed FigCraft 0.15.0 and active document.
- Dragging icons directly out of the companion into another app is not implemented yet; use the named Insert action, Copy SVG, or Export SVG.
- The macOS package is unsigned. Developer ID signing and notarization require the owner's Apple Developer ID credentials.
- Windows and Linux are included in CI compile checks; native installers and interactive runtime testing remain future work after macOS behaviour is validated.

## Licensing

Companion source: MIT. Lucide SVG assets: ISC, as published by the Lucide project. See https://github.com/lucide-icons/lucide.
