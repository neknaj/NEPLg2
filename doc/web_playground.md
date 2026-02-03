# Web playground

The web playground lives under `web/` and is built by Trunk (`Trunk.toml` targets
`web/index.html`).

## Local development

- Run `trunk serve` and open `http://127.0.0.1:8080/`.
- The embedded editor is optional. If `web/vendor/editorsample` is missing, the fallback textarea is used.
- GitHub Pages builds pass `--public-url /NEPL-g2/` so the published site resolves assets under that prefix.
- If you set `--public-url` to a subpath (e.g. `/web/dist/`), `trunk serve` will also expect that base path. Open `http://127.0.0.1:8080/web/dist/` or pass `--serve-base / --ws-base /` to serve from root while keeping asset URLs under the subpath.

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
