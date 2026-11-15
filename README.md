# Lucide Companion for FigCraft

A native macOS companion for browsing the full Lucide icon set, styling SVGs, and inserting editable vector geometry into FigCraft.

**Author:** Jayanta  
**Version:** 0.3.0  
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
- Right-click any icon for Copy icon (SVG + PNG), Insert into FigCraft, and Favorites actions.
- Copy icon writes both the original 24 × 24 SVG vector and a transparent PNG fallback to the macOS clipboard, allowing compatible apps to choose the best representation.
- Catalogue download and FigCraft actions run in background threads so the UI stays responsive.

### Keyboard
- `⌘F`: focus icon search.
- `↑` / `↓`: move through the filtered icon list when search is not focused.
- `Enter`: select the first result when no result is selected.
- `Esc`: clear the search query.

## What's new in v0.3.0

- Category picker is now a vertically scrollable list with outside-click dismissal.
- Long category names stay right-aligned with consistent spacing before the dropdown chevron.
- Category dropdown scrollbar matches the slim main icon-grid scrollbar.
- Navigation and button labels provide brighter hover feedback.
- "Craft" in the Open FigCraft button uses the `#0cdb96` accent colour.

## Download

<https://github.com/dsvictor/Lucid-Icon-Companion/releases/latest>

Download **Lucide-Companion-macOS-arm64.dmg** from the latest release.

### Install on macOS

1. Download the DMG from the [latest GitHub release](https://github.com/dsvictor/Lucid-Icon-Companion/releases/latest).
2. Open the DMG.
3. Drag **Lucide Companion.app** into your Applications folder.
4. Eject the mounted DMG, then launch the app from Applications.

**Compatibility:** macOS 13 or later, Apple Silicon (ARM64).

### First launch: app is not notarized

This release is currently **not code-signed or notarized**. macOS Gatekeeper may prevent it from opening because Apple has not notarized the app. Only proceed if you downloaded the DMG from this project's official GitHub Releases page and trust the software.

You can use either of the following methods. Usually, the manual method is preferable because it approves this app through macOS settings without using Terminal.

#### Method 1 — Manual approval (recommended)

1. Make sure **Lucide Companion.app** is in your **Applications** folder.
2. Try to open the app once. If macOS displays a security warning, dismiss it using **Done** or **Cancel**.
3. Open **System Settings → Privacy & Security**.
4. Scroll down to the **Security** section. You should see a message saying that Lucide Companion was blocked from use.
5. Select **Open Anyway**. Authenticate with your Mac password or Touch ID if requested.
6. Confirm **Open** in the final prompt.

The **Open Anyway** option generally appears only after you have attempted to launch the app and macOS has blocked it. The wording and placement can vary slightly between macOS versions.

#### Method 2 — Terminal

After copying the app into **Applications**, open Terminal and run:

```sh
xattr -dr com.apple.quarantine "/Applications/Lucide Companion.app"
```

Then open **Lucide Companion.app** from Applications.

This command removes the quarantine attribute from this app bundle and its contents so Gatekeeper no longer applies the downloaded-file quarantine check to it. It does **not** code-sign or notarize the app. Check that the app came from the official release before running the command.

If you installed the app somewhere other than Applications, replace the path in the command with the actual path to **Lucide Companion.app**. You can also type `xattr -dr com.apple.quarantine ` (including the trailing space) in Terminal, drag the app into the Terminal window to insert its path, and press Return.

**Important:** Do not disable Gatekeeper globally with commands such as `spctl --master-disable`. The steps above apply only to this app.

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

FigCraft must have been started with the optional control channel enabled. A FigCraft instance that was opened normally cannot be attached to retroactively by the companion. If it is already running without control, quit it and use **Open FigCraft** in Lucide Companion. The app will not force-open a second FigCraft instance.

The import command accepts X/Y coordinates and a `center` flag. When centred mode is enabled, no explicit coordinates are sent and FigCraft uses the current viewport centre.

## Known limitations

- Lucide's static package publishes tags and tag groups, not a separate category taxonomy; the app therefore exposes upstream tag groups rather than inventing categories.
- FigCraft's control channel is opt-in at application startup. If FigCraft is already open without the control channel, quit it completely and click Open FigCraft in Lucide Companion to relaunch it with control enabled. The companion intentionally avoids spawning a duplicate FigCraft window.
- Dragging icons directly out of the companion into another app is not implemented yet; use the named Insert action, Copy SVG, or Export SVG.
- The macOS package is unsigned and not notarized. Developer ID signing and notarization require the owner's Apple Developer ID credentials.
- Windows and Linux are included in CI compile checks; native installers and interactive runtime testing remain future work after macOS behaviour is validated.

## Licensing

Companion source: MIT. Lucide SVG assets: ISC, as published by the Lucide project. See https://github.com/lucide-icons/lucide.
