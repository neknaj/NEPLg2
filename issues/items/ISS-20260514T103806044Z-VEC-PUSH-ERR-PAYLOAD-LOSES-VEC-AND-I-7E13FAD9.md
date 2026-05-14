---
id: ISS-20260514T103806044Z-VEC-PUSH-ERR-PAYLOAD-LOSES-VEC-AND-I-7E13FAD9
title: "Vec push Err payload loses Vec owner under Stage 6 owner-preserving update"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/list, stdlib/alloc/hash/sha256, stdlib/std/fs, stdlib/neplg2/core, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T103806044Z-VEC-PUSH-ERR-PAYLOAD-LOSES-VEC-AND-I-7E13FAD9: Vec push Err payload loses Vec owner under Stage 6 owner-preserving update

## 概要

Vec.push consumes Vec<T> but returned Result<Vec<T>, StdErrorKind>. On allocation or invalid state failure the caller could not recover the consumed Vec owner, so the API contract hid ownership transfer in implementation discipline instead of expressing it in the type.

## 対象

- `stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/list, stdlib/alloc/hash/sha256, stdlib/std/fs, stdlib/neplg2/core, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- Stage 6 の設計では、fallible update の成功/失敗どちらでも owner の行方が型で表される必要がある。
- `Vec.push<T: Copy>` は item を Copy payload として扱うため、現段階の Err payload が返すべき linear owner は `Vec<T>` である。
- 将来 `T: Copy` 制約を外す場合は、`OwnedBuffer<T>` / initialized prefix / moved state と合わせて item owner を Err payload に載せる別設計が必要になる。

## 問題

Vec.push consumes Vec<T> but returned Result<Vec<T>, StdErrorKind>. On allocation or invalid state failure the caller could not recover the consumed Vec owner, so the API contract hid ownership transfer in implementation discipline instead of expressing it in the type. Existing caller code also assumed failure consumed or freed the owner implicitly, which is incompatible with ResourceIR proving owner transfer from source structure.

## 影響

Stage 6 static safety cannot prove fallible collection updates by type. The old API encouraged freeing or dropping owners inside `push` and made higher-level APIs silently rely on implementation discipline instead of explicit owner flow.

## 修正方針

Introduce a named `VecPushError<T>` payload containing the recovered `Vec<T>` owner and `StdErrorKind`. Change `push<T: Copy>` to return `Result<Vec<T>, VecPushError<T>>` and ensure every Err path returns the input Vec owner instead of freeing old storage. Update caller code so APIs that return the collection keep the recovered owner, and APIs that collapse the error to `Diag` / errno / `StdErrorKind` explicitly free the recovered owner before returning. Update docs, source policy, and focused doctests.

## 対応

- `VecPushError<T>` を追加し、`push<T: Copy>` の失敗 payload が `Vec<T>` owner と `StdErrorKind` を明示的に返す設計に変更した。
- grow helper は `vec_realloc_region_or_keep` に改め、realloc 失敗時に旧 `RegionToken<T>` を解放せず error payload に戻すようにした。
- list、fs、diag、kpgraph、TUI、selfhost core、SHA-256 の `push` 呼び出しを更新し、失敗時に返った `Vec` owner を継続利用するか、外側 API が error kind だけを返す場合は明示的に free するようにした。
- SHA-256 streaming update は `Sha256UpdateError` に state owner を戻す形へ変更し、caller が failure cleanup を選べるようにした。
- ResourceIR 側は owner return summary の比較を canonical 化し、owner summary update が field/fact order や nested condition の並びで不要に振動しないようにした。
- i32 alias condition query に active guard と memo を入れ、ResourceIR の条件導出が循環関係で停止性を失わないようにした。

## 検証

- `cargo fmt --package nepl-core --check`: passed
- `cargo test -p nepl-core resource::initialized_alias_i32_condition_tests --lib`: passed
- `cargo test -p nepl-core resource::owner_summary_update --lib`: passed
- `trunk build`: passed
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree -o tmp/agent1-vec-push-owner-error-push-final.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/agent1-vec-push-owner-error-vec-md-final.json -j 1 --dist web/dist --assert-io`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/tests/list.n.md --no-tree -o tmp/agent1-vec-push-owner-error-list-md-final.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i tests/stdlib/list_collections.n.md --no-tree -o tmp/agent1-vec-push-owner-error-list-collections-final.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md --no-tree -o tmp/agent1-vec-push-owner-error-hash-md-final.json -j 1 --dist web/dist --assert-io`: total=1, passed=1

## 関連観測

- `tests/stdlib/fs.n.md` の focused run は 5/7 passed。`doctest#4` の direct raw store は現在の raw-memory boundary 方針と衝突し、`doctest#5` は `fs_path_filetype` の normalized builder owner flow で `resource.cell.moved` / `resource.cell.uninit` が再発したため、既存 issue `ISS-20260505T021408593Z-FS-PATH-FILETYPE-LEAKS-NORMALIZED-ST-2B0962CF` を再オープンして追跡する。
- `tests/stdlib/neplg2_type_arena.n.md` は 0/5 で、`SelfhostTypeRecord` enum payload 分割後も doctest が旧 `SelfhostTypeKind::*` を `selfhost_type_arena_add_primitive` に渡しているため、新規 issue `ISS-20260514T123100000Z-SELFHOST-TYPE-ARENA-DOCTESTS-USE-OLD-PRIMITIVE-4C60C45A` で分離する。
- `tests/stdlib/neplg2_module_graph.n.md` は current `web/dist` で 3/3 compile timeout。既存 timeout issue `ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D` を再オープンして、lexer/parser/loader 側の静的検査コスト再発として扱う。
