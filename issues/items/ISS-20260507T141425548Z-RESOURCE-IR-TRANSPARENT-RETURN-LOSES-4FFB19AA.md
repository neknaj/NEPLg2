---
id: ISS-20260507T141425548Z-RESOURCE-IR-TRANSPARENT-RETURN-LOSES-4FFB19AA
title: "Resource IR transparent return region_ptr non-owning view lacked regression coverage"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/lower_raw_address_return.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T141425548Z-RESOURCE-IR-TRANSPARENT-RETURN-LOSES-4FFB19AA: Resource IR transparent return region_ptr non-owning view lacked regression coverage

## 概要

`region_ptr &token` を user helper が返したあと、caller がその `MemPtr` を `region_new` へ渡す経路について、full compiler gate が `resource.owner.no_free_obligation` まで到達することを直接固定する regression がなかった。調査中に、helper が `&RegionToken` parameter をそのまま `region_ptr` へ渡す場合、Resource IR lowering coverage が HIR 側の reference projection count とずれて `resource.lower.incomplete` で先に停止することも確認した。

## 対象

- `nepl-core/src/resource/lower_raw_address_return.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `MemPtr` は non-owning pointer、`RegionToken` は free obligation owner へ分離する方針であり、borrowed pointer projection は helper / return boundary を越えても owner token へ昇格してはならない。
- 2026-05-07 の調査では、owner checker 単体はこの経路を既に拒否していた。一方で full compile pipeline では、`region_ptr token` の HIR coverage が `&token` と `token: &RegionToken` を同じ reference projection として扱えず、`resource.lower.incomplete` が先に発生した。
- `region_ptr &token` は `AddrOf` から実体 local を得られるため追加 deref projection ではない。一方、helper parameter `token: &RegionToken` をそのまま projection する `region_ptr token` は Resource IR 上で `Deref` projection を使うので、HIR coverage も同じ semantic deref として数える必要がある。
- Stage 4 の owner/provenance 分離では、lowering coverage が先に誤停止すると owner diagnostic の regression を固定できず、後続 refactor で non-owning marker の欠落を見逃しやすい。

## 問題

`region_ptr` の helper return 経路は、`RegionToken` の正規 owner consumption と見た目が近いため、Resource IR lowering / owner summary の責務分割中に non-owning marker が落ちやすい。既存テストは「borrowed pointer で store/load した後に元 token を dealloc できる」ことは確認していたが、「borrowed pointer を `region_new` で forged token にして free することは拒否される」ことを full compiler pipeline では直接確認していなかった。

## 影響

回帰が入った場合、borrowed `RegionToken` projection が helper return boundary を越えたところで free obligation owner のように扱われ、`MemPtr = non-owning pointer` / `RegionToken = owner token` の分離が崩れる。これは Stage 4 の Resource IR owner/provenance 分離に対する memory safety regression になる。

## 修正方針

`region_ptr` の reference projection を HIR coverage 側でも型付きに数え、`AddrOf` で実体 local へ解決できる場合は deref projection として数えない。これにより Resource IR coverage と lowering の責務境界を一致させたうえで、Resource IR unit regression と end-to-end stdlib memory safety doctest の両方で `resource.owner.no_free_obligation` まで到達することを固定する。

## 対応

- `nepl-core/src/resource/coverage_hir_projection.rs` で `region_ptr` を reference address projection として扱い、`AddrOf` 由来の直接 borrow は追加 deref として数えないようにした。
- `nepl-core/tests/resource_ir.rs` に `resource_ir_owner_check_rejects_region_token_forged_from_region_ptr_helper` を追加し、owner checker 単体と full compiler gate の両方を確認した。
- `tests/stdlib/memory_safety.n.md` に `helper が返した region_ptr は owner token にできない` の compile_fail doctest を追加した。
- どちらも expected diagnostic は `resource.owner.no_free_obligation` とし、borrowed pointer を forged `RegionToken` owner として扱わないことを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_region_ptr_helper -- --nocapture`
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-region-ptr-helper-forge.json -j 1 --dist web/dist`
