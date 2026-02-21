2026-02-10 競プロ向け改善タスク（整理版）

方針
- まず再現テストを固定し、実装差（stdlib起因かWASI host起因か）を分離する。
- 競プロの主要ボトルネックである「入力の安定性」「64-bit」「ソート/二分探索」を優先する。
- 仕様は `README` と `doc` に明文化し、`tests`/`nodesrc` で回帰検証する。

未完了タスク
1. `kpread` 不安定挙動の再現固定
- Node.js WASI での「stdin使用後にNUL混入/0化/値破壊」を最小再現ケースとして `tests/` に追加する。
- 同一 wasm を `wasmtime` と `node:wasi` で比較する差分テストを用意する。
- 判定方針:
  - Nodeのみ再現: host依存/ABI差を優先調査
  - 両方で再現: stdlib実装を優先調査

2. WASI preview1 ABI の固定化（IOVec）
- `kp/kpread` / `kp/kpwrite` の iovec を `wasm32` 前提 `ptr32 + len32`（8 bytes）で統一する。
- `fd_read`/`fd_write` ラッパを stdlib に追加し、直接 iovec 組み立てを排除する。
- `nread==0` は EOF として扱い、errno 系と分離する仕様に統一する。
- `README`/`doc` に ABI を明記する:
  - `WASI preview1`
  - `wasm32`
  - `iovec = 8 bytes (ptr32 + len32)`

3. 64-bit 最小機能の提供
- 境界値テスト（0, -1, min/max近傍, 19桁）を追加する。

4. 10進変換の共通化
- `itoa` / `utoa` を共通モジュール化して再利用可能にする。
- `kpwrite` / `stdio` / `string` の重複実装を統合する。

5. ソート/API の競プロ最適化
- 非比較ソートとして `counting_sort`（整数限定）を追加する。
- `radix_sort`（32-bit 整数）を追加する。
- 必要であれば stable sort を追加する。

6. 二分探索と頻出ユーティリティ
- `fill_u8` / `fill_i32` / `memset` 相当の初期化 API を追加する。

7. Vec API の in-place 化
- `vec_push_in_place` を追加し、競プロ向け推奨 API とする。
- 大量要素（2e5+）で push 系オーバーヘッド比較テストを追加する。

8. 競プロ標準 I/O の仕様化
- `kp/kpread` と `kp/kpwrite` を「標準の正解実装」として仕様化する。
- 仕様項目:
  - 対応整数型（i32/i64）
  - 符号付き/符号なし
  - EOF
  - 改行/空白
  - バッファサイズ
  - エラー処理
- 巨大入力・境界値・異常系テストを stdlib 側に追加する。

9. ランタイム互換検証
- Node/uvwasi, wasmtime, wasmer の実行差を CI またはローカル検証スクリプトで可視化する。
- 入力と出力が一致しない場合、ABI層かstdlib層かをログで切り分ける。

10. 使い勝手と安全性
- `read_line` の固定長仕様を見直し、切り捨て検知か可変拡張を提供する。
- 可能なら `free(ptr)` 系（サイズ管理内包）を導入し、`dealloc(ptr,size)` の誤用を減らす。

11. デバッグ支援
- `assert` 失敗時のメッセージと位置情報を強化する。
- `debug_print_i32` など提出時無効化可能な低コストデバッグ API を追加する。
