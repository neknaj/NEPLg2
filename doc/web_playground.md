# Web playground

> **対象実装**: このドキュメントは現行 Web Playground（Bootstrap 実装）について記述する。NEPLg2.1 の正の仕様は `doc/2.1spec/` を参照。

The web playground lives under `web/` and is built by Trunk (`Trunk.toml` targets
`web/index.html`).

## Local development

- Run `trunk serve` and open `http://127.0.0.1:8080/`.
- GitHub Pages builds pass `--public-url /NEPL-g2/` so the published site resolves assets under that prefix.
- If you set `--public-url` to a subpath (e.g. `/web/dist/`), `trunk serve` will also expect that base path. Open `http://127.0.0.1:8080/web/dist/` or pass `--serve-base / --ws-base /` to serve from root while keeping asset URLs under the subpath.

## Panel workspace

The playground now uses a split-tree workspace instead of the old fixed three-pane layout.

- The root layout starts with `Explorer | (Editor / Terminal)`.
- Each leaf panel owns its own shell, focus state, and, for editor panels, its own tab state.
- Split ratios and focused panel state are saved in localStorage and restored on the next launch.
- Toolbar actions such as `Run`, `Compile`, `Help`, and `Save` target the focused editor panel.
- File open requests from the explorer also target the focused editor panel, creating or reusing editor state through the workspace manager.
- Drag-and-drop currently supports moving panels across `left`, `right`, `top`, and `bottom` drop zones. Center-drop merges are implemented for editor panels by merging tab sets.
- Explorer duplication is intentionally blocked, and the last explorer or last editor panel cannot be closed.
- Editor panels keep zoom per active tab, and terminal panels keep zoom per panel.
- Zoom controls are `Ctrl+Wheel`, `Ctrl++`, `Ctrl+-`, `Ctrl+0`, and two-finger pinch on touch devices. The current zoom is shown as a temporary badge overlay in the panel.

## Terminal features

The embedded terminal can:

- `run`: compile the current editor source to WASM and execute it in the browser.
- `test`: compile and execute stdlib tests (from `stdlib/tests`).
- `clear`: clear terminal output.

WAT generation is provided by the "WATを生成" button in the editor panel.

Standard input is provided via the terminal `stdin` textarea. Output is captured
from WASI `fd_write` and rendered in the terminal pane.

## Notes

- The compiler runs in WebAssembly and uses an in-memory stdlib source map.
- Diagnostics are rendered as text with line/column information.
- The terminal is a browser-only convenience; it does not execute `cargo` commands.
- Only stdlib imports are available in the browser; local file imports are not supported yet.

## Editor redevelopment test path

The playground editor redesign is expected to stay testable without a browser.

- The app entrypoint now creates the editor through the new `editor-core` browser adapter instead of calling the old global factory directly.
- Build the web TypeScript side first with `npm --prefix web run build:ts`.
- When the Rust / WASM side changes, run `trunk build` before CLI verification.
- The formal CLI check is `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=/tmp/playground-editor-tests.json`.
- Inspect the generated JSON summary to confirm case counts, failures, and per-case snapshots.
- The CLI suite now covers keyboard/state fixtures, pure text editing, left-right and vertical cursor movement, Home/End, PageUp/PageDown, and pure analysis fixtures for highlight payloads, problems, hover, definition, and occurrences.
- `trunk build` is still a hard requirement before commit, but it depends on the `trunk` binary being available in the environment.
