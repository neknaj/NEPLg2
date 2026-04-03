2026-04-03 Web Playground editor

- `web/src/editor/` の state 更新責務をさらに薄くし、browser adapter / pure core 側へ移す
- `tests/playground_editor/` に multi-file import、completion、fold、problem list 表示の fixture を追加する
- pointer 操作、fold click、scroll、completion UI の状態遷移も CLI で再現できるように整理する
- GitHub Actions と同じ前提で `trunk build` を通せるように、ローカルの wasm32 target / wasm-bindgen-cli / trunk 環境差分を解消する
