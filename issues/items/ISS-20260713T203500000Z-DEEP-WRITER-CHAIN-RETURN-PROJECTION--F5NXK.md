---
id: ISS-20260713T203500000Z-DEEP-WRITER-CHAIN-RETURN-PROJECTION--F5NXK
title: "Deep registered writer chain return projection reuses moved payload"
area: RESOURCE
status: verified
resolved: true
priority: P1
type: bug
created: 2026-07-13
updated: 2026-07-13
target: nepl-core/src/resource/owner_variant.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/gui_font_registered_face.n.md
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

- outer Result armを適用するとき、nested error variantsのreturn targetは異なるenum payload projectionとして同時に列挙される。
- それらのtargetは実行時には相互排他的だが、同じcanonical source storageを返すmappingを持つ。
- `apply_match_arm_returns`が各targetへ通常transferしたため、最初のtargetでsourceをMovedにし、2件目以降を正当な条件付きreturnではなく二重moveとして診断した。

修正境界は`owner_variant.rs`のmatch-arm return適用である。同じcanonical sourceかつ最初に異なるtarget projectionが別enum payloadである場合だけ、最初にmaterializeしたtargetのowner state/storage、raw alias/view、storage originを相互排他的targetへ条件付き複製する。既存target state、非transferable source、同一または非排他的targetは従来のtransferと診断経路へ戻し、Moved sourceの一般無視は行わない。

# 再開条件

- genuine double move negative controlを維持する。
- 同じBudgetStepからownerを2回take、既存double-take retry、非排他的同一target候補、Err cleanup欠落をnegative controlとして維持する。
- F5nxj controlled 8-command fixtureがnormal Resource checkを通る。
- F5nxkでcommand index 0と4のindexed span、index -1、typed span lookup failure、identity mismatch、single freeを実行して1 / 1通過する。

# Compiler checkpoint

production suffix depthと同じ5 owner leafを持つbudget Resultを再帰的に連結する回帰を追加した。outer Result variantを選択した時点では、nested error variantsのtarget projectionは相互排他的だが同じsource storageを返す。従来は各targetへ通常transferして2件目以降をMovedとした。

match arm内で最初にmaterializeしたtargetのowner state、raw alias/view、storage originを記録し、同じsourceかつtarget suffixが異なるenum payloadで排他的な場合だけ、同じstorage identityを条件付きtargetへ複製する。targetに既存stateがある場合やsourceがtransferableでない場合はshortcutせず従来の診断経路へ戻す。一般的なMoved無視や非排他的targetの抑制は行わない。

synthetic production-depth回帰、既存deep owner-summary回帰、same/different source、same/exclusive target、pre-owned Live/Moved targetのfocused testsは通過した。

# Production runtime confirmation

最初のtargetを探す適用履歴を全source共通の線形列として保持すると、巨大なregistered-face callerでresource static checkが300秒を超えた。履歴をsource別`BTreeMap<Place, Vec<Place>>`へ変更し、異なるsourceのtargetを相互排他候補として走査しないようにした。

controlled fixtureはF5nxj plan/allocation、8回のwriter commit、budget 0 terminal、checked sealを通り、F5nxk ownerへ移してcommand index 0 / 4のchecked commandとindexed span identity、index -1、typed span lookup failure、identity mismatchを借用で検査した後、sourceとwriter storageをexactly once解放する。trunk後のfocused doctest通過を最終gateとする。compiler correctnessとproduction runtime再現の両方が復帰したため、このblocker issueをresolvedとする。
