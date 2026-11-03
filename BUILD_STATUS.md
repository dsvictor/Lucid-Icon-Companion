# Build status — 0.2.0 development

## Implemented
- Native Rust/egui desktop app for macOS Apple Silicon.
- Dynamic latest-release Lucide static package downloader.
- Full SVG extraction with staging cache replacement, plus upstream `tags.json`.
- Multi-token name/tag search and tag-group filtering.
- All, Favorites and Recent library views.
- Persistent favourites, recent history, appearance settings and FigCraft preferences.
- SVG preview, stroke/fill customization, SVG export and clipboard copy.
- Background catalogue download and background FigCraft control operations.
- FigCraft launch, connection test using `window.list`, and editable SVG import.
- Viewport-centred or explicit X/Y placement.
- Keyboard search/navigation shortcuts.
- Apple Silicon app bundle and DMG CI packaging.

## CI evidence
- Previous 0.1.0 Apple Silicon build, tests, DMG packaging and artifact upload passed.
- 0.2.0 feature branch is under CI validation. The latest successful result must be checked before publishing a replacement artifact.

## Not yet verified on the user's Mac
- App launch and visual usability on macOS 27.2.
- Latest catalogue download and exact icon count against the npm package.
- Actual control connection and SVG insertion against installed FigCraft 0.15.0.
- Placement in the user's active document and viewport.
- Gatekeeper, clipboard and native save-dialog behaviour.

## Remaining release work
- Hands-on target-machine testing is required; remote CI cannot validate the user's installed FigCraft runtime.
- Direct drag-out from the icon browser is not supported by the current GUI toolkit integration.
- Windows/Linux compile checks are included; platform-specific installers and runtime validation remain future work. Signed/notarized macOS distribution remains future work.
- Signing/notarization requires Developer ID credentials owned by the publisher.
