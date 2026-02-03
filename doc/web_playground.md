# Web playground

The web playground lives under `web/` and is built by Trunk (`Trunk.toml` targets
`web/index.html`).

## Terminal features

The embedded terminal can:

- `run`: compile the current editor source to WASM and execute it in the browser.
- `test`: compile and execute stdlib tests (from `stdlib/tests`).
- `clear`: clear terminal output.

Standard input is provided via the terminal `stdin` textarea. Output is captured
from WASI `fd_write` and rendered in the terminal pane.

## Notes

- The compiler runs in WebAssembly and uses an in-memory stdlib source map.
- Diagnostics are rendered as text with line/column information.
- The terminal is a browser-only convenience; it does not execute `cargo` commands.
- Only stdlib imports are available in the browser; local file imports are not supported yet.
