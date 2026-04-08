2026-04-08 Playground

- `web/src/workspace/` の panel drag/drop と center merge の edge case を CLI で固定する
- 複数 terminal panel を shared terminal session / shared shell backend に寄せる
- mobile での workspace 操作を見直し、touch 環境では split/drag UI を明示的に縮退させる
- `tests/playground_editor/` に multi-file import、completion、fold、problem list 表示の fixture を追加する
- pointer 操作、fold click、scroll、completion UI の状態遷移も CLI で再現できるように整理する
