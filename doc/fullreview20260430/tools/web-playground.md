# web playground review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `nepl-web/src/lib.rs`
- `nepl-web/build.rs`
- `web/src/main.ts`
- `web/src/runtime/**`
- `web/src/workspace/**`
- `web/src/editor/**`
- `web/src/editor-core/**`
- `web/src/language/**`
- `web/styles.css`
- `tests/playground_editor/**`
- `nodesrc/playground_*_test_runner.js`

## 良い点

`nepl-web` は Rust compiler を wasm-bindgen で公開し、compile、WAT generation、VFS stdlib injection、analysis payload、diagnostics rendering を browser 側へ提供している。diagnostic payload は stable code を含み、editor contract と整合している。

`web/src/main.ts` は bundled stdlib と examples を VFS に mount し、workspace UI を起動する。初期ファイルとして `/examples/rpn.nepl` を開くため、examples の品質は playground 第一印象に直結する。

`web/src/editor-core` は editor state / reducer / keymap / language-analysis bridge を分けている。`tests/playground_editor` と `nodesrc/playground_editor_test_runner.js` は DOM なしで editor behavior と analysis request を確認できる。

`web/src/language/neplg2/neplg2-provider.ts` は wasm analysis と editor payload bridge を使い、hover、definition、occurrences、completion、diagnostics を供給する。byte offset と JS index の mapping を持つため、UTF-8 / char 変更時の重要境界である。

## 問題とリスク

`nepl-web/src/lib.rs` は wasm binding、WAT decoration/minify、diagnostic rendering、analysis JSON、VFS merge、compiler invocation を 1 ファイルに持つ大きい境界である。Rust compiler 側の backend 巨大 file 問題と同じく、今後 web API 追加時には binding / diagnostics / analysis / compile outputs を分離する必要がある。

playground は bundled examples を直接ユーザーへ見せるが、examples doctest は CI gate に入っていない。このため `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895` が playground 品質にも影響する。

analysis bridge は Rust `nepl-language` / `nepl-lsp` と似た責務を TypeScript 側にも持っている。将来的に hover/definition/semantic token logic が二重実装になりすぎると、browser と LSP の挙動差が増える。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `nepl-web` compile API | wasm-bindgen で compiler API を公開。 | 良いが lib.rs 分割余地あり。 |
| bundled stdlib/examples | VFS へ mount。 | 良い。examples CI gap に注意。 |
| workspace/panels | editor/terminal/file explorer を統合。 | 実用段階。 |
| editor-core | reducer/state/keymap/analysis bridge。 | 良い。test runner あり。 |
| NEPLg2 provider | wasm analysis から hover/definition を生成。 | UTF-8 boundary 重要。 |
| playground tests | CLI snapshot と worker regressions。 | 良い。CI 接続範囲の継続確認が必要。 |

## 推奨対応

- `nepl-web/src/lib.rs` は binding API、diagnostic render、analysis conversion、compile output generation へ段階分割する。
- examples doctest CI gap を修正し、playground 初期サンプルの regression を main gate に入れる。
- TypeScript analysis bridge と Rust `nepl-language` の contract を揃え、hover/definition の期待 JSON を共有する。
