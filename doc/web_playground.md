# Web playground

> **対象実装**: このドキュメントは現行 Web Playground（Bootstrap 実装）について記述する。NEPLg3 の正の仕様は `doc/neplg3/spec/` を参照。

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
- Drag-and-drop supports moving panel shells across `left`, `right`, `top`, and `bottom` drop zones.
- Editor tabs can be dragged onto another editor panel to merge there, or dropped on a panel edge to create a new split and move the tab into it.
- Explorer files can be dragged onto an editor panel to open there, or dropped on a panel edge to create a new editor split and open the file there.
- Dragging onto an editor panel's tab bar is treated as tab attachment, not as split creation.
- Center-drop merges for editor panels preserve tab snapshots instead of re-reading from VFS.
- Tab switching is treated as a full-document replace, so syntax highlighting does not reuse incremental payloads across unrelated files.
- Each editor panel owns its own language provider instance; analysis state is not shared across panels.
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
- `neplg2 build` / `neplg2 run` and `wasmi` execution now run in a dedicated web worker, so compile and execution no longer block the workspace UI.
- The worker owns long-running WASI execution and streams stdout/stderr back to the terminal while stdin continues to be accepted from the focused terminal panel.

## Notes

- The compiler runs in WebAssembly and uses an in-memory stdlib source map.
- Diagnostics are rendered as text with line/column information.
- The terminal is a browser-only convenience; it does not execute `cargo` commands.
- Only stdlib imports are available in the browser; local file imports are not supported yet.

## Editor Language Highlighting

The editor receives compiler analysis output and normalizes it in `web/src/editor-core/language-analysis.ts` before the canvas renderer sees it.

Contract:

- Lexer token kinds, diagnostics, resolution data, and semantic token data are converted into editor-facing token decorations without exposing DOM or canvas types.
- Latest NEPLg2 syntax markers such as `pub`, `%fn`, `%impure`, `\arg`, `&Type`, `Result::Ok`, `#import`, `@`, and `@merge` are classified at the analysis boundary.
- Prefix expression ranges come from Rust semantic analysis. `expr_span` is the expression range for a token, and `arg_span` is the callee argument range when the token belongs to a prefix-call argument. The older `expression_range` / `arg_range` field names remain accepted as compatibility aliases.
- `%T expr` type annotation highlighting is driven by Rust AST ranges. The outer annotation range covers `%T`, while the inner type-expression range covers the type after `%`; this prevents lower-case type names such as `%widget_state` from falling back to variable coloring.
- `void` is highlighted as a zero-argument marker, not as a type. `unit` is highlighted as a type inside compiler-reported type syntax ranges and as a unit literal in value expression position.
- Name colors are compiler-driven when name-resolution data is available: function names/calls are `function`, immutable `let` names are `constant`, mutable names and parameters are `variable`, and type declarations/usages are `type`.
- Path-like names such as `group1::group2::name` classify the left namespace/group segments as `namespace` so they can be drawn dimmer than the final member name.
- Absence and errors remain explicit in the language provider boundary; browser rendering code consumes a completed update payload.
- Token color names are stable editor categories such as `keyword`, `type`, `function`, `constant`, `variable`, `namespace`, `operator`, `punctuation`, `literal-string`, `literal-char`, `literal-number`, `literal-bool`, `literal-unit`, `literal-void`, and `comment`.

Current implementation:

- `nepl-web` emits `syntax_ranges` and `token_classifications` from the parsed AST before type checking completes, so type-expression highlighting remains available even while a program has type errors.
- `nepl-web` also folds lexer marker tokens and name-resolution trace into `token_classifications`, so `fn void T`, `\void`, function names, parameter names, and immutable local names do not depend on TypeScript guessing.
- `nepl-web` emits path namespace/member classifications from compiler token context. The editor theme draws `namespace` with a low-contrast color and keeps the final member as `constant`, `function`, or another resolved category.
- `web/src/editor-core/language-analysis.ts` consumes `token_classifications` before lexical fallback colors. Rust-provided classifications therefore override heuristic identifier coloring within the reported compiler range.
- `token_resolution` remains available for hover, definition jump, and compatibility fallback. It must not override `token_classifications`, because the compiler classification is the single authority for editor colors.
- `web/src/language/neplg2/neplg2-provider.ts` uses `analyze_semantics_with_vfs` for editable `.nepl` files when the playground VFS is available. The current unsaved editor text is overlaid onto `serializeForCompile()` before analysis so imports and span source paths match the visible document.
- `Kw*` and directive tokens become `keyword`.
- primitive type names and upper-case identifiers remain a fallback only. Compiler-provided type syntax and name-resolution classifications take priority when available.
- `%`, `\`, `&`, `::`, arithmetic symbols, and pipe/arrow-like tokens become `operator`.
- directive bodies are split so paths stay `string`, `as` / `@merge` stay `keyword`, and wildcard import markers stay `operator`; standalone `@` also remains a keyword token for incomplete editor input.
- `nodesrc/test_analysis_api.js`, `nodesrc/test_editor_current_syntax_highlighting.js`, and `nodesrc/test_neplg2_language_provider_vfs.js` fix this contract for current NEPLg2 syntax, prefix-call argument ranges, `%` type annotation ranges, and VFS-backed playground analysis.

## Editor redevelopment test path

The playground editor redesign is expected to stay testable without a browser.

- The app entrypoint now creates the editor through the new `editor-core` browser adapter instead of calling the old global factory directly.
- Build the web TypeScript side first with `npm --prefix web run build:ts`.
- When the Rust / WASM side changes, run `trunk build` before CLI verification.
- The formal CLI check is `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=/tmp/playground-editor-tests.json`.
- Inspect the generated JSON summary to confirm case counts, failures, and per-case snapshots.
- The CLI suite now covers keyboard/state fixtures, pure text editing, left-right and vertical cursor movement, Home/End, PageUp/PageDown, and pure analysis fixtures for highlight payloads, problems, hover, definition, and occurrences.
- Workspace-specific headless checks include `node nodesrc/playground_workspace_test_runner.js` and `node nodesrc/playground_tab_transfer_test_runner.js`.
- Drag/drop intent checks include `node nodesrc/playground_drag_drop_test_runner.js`.
- Terminal worker protocol checks include `node nodesrc/playground_shell_worker_test_runner.js`.
- `trunk build` is still a hard requirement before commit, but it depends on the `trunk` binary being available in the environment.
