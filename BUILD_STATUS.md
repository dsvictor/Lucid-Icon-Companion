# Build status — development branch

## Implemented
- Native Rust/egui desktop application.
- Runtime download of the latest published `lucide-static` package.
- Complete SVG extraction plus upstream `tags.json` metadata cache.
- Multi-token name/tag search.
- All, Favorites and Recent views.
- Persistent favourites, recents, appearance, import position and port settings.
- SVG preview, stroke/fill customization, export and clipboard copy.
- Async catalogue update and async FigCraft import to keep the UI responsive.
- FigCraft launch, control-channel ping, viewport-centred or explicit X/Y SVG import.
- Apple Silicon macOS app bundle and DMG workflow.

## Verified by GitHub Actions
- Apple Silicon Rust tests and release compilation passed.
- App bundle/DMG packaging passed.
- Artifact upload passed for the initial 0.1.0 build.

## Still requires user-machine verification
- Launch and interact with the app on macOS 27.2.
- Download the latest catalogue and compare the cached icon count to the upstream package.
- Test control ping and actual SVG insertion with FigCraft 0.15.0.
- Verify placement coordinates against the user's active document and viewport.
- Verify file picker, clipboard and Gatekeeper behaviour.

## Release blockers
- Add and validate category metadata if upstream exposes a suitable package index.
- Add keyboard navigation, icon grid/drag export and richer visual controls.
- Add Windows/Linux CI once macOS functionality is validated.
- Code signing and notarization need Developer ID credentials.

Do not call this production-ready until real FigCraft insertion is tested on the target Mac.
