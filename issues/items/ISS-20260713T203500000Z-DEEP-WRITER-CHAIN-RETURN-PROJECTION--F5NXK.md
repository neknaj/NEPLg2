---
id: ISS-20260713T203500000Z-DEEP-WRITER-CHAIN-RETURN-PROJECTION--F5NXK
title: "Deep registered writer chain return projection reuses moved payload"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-13
updated: 2026-07-13
target: nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_variant_apply.rs
---

# 概要

F5nxj production writerを8回順次commitしてF5nxkへ渡すvalid callerが、各`Result<BudgetStep, StepError>`から回収済みowner leafをfunction return時に再利用したとして`resource.owner.use_after_move`で拒否される。

# 再現条件

再現sourceは`issues/repro/ISS-20260713T203500000Z-deep-writer-chain-return-projection.nepl`に隔離している。通常test scanには入れず、compiler修正時にproduction型をimportしたfocused compileへ使用する。

- 入力は`GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandSinkWritingOwner`。
- `gui_font_registered_face_simple_glyph_indexed_path_command_sink_writer_step_budget(owner, 1)`を8回順次適用する。
- 各Okはstatusをborrowした後、`budget_step_owner`で次ownerをexactly once回収する。
- Errは`step_error_free`でexactly once解放する。
- 最後にbudget 0のCompleted、checked seal、F5nxk start/read/freeへ進む。
- recursive drain、8個の非再帰helper、単一関数8段unrollの全てで同じdeep source 3 leaf / writer 2 leaf projection群が`ReturnValue ... found Moved`となる。

# 既存issueとの差分

`ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D`は同一parameter enumの別payloadが同一targetへ合流する代替mappingを保持した。本件はsuccessive Ok ownerを次のcallへ消費した後、boolを返すcallerで一時Result owner leafがReturnValue候補として残る。別variant retryだけでなく、連続success transferのsummary compositionまたはreturn target pruningを調べる必要がある。

# 根本原因境界

- match bindでResult payloadのpending owner effectsがbind localへ複製される。
- `budget_step_owner step`が実ownerを次ownerへ移しても、bind local側の条件付きreturn alternativeが残る。
- arm終端の`resolve_result`は元scrutineeを解決するが、bind local側の残存候補を全て消費しない。
- apply側の`should_skip_unavailable_alternative`はenum projectionの相互排他だけを見て、generation側が保持した`source_condition`を判定しない。
- このため既にMovedのTemporary Result leafが後段control outputを通ってbool returnまで残る。

修正境界は`owner_control.rs`のmatch bind lifecycleと`owner_variant.rs`の`materialize_return_owner_for_target` / `retain_unmaterialized_sources` / availability判定である。target projectionが具体化・移送済みのとき、そのtargetに対応するpending return alternativeだけを`source_condition`込みでpruneする。Moved sourceの一般無視は禁止する。

# 再開条件

- genuine double move negative controlを維持する。
- 同じBudgetStepからownerを2回take、既存double-take retry、非排他的同一target候補、Err cleanup欠落をnegative controlとして維持する。
- F5nxj controlled 8-command fixtureがnormal Resource checkを通る。
- F5nxkでcommand index 0と4のindexed span、index -1、typed span lookup failure、identity mismatch、single freeを実行して1 / 1通過する。
