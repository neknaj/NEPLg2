---
id: ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB
title: "Fallible owning collection updates lose input owners on allocation failure"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/collections/list.nepl, stdlib/alloc/collections/hashmap.nepl, nodesrc/test_stdlib_hashmap_owner_contract.js, stdlib/tests/hashmap.n.md, stdlib/tests/hashmap_str.n.md, tests/stdlib/hash_collection_rehash.n.md, tests/stdlib/pipe_collections.n.md, tests/stdlib/traits_hash.n.md, tests/stdlib/selfhost_req.n.md, tests/stdlib/collections_diag.n.md"
---

# ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB: Fallible owning collection updates lose input owners on allocation failure

## 概要

Resource IR owner checking now tracks raw owners stored inside collection backing memory. list cons/push style APIs and HashMap grow/new paths can consume an owning collection or backing entries, then return Err without returning or freeing all transferred owners on every failure path.

## 対象

- `stdlib/alloc/collections/list.nepl, stdlib/alloc/collections/hashmap.nepl`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node -- --nocapture` は pass。Resource IR は raw node field に格納した tail owner を `load_i32` で取り出し、tail と新 node の両方を free できる。
- `cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture` は `cons__T_List_T_T__Result_T_E_List_T_T_Diag__imp_i32` の `Result::Ok(List).ptr` owner leak を報告する。allocation failure path が入力 tail owner を返すか解放する契約になっていない。
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` は `hashmap_new_with_capacity...` の header/entries owner と `hashmap_rehash_to...` の backing entries owner leak を報告する。HashMap grow/new path が途中で獲得または移動した backing owner を全 failure path で閉じていない。
- `cargo test -p nepl-core --test neplg2 llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture` も同じ HashMap backing owner contract に依存して失敗する。
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check。

## 問題

Resource IR owner checking now tracks raw owners stored inside collection backing memory. list cons/push style APIs and HashMap grow/new paths can consume an owning collection or backing entries, then return Err without returning or freeing all transferred owners on every failure path.

## 影響

Strict memory safety gates cannot pass for List/HashMap integration tests without either weakening owner checking or fixing the stdlib API/implementation. The issue is a real ownership contract problem, not a Resource IR false positive.

## 修正方針

Redesign fallible owning collection APIs so failure paths either preserve and return the original owner, or fully release/roll back every owner consumed before returning Err. Add focused tests for allocation failure paths and successful free traversal.

## 検証

Run focused List/HashMap neplg2 tests and Resource IR owner regressions after the stdlib ownership contract is fixed.

## 2026-04-30 状況更新

`list_get_out_of_bounds_err` の失敗は、List の allocation failure rollback ではなく、`unwrap_ok` / `uwok` の `Err => unreachable` 経路で owned `Result::Err` payload が caller に残る Resource owner summary 問題だった。これは `ISS-20260429T170036695Z-RESOURCE-OWNER-SUMMARY-KEEPS-OWNED-E-BFDE4F98` として core 側で修正済みで、`cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture` は pass する。

この issue の残件は HashMap 側に集中している。`hashmap_rehash_to` の `hdr + 8` entries owner、`insert` の local `entries` owner、main の `map1` / `hms` / `hmk` header と entries owner leak は引き続き再現する。次の修正では HashMap grow / rehash / insert / free の owner contract を、Resource IR owner gate を弱めずに整理する。

## 2026-04-30 状況更新 2

`hashmap_rehash_to` の `hdr + 8` entries owner leak は、stdlib の rollback 契約ではなく core Resource owner checker の raw address alias 継承漏れだった。`ISS-20260429T172032098Z-RESOURCE-OWNER-CHECKER-LEAVES-RAW-CE-8EA40ADE` で修正済み。

この issue の残件は以下に絞られた。

- `insert...` の local `entries` owner may leak。
- `main` 側の `map1` / `hms` / `hmk` header と entries owner leak。

特に caller-side leak は、HashMap を作成して `get` / `len` / `contains` で参照した後に `free` しない fixture/API 契約の問題であり、次の修正では `HashMap` の read API と test fixture の ownership contract を整理する。

