# Build status

## Implemented in this development branch
- Native Rust/egui desktop shell.
- Dynamic resolution of the latest published `lucide-static` package.
- Extraction and local caching of package SVG assets.
- Search by icon name.
- Rendered SVG preview and basic size, stroke-width, stroke-colour and fill controls.
- SVG export and clipboard copy.
- FigCraft launch with the loopback control channel enabled.
- Editable SVG import through the documented `engine.execute` / `file.import` protocol.
- Apple Silicon macOS packaging and GitHub Actions artifact workflow.

## Validation still required
- Verify the npm tarball's exact archive layout and compare extracted icon count with the package's published contents in CI.
- Run the application on macOS 27.2; confirm native preview, file dialogs and clipboard behaviour.
- Test the control socket and editable SVG import against the user's installed FigCraft 0.15.0. The command format is grounded in upstream source, but the target-machine runtime test has not yet happened.
- Check SVG import warnings and edge cases, including unusual SVG attributes.
- Add upstream tags/categories, favourites, recents, drag-and-drop, placement settings and keyboard navigation.
- Code signing and notarization require Apple Developer ID credentials.

## Release gate
Do not call a build production-ready until catalogue completeness, Apple Silicon runtime behaviour, and real FigCraft insertion have all been checked.