## 2026-04-30 状況更新 3

`insert...` の local `entries` owner may leak は、`load_i32 add hdr 8` が backing entries の所有権移動として扱われていた core Resource owner checker 問題だった。`ISS-20260429T173344520Z-RESOURCE-OWNER-CHECKER-MOVES-RAW-ADD-D665B59D` で修正済み。

この修正後、HashMap focused tests の残件は以下に更新された。

- `insert...` の `hdr.StorageOffset(8).Deref` owner may leak。
- caller 側の `map1` / `hms` / `hmk` header と entries owner leak。

次は `insert` 内で helper call / loop / raw slot address 計算を通った後も、entries owner cell が `ready.field0 + 8` 配下に残るべきか、あるいは stdlib API が明示的な borrow/view helper を必要とするかを切り分ける。

## 2026-04-30 状況更新 4

`insert...` の `hdr.StorageOffset(8).Deref` owner may leak は、core Resource owner checker が raw address alias graph と owner table を別々に merge し、さらに projected alias copy 時に基底 pointer alias を落としていた問題だった。`ISS-20260429T174311311Z-RESOURCE-OWNER-CHECKER-LOSES-AGGREGA-8E245CC4` で修正済み。

この修正後、`cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` は `insert...` 内部では失敗しなくなり、残件は `main` 側の `map1` header / entries owner leak に絞られた。これは HashMap を `get` / `contains` / `len` などの read API に渡した後、所有者を返すのか明示的に `free` するのかが fixture/API 契約として閉じていない問題として扱う。

## 2026-04-30 修正内容

HashMap の read/free API 契約を、ResourceIR が検査できる形に整理した。

- `HashMap<.K,.V,.H>` の layout を `hdr <MemPtr<u8>>`, `entries <MemPtr<u8>>`, `hasher <.H>` に変更し、entries owner を header raw cell に隠さないようにした。
- header は count/cap/tombstones の 12 byte metadata に縮小し、entries owner は struct field として保持する。
- `hashmap_alloc_entries` は `Result<MemPtr<u8>, Diag>` を返すようにし、constructor / rehash / insert / free が entries owner を明示的に移動できるようにした。
- `hashmap_rehash_to` は成功時に旧 entries を解放して新 entries owner を返却 HashMap へ移し、確保失敗時は消費済みの旧 entries と header を解放してから `Err` を返す。
- `get` / `contains` / `len` は `&HashMap` を受け取る read API に変更し、読み取りで map owner を消費しない契約にした。
- key/value の deep drop を行わない設計を明確化し、`.V: Copy` bound を public API に追加した。
- caller fixture は read 後に `free` を呼ぶように更新した。
- `collections_diag` の HashMap unexpected `Ok` branch は、返ってきた map owner を捨てずに `free` するようにした。
- `nodesrc/test_stdlib_hashmap_owner_contract.js` を追加し、header/entries owner 分離、borrow read API、rehash failure cleanup、free の owner 解放を source policy として固定した。

`tests/stdlib/collections_diag.n.md::doctest#1` では、HashMap remove の `Err(Diag)` を観測したあと `Diag` の `str` payload owner が残る別問題を発見した。これは HashMap owner contract ではなく diagnostic ownership contract の問題として `ISS-20260429T191522911Z-DIAG-BY-VALUE-OBSERVERS-LEAVE-OWNED--9A8FBE5F` に分離した。

## 検証結果

- `cargo test -p nepl-core --test neplg2 hashmap -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test overload -- --nocapture`: 8 passed
- `trunk build`: passed
- `node nodesrc/test_stdlib_hashmap_owner_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashmap_str.n.md --no-tree -o tmp/hashmap-owner-contract-stdlib-only.json -j 1 --dist web/dist`: total=11, passed=11
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 3 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 5 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 5 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 5 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 4 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 6 --dist web/dist`: passed

補足: `cargo test -p nepl-core --test selfhost_req req -- --nocapture` は既存の ByteBuf/String/FS owner 問題 3 件で fail したが、HashMap 関連の `test_req_string_map` と `test_req_trait_extensions` は pass した。
