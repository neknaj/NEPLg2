# NEPLg2 静的検査の複雑化解消計画

作成日: 2026-04-28

## 目的

NEPLg2 の Rust compiler は、型検査、effect 判定、move/borrow/lifetime、drop 挿入、raw memory provenance を後付けで積み重ねてきた。その結果、`typecheck.rs` と `passes/move_check.rs` が巨大化し、修正ごとに局所的な summary や alias map を増やす構造になっている。

この文書は、静的検査を弱めずに、不必要な複雑化を解消するための大規模修正の仕様と実装計画を定める。目標は「検査を形だけ残す」ことではなく、memory safety、type safety、effect safety を compiler が一貫した中間表現で検査できる状態にすることである。

関連 issue:

- [ISS-20260425T000000Z-RV-CORE-002-D17C4B3C](../../issues/items/ISS-20260425T000000Z-RV-CORE-002-D17C4B3C.md): `typecheck.rs` / `move_check.rs` の責務集中。
- [ISS-20260425T000000Z-RV-CORE-009-58589A3F](../../issues/items/ISS-20260425T000000Z-RV-CORE-009-58589A3F.md): Resource IR 不在による move/borrow/drop の後付け実装。
- [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](../../issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md): raw memory operation の effect / ownership 境界。
- [ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF](../../issues/items/ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md): `MemPtr` / `RegionToken` の provenance / owner model。
- [ISS-20260514T054314434Z-COPY-IMPL-CAN-MARK-COMPILER-OWNER-TO-D6C08048](../../issues/items/ISS-20260514T054314434Z-COPY-IMPL-CAN-MARK-COMPILER-OWNER-TO-D6C08048.md): compiler owner token への `Copy` capability impl を typecheck boundary で拒否する。
- [ISS-20260514T164856024Z-OWNER-BACKED-AGGREGATE-CONSTRUCTORS--61400B84](../../issues/items/ISS-20260514T164856024Z-OWNER-BACKED-AGGREGATE-CONSTRUCTORS--61400B84.md): compiler owner token を直接 field に持つ aggregate の constructor / field projection を compiler owner aggregate boundary で制限する。
- [ISS-20260514T230404748Z-OWNER-BACKED-AGGREGATE-POLICY-DOES-N-7D995A6B](../../issues/items/ISS-20260514T230404748Z-OWNER-BACKED-AGGREGATE-POLICY-DOES-N-7D995A6B.md): owner-backed aggregate constructor policy を nested owner field へ伝播させる。
- [ISS-20260514T231627302Z-OWNER-BACKED-AGGREGATE-FIELD-PROJECT-290DED97](../../issues/items/ISS-20260514T231627302Z-OWNER-BACKED-AGGREGATE-FIELD-PROJECT-290DED97.md): owner-backed aggregate field projection を compiler owner aggregate boundary で制限する。
- [ISS-20260514T233136936Z-GENERIC-OWNER-BACKED-AGGREGATE-CONST-6E024598](../../issues/items/ISS-20260514T233136936Z-GENERIC-OWNER-BACKED-AGGREGATE-CONST-6E024598.md): generic type application 後に owner-backed になる aggregate constructor を compiler owner aggregate boundary で制限する。
- [ISS-20260515T020307026Z-OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI-91ECE78D](../../issues/items/ISS-20260515T020307026Z-OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI-91ECE78D.md): owner aggregate constructor capability を constructor 名ごとの source proof に狭める。
- [ISS-20260515T023829013Z-CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO-EC25363F](../../issues/items/ISS-20260515T023829013Z-CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO-EC25363F.md): checked owner helper 呼び出しを raw memory boundary evidence として扱わない。
- [ISS-20260515T024851827Z-RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W-2A06192D](../../issues/items/ISS-20260515T024851827Z-RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W-2A06192D.md): raw memory operation / raw body operation の source capability を enum operation 単位に狭める。
- [ISS-20260515T110646911Z-CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK-B846CF4C](../../issues/items/ISS-20260515T110646911Z-CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK-B846CF4C.md): checked `MemPtr` 証明で `RegionToken` 返り値 provenance を internal summary に保持する。
- [ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1](../../issues/items/ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1.md): `RegionToken<T>` から最後の `MemPtr<T>` owner-like field を削除し、Resource IR summary を direct raw owner field に合わせる。
- [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](../../issues/items/ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md): public `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` API が `MemPtr<T>` に free obligation を残す Stage 6 残件。2026-05-16 に fixed。
- [ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44](../../issues/items/ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44.md): `core/mem` safe facade が低レベル `alloc_ptr` owner wrapper を再公開する Stage 6 子 issue。2026-05-16 に fixed。
- [ISS-20260515T171022724Z-STDIO-PRINT-BYTE-SCRATCH-STILL-USES--2C662B42](../../issues/items/ISS-20260515T171022724Z-STDIO-PRINT-BYTE-SCRATCH-STILL-USES--2C662B42.md): `std/stdio/write/byte.nepl` の 1 byte scratch buffer が `MemPtr<u8>` owner API に依存する Stage 6 子 issue。
- [ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA](../../issues/items/ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA.md): `std/stdio/write/fd.nepl` の fd_write iovec / nwritten scratch が `MemPtr<u8>` owner API に依存する Stage 6 子 issue。
- [ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719](../../issues/items/ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719.md): `std/stdio/read` の read_all / read_line buffer と fd_read scratch が `MemPtr<u8>` owner API に依存する Stage 6 子 issue。
- [ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134](../../issues/items/ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134.md): `VecStorageState` と split `RegionToken<T>` field の相関を型で証明できなかった Stage 6 issue。2026-05-16 に fixed。
- [ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535](../../issues/items/ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535.md): stdio read owner 移行後に残っていた ByteBuf-to-str / `string_from_mem_unchecked_result` owner transfer 境界。2026-05-16 に fixed。
- [ISS-20260515T181501164Z-STD-FS-FD-WRITE-SCRATCH-STILL-USES-M-42F15E1B](../../issues/items/ISS-20260515T181501164Z-STD-FS-FD-WRITE-SCRATCH-STILL-USES-M-42F15E1B.md): `std/fs/write/fd.nepl` の fd_write iovec / nwritten scratch が `MemPtr<u8>` owner API に依存する Stage 6 子 issue。2026-05-16 に fixed。
- [ISS-20260515T182041827Z-STD-FS-OPEN-FD-OUT-SCRATCH-STILL-USE-7C3B2667](../../issues/items/ISS-20260515T182041827Z-STD-FS-OPEN-FD-OUT-SCRATCH-STILL-USE-7C3B2667.md): `std/fs/fd.nepl` の path_open fd_out scratch が `MemPtr<u8>` owner API に依存する Stage 6 子 issue。2026-05-16 に fixed。
- [ISS-20260515T182445783Z-STD-FS-STAT-BUFFER-STILL-USES-MEMPTR-DF3210E8](../../issues/items/ISS-20260515T182445783Z-STD-FS-STAT-BUFFER-STILL-USES-MEMPTR-DF3210E8.md): `std/fs/stat.nepl` の path_filestat_get out-buffer scratch が `MemPtr<u8>` owner API に依存する Stage 6 子 issue。2026-05-16 に fixed。
- [ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD](../../issues/items/ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD.md): `Result<Vec<T>, E>` owner return を raw identity escape と誤診断する Stage 6 core false positive。2026-05-16 に fixed。
- [ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4](../../issues/items/ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4.md): raw identity 修正後に露出した `std/fs/path/normalize` の `Result<Vec<T>, E>` / `Result<StringBuilder, E>` owner summary leak。2026-05-16 に fixed。
- [ISS-20260516T010329239Z-RESOURCE-PROOF-PRIMITIVE-CLASSIFICAT-12B44B46](../../issues/items/ISS-20260516T010329239Z-RESOURCE-PROOF-PRIMITIVE-CLASSIFICAT-12B44B46.md): Resource proof primitive classification を typed registry に集約し、Resource IR / source capability の direct string 判定を減らす Stage 6 compiler-core issue。2026-05-16 に fixed。
- [ISS-20260516T020917148Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-ACCE-5E35D33F](../../issues/items/ISS-20260516T020917148Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-ACCE-5E35D33F.md): owner aggregate constructor source evidence が call-head 以外の大文字 symbol や same-module enum variant を構築子証拠として扱う Stage 6 compiler-core issue。2026-05-16 に fixed。
- [ISS-20260516T021926423Z-RAW-MEMORY-SOURCE-EVIDENCE-ACCEPTS-N-88427FD2](../../issues/items/ISS-20260516T021926423Z-RAW-MEMORY-SOURCE-EVIDENCE-ACCEPTS-N-88427FD2.md): raw memory source evidence が call-head 以外の raw helper symbol を raw operation / structural boundary 証拠として扱う Stage 6 compiler-core issue。2026-05-16 に fixed。
- [ISS-20260516T022823182Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-MISS-2CBBEB43](../../issues/items/ISS-20260516T022823182Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-MISS-2CBBEB43.md): owner aggregate source evidence が `let` / type annotation 後の prefix initializer call-head を見落とす Stage 6 compiler-core issue。2026-05-16 に fixed。
- [ISS-20260516T024129752Z-OWNER-AGGREGATE-FIELD-EVIDENCE-ACCEP-22239551](../../issues/items/ISS-20260516T024129752Z-OWNER-AGGREGATE-FIELD-EVIDENCE-ACCEP-22239551.md): owner aggregate field evidence が unrelated `get` helper 名だけで field boundary を許可する Stage 6 compiler-core issue。2026-05-16 に fixed。
- [ISS-20260516T025931471Z-WINDOWS-STDLIB-PATH-CANONICALIZATION-5C6E2D4E](../../issues/items/ISS-20260516T025931471Z-WINDOWS-STDLIB-PATH-CANONICALIZATION-5C6E2D4E.md): Windows の stdlib path canonicalization 差異で仮想 stdlib source の SourceCapabilities が落ちる Stage 6 compiler-core issue。2026-05-16 に fixed。
- [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](../../issues/items/ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md): stdlib raw-memory-backed API の段階移行。
- [ISS-20260514T234418963Z-HASHMAP-AND-HASHSET-ROOT-FACADES-RE--68724B49](../../issues/items/ISS-20260514T234418963Z-HASHMAP-AND-HASHSET-ROOT-FACADES-RE--68724B49.md): HashMap / HashSet root facade から internal storage/probe/rehash helper を閉じる。
- [ISS-20260515T000336641Z-VEC-STORAGE-FACADE-RE-EXPORTS-ALLOCA-4F004371](../../issues/items/ISS-20260515T000336641Z-VEC-STORAGE-FACADE-RE-EXPORTS-ALLOCA-4F004371.md): Vec storage facade から allocation / cleanup internal helper を閉じる。
- [ISS-20260515T002636772Z-ALLOC-STRING-FACADE-SOURCE-POLICY-ST-1530FB1C](../../issues/items/ISS-20260515T002636772Z-ALLOC-STRING-FACADE-SOURCE-POLICY-ST-1530FB1C.md): alloc/string root が raw storage / UTF-8 helper を再公開しない Stage 6 source policy へ更新する。
- [ISS-20260515T003514038Z-VEC-SORT-MERGE-SOURCE-POLICY-STILL-E-BD811427](../../issues/items/ISS-20260515T003514038Z-VEC-SORT-MERGE-SOURCE-POLICY-STILL-E-BD811427.md): Vec sort/merge facade が raw merge helper を再公開しない Stage 6 source policy へ更新する。
- [ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF](../../issues/items/ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF.md): `VecDataLen` raw storage view carrier を削除し、`MemPtr` owner-like field baseline を下げる。
- [ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F](../../issues/items/ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F.md): `StringBuilder` 固有の raw `MemPtr` owner state を `ByteBuilder` owner boundary へ集約する。
- [ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159](../../issues/items/ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159.md): `ByteBuf` / `ByteBuilder` の raw `MemPtr` owner field を `RegionToken` owner boundary へ集約する。
- [ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A](../../issues/items/ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A.md): `Vec.data` raw `MemPtr` owner field を廃止し、backing storage owner を `RegionToken<T>` field へ集約する。
- [ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E](../../issues/items/ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E.md): `ByteBuilder` grow failure cleanup の `unreachable` を廃止し、`RegionToken<u8>` owner を直接消費する。
- [ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F](../../issues/items/ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F.md): `Vec` / `ByteBuilder` の `RegionToken` realloc を core/mem owner-preserving helper へ集約し、Vec capacity grow の overflow proof を追加する。
- [ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660](../../issues/items/ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660.md): `sort_merge_ret<T>` の失敗 payload に `Vec<T>` owner を戻し、scratch buffer を `RegionToken<T>` owner へ移す。
- [ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D](../../issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md): Resource IR / self-host model に合わせた compiler diagnostic 再設計。
- [ISS-20260506T022705566Z-MOVE-CHECK-DOCTESTS-ARE-STALE-AFTER--EDD8402F](../../issues/items/ISS-20260506T022705566Z-MOVE-CHECK-DOCTESTS-ARE-STALE-AFTER--EDD8402F.md): Resource IR field projection / Never merge / move_check doctest authority の同期。
- [ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B](../../issues/items/ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B.md): monomorphized Resource IR function と source HIR function の対応を `origin_name` metadata で保持する。
- [ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013](../../issues/items/ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013.md): checked `ResourceDropElaborationPlan` を compiler pipeline artifact として保持する。
- [ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54](../../issues/items/ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54.md): checked drop plan が source HIR origin / binding / scope span へ戻せることを gate する。
- [ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744](../../issues/items/ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744.md): bare callable value reference を local read と誤認しない Resource IR lowering / coverage rule。
- [ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E](../../issues/items/ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E.md): assignment overwrite drop obligation を checked Resource IR drop elaboration plan に含める。
- [ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860](../../issues/items/ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860.md): checked ResourceDropElaborationPlan を実 drop call 生成で消費し、旧 HIR VarState drop walker を削除する。
- [ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C](../../issues/items/ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C.md): configured stdlib `alloc/string.nepl` と `alloc/string/storage.nepl` を exact raw-memory-boundary capability として扱う。
- [ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52](../../issues/items/ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52.md): raw-memory-backed scanner / byte helper の Stage 5 boundary と Stage 6 public API 移行を整理する。
- [ISS-20260514T183506445Z-STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B-C76C9E1E](../../issues/items/ISS-20260514T183506445Z-STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B-C76C9E1E.md): `std/env/cliarg` root facade と raw argv ABI boundary を分離する。
- [ISS-20260514T190700987Z-SELF-HOST-CLI-ARGS-PARSER-READS-VEC--5193AE7F](../../issues/items/ISS-20260514T190700987Z-SELF-HOST-CLI-ARGS-PARSER-READS-VEC--5193AE7F.md): self-host CLI args parser の `Vec<str>` read-only observer を raw storage 走査から public `Vec` API へ移す。
- [ISS-20260514T195643983Z-KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX-416FBAB5](../../issues/items/ISS-20260514T195643983Z-KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX-416FBAB5.md): `kp/kpprefix` の copyable raw prefix storage owner を `Vec<i32>` owner boundary へ移す。
- [ISS-20260514T200755109Z-KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O-953345F8](../../issues/items/ISS-20260514T200755109Z-KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O-953345F8.md): `kp/kpfenwick` / `kp/kpdsu` の public raw `i32` owner handle を typed collection owner boundary へ移す。
- [ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB](../../issues/items/ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB.md): `kp/kpgraph` の dense matrix raw pointer API を `AdjacencyMatrix` owner boundary へ移す。
- [ISS-20260515T004634650Z-KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E-1F2C94F7](../../issues/items/ISS-20260515T004634650Z-KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E-1F2C94F7.md): `kp/kpgraph` の unsafe-unwrap source policy を現行 typed owner BFS API に合わせる。
- [ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D](../../issues/items/ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D.md): `kp/kpsearch` の public Vec helper 実装を raw storage view から `Vec` API boundary へ移す。
- [ISS-20260514T051405052Z-STREAMSCANNER-HIDES-BUFFER-OWNER-BEH-0977B2E3](../../issues/items/ISS-20260514T051405052Z-STREAMSCANNER-HIDES-BUFFER-OWNER-BEH-0977B2E3.md): `StreamScanner.header` の raw `MemPtr` owner field を廃止し、ByteBuf owner と typed cursor storage へ分離する。
- [ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F](../../issues/items/ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F.md): fs / stdio read scratch owner cleanup の Resource IR owner summary。
- [ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8](../../issues/items/ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8.md): KP stream scanner float doctest runtime timeout。
- [ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA](../../issues/items/ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA.md): `unwrap_ok dealloc` 経由の checked raw owner consumption を Resource IR summary に反映する。
- [ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3](../../issues/items/ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3.md): `Result` payload owner を unconditional projection return として materialize せず variant owner return として扱う。
- [ISS-20260512T230732771Z-RESOURCE-OWNER-SUMMARY-TREATS-VARIAN-0C3269E3](../../issues/items/ISS-20260512T230732771Z-RESOURCE-OWNER-SUMMARY-TREATS-VARIAN-0C3269E3.md): Resource owner summary の variant path 条件を OR alternatives として保持し、到達可能 arm を誤って落とさない。
- [ISS-20260515T115250172Z-RESOURCE-OWNER-SUMMARY-LOSES-NESTED--28CFC4D8](../../issues/items/ISS-20260515T115250172Z-RESOURCE-OWNER-SUMMARY-LOSES-NESTED--28CFC4D8.md): helper summary が参照する nested `Result` payload owner を caller 側で materialize し、owner-preserving collection helper を Resource IR 上で証明する。
- [ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912](../../issues/items/ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912.md): `alloc/string/access.nepl` / `alloc/string/scanner.nepl` 分割後の exact raw-memory-boundary capability 追従。
- [ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71](../../issues/items/ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71.md): `alloc/string/integer.nepl` 分割後の exact raw-memory-boundary capability 追従。
- [ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4](../../issues/items/ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4.md): `alloc/string/builder.nepl` 分割後の exact raw-memory-boundary capability 追従。
- [ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232](../../issues/items/ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232.md): Resource IR authority path の deep-prefix compile-time budget 監査。
- [ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573](../../issues/items/ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573.md): Stage 6 の `core/mem` internal/public 分離の前提となる import visibility enforcement。
- [ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4](../../issues/items/ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4.md): fs / stdio private scratch dealloc が owner alias move 後の free obligation を失う。
- [ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1](../../issues/items/ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1.md): raw address lowering の return/source classification 責務が再集中している。
- [ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A](../../issues/items/ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A.md): initialized alias tracking module の raw alias group / value origin / i32 scalar fact 責務分割。

## 現状の問題

### 1. HIR 直走査に検査責務が集中している

現在の `move_check` は HIR tree を直接走査しながら、local variable state、borrow state、field move、raw memory place、function raw effect summary、enum payload alias、aggregate field alias を同時に扱っている。これにより、ある経路を塞ぐたびに別の容器や関数境界に対応する補助 map が増える。

この構造では、次の問いに単一の答えを持てない。

- この値は所有値か、borrow projection か、raw pointer か。
- この storage の free obligation は誰が持つか。
- この cell は initialized / moved / uninitialized のどれか。
- この borrow はどの resource id に紐づくか。
- この関数呼び出しは resource state をどう変化させるか。

### 2. `MemPtr` が複数の責務を兼ねている

`MemPtr<T>` は stdlib 上では Copy な non-owning pointer と説明されている。一方で、collection storage、single-cell owner、self-host outcome では owning storage handle としても使われている。

今後の方針は `MemPtr` を拡張し続けることではない。役割を次のように分ける。

| 役割 | 型・IR 表現 | 意味 |
|---|---|---|
| non-owning pointer | `MemPtr<T>` | Copy 可能な projection。free obligation を持たない。 |
| storage owner | `OwnedRegion<T>` / `Storage<T>` | allocator が発行した free obligation owner。Copy 不可。 |
| initialized cell | `InitializedCell<T>` / Resource IR cell state | 値が入っている、move 済み、drop obligation が残る、を表す。 |
| compiler capability | compiler-issued token | stdlib code から forge できない resource id / provenance。 |

### 3. effect が surface 表現しか持たない

現行の effect は主に `Pure` / `Impure` であり、raw allocation や internal buffer mutation を「外部から観測できない内部効果」として扱う層がない。そのため raw primitive を単純に impure 化すると stdlib が広範囲に壊れ、逆に pure のままにすると safe source から raw memory discipline を構成できる。

必要なのは、内部効果と surface effect の分離である。

| 内部 effect | surface fold | 条件 |
|---|---|---|
| `InternalAlloc` | `Pure` | raw identity / owner token が public surface へ漏れない。 |
| `UnsafeMemory` | fold 不可 | 明示 unsafe / compiler-owned boundary 内だけで許可する。 |
| `ExternalIO` | `Impure` | I/O など外部観測可能な効果。 |
| `Nondet` | `Impure` | 時刻、乱数、環境依存など。 |

### 4. drop 挿入と move check が別々に状態を推測している

`drop_insertion` は scope exit と structural field drop を見て drop を後付けする。`move_check` は別の走査で moved / borrowed / raw place state を推測する。この 2 つが同じ resource state を共有していないため、stdlib 側で drop loop を追加した場合に、将来の auto drop と衝突する危険がある。

## 目標仕様

### 検査の層

静的検査は次の依存方向に分ける。

| 層 | 責務 | 出力 |
|---|---|---|
| resolve | import、name、scope、overload candidate の収集 | resolved AST / symbol table |
| type inference | 型変数、trait capability、overload 決定 | typed HIR |
| effect inference | 関数 effect、internal effect、surface fold | effect signature |
| resource lowering | HIR から resource operation へ変換 | Resource IR |
| resource check | move、borrow、lifetime、initialized、drop obligation、raw provenance | checked Resource IR / diagnostics |
| drop elaboration | Resource IR 上で auto drop を挿入 | drop-elaborated Resource IR |
| backend lowering | WASM / LLVM 用 HIR または backend IR へ変換 | backend input |

後段は前段の内部実装へ戻ってはならない。特に resource check は `typecheck.rs` の local helper や HIR の表面的な call name 推測に依存しない。

### Resource IR の最小モデル

Resource IR は CFG を持つ中間表現とし、少なくとも次を第一級に表す。

| 要素 | 説明 |
|---|---|
| `ResourceId` | 所有値、storage owner、borrow target を識別する compiler-owned id。 |
| `Place` | local、field、enum payload、tuple field、storage offset、projection を表す。 |
| `StorageId` | allocator が発行した storage。byte range と layout plan を持つ。 |
| `CellState` | `Uninit` / `Initialized(T)` / `Moved` / `Dropped` / `MaybeMoved`。 |
| `OwnerState` | free obligation の有無、owner token の移動状態。 |
| `BorrowState` | shared borrow set、unique borrow、borrow lifetime end。 |
| `PointerProvenance` | `MemPtr` projection の base resource、offset、unknown-offset 保守情報。 |
| `EffectOp` | internal allocation、unsafe memory、external I/O、user call の resource effect。 |

### `MemPtr` / `Storage` / `InitializedCell` の規則

1. `MemPtr<T>` は non-owning pointer であり、Copy できる。
2. `MemPtr<T>` の copy は free obligation を複製しない。
3. `Storage<T>` / `OwnedRegion<T>` は Copy 不可であり、free obligation を持つ。
4. `Storage<T>` の projection から `MemPtr<T>` を作れるが、`MemPtr<T>` から `Storage<T>` は作れない。
5. initialized value を持つ cell を storage-only free することは禁止する。
6. `load<T>` は `T: Copy` の read と、non-Copy の move-out を分ける。
7. `store<T>` は uninitialized cell の initialize と、initialized cell の overwrite を分ける。non-Copy overwrite は既存 value の drop/consume が証明された場合だけ許可する。
8. raw address `i32` は compiler-owned internal boundary 外へ出さない。移行中は既存 API を `resource.cell.*` / `resource.owner.*` / `resource.raw.*` / `effect.*` 系の検査で保守的に塞ぎ、cell state と owner obligation の原因分類を混ぜない。

### function effect と resource summary

Resource IR 導入後の関数 summary は、現行の raw alias summary の延長ではなく、関数境界をまたぐ resource effect として表す。

- 引数 resource の consume / borrow / projection。
- 戻り値 resource の owner transfer / borrowed projection / copy value。
- storage cell の initialized / moved / dropped 変化。
- `InternalAlloc` が外部へ漏れたかどうか。
- unknown callback は保守的に effect set を上げる。

function value、enum payload、aggregate field、branch merge を別々の alias map で扱わず、Resource IR の `Place` と `EffectOp` に統合する。

## 実装計画

### Stage 0: 現状固定と回帰境界の明確化

目的: 大規模修正中に安全検査を弱めないため、既存の暫定防壁を固定する。

作業:

- `tests/compiler/move_effect.n.md` の raw ownership / raw effect regression を現行 baseline として維持する。
- raw memory / borrow / function effect 関連の compile_fail に `diag_code` を可能な範囲で固定する。
- `node nodesrc/issues.js check` と focused compiler test を CI / local の確認手順へ明記する。

commit 単位:

1. test naming と出力 JSON baseline の整理。
2. Resource IR 導入前提の regression 一覧更新。

### Stage 1: module 境界の切り出し

目的: behavior を変えず、`typecheck.rs` と `move_check.rs` の責務境界を作る。

作業:

- `typecheck.rs` から symbol/env、overload、trait lookup、effect inference、HIR lowering 補助を分割する。
- `move_check.rs` から raw helper classifier、place/provenance 型、branch merge、function summary 型を分割する。
- この段階では検査 semantics は変えない。

commit 単位:

1. 型定義と helper の移動のみ。
2. raw helper classifier の module 化。
3. function summary / branch merge 型の module 化。
4. diagnostics と tests の import path 調整。

### Stage 2: Resource IR 型定義と dump の追加

目的: 新しい検査モデルを実装前に可視化する。

作業:

- `nepl-core/src/resource/` を作成し、`ResourceModule`、`ResourceFunction`、`ResourceBlock`、`ResourceOp`、`Place`、`ResourceState` を定義する。
- HIR から Resource IR へ lowering する skeleton を追加する。
- 最初は enforcement しない dump / snapshot 用の IR として扱う。

commit 単位:

1. Resource IR data structure。
2. HIR lowering skeleton。
3. dump / debug snapshot test。

### Stage 3: Resource IR lowering の充実

目的: HIR の静的検査情報を Resource IR に移す。

作業:

- local let/set、function call、branch、loop、match、aggregate construction、field projection を Resource IR op に下げる。
- `MemPtr` projection、storage owner、raw load/store/dealloc/realloc/bulk copy を Resource IR op に下げる。
- HIR 直走査の raw alias 推測と Resource IR lowering の結果を比較する debug check を追加する。

commit 単位:

1. local / aggregate / branch lowering。
2. raw memory operation lowering。
3. function call / callback effect lowering。
4. old checker との comparison diagnostics。

### Stage 4: resource check への移行

目的: move/borrow/lifetime/drop obligation を Resource IR 上の検査へ移す。

作業:

- `CellState` と `OwnerState` による move / initialized 検査を実装する。
- `BorrowState` による shared / unique / lifetime end 検査を実装する。
- branch / loop merge を Resource IR state merge に統一する。
- old `move_check` は比較用に残し、差分がある場合は issue 化する。
- Resource IR diagnostic を粗い互換 bucket へ押し込まず、[compiler diagnostic 再設計計画](./compiler_diagnostics_redesign_plan.md)に従って cell / owner / borrow / lowering の stable code を保持する。

commit 単位:

1. initialized / moved state。
2. owner token / free obligation。
3. borrow / lifetime。
4. branch / loop merge。
5. old checker との gating 切り替え。

進捗:

- 2026-04-29: Resource IR owner obligation gate が generic aggregate store/load regression を拒否していた件を再確認し、原因が compiler 側の false positive ではなく test helper の `alloc_raw` storage leak であることを切り分けた。generic helper は `load<T>` 結果を保持してから `dealloc_raw` する形へ直し、free obligation model を弱めずに generic aggregate 回帰を通した。`List` / `HashMap` の `RawMemoryLoadCell Uninit` は stdlib raw-memory-backed collection / Resource IR lowering の別残件として扱う。
- 2026-05-06: Resource IR cell gate を raw-memory cell operation 専用から通常 read/move/drop/call/construct/branch/match/return まで広げた。`ResourceCheckDiagnostic::CellUnavailable` はすべて `resource.cell.*` として compiler diagnostic へ写像され、old move checker が見逃した通常 cell-state violation も Resource IR boundary で止める。残る Stage 4 の主な未完了点は old move checker と HIR drop insertion の統合削除である。
- 2026-05-06: `run_move_check` の実行順序を見直し、Resource IR lowering coverage / cell / borrow / effect / owner gate を旧 `passes::move_check::run` より先に実行するようにした。旧 checker は Resource IR gate 通過後の fallback 防壁として残す。これにより Resource IR diagnostic が legacy HIR diagnostic に fail-fast で隠される問題を解消した。回帰防止として `nodesrc/test_resource_gate_order.js` を source policy runner に追加した。残る Stage 4 の主な未完了点は、fallback として残る old move checker の削除と HIR drop insertion の Resource IR drop elaboration への統合である。
- 2026-05-06: `tests/compiler/move_effect.n.md` を Resource IR / effect gate 後の authority に合わせ直し、pure raw operation は `effect.pure.calls_impure`、raw cell state は impure fixture、move 後の raw load は `resource.cell.*` で検証する形へ整理した。あわせて direct `Result::Ok` payload match を介した raw address alias で canonical address が新規束縛名へ揺れ、moved cell が uninit と誤診断される問題を `RawCellAddressAliases` の合流規則で修正した。
- 2026-05-06: `tests/compiler/move_check.n.md` を Stage 4 Resource IR authority に合わせ直し、52/52 passing にした。`field::get_ref` は typed `get_field_ref` intrinsic と Resource IR `Borrow` lowering で field cell state を保持し、compiler-lowered `add &owner offset` も field projection として coverage / initialized check が扱う。`Never` value の branch / match arm は initialized-state merge から除外し、到達不能 path が reachable cell state を汚染しないようにした。残る Stage 4 の主な未完了点は、旧 `passes::move_check::run` fallback の削除と HIR drop insertion の Resource IR drop elaboration への統合である。
- 2026-05-06: `ISS-20260506T025727360Z-REMOVE-LEGACY-MOVE-CHECK-FALLBACK-AF-C143E79B` として、compiler pipeline から旧 `passes::move_check::run` fallback を削除し、`nepl-core/src/passes/move_check*` も compiled pass から除去した。`run_resource_static_check` は Resource IR lowering coverage / cell / borrow / effect / owner gate だけを実行する。fallback 削除で露呈した deep prefix chain の owner gate 膨張は、user function return raw-address alias を lowering で二重 materialize していたことが原因だったため、plain user call の identity / owner transfer は Resource IR summary gate に一本化した。残る Stage 4 の主な未完了点は、HIR `passes::insert_drops` を Resource IR drop elaboration へ統合することである。
- 2026-05-06: 旧 fallback 削除後の `tests/compiler/move_check.n.md` 52 件を Resource IR だけで通すため、borrow/lifetime gate は borrow token を aggregate / enum payload / field projection を含む prefix/suffix tree として伝播するようにした。Read / Move / Assign / Construct / Match bind / call return summary は exact local ではなく projected `Place` を基準に token を移す。branch / match arm の検査は外側 continuation を順序付きに見て、使用より前に token scope が終わる場合は外側 EndScope による過剰保持を避ける。これにより `move_check.n.md` は 52/52 passing になった。一方で `move_effect.n.md` は 105/110 で、raw address helper literal offset と higher-order / aggregate / enum payload function value raw write の effect/cell summary 残件があるため `ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2` を再オープンした。
- 2026-05-06: `ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2` を再解決し、`tests/compiler/move_effect.n.md` は 110/110 passing になった。専用 lowering を持たない user helper について、return expression が引数由来の raw address projection だけで構成される場合に限り Resource IR lowering で透明な return projection を発行する。unknown impure indirect call は `MemPtr` / `RegionToken` 引数を保守的な raw cell store release requirement として summary に反映し、高階関数、aggregate field、enum payload に保存された callback raw write を caller 側の initialized cell state 上書きとして検出する。旧 HIR checker fallback は復活させない。
- 2026-05-06: `ISS-20260425T000000Z-RV-CORE-009-58589A3F` の Stage 4 進捗として、compiler pipeline の Resource IR gate を HIR `passes::insert_drops` より前へ移した。Resource IR gate は typecheck 直後の未単相化 HIR 全体ではなく、drop 未挿入 source semantics を保持したまま monomorphize した reachable HIR を検査する。未単相化 HIR 全体を検査すると `#target std` の未使用 stdlib まで対象になり `move_effect.n.md::doctest#108` が timeout するためである。deep prefix chain では HIR の再帰 `clone()` も stack overflow するため、Resource IR 用 HIR と codegen 用 HIR は clone ではなく typecheck を二度実行して分離する。残る Stage 4 の主な未完了点は、HIR `passes::insert_drops` 自体を Resource IR drop elaboration へ置き換え、この二重経路を統合することである。
- 2026-05-06: Resource IR initialized/cell checker が `EndScope` で live non-Copy local を auto-drop state transition として扱うようにした。これにより、source Resource IR check は HIR `passes::insert_drops` の生成済み `drop` 式に依存せず scope exit の drop obligation を表現できる。同名・同型 shadowing では inner auto-drop が outer local を壊さないように、Resource IR lowering が有効範囲内の shadowed local place を `x#N` 形式で固有化する。残る Stage 4 の未完了点は、codegen 側の HIR `passes::insert_drops` を Resource IR drop elaboration の結果から生成する構造へ置き換えることである。
- 2026-05-06: EndScope auto-drop を checker 内部の暗黙処理に閉じず、`ResourceDropPlan` / `ResourceDropFunctionPlan` / `ResourceAutoDrop` / `ResourceAutoDropKind` として明示データ化した。`compute_resource_drop_plan` は nested Branch / Loop / Match を含む Resource IR を走査し、non-Copy scope local の auto-drop 候補を列挙する。initialized/cell checker も同じ候補列挙を使うため、次に codegen 側の HIR `passes::insert_drops` を置き換える際に、checker と codegen が別々の drop 対象推定を持たない。
- 2026-05-06: `ResourceDropPlan` の auto-drop 候補へ `ResourceDropRequirement` を追加し、`StateOnly` / `WholeValue` / `DynamicEnumPayload` / `Structural` を enum として分類するようにした。これにより、direct Drop impl、structural field Drop、runtime tag 依存の enum payload Drop を codegen 側が文字列や独自 flag で再推定しない。残る Stage 4 の未完了点は、この分類済み plan を実 drop call 生成へ接続し、HIR `passes::insert_drops` を削除することである。
- 2026-05-06: HIR `passes::insert_drops` の内部に残っていた drop-needed 再推定を削除し、`ResourceDropRequirement` を消費する `drop_lines_for_requirement` へ統合した。旧 `structural_drop_fields` / `structural_enum_field_drop_lines` / `type_needs_structural_drop` は削除済みで、partial field move でも残存 field の requirement を enum match で扱う。残る Stage 4 の未完了点は、HIR scope walker 自体を Resource IR drop elaboration へ置き換え、compiler pipeline から `passes::insert_drops` を外すことである。
- 2026-05-06: `ResourceDropFunctionPlan` に `drop_points` を追加し、EndScope 単位の auto-drop group を保持するようにした。flat `auto_drops` は `drop_points` から flatten した互換 view として維持する。これにより codegen 移行時に、nested block / branch / match の scope end を HIR 側で再推定せず、Resource IR の drop point を消費できる。残る Stage 4 の未完了点は、drop point を実 drop call 生成へ接続することである。
- 2026-05-06: `ResourceDropPoint` に `ResourceDropPointPath` を追加し、block id と `Op` / `BranchThen` / `BranchElse` / `LoopCondition` / `LoopBody` / `MatchArm` の enum step で EndScope の Resource IR 構造上の位置を保持するようにした。span だけに依存せず、codegen が typed path を辿れる形へ進める。残る Stage 4 の未完了点は、この path を実 drop call 挿入位置へ接続し、HIR scope traversal を削除することである。
- 2026-05-06: `ResourceDropPointPath` を実際の Resource IR op へ解決する `resolve_resource_drop_point_path` / `resolve_resource_drop_point_end_scope` を追加した。無効 path は `ResourceDropPointResolutionError` enum で分類し、block 不在、op index 範囲外、container step と実 op の不一致、match arm 範囲外、EndScope 以外の選択を黙って無視しない。これにより drop point path は単なる metadata ではなく、codegen が消費前に検証できる typed insertion anchor になった。残る Stage 4 の未完了点は、この EndScope resolver を HIR/Wasm drop call 生成へ接続し、`passes::insert_drops` の scope walker を削除することである。
- 2026-05-06: `ISS-20260506T083026784Z-RESOURCE-IR-DROP-PLAN-LACKS-LIVE-DRO-358D2C7E` として、candidate drop plan と live drop fact の混同を分離した。`ResourceFunctionCheck::auto_drop_points` は initialized-state traversal が実際に `Initialized` と判定して drop した point だけを保持し、move 済み outer local は live drop fact に出ない。あわせて non-Copy function parameter の EndScope anchor を Resource IR lowering に追加し、HIR `insert_drops` の outer parameter scope に残っていた drop obligation を Resource IR 上にも表現した。残る Stage 4 の未完了点は、この live drop fact を HIR/Wasm drop call 生成へ接続し、candidate plan ではなく checked state を codegen authority にすることである。
- 2026-05-06: `ISS-20260506T084621972Z-RESOURCE-IR-LIVE-DROP-FACTS-LACK-COD-9EB91BC5` として、`ResourceDropElaborationPlan` を追加した。これは candidate `ResourceDropPlan` ではなく、initialized-state checker が実際に auto-drop した `ResourceFunctionCheck::auto_drop_points` だけから作る codegen-facing plan である。構築時に function/check 対応、typed path の EndScope 解決、auto-drop place と EndScope locals の一致を `ResourceDropElaborationPlanError` enum で検証し、compiler pipeline でも Resource IR cell gate 直後に hard gate として実行する。残る Stage 4 の未完了点は、HIR `passes::insert_drops` の scope walker をこの checked live plan の消費側へ置き換えることである。
- 2026-05-06: `ISS-20260506T090109381Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-82B39C85` として、drop elaboration plan に source binding 名を持たせた。Resource IR lowering は `DeclareLocal` に `source_name` を記録し、shadowed local の内部 place が `x#...` になっても backend/HIR が参照する source 名 `x` を失わない。`ResourceDropElaborationDrop` は checked place、source_name、drop requirement を一体で保持し、binding が解決できない場合は `MissingDropBinding` enum error で hard gate する。残る Stage 4 の未完了点は、この source binding 付き plan を実際の HIR/Wasm drop call 挿入へ接続することである。
- 2026-05-06: `ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B` として、monomorphized Resource IR function から source HIR function へ戻すための `origin_name` metadata を追加した。`HirFunction` は typecheck 時点の source 関数名を保持し、monomorphize で `name` が specialized symbol へ変わっても `origin_name` は維持される。`ResourceFunction` と `ResourceDropElaborationFunction` も `origin_name` を持つため、次の HIR/Wasm drop call 生成は mangled name の prefix parsing ではなく構造化 metadata で source function と対応できる。残る Stage 4 の未完了点は、この function origin / source binding / checked drop point を消費して HIR `passes::insert_drops` を削除することである。
- 2026-05-06: `ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013` として、`run_resource_static_check` が checked `ResourceDropElaborationPlan` を返し、`PreparedProgram` がそれを保持するようにした。これにより plan は gate で検証されて捨てられる metadata ではなく、codegen bridge が消費する compiler pipeline artifact になった。残る Stage 4 の未完了点は、この prepared plan を HIR/Wasm drop call 生成に渡し、旧 `passes::insert_drops` の scope walker を削除することである。
- 2026-05-06: `ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54` として、checked drop plan が source HIR 側の origin / binding / scope span へ戻せることを `validate_resource_drop_elaboration_hir_bridge` で検証するようにした。compiler pipeline は HIR `passes::insert_drops` の前にこの bridge gate を実行し、欠落は `ResourceDropElaborationHirBridgeError` enum から `resource.lower.incomplete` へ写像する。残る Stage 4 の未完了点は、この bridge 済み plan を実際の HIR/Wasm drop call 生成へ渡し、旧 scope walker の drop 対象推定を削除することである。
- 2026-05-06: `ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744` として、裸の callable value reference が `HirExprKind::Var` として Resource IR lowering に届いた場合でも、active local binding がなければ typed `origin_name` / function type から canonical function symbol を解決し、`ResourceOp::FunctionValue` として lowering するようにした。HIR coverage gate も同じ local-shadowing-aware callable rule へ更新し、coverage の scope state は `coverage_hir_scope.rs` に分離した。これにより function value を未初期化 local と誤診断せず、cell checker を弱めずに first-class function / branch return / lambda 系の false positive を解消する。残る Stage 4 の未完了点は、bridge 済み drop elaboration plan を HIR/Wasm drop call 生成へ接続し、旧 `passes::insert_drops` の scope walker を削除することである。
- 2026-05-06: `ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E` として、`set` / `Assign` による initialized non-Copy target の上書き前 Drop obligation を `ResourceAutoDropKind::AssignmentOverwrite` として明示した。initialized-state traversal は target が到達時点で `Initialized` の場合だけ live overwrite drop fact を記録し、move 済み target の再初期化では記録しない。`ResourceDropElaborationPlan` は assignment path を typed resolver で検証し、source HIR bridge も `set` span / target binding を確認する。残る Stage 4 の未完了点は、ScopeLocal と AssignmentOverwrite の両方を消費して実 drop call を生成し、旧 `passes::insert_drops` の VarState scope walker を削除することである。
- 2026-05-06: `ISS-20260506T113709479Z-RESOURCE-DROP-ELABORATION-PLAN-IS-NO-6CFFA860` として、compiler pipeline の実 drop call 生成を checked `ResourceDropElaborationPlan` consumer へ置き換えた。`passes::insert_resource_drops` は `ResourceAutoDropKind::ScopeLocal` / `AssignmentOverwrite` を enum で分岐し、`ResourceDropRequirement` の exhaustive match から Drop call / structural field Drop / dynamic enum payload Drop を生成する。旧 HIR `VarState` / `var_stacks` scope walker と `passes::insert_drops` 呼び出しは削除済みであり、drop 対象を HIR から再推定する二重 authority は残さない。`prepare_module_for_codegen_with_source_map` は drop 未挿入の monomorphized HIR を Resource IR check し、その同じ HIR へ plan-based drop insertion を行い、final monomorphize で生成 Drop trait call を解決する。後挿入された Drop call の impl method body が欠落しないよう、`monomorphize_internal` は `HirModule.impls` に保持されている impl method function も function table へ再登録する。Stage 4 の主な残件は、この新 authority で full review / regression を通し、Stage 5/6 の raw memory / stdlib public API 境界へ進めることである。
- 2026-05-06: `ISS-20260425T000000Z-RV-CORE-009-58589A3F` の完了監査として、compiler pipeline に旧 `passes::move_check::run` fallback と旧 `passes::insert_drops` 呼び出しが残っておらず、checked `ResourceDropElaborationPlan` が `insert_resource_drops` で消費されることを再確認した。この親 issue は Resource IR authority 化完了として fixed にし、raw-memory-backed stdlib API / `MemPtr` owner token 分離 / collection drop obligation は既存 Stage 5/6 issue で追跡する。監査中に deep-prefix `check_pipeline` focused regression が local 240 秒 budget を超えたため、compile-time complexity / regression sizing 問題を `ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232` として分離した。
- 2026-05-06: `ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232` を解決した。通常 i32 copy を raw-address alias group として seed していた `copy_alias_or_seed` を廃止し、既存 raw relation だけを伝播する `copy_alias_if_tracked` と、`RawAddressAlias` / `RawAddressView` だけが seed する `copy_explicit_raw_address_alias` に分けた。raw memory address の local origin は alias group ではなく value-origin fact として保持し、canonicalize 時にだけ使う。さらに transparent raw-address return lowering は bare i32 parameter return を raw helper とみなさず、`add` / `sub` / `mem_ptr_*` / `region_*` など raw-address operation の operand に限定した。これにより deep-prefix Resource IR static check は 240 秒 timeout から 9.33 秒、prepare_codegen は 9.39 秒へ戻り、higher-order function value raw write regression は維持した。Stage 4 authority path の残件は、full review / regression を継続しつつ Stage 5/6 の raw memory boundary と stdlib public API 分離へ進むことである。
- 2026-05-06: `ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F` として、owner summary の false positive を checker 緩和ではなく Result owner effect の materialization と同一 storage replacement の明示で修正した。branch / match / return 境界では pending `Result` payload owner transfer を外側 state に渡す前に実体化し、unconditional consumption と variant-conditioned consumption の二重消費を避ける。fs/stdio private scratch は checked API の `Err` 握りつぶしではなく internal raw boundary の exact `dealloc_raw` に統一した。残る Stage 4 の主な残件は、`ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の dynamic initialized range summary と、`ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` の `unwrap_ok dealloc` checked consumption summary である。
- 2026-05-06: `ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` として、`unwrap_ok` のような reachable `Result::Ok` arm だけを返す helper を `resolved_parameter_variants` summary として表現した。summary 収集は `Read` / `Move` / local initializer / assignment の透明な値 alias を辿り、`expr LocalRead` などの注釈 op では alias を消さない。一方で call / construct / borrow / raw / match output は変換値として alias を切る。これにより `dealloc` の `Result::Ok` success branch に保留された owner consume が `unwrap_ok dealloc` 経由で呼び出し元の raw owner に適用され、checked cleanup API を raw API へ落とさずに false positive を解消した。残る Stage 4 の主な残件は、`ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の dynamic initialized range summary である。
- 2026-05-06: `ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53` として、dynamic raw address view が stable local origin を失う問題を修正した。`ValueOrigin` を exact place だけでなく prefix にも適用し、`tmp[+?]` を `%pref[+?]` のような stable origin plus suffix へ正規化する。通常 i32 copy は raw alias group を seed しないため、deep-prefix alias explosion を再発させずに、`fill_i32 pref pref_len 0` の dynamic initialized Copy range と後続の別 read 由来 `load_i32 add pref off` が同じ cell fact を参照できる。`kpread_to_kpwrite_prefixsum_i32` の `resource.cell.uninit` blocker は解消し、次の別件として fs/stdio scratch dealloc の `resource.owner.no_free_obligation` を `ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4` に分離した。親 issue `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` は、length/guard と結び付いた dependent range summary の残件として open のまま維持する。
- 2026-05-07: `ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4` を解決した。原因は stdlib cleanup ではなく、`RawMemory::Alloc` temporary から local / enum payload / field へ owner transfer した後に `RawCellAddressAliases::move_owner_aliases` が owner mark だけを移し、raw owner value の alias group を再作成していなかったことだった。moved target と moved marked projection を再度 alias group に入れることで、通常 i32 copy は raw alias group を seed しない方針を維持しつつ、owner mark 済み storage root の exact read copy だけが `dealloc_raw` の free obligation へ解決される。`fs_open_with_flags__`、`fs_read_fd_bytes__`、`stdio_read_all_bytes_result__`、`stdio_write_fd_mem_result__` の scratch owner diagnostics を固定する Resource IR 回帰を追加し、`kpread_to_kpwrite_prefixsum_i32` も通過した。Stage 4 authority path の残件は full review / regression と、Stage 5/6 の raw-memory-backed stdlib API 境界整理へ移ることである。
- 2026-05-07: `ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1` を解決した。`lower_raw_address.rs` から transparent user return projection の解析を `lower_raw_address_return.rs` へ分離し、raw wrapper / actual call semantics と user return-expression classification の責務を分けた。`nodesrc/test_resource_checker_responsibility.js` には新 module の存在、`mod` 宣言、line limit、主要 entry point を追加した。これにより `lower_raw_address.rs` は 620 line limit を下回った。source policy は次の別件として `initialized_alias.rs` の責務集中に到達したため、`ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A` に分離した。
- 2026-05-07: `ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A` を解決した。`RawCellAddressAliases` から stable value origin を `initialized_alias_origin.rs` へ、i32 value / condition fact store を `initialized_alias_scalar.rs` へ分離した。raw address alias group と owner cell canonicalization は `initialized_alias.rs` に残し、branch merge は alias group / origin / scalar fact の各責務へ委譲する。これにより memory-safety-critical な raw owner alias table と、補助的な value-origin / condition fact が同一 file に再集中しない。source policy は Resource IR checker responsibility を warning 0 で通過した。
- 2026-05-12: `ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3` を解決した。`Result` payload owner が generic `ok` / `err` helper 経由で通常 `projection_returns` に残り、caller で runtime 不可能な `Ok` / `Err` payload owner まで同時に materialize される問題を、複数 variant payload が混在する projection return だけの variant return 正規化で修正した。あわせて raw owner summary alias walk は `read` / `DeclareLocal` / branch / match bind の projection suffix と `RawMemoryOp::Store` value consumption を保持し、`Result::Ok.field0` や raw node field に移した owner seed を見落とさない。`EndScope` owner auto-drop は `str` などの状態所有 leaf を落としつつ、`i32` raw address / `MemPtr` のような非所有 pointer を `StateOnly` として自動消費しないため、実 drop code が存在しない raw owner leak を隠さない。Stage 4 Resource IR owner summary は resolved variant と owner materialization の境界がより明確になった。
- 2026-05-07: `ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8` を解決した。KP doctest timeout の主因は stdlib runtime ではなく、Resource IR summary builder が caller/callee 依存を見ずに全関数を全反復で再計算していた compile-time complexity だった。`initialized` / `owner` summary は in-place fixed point 更新と関数 summary dependency worklist に移行し、direct call / function value / nested branch / loop / match / self recursion の依存抽出を単体回帰で固定した。`NEPL_COMPILE_STAGE_TIMING=1` の host-only stage timing で `resource_static_check` は約 15.9 秒から約 6.7 秒へ低下し、`tests/stdlib/kp.n.md` focused suite は 7/7 passing になった。Stage 4 authority path の残件は、full review / regression を継続しつつ、残る compile-time hot path を別 issue として必要に応じて切り分けることである。
- 2026-05-07: `ISS-20260506T201433509Z-RESOURCE-CONDITION-FACTS-DROP-NONZER-5EE6B7A6` を解決した。`ResourceConditionFact` に typed `I32Relation` と `ResourceI32RelationOp` を追加し、`lt i len` のような nonzero relational guard を Resource IR に残す。zero-value fact は owner / realloc refinement 用に維持し、relation fact は exhaustive `match` で明示的に扱う。これは `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の returned raw header range summary に必要な typed precondition であり、残件は relation fact と symbolic raw offset を結び付ける range model である。
- 2026-05-07: `ISS-20260506T202600181Z-RESOURCE-RAW-OFFSETS-ERASE-SYMBOLIC--E5DDB5A0` を解決した。`ResourceOffset { bytes: Option<usize> }` に known byte offset / dynamic index / unknown wildcard を混在させる設計をやめ、`ResourceOffset::Known` / `Symbolic` / `Unknown` の enum に分離した。`RawAddressOffset` も simple dynamic offset place を `Symbolic` として保持し、`mem_ptr_add ptr idx` が `base[+symbolic]` として Resource IR に残る。general overlap 判定は `Symbolic` / `Unknown` を may-overlap として安全側に維持するため、静的検査を緩めずに後続 range summary の identity 情報だけを失わない。Stage 4 の残件は、`I32Relation` と `ResourceOffset::Symbolic` を照合して initialized byte range を typed fact として伝播する本体実装である。
- 2026-05-07: `ISS-20260506T203942617Z-RESOURCE-BRANCH-PATHS-DO-NOT-RETAIN--4242E13D` を解決した。`ResourceConditionFact::I32Relation` を branch path の `I32RelationFacts` に保存し、truthy branch は元 relation、false branch は negated relation として保持する。relation fact は value / unary condition から分離しつつ copy / prefix replacement / clear / merge の対象になり、query は reversed relation も扱う。これにより後続 range summary は HIR 条件式を再走査せず Resource IR state から `i < len` / `i >= len` を問い合わせられる。Stage 4 の残件は、保存済み relation fact と `ResourceOffset::Symbolic` を cell availability / initialized range fact へ接続することである。

- 2026-05-07: `ISS-20260506T210407334Z-INITIALIZED-RESOURCE-BRANCH-PATHS-DO-F88296F7` を解決した。owner checker だけでなく initialized checker の branch path でも `record_condition_fact_value_constraints` を適用し、then / else の `RawCellAddressAliases` に typed condition fact を反映する。既存 realloc condition handling はその後に適用するため、realloc 成否と relation proof の両方が path state に残る。Stage 4 の残件は、この initialized checker から参照可能になった relation proof と `ResourceOffset::Symbolic` を raw memory load の availability 判定へ接続することである。

- 2026-05-07: `ISS-20260506T211740745Z-SYMBOLIC-COPY-STORES-ERASE-UNKNOWN-O-0BD91F6C` を解決した。`RawMemoryOp::Store` の汎用 clear を store 専用の typed clearing に分け、symbolic Copy store が `pref[+?].deref` の initialized Copy fact を過剰に消さないようにした。non-Copy / moved / uninit state は従来どおり保守的に消すため、memory safety の緩和ではない。Stage 4 の残件は、loop condition fact と guarded initialized range summary を接続して、明示 proof のある dynamic raw load だけを通すことである。

- 2026-05-07: `ISS-20260506T212446487Z-RESOURCE-LOOPS-DO-NOT-CARRY-TYPED-CO-FD0086F2` を解決した。`ResourceOp::Loop` に `condition_fact` を追加し、`while lt i len` のような typed relation guard を Branch と同じ `ResourceConditionFact::I32Relation` として Resource IR に保持する。initialized / owner checker は condition evaluation 後、body path に truthy fact、exit path に negated false fact を適用してから state merge するため、loop body の range summary が HIR 条件式を再走査せず `RawCellAddressAliases` から `i < len` を問い合わせられる。Stage 4 の残件は、保持された loop relation fact と `ResourceOffset::Symbolic` を raw cell availability / initialized range fact へ接続することである。

- 2026-05-07: `ISS-20260506T215615927Z-RESOURCE-RAWADDRESSVIEW-TREATS-ORDIN-B3C620DA` を解決した。`RawAddressView` は lowering で `add` / `sub` から広めに生成されるため、checker 側で既存の raw-address proof がある場合だけ raw alias / non-owning view として扱う。proof は alias table の exact/prefix raw address、initialized checker の raw cell / owned raw storage / external raw storage、owner checker の owner state / storage origin に限定し、scalar `ValueOrigin` だけでは raw pointer とみなさない。さらに storage-offset view を local に束縛しても `pref[+?].deref` の broad initialized fact を view local へ rekey しない。これにより unrelated impure `i32` arithmetic が raw alias state を seed せず、`fill_i32 pref pref_len 0` 後の symbolic load と `kpread_to_kpwrite_prefixsum_i32` は通過する。Stage 4 の残件は、returned header / length field / guard relation をまたぐ dependent initialized range summary を typed model として表現することである。

- 2026-05-07: `ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A` を解決した。self-host lexer の empty `lex_all_with_file_id` smoke で `resource_initialized_moves` に入る前に止まっていた原因は、raw cell address return summary が `idx` / `file_id` のような通常 `i32` parameter まで raw address seed として扱い、token construction と branch merge を通じて bogus alias を膨張させていたことだった。summary 計算を `SummaryWorklist` に移行し、seed 対象を `MemPtr` / `RegionToken` / それらを含む aggregate / reference に限定した。明示的な `RawAddressAlias` / `RawAddressView` は引き続き raw relation を作れるため、raw helper の正当な i32 address return は lowering の typed ResourceOp が authority になる。timeout は owner diagnostics まで進む状態に改善し、残る lexer owner flow は `ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE` として分離した。Stage 4 の残件は、compiler summary の計算量を抑えたまま Resource owner diagnostics の正確性を維持し、self-host lexer の owner transfer を通すことである。

- 2026-05-07: `ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903` を解決した。returned aggregate の `buf` / `len` field をまたぐ initialized raw range が caller local へ束縛される時に失われていた原因は、値コピーが range count だけを複写し、address / count の dependent pair を同時に projection copy していなかったことだった。`CellTable::copy_initialized_raw_byte_ranges_through_value` を `cell_state_raw_range_value.rs` に追加し、`DeclareLocal` / `Read` / `Assign` / `Move` / branch / match / raw memory `Load` / raw memory `Store` / aggregate `Construct` で initialized range の address と count を value projection として複写する。assignment / raw memory store では overwritten target 配下の stale range fact を消す。guard なし symbolic load は引き続き拒否し、`0 <= i && i < len` が Resource IR relation fact として証明された場合だけ通すため、静的検査の正確性は緩めていない。責務分割 policy は緩めず、追加 helper と既存 cover test を module 分離した。Stage 4 の残件は、returned aggregate projection 修正後も残る full scanner-style の `fd_read` loop / realloc / capacity field を含む dependent range model と、別件 `ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730` の external I/O effect helper 分割である。

- 2026-05-07: `ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730` を解決した。`fd_read` / `fd_pread` の single-iov payload range construction を `initialized_external_io_payload.rs` に分離し、`initialized_external_io_effect.rs` は external I/O effect entry point と nread exact-cell initialization に戻した。iovec descriptor 探索、single-iov 判定、payload alias filtering、`nread` を count とする `InitializedRawRangeUnit::Bytes` 登録は新 module が担当する。line limit は緩めず、`node nodesrc/test_resource_checker_responsibility.js` は passed になった。Stage 4 の残件は、full scanner-style の `fd_read` loop / realloc / capacity field を含む dependent range model である。

- 2026-05-07: `ISS-20260507T051545017Z-CELL-STATE-RAW-RANGE-EXCEEDS-SPLIT-L-76536EAC` を fixed として整理した。`cell_state_raw_range.rs` の責務超過は、raw range mutation / value projection / guarded cover proof / cover test を module 分離した結果、129/140 lines まで戻っている。`node nodesrc/test_resource_checker_responsibility.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` はどちらも warning なしで通過しており、line limit を上げる形の回避はしていない。Stage 4 の残件は、full scanner-style の `fd_read` loop / realloc / capacity field を含む dependent range model である。

- 2026-05-07: `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` を解決した。full scanner-style の `fd_read` loop / `realloc_raw` / returned header capacity field をまたぐ initialized raw range は、`sub next_cap cap` の typed scalar difference fact、append fill range composition、raw alias aware branch/loop/match range merge、`canonicalize_scalar` による count と non-owning pointer view の分離で表現する。guard なし symbolic load は引き続き拒否し、caller 側は `0 <= i && i < len && i < cap` が Resource IR fact として証明される場合だけ `load_u8 add data i` を許可する。Stage 4 の残件は、Resource IR authority path の full review / regression と、以後発見された owner / effect / borrow の個別 issue 解決である。

- 2026-05-07: `ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE` を解決した。match payload binding が外側 local / parameter を shadow した場合、Resource IR lowering が payload 初期化対象を `%e`、arm body の参照を `%e#0` に分裂させ、`sha256_rounds_loop` の `Result::Err e` を `resource.cell.uninit` と誤診断していた。match arm binding は `ctx.declare_local` が返す固有 Place を checked authority とし、drop elaboration bridge 用の source binding 名は `bind_source_name` として別に保持する。checker の cell-state 判定は緩めず、shadowed Copy payload と non-Copy payload drop bridge の Rust 回帰、および SHA-256 known-vector doctest で固定した。Stage 4 の残件は、full scanner-style の dependent range model と、Resource IR authority path の継続的な full regression である。

- 2026-05-07: `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` の部分対応として、`str_addr` と borrowed `region_ptr` の lowering を `RawAddressViewKind::NonOwningProjection` として表現するようにした。`RawAddressViewKind::Offset` と分けることで、通常 i32 arithmetic の raw pointer proof gate は維持しつつ、仕様上 non-owning な pointer projection は source 未解決でも dealloc / realloc の owner として扱わない。`mem_ptr_addr` と `str_from_addr_unchecked` の owner transfer 経路は残しており、静的検査を緩めずに `str_addr` helper 経由の free bypass を拒否する。Stage 4 の残件は、`MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の最終分離である。

- 2026-05-07: `ISS-20260507T085434323Z-RESOURCE-OWNER-CHECKER-LOSES-NON-OWN-344F2372` を解決した。`str_addr` 由来の non-owning raw address view は direct local では owner として拒否されていたが、`Result::Ok` などの aggregate payload に入れてから match bind / read を通ると non-owning raw view fact が落ちていた。Resource IR owner summary は payload projection に non-owning raw view marker を生成できるため、summary 生成を緩めるのではなく、`RawAddressViewTable` で通常 raw address view と non-owning raw address view を分け、construct / branch / match / call return summary / read の value-preserving owner-flow に non-owning fact copy を接続した。これにより `OwnerState::NoFreeObligation` を pointer authority として流用せず、`MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の分離方針を弱めず、payload 経由の `dealloc_raw` / `realloc_raw` も `OwnerUnavailable` で拒否する。Stage 4 の残件は、Resource IR authority path の full review / regression と、stdlib public API 側の owner token 分離である。
- 2026-05-07: `ISS-20260507T125821563Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-992DF2EE` を解決した。returned / param / variant param の raw byte range と `KnownI32` / projection count enum を `initialized_summary_byte_range_model.rs` へ分離し、`initialized_summary.rs` を function summary と raw cell/variant requirement contract に戻した。line limit は緩めていない。Stage 4 の残件として、次に露出した `initialized_summary_apply.rs` の responsibility split を `ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6` で追跡する。
- 2026-05-07: `ISS-20260507T130937432Z-RESOURCE-INITIALIZED-SUMMARY-APPLY-E-7FFA13D6` を解決した。caller-side param cell / param byte range application と `RawCellInitializationParamCount` の解決を `initialized_summary_apply_param.rs` へ分離し、`initialized_summary_apply.rs` を summary lookup と application orchestration に戻した。Stage 4 の残件として、次に露出した `initialized_summary_byte_ranges.rs` の builder 分割を `ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0` で追跡する。
- 2026-05-07: `ISS-20260507T131613193Z-RESOURCE-INITIALIZED-SUMMARY-BYTE-RA-F56D00D0` を解決した。returned / param raw byte range builder と count-source extraction を 4 module に分離し、`initialized_summary_byte_ranges.rs` の集中を削除した。Stage 4 の残件として、次に露出した `initialized_summary_variant_build.rs` の分割を `ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691` で追跡する。
- 2026-05-07: `ISS-20260507T132339456Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-32AEE691` を解決した。variant param cell / byte range の uniqueness helper を `initialized_summary_variant_unique.rs` へ分離し、`initialized_summary_variant_build.rs` を variant return path traversal と variant-gated summary construction に戻した。`node nodesrc/test_resource_checker_responsibility.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` は warning なしで通過しており、Stage 4 ResourceIR proof module の責務分割 policy は現時点で緑である。
- 2026-05-07: `ISS-20260507T134613401Z-RESOURCE-OWNER-SUMMARY-IGNORES-NON-O-9A39F228` を解決した。`str_addr` 由来の non-owning raw view を `mem_ptr_wrap` / `region_new` で `RegionToken` に詰め直し、`dealloc_region` の helper parameter consumption 経由で free obligation owner に見せる経路を拒否する。callee owner summary が consumed parameter projection を要求する場合、caller actual が non-owning raw address view なら `resource.owner.no_free_obligation` を出すようにし、owner consumption の責務は `owner_consumption.rs` に分離した。これは `MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の分離を Stage 4 owner summary 適用境界へ拡張する対応である。
- 2026-05-07: `ISS-20260507T143247279Z-RESOURCE-IR-OWNER-CHECKER-LOSES-NON--66D5734F` を解決した。`region_ptr_at<T,U>` の `Result::Ok(MemPtr<U>)` payload は borrowed `RegionToken` 由来の bounds-checked non-owning projection だが、stdlib 実装が `region_token_ptr_ref` / `mem_ptr_addr` / `mem_ptr_wrap` / `Result::Ok` を経由するため owner summary に non-owning view fact が残らず、`region_new` で forged owner token に見せられた。`region_ptr_at` の Ok payload raw field と borrowed `region_token_ptr_ref` の raw field を Resource IR lowering 時点で `NonOwningProjection` として表現し、coverage gate は reference projection の HIR/ResourceIR 対応を保ったまま `RawAddressView` target を alias metadata として扱うようにした。これにより `region_ptr_at` の正常な pointer read/write と元 token の dealloc は維持しつつ、Ok payload を owner に昇格する経路は `resource.owner.no_free_obligation` で拒否する。Stage 4 の残件は、compiler-issued owner token と stdlib public API の最終分離である。
- 2026-05-13: `ISS-20260512T202418246Z-RESOURCE-OWNER-POLICY-DOES-NOT-GUARD-2B46D8D5` を解決した。Stage 4 の `NonOwningProjection` は `MemPtr` projection が free obligation owner に戻らないための compiler-side 境界だが、source policy はこれまで enum の存在と module 分割しか見ていなかった。`nodesrc/test_resource_checker_responsibility.js` で `raw_address_view_carries_owner_alias` と `non_owning_raw_view_return_kind` の match arm を直接監視し、`RawAddressViewKind::NonOwningProjection => false` と `RawAddressViewOwnership::NonOwningProjection => ProjectionView` を wildcard なしで固定した。これにより `MemPtr = non-owning pointer` と `OwnedRegion/Storage = free obligation owner` の分離を regression policy として維持する。
- 2026-05-13: `ISS-20260512T202946482Z-TYPECHECK-CONSTRUCTOR-CAPABILITY-BOU-14965EAB` を解決した。`MemPtr` / `RegionToken` の direct constructor restriction は compiler-issued raw pointer / owner token capability の入口であり、後段 Resource IR だけに任せると forged token が typed HIR に残る。`nodesrc/test_static_check_boundary_responsibility.js` で `StructConstructorPolicy` / `RestrictedStructConstructor` enum、raw-memory-boundary based の `MemPtr => RawPointer` / `RegionToken => OwnerToken` 分類、`apply_struct_constructor` の `RawMemoryBoundaryOnly` gate、個別 diagnostic code への分岐を監視する。これにより Stage 4 の owner capability boundary を typecheck source policy でも維持する。
- 2026-05-13: `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` を compiler core 側では fixed / resolved とした。`MemPtr` / `RegionToken` direct constructor boundary、`NonOwningProjection` raw view、`StorageOrigin::Owned`、returned owner summary、helper consumption gate、source policy が揃い、fixed raw address や borrowed projection を free obligation owner に偽装する主要経路は Resource IR / typecheck で拒否される。`tests/stdlib/memory_safety.n.md` は 23/23、`tests/compiler/move_effect.n.md` は 110/110、`region_token_forged` は 6/6 passing である。残る `core/mem` public API、collection/string/self-host buffer の safe public discipline は Stage 6 stdlib issue へ分離して継続する。
- 2026-05-14: `ISS-20260514T054314434Z-COPY-IMPL-CAN-MARK-COMPILER-OWNER-TO-D6C08048` を解決した。`RegionToken<T>` は構造上 `MemPtr<T>` と `i32` に見えるが、free obligation を持つ compiler owner token なので `Copy` capability target にはできない。Copy impl target validation は `StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken)` を参照し、compiler-owned owner token だけを `type.copy_impl.target_not_copy` で拒否する。同名 user struct は struct table の type identity と policy が一致しないため通常の構造的 Copy 判定に残る。これにより `MemPtr = non-owning pointer` と `RegionToken/OwnedRegion = free obligation owner` の分離を trait capability layer でも維持する。
- 2026-05-13: `ISS-20260513T230312662Z-RESOURCE-OWNER-VARIANT-RETURN-MODULE-817EC208` を解決した。`owner_summary_variant_return.rs` は returned owner source collection と variant payload return materialization を同居させていたため、returned value / descendant / alias descendant から owner source を集める処理を `owner_summary_variant_return_sources.rs` へ分離した。分離後に露出した `owner_variant_utils.rs` の責務超過も、`OwnerValueCondition` truth evaluation を `owner_variant_condition_truth.rs` へ移して解消した。line limit は緩めず、新 module も `nodesrc/test_resource_checker_responsibility.js` の監視対象に入れ、Resource IR owner summary / pending variant owner effect の監査境界を維持した。
- 2026-05-07: `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF` を解決した。`check_module_with_source_map` は typecheck で止まらず、compile preparation と同じ非再帰 prepare phase を共有して Resource IR lowering coverage、initialized cell、drop plan、borrow、effect、owner、drop bridge gate を通す。`--check` は成果物 emission には進まないが、compile/codegen pipeline と同じ memory/resource safety authority を通るため、CI や selfhost tooling が check-only を使っても Resource IR diagnostic を取りこぼさない。deep prefix regression と CLI Resource IR regression で、stack overflow 回避と static gate enforcement の両方を固定した。
- 2026-05-12: `ISS-20260512T032320909Z-RESOURCE-OWNER-SUMMARY-REPORTS-STDIO-C9FC40C9` を解決した。`print_i32` / `ansi_text_style_code` で露出した `str` temporary の `resource.owner.maybe_leak` は、stdio が `str` を消費すべきという問題ではなく、Resource IR lowering が allocation-returning Copy `str` temporary の statement lifetime を表現していなかったことが原因だった。HIR block line ごとに新規 top-level op の temporary output を確認し、Copy だが state-only owner scoping が必要な `str` temporary へ line-end `EndScope` を挿入する。非 dropped line result は `EndScope.result` で保存するため、関数返却値や block result を誤って消費しない。これにより stdio/ANSI false positive は解消し、`resource.owner.maybe_leak` 自体は弱めていない。examples の残り 1 件は `cliarg_count` / `cliarg_get` の raw argv scratch owner flow であり、`ISS-20260512T041752474Z-RESOURCE-OWNER-SUMMARY-REPORTS-CLIAR-97FEDA3D` として分離した。

### Stage 5: effect model の拡張

目的: raw memory を safe surface から閉じつつ、stdlib 内部の正当な allocation を表現する。

作業:

- internal effect と surface fold を導入する。
- raw memory primitive は compiler-owned boundary では `InternalAlloc` / `UnsafeMemory` として扱う。
- public pure API から raw identity が漏れた場合は fold 不可として `resource.raw.*` / `effect.*` の diagnostic にする。
- user source から raw address escape を構成できる経路を compile_fail にする。
- raw identity escape と ordinary impure call を同じ表示 bucket に依存させず、[compiler diagnostic 再設計計画](./compiler_diagnostics_redesign_plan.md)の `resource.raw.*` / `effect.*` code へ分ける。

commit 単位:

1. effect enum / fold 関数。
2. raw primitive effect 分類。
3. stdlib internal boundary の暫定許可。
4. public escape diagnostics。

進捗:

- 2026-05-06: compiler-owned raw-memory-boundary capability は `SourceCapabilities` と SourceMap を通して Resource IR effect gate に届く。`UnsafeMemoryInPureFunction` は raw-memory-boundary でない source では `effect.pure.calls_impure` として error 化済みである。
- 2026-05-06: `ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C` として、configured stdlib の `alloc/string.nepl` と `alloc/string/storage.nepl` を `core/mem.nepl` と同じ exact raw-memory-boundary capability の対象に加えた。これは string / str owned storage helper の内部 raw `load` / `store` / `bulk_copy` を Stage 6 移行中に許可するもので、stdlib 全体や arbitrary suffix path を許可するものではない。`Loader` は configured `stdlib_root` から canonical path を計算し、該当する exact path だけを許可する。
- 2026-05-06: wasm doctest で、`alloc/io.nepl` / `alloc/string/utf8.nepl` / `std/text.nepl` / `std/streamio/scanner/state.nepl` にも同種の raw-memory-backed boundary 未整理が残ることを確認し、`ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` として分離した。これらは安易に stdlib 全体を許可せず、true internal boundary と safe public wrapper の責務を確認してから exact capability か Stage 6 API 移行で解く。
- 2026-05-06: `ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` として、`alloc/io.nepl` / `alloc/string/utf8.nepl` / `std/text.nepl` / `std/streamio/scanner/state.nepl` を configured exact boundary table に追加した。`tests/stdlib/kp.n.md` から `effect.pure.calls_impure` は消え、残りは fs/stdio read owner summary、`pref` dynamic range summary、f64/f32 runtime timeout として分離された。
- 2026-05-06: remote main の string responsibility split 後、`alloc/string/access.nepl` の `len` / `string_byte_at_unchecked` と `alloc/string/scanner.nepl` の scanner byte helper が exact raw-memory-boundary capability に追従しておらず、`effect.pure.calls_impure` が再発した。`ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912` として、両 module を configured stdlib の exact boundary table に追加した。Stage 6 移行完了までの internal string layout boundary は、module split ごとに loader capability table と regression を同時更新する。
- 2026-05-06: remote main の integer conversion split 後、`alloc/string/integer.nepl` の `from_u128_radix` が raw `store_u8` で文字列 buffer を構築するにもかかわらず exact raw-memory-boundary capability に追従していなかった。`ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71` として `alloc/string/integer.nepl` を loader の exact boundary table に追加した。併せて `alloc/string/float.nepl` は直接 raw memory 操作を持たず `StringBuilder` / integer conversion へ委譲していることを確認し、過剰な raw boundary capability は付与しない。
- 2026-05-06: KP doctest の次 blocker として、`alloc/string/builder.nepl` の `sb_append_result` / `sb_append_byte_result` / `sb_build_result` が raw `store_u8` / `mem_copy` を使うにもかかわらず exact raw-memory-boundary capability に追従していないことを確認した。`ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4` として `alloc/string/builder.nepl` を loader の exact boundary table に追加した。StringBuilder は owned byte buffer の内部構築境界であり、Stage 6 の owner-token API 移行が完了するまでは safe public surface ではなく compiler-owned internal boundary として扱う。
- 残件は、raw-memory-backed stdlib public API を Stage 6 で internal/public 境界へ分け、raw identity と owner token が safe surface へ漏れない最終 API に移行することである。
- 2026-05-13: `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04` を core compiler 側では verified / resolved とした。`UnsafeMemoryInPureFunction` hard gate、Resource IR raw identity / cell / owner gate、exact raw-memory-boundary capability regression が揃っており、user source から raw memory operation を pure bypass する元の問題は閉じている。Stage 6 の stdlib internal/public API 移行は引き続き stdlib issue 側で追跡する。
- 2026-05-13: `ISS-20260513T023254911Z-CORE-MEM-FACADE-STILL-CARRIED-RAW-ME-FEEF633F` を解決した。`core/mem.nepl` root は public facade に縮小し、`types` / `raw` / `allocator` / `pointer` submodule へ実装責務を分離した。loader の exact raw-memory-boundary capability も root facade から外し、実装 submodule のみに付与する。raw helper の public re-export 閉鎖は未完だが、Stage 6 の前提である「public facade 自体が raw boundary privilege を持たない」状態に進めた。
- 2026-05-13: `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D` を解決した。`core/mem` root facade は `types` / `layout` / checked `pointer` API だけを公開し、`mem_ptr_wrap` / `mem_ptr_addr` / `region_new` / raw allocator / raw load-store は internal/raw implementation module へ閉じた。compiler resolver も private import を public facade 越しに推移公開しないため、`#import "core/mem" as *` から raw address escape を構成できない。残る Stage 6 の焦点は、direct internal/raw module の利用 discipline、Vec/StringBuilder/collection/self-host buffer の owner token API 移行、stdlib 全体の raw-memory-backed public API 整理である。
- 2026-05-13: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` の一部として、loader の raw-memory-boundary authority を exact module allowlist から source capability proof へ移した。`RAW_MEMORY_BOUNDARY_STDLIB_PATHS` は削除し、configured stdlib root 配下の compiler-owned source で、AST 上に raw body instruction / raw address helper / raw owner helper / raw helper call / raw intrinsic / restricted constructor の証拠がある場合だけ capability を付与する。これにより module 分割のたびに compiler allowlist を追従する設計を廃止し、user source や prefix-like path は raw 証拠を持っていても capability を受け取らない。
- 2026-05-13: `ISS-20260513T090733651Z-VEC-STORAGE-CLEANUP-DEALLOCATES-THRO-4A132C97` を解決した。`vec_free_storage` / `push` realloc failure cleanup / merge sort scratch cleanup は `mem_ptr_addr` で `MemPtr` を raw `i32` owner へ落とさず、typed `dealloc_ptr<T>` で free obligation を閉じる。確保直後 scratch buffer の dealloc failure branch は到達不能として扱い、Resource IR に owner leak branch を残さない。`stdlib/alloc/collections/vec` focused doctest は 32/32 passing になった。これは `OwnedBuffer<T>` 完成ではないが、Stage 6 の `MemPtr = non-owning pointer` 方針に沿って raw address 経由の owner proof loss を取り除く局所前進である。
- 2026-05-13: `ISS-20260513T092818532Z-VEC-CLEANUP-FREE-ACCEPT-NON-COPY-PAY-497499BC` を解決した。`Vec.clear` / `Vec.free` / `vec_free_storage` は initialized element を走査せず storage-only cleanup を行うため、`OwnedBuffer<T>` と element drop traversal が入るまでは `.T: Copy` に限定する。`Vec<CleanupPayload>` の `clear` / `free` compile-fail と source policy で、unsupported non-Copy payload が安全に破棄できるように見える退行を防ぐ。
- 2026-05-13: `ISS-20260513T095201685Z-RAW-MEMORY-SOURCE-CAPABILITY-TREATS--389248CD` を解決した。raw-memory-boundary source capability scanner は、raw helper と同名の parameter / local / same-module safe helper を raw evidence として扱わない。`RawMemoryBoundaryScope` を分離し、function parameter、block `let`、match payload binding、top-level 定義による lexical shadowing を scanner に反映した。これにより source capability proof が単なる identifier spelling ではなく、shadow されていない raw operation / raw helper / restricted constructor の AST evidence に基づく。
- 2026-05-13: `ISS-20260513T214506607Z-BTREE-KEY-EQUALITY-HELPERS-ACCEPT-OR-BFADC667` を解決した。`BTreeMap` / `BTreeSet` の key equality helper は `ord_lt` を値渡しで 2 回呼ぶため、borrowed key comparison と `OwnedBuffer<T>` / initialized cell based non-Copy collection が入るまでは `.K: Ord&Copy` / `.T: Ord&Copy` に限定する。これにより sorted-array BTree API の Stage 6 Copy-only 境界を helper から迂回できない。
- 2026-05-14: `ISS-20260514T045613682Z-STREAMWRITER-STORES-RAW-MEMPTR-OWNER-448F8E4F` を解決した。`StreamWriter` は direct `MemPtr<u8>` / capacity / pending length field を保持せず、`ByteBuilder` を owner boundary として保持する形へ移行した。flush は `ByteBuilder.ptr` の non-owning view を使い、close は `ByteBuilder` owner を move して解放する。これにより stream writer public state から raw pointer owner field が消え、Stage 6 の transitional MemPtr owner field policy は 8 件から 7 件へ減った。
- 2026-05-14: `ISS-20260514T051405052Z-STREAMSCANNER-HIDES-BUFFER-OWNER-BEH-0977B2E3` を解決した。`StreamScanner` は raw `MemPtr<u8>` header に buffer pointer / length / cursor position を詰める設計を廃止し、input owner を `ByteBuf` field、cursor position を typed `Vec<i32>` storage として保持する。scanner byte access と token slice construction は state helper に集約し、cursor mutation は `vec::get` / `vec::replace` 経由で Resource IR が typed initialized cell として追跡できる形にした。これにより `StreamScanner.header` の transitional exception を削除し、Stage 6 の MemPtr owner field policy は 7 件から 6 件へ減った。
- 2026-05-14: `ISS-20260514T055830236Z-VECDATALEN-CARRIES-RAW-VEC-STORAGE-V-B662D7DF` を解決した。`VecDataLen<T>` は `Vec.data: MemPtr<T>` と `len` を public struct field としてまとめる raw storage view carrier だったため、Copy-only 制約を維持しても `MemPtr` owner-like field migration の例外を増やしていた。互換 alias は残さず `VecDataLen` / `data_len` を削除し、呼び出し側は `len<T>(&Vec<T>)` と `data_mem_ptr<T>(&Vec<T>)` を明示的に別観測する形へ移した。これにより Stage 6 の MemPtr owner field policy は 6 件から 5 件へ減った。
- 2026-05-14: `ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F` を解決した。`StringBuilder` は `Option<MemPtr<u8>>` / `len` / `cap` を `ByteBuilder` と重複して保持していたため、text builder 固有の raw owner field 例外を残していた。`StringBuilder` は `ByteBuilder` owner を保持する typed wrapper に変更し、capacity / append / free / build は `ByteBuilder` / `ByteBuf` の owner boundary へ委譲する。safe buffer API は pure surface に揃え、raw memory effect は各 raw-memory-boundary source 内の Resource IR / source capability gate で検査する形にした。これにより Stage 6 の MemPtr owner field policy は 5 件から 4 件へ減った。
- 2026-05-14: `ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159` を解決した。`ByteBuf` / `ByteBuilder` は `Option<MemPtr<u8>>` を owned storage field として持つ過渡設計をやめ、free obligation owner を `RegionToken<u8>` field に集約した。`MemPtr<u8>` は `io_bytebuf_data_ptr_ref` / `byte_builder_data_ptr_ref` が参照から返す non-owning projection に限定し、append / flush / write / UTF-8 変換は owner を動かさず view を使う。compiler ResourceIR 側も function summary の raw owner alias 伝播で non-owning projection 由来の `mem_ptr_add` を owner alias と誤認しないよう、raw view state を summary traversal に接続した。これにより Stage 6 の MemPtr owner field policy は 4 件から 2 件へ減った。
- 2026-05-14: `ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A` を解決した。`Vec<T>` は `data: MemPtr<T>` を storage owner field として持つ過渡設計をやめ、`region: RegionToken<T>` を free obligation owner として保持する。`data_mem_ptr<T>` / `vec_storage_mem_ptr<T>` / sort・map・filter・prefix・mutation 系は参照から得る non-owning `MemPtr<T>` view を使い、戻り値では `RegionToken<T>` owner を移す。これにより Stage 6 の MemPtr owner field policy は `RegionToken.ptr` だけの 1 件へ減った。ただし `RegionToken` はまだ forgeable であり、`OwnedBuffer<T>` / initialized prefix / non-Copy payload drop traversal は Stage D の残件として継続する。
- 2026-05-14: `ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E` を解決した。`byte_builder_realloc_region_or_free` は grow failure cleanup で `RegionToken<u8>` を ptr/size に分解して `dealloc_ptr` へ渡すのをやめ、`dealloc_region<u8> region` へ owner tokenを丸ごと渡す。Err branch は typed `OutOfMemory` に畳み、通常 stdlib 実装の `unreachable` を排除した。これにより Stage 6 の byte buffer owner boundary は `RegionToken` owner 消費として一貫した。
- 2026-05-15: `ISS-20260514T160255919Z-VEC-DATA-PTR-EXPOSES-RAW-I32-STORAGE-546EA2EB` を解決した。`Vec.data_ptr<T>(&Vec<T>) -> i32` は raw address observer として public API に残さず削除した。必要箇所は `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` を明示的に観測し、raw-memory-boundary 実装箇所だけが `mem_ptr_addr` へ変換する。同じ根として、この時点では `kpsearch` の raw `i32` pointer helper を private にし、公開面を `Vec<i32>` wrapper に揃えた。後続の `ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D` で `kpsearch` 自体の raw storage view 依存も削除済みである。
- 2026-05-15: `ISS-20260514T220733927Z-ALLOC-STRING-ROOT-RE-EXPORTS-RAW-STR-BF0F0254` を解決した。`alloc/string` root は通常利用者向け safe facade に戻し、`alloc/string/storage` と `alloc/string/utf8` の raw `MemPtr` helper を public wildcard re-export しない。`std/fs` / `std/stdio` / `std/env/cliarg` / `std/streamio` など raw OS/storage boundary 側だけが explicit raw helper module を import する。これにより `core/mem` / `Vec` と同じ public/raw facade split が string でも揃った。
- 2026-05-15: `ISS-20260514T161819706Z-VEC-STORAGE-MEMPTR-HELPER-EXPOSES-LO-A9C5BC02` を解決した。`vec_storage_mem_ptr<T>(VecStorageState, &RegionToken<T>) -> MemPtr<T>` は lower-level storage state pieces を受ける公開 helper だったため削除した。`data_mem_ptr<T>(&Vec<T>)` が `VecStorageState` を直接 match し、`Empty` は 0 address view、`Owned` は `region_ptr` 由来 view を返す。public surface は `&Vec<T>` observer に集約した。

### Stage 6: stdlib memory API の段階移行

目的: compiler の Resource IR と stdlib の公開 API を同期する。

作業:

- `core/mem` を internal raw module と safe public wrapper に分ける。
- collection は `Copy` read、borrowed read、owned remove/pop、container drop を API と型制約で分ける。
- `dealloc_*` は storage-only dealloc と initialized payload destruction を分ける。
- self-host compiler の buffer / diagnostic / outcome は raw `MemPtr` を直接持たず、safe wrapper を使う。

commit 単位:

1. `core/mem` internal/public 境界。
2. `Vec` / `StringBuilder` の owner token 移行。
3. collection drop contract。
4. self-host buffer API 移行。

進捗:

- 2026-05-15: `ISS-20260514T164856024Z-OWNER-BACKED-AGGREGATE-CONSTRUCTORS--61400B84` を解決した。compiler typecheck は struct field 型を走査し、direct field に compiler owner token を含む public aggregate を `OwnerBackedAggregateBoundaryOnly` として分類する。`Vec<T>` などの特定名 allowlist ではなく、`RegionToken<T>` の restricted constructor policy と型形状から導出する。
- 同修正で、owner-backed aggregate の direct constructor は `OwnerAggregateConstructorBoundary` source capability を持つ compiler-owned stdlib 実装 source に限定した。configured stdlib 配下でも無条件には付与せず、parsed source に aggregate constructor evidence がある場合だけ capability を付ける。user source では evidence があっても capability は付与せず、`type.owner_aggregate.constructor_restricted` を出す。
- 2026-05-15: `ISS-20260514T230404748Z-OWNER-BACKED-AGGREGATE-POLICY-DOES-N-7D995A6B` を解決した。owner-backed aggregate 判定は direct `RegionToken<T>` field だけでなく、`Vec<T>` を field に持つ wrapper、`HashMapStorage<K,V>`、`HashMap<K,V,H>` のように owner-backed aggregate を入れ子に含む型へ fixed-point で伝播する。これにより user source が collection storage state を通常 struct constructor で再構築する経路も `type.owner_aggregate.constructor_restricted` で拒否される。判定は stdlib 名 allowlist ではなく、compiler owner token policy と struct field 型から導出する。
- owner token field projection も field 型を解決した上で boundary 外では `type.owner_token.field_access_restricted` とするため、aggregate を経由して free obligation owner を forge / extract する経路を閉じた。こちらは `OwnerAggregateFieldBoundary` source capability へ分離され、`RawMemoryBoundary` とも分離している。stdlib 実装 module が `Vec` / `ByteBuilder` などの owner aggregate を移動・再構築できても、raw memory operation authority までは広がらない。
- 2026-05-15: `ISS-20260514T183506445Z-STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B-C76C9E1E` を解決した。`std/env/cliarg` root facade は `core/mem/raw` / `core/mem/internal` を直接 import せず、`cliarg_count` / `cliarg_get` は qualified `std/env/cliarg/raw` helper へ委譲する。argv scratch allocation、raw address conversion、out pointer 初期化、`args_get`、raw slot load は `cliarg_count_result` / `cliarg_get_checked` に集約し、C string helper は `std/env/cliarg/cstr` を明示 import する境界へ分けた。
- 2026-05-15: `ISS-20260514T190700987Z-SELF-HOST-CLI-ARGS-PARSER-READS-VEC--5193AE7F` を解決した。`stdlib/neplg2/cli/args/parse.nepl` は `core/mem` / `core/mem/raw` を import せず、`selfhost_cli_arg_at` / `selfhost_cli_parse_loop` は borrowed `Vec<str>` と `v::get<str>` / `v::len<str>` だけで CLI token を観測する。parser caller 側の `Vec<str>` owner obligation は doctest 側で明示的に解放し、Resource IR regression で `Vec.get<str>` が Copy read として owner storage を移動しないことを固定した。focused parser suite の合計 compile time 問題は `ISS-20260514T193353066Z-SELFHOST-CLI-ARG-PARSER-DOCTEST-SUIT-CF8C1BA8` に分離した。
- 2026-05-15: `ISS-20260514T195643983Z-KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX-416FBAB5` を解決した。`kp/kpprefix` は `PrefixI32` を raw `i32` pointer / len の Copy handle として公開するのをやめ、`data <Vec<i32>>` の owner handle に変更した。`prefix_build_i32` / `prefix_range_sum_i32` の public raw address API は互換 alias を残さず削除し、公開面は `prefix_build_vec_i32(Vec<i32>) -> Result<PrefixI32, Diag>` と `prefix_sum_i32(&PrefixI32, i32, i32) -> Result<i32, Diag>` に揃えた。内部も `vec::filled` / `vec::get` / `vec::replace` / `vec::free` だけを使うため、KP prefix helper は raw-memory-boundary module ではなく typed Vec owner boundary の利用者になった。
- 2026-05-15: `ISS-20260514T200755109Z-KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O-953345F8` を解決した。`kp/kpfenwick` は raw allocation handle を public `i32` として返す API をやめ、`Fenwick` owner、`FenwickAddError`、borrowed query `Result` に揃えた。`kp/kpdsu` も raw parent/size handle を廃止し、`DisjointSet` owner と `DisjointSetUpdateError` を使う API に移した。どちらも ordinary caller が raw storage identity を保持しないため、owner/free obligation が public 型に残る。
- 2026-05-15: `ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB` を解決した。`kp/kpgraph` は `DenseGraph.mat <i32>` と `dense_graph_bfs_dist_raw(n, mat, start)` を削除し、`DenseGraph` を `AdjacencyMatrix` owner wrapper にした。構築・読み込み・BFS は `Result` で失敗を返し、BFS の距離配列と queue も `Vec<i32>` API だけで扱う。doctest も returned Vec の raw storage を読まず、`v::get<i32>` で stdout を出す。
- 2026-05-15: `ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D` を解決した。`kp/kpsearch` は raw memory import、`mem_ptr_addr data_mem_ptr`、raw `i32` helper を削除し、query API を `&Vec<i32>` borrowed read に変更した。二分探索は `Vec.len` / `Vec.get` だけで実装し、`unique_sorted_vec_i32` は owner-consuming のまま `Vec.get` / `Vec.replace` で in-place compaction する。
- 2026-05-15: `ISS-20260514T204735670Z-VEC-SORT-FACADE-RE-EXPORTS-RAW-MEMPT-6646B4EF` を解決した。canonical `alloc/collections/vec/sort` facade は raw `MemPtr` helper と `sort_i32` raw slice adapter を再公開せず、ordinary caller には safe `Vec` sort API と `sort_is_sorted` observer だけを見せる。unchecked traversal は `sort/raw/access` / `sort/raw/quick` / `sort/raw/heap` の explicit import 境界へ移した。
- 2026-05-15: `ISS-20260514T211956079Z-OWNER-AGGREGATE-BOUNDARY-TREATS-QUAL-8D858CD3` を解決した。compiler の `OwnerAggregateBoundary` source capability 判定は、`Result::Ok` / `Option::Some` のような qualified enum variant を owner aggregate constructor evidence として扱わないようにした。constructor evidence は unqualified constructor-like symbol に限定し、field accessor evidence は explicit helper として維持する。これにより、ordinary enum construction が owner-backed aggregate constructor / owner-token field projection の capability を過大付与する経路を閉じた。
- 2026-05-15: `ISS-20260514T212804383Z-OWNER-AGGREGATE-CONSTRUCTOR-AND-OWNE-58143AB3` を解決した。`OwnerAggregateBoundary` を `OwnerAggregateConstructorBoundary` と `OwnerAggregateFieldBoundary` に分離し、source evidence と許可操作の対応を細かくした。constructor evidence だけでは owner token field projection を許可せず、field accessor evidence だけでは owner-backed aggregate direct constructor を許可しない。
- 2026-05-15: `ISS-20260515T020307026Z-OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI-91ECE78D` を解決した。`OwnerAggregateConstructorBoundary` は file-wide な bool ではなく constructor 名付き capability とし、loader は `Vec` evidence なら `Vec` だけ、`Diag` evidence なら `Diag` だけを記録する。typecheck の owner-backed aggregate constructor gate も実際に適用中の constructor 名を照合するため、同じ compiler-owned stdlib source に unrelated constructor evidence があっても `Vec` / `HashMap` / owner wrapper の direct constructor までは許可されない。
- 2026-05-15: `ISS-20260515T023829013Z-CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO-EC25363F` を解決した。`alloc_ptr` / `alloc_region` / `dealloc_region` などの checked owner wrapper は safe public API であり、これを使うだけでは raw-memory-boundary source evidence としない。raw authority は actual raw operation、raw address identity helper、restricted compiler memory constructor、raw address intrinsic に限定する。
- 2026-05-15: `ISS-20260515T024851827Z-RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W-2A06192D` を解決した。raw memory source capability は `RawMemoryStructuralBoundary`、`RawMemoryOperationBoundary(RawMemoryOp)`、`RawBodyMemoryOperationBoundary(RawBodyMemoryOp)` に分かれた。raw address identity helper / `MemPtr` / `RegionToken` constructor は structural capability だけを付与し、`load` / `store` / `alloc` などの actual raw helper や intrinsic は operation enum で記録する。`#wasm` / `#llvm` body の memory instruction も backend operation enum で記録し、typecheck と ResourceEffectBoundary diagnostic suppression は使用中の operation と capability を照合する。これにより、同じ compiler-owned stdlib source 内でも `load` evidence が `store` を許可したり、structural raw address helper が raw load/store suppression を広げたりしない。
- 2026-05-15: `ISS-20260515T110646911Z-CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK-B846CF4C` を解決した。public raw escape diagnostic と internal raw identity summary の filter を分離し、`str` のような opaque high-level owner は summary から隠したまま、`RegionToken` の allocator-derived owner provenance は `RawIdentityReturnSummary` に残す。これにより `alloc_region -> Result::Ok(RegionToken)`、`region_ptr_at -> Result::Ok(MemPtr)`、callback-returned `MemPtr` の経路を checked `store` / `load` / `fill` が Resource IR 上で証明できる。公開面では `RegionToken` は引き続き owner-protected なので raw address escape diagnostic は出ない。
- 2026-05-15: `ISS-20260515T153348188Z-PUBLIC-MEM-PTR-ADD-BYPASSES-REGION-B-F82F9BBB` を解決した。`mem_ptr_add` は `RegionToken` 由来の `MemPtr` から任意 offset の `MemPtr` を作れるため、safe caller が `region_ptr_at` の bounds / alignment 証明を迂回できた。Resource IR lowering は `RawAddressViewKind::MemPtrOffset` を導入し、一般的な raw address arithmetic である `Offset` と、public `MemPtr` を作る pointer arithmetic を enum 上で分ける。effect boundary は `MemPtrOffset` だけを raw structural boundary operation として診断し、compiler-owned raw-memory-boundary source 以外では `resource.raw.memory_outside_boundary` とする。`RawAddressViewKind::NonOwningProjection` は検査済み projection として別 enum variant のまま扱い、`region_ptr_at` の Ok payload や `region_ptr` 由来 view と混同しない。
- 2026-05-15: `ISS-20260514T215003679Z-VEC-EMPTY-CONSTRUCTOR-ACCEPTS-NON-CO-258C7574` を解決した。`vec_empty<T>` は allocation を行わなくても public `Vec<T>` owner aggregate を構築するため、現行の Copy-only collection cleanup contract では例外にせず `.T: Copy` に限定した。`Vec<NonCopyPayload>` の empty construction は `type.trait_bound.unsatisfied` で拒否される。
- 2026-05-15: `ISS-20260514T223113919Z-STD-TEXT-ROOT-RE-EXPORTS-RAW-UTF-8-M-7F3A2723` を解決した。`std/text` root は checked `ByteBuf -> str` conversion だけを再公開し、raw `MemPtr<u8>` based validation / decode helper は `std/text/validate` / `std/text/decode` の explicit import 境界へ閉じた。invalid UTF-8 doctest fixture も raw address store をやめ、checked `MemPtr` store と owner cleanup で構成する。
- 2026-05-16: `ISS-20260515T155626605Z-REGIONTOKEN-STILL-STORES-MEMPTR-AS-O-C0F9E4D1` を解決した。`RegionToken<T>` は `ptr: MemPtr<T>` を持たず、`raw: i32, size: i32` の owner token layout になった。`MemPtr<T>` は `region_ptr<T>(&RegionToken<T>)` や `region_ptr_at<T,U>` が返す non-owning projection に限定し、Resource IR lowering / initialized summary / raw-address return summary は direct `RegionToken.raw` を owner identity として扱う。owner summary seed は `dealloc_region -> dealloc_ptr` のような callee summary 経由の raw owner consumption も見るため、direct raw field 化しても free obligation consumption が関数境界で失われない。Stage 6 の `MemPtr` owner-like field policy は transitional allowlist 0 件になった。
- 2026-05-16: `ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719` を解決した。`std/stdio/read` の read_all / read_line は main buffer と fd_read scratch を direct `alloc_ptr` / `dealloc_ptr` / `realloc_ptr` で扱わず、`RegionToken<u8>` owner と `region_ptr` non-owning view に分離した。`stdio_finish_read_buffer` / `stdio_discard_read_buffer` も `RegionToken<u8>` owner を消費する helper へ変更し、cleanup branch から unsafe `unreachable` を削除した。read/text doctest は ByteBuf-to-str 側の `string_from_mem_unchecked_result` owner transfer 残件で止まるため、`ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535` に分離した。
- 2026-05-16: `ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686` を解決した。`stdlib/core/mem/pointer/alloc.nepl` を削除し、`alloc_ptr` / `realloc_ptr` / `dealloc_ptr` が public / direct import 可能な `MemPtr<T>` owner API として残る経路を閉じた。`alloc_region_bytes` / `realloc_region_bytes_keep` は `RegionToken<T>` owner と direct `RegionToken.raw` / `RegionToken.size` field に基づく owner-consuming boundary へ揃え、`MemPtr<T>` は non-owning view としてだけ扱う。Resource IR には `OwnerStorageExtent::RegionTokenSize` と projection owner return への consumed extent requirement 適用を追加し、`region_new` summary 経由でも raw owner extent と token size の対応を証明する。
- 2026-05-16: `ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134` を解決した。`VecStorageState` と split `region: RegionToken<T>` field を廃止し、`VecStorage<T>::Empty | Owned(RegionToken<T>)` へ移行した。`Vec<T>` は `len/cap/storage` の 3 field になり、free obligation owner は `Owned` variant payload にだけ存在する。`vec_free_storage<T>` は owner-carrying enum を消費し、`Empty` branch の no-op と `Owned` branch の `dealloc_region` を source type と `match` の網羅性で証明する。これを実用化するため、borrowed enum match は `&VecStorage<T>` payload を `&RegionToken<T>` として束縛し、match typecheck は期待型に引きずられて borrowed scrutinee を owned enum と誤推論しないよう retry / rollback を持つ。owner-backed aggregate field gate は field type 基準へ絞り、metadata projection は許可しつつ owner field extraction は拒否する。
- 2026-05-16: `ISS-20260516T010329239Z-RESOURCE-PROOF-PRIMITIVE-CLASSIFICAT-12B44B46` を解決した。compiler memory type と memory helper primitive の分類を `resource_primitives` registry に集約し、`MemPtr` / `RegionToken` / `region_*` / `mem_ptr_*` の Resource IR / source capability direct string 判定を registry query へ置換した。これは stdlib module ごとの allowlist ではなく、型解決済みの compiler memory type と helper primitive enum を通す proof boundary である。今後 `OwnedBuffer` / `OwnedRegion` などを追加する場合も registry の enum / query へ追加し、個別 module ごとの証明器を増やさない。

### Stage 7: 旧 summary の削除

目的: 複雑化の原因になっていた HIR 個別 summary を取り除く。

作業:

- raw alias / enum payload alias / aggregate field alias / function value raw effect summary を Resource IR summary へ統合する。
- `move_check.rs` の旧 state map を削除する。
- `drop_insertion` を Resource IR drop elaboration へ統合する。

commit 単位:

1. old summary read path の停止。
2. old summary 型の削除。
3. old move_check / drop_insertion の統合削除。

## Issue 整理方針

| issue | 位置づけ | 完了条件 |
|---|---|---|
| `RV-CORE-002` | Stage 1 の親 issue。module 境界と責務分離を追跡する。 | `typecheck.rs` / `move_check.rs` の主要責務が module 化され、focused regression が維持される。 |
| `RV-CORE-009` | Stage 2-4 の親 issue。Resource IR と resource check を追跡する。 | Resource IR 上で move/borrow/lifetime/drop obligation を検査し、旧 checker 依存を除去する。 |
| `CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS` | Stage 5 の compiler-core issue。raw memory effect / ownership boundary を追跡した。 | 2026-05-13 に core 側は verified / resolved。stdlib public API migration は Stage 6 の stdlib issue へ分離する。 |
| `MEMPTR-AND-REGIONTOKEN` | Stage 3/4 の compiler-core issue。`MemPtr` / owner token / initialized cell の compiler 側分離を追跡した。 | 2026-05-13 に core 側は fixed / resolved。stdlib public API と collection/string/self-host buffer の移行は Stage 6 の stdlib issue へ分離する。 |
| `COPY-IMPL-CAN-MARK-COMPILER-OWNER` | Stage 4 の compiler-core issue。owner token の線形性を trait capability layer で崩せる経路を追跡した。 | 2026-05-14 に fixed。`StructConstructorPolicy::RawMemoryBoundaryOnly(OwnerToken)` に基づき compiler owner token への `Copy` impl を拒否する。 |
| `OWNER-BACKED-AGGREGATE-CONSTRUCTORS` | Stage 6 の compiler-core issue。owner token を direct field に持つ aggregate の forge / projection 経路を追跡した。 | 2026-05-15 に fixed。型形状から owner-backed aggregate を分類し、constructor と owner-token field projection を owner aggregate boundary へ限定する。 |
| `OWNER-BACKED-AGGREGATE-POLICY-DOES-N` | Stage 6 の compiler-core issue。owner-backed aggregate policy が nested owner field に伝播しない問題を追跡した。 | 2026-05-15 に fixed。fixed-point 構造判定で `Vec` wrapper、HashMapStorage、HashMap など二段目以降の owner-backed aggregate constructor も boundary 外で拒否する。 |
| `OWNER-BACKED-AGGREGATE-FIELD-PROJECT` | Stage 6 の compiler-core issue。constructor 側で閉じた owner-backed aggregate を field projection から取り出せる問題を追跡した。 | 2026-05-15 に fixed。field access も同じ構造判定を使い、`HashMap.storage` や `Vec` wrapper field の projection を owner aggregate field boundary 外で拒否する。 |
| `GENERIC-OWNER-BACKED-AGGREGATE-CONST` | Stage 6 の compiler-core issue。generic type application 後に owner-backed になる aggregate constructor を追跡した。 | 2026-05-15 に fixed。constructor result の applied type を同じ構造判定へ通し、`OwnerBox<Vec<i32>>` のような generic wrapper constructor も boundary 外で拒否する。 |
| `OWNER-AGGREGATE-BOUNDARY-TREATS-QUAL` | Stage 6 の compiler source capability issue。qualified enum variant を owner aggregate constructor evidence と誤分類していた。 | 2026-05-15 に fixed。constructor evidence は unqualified symbol に限定し、`Result::Ok` / `Option::Some` だけでは owner aggregate constructor capability を付与しない。 |
| `OWNER-AGGREGATE-CONSTRUCTOR-AND-OWNE` | Stage 6 の compiler source capability issue。constructor と owner field projection が 1 capability を共有していた。 | 2026-05-15 に fixed。`OwnerAggregateConstructorBoundary` と `OwnerAggregateFieldBoundary` へ分離し、evidence kind と許可操作を対応させる。 |
| `OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI` | Stage 6 の compiler source capability issue。constructor evidence が file-wide で別 constructor へ過大適用されていた。 | 2026-05-15 に fixed。`OwnerAggregateConstructorBoundary(String)` として constructor 名ごとに記録し、typecheck は適用中の constructor 名と照合する。 |
| `CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO` | Stage 6 の compiler source capability issue。checked owner wrapper 呼び出しだけで raw boundary authority が付与されていた。 | 2026-05-15 に fixed。`alloc_ptr` / `alloc_region` / `dealloc_region` など safe wrapper は raw evidence から外し、actual raw operation / raw address identity helper / restricted constructor を evidence とする。 |
| `RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W` | Stage 6 の compiler source capability issue。raw operation と raw body instruction が file-wide raw boundary に畳まれていた。 | 2026-05-15 に fixed。raw structural boundary、`RawMemoryOp`、`RawBodyMemoryOp` を別 capability にし、typecheck / ResourceEffectBoundary suppression は実際の operation を照合する。raw identity escape も checked wrapper 名ではなく発生元 `RawMemoryOp` を保持して照合する。 |
| `CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK` | Stage 6 の Resource IR provenance issue。public escape filter と internal summary filter を共有していたため、`RegionToken` 返り値 provenance が checked `MemPtr` 証明へ届かなかった。 | 2026-05-15 に fixed。public raw escape では `RegionToken` を owner-protected のまま扱い、internal `RawIdentityReturnSummary` には allocator-derived owner provenance を残して `region_ptr` / `region_ptr_at` / callback 経由の checked access を証明する。 |
| `PUBLIC-MEM-PTR-ADD-BYPASSES-REGION-B` | Stage 6 の Resource IR raw address view boundary issue。public `mem_ptr_add` が `region_ptr_at` の bounds / alignment proof を迂回できた。 | 2026-05-15 に fixed。`RawAddressViewKind::MemPtrOffset` を導入し、compiler-owned raw-memory-boundary source 以外では `resource.raw.memory_outside_boundary` として拒否する。 |
| `REGIONTOKEN-STILL-STORES-MEMPTR-AS-O` | Stage 6 の core/mem owner-token layout issue。`RegionToken<T>` が最後の `MemPtr<T>` owner-like field を保持していた。 | 2026-05-16 に fixed。`RegionToken<T>` は direct `raw: i32` owner identity と `size: i32` を持ち、`MemPtr<T>` は checked non-owning projection としてだけ構築する。Resource IR owner summary は callee summary 経由の raw owner consumption も seed し、`dealloc_region` の owner consumption を関数境界で証明する。 |
| `PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F` | Stage 6 の core/mem public API issue。`alloc_ptr` / `realloc_ptr` / `dealloc_ptr` が `MemPtr<T>` を free obligation carrier として公開していた。 | 2026-05-16 に fixed。`stdlib/core/mem/pointer/alloc.nepl` を削除し、safe facade と direct import の両方から public `MemPtr<T>` allocation owner API を撤去した。`RegionToken<T>` owner + `MemPtr<T>` non-owning projection の境界に揃えた。 |
| `VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE` | Stage 6 の Vec owner-state issue。`VecStorageState` と `RegionToken<T>` が split field だったため、`Empty` と allocated token の相関を source type が証明できなかった。 | 2026-05-16 に fixed。`VecStorage<T>::Empty | Owned(RegionToken<T>)` に移行し、`Vec<T>` は `len/cap/storage` だけを持つ。borrowed enum match により observer は `&RegionToken<T>` payload を参照でき、`vec_free_storage` は owner-carrying enum の `match` で free obligation を閉じる。 |
| `STDIO-READ-BYTES-STILL-USES-MEMPTR-O` | Stage 6 の stdlib scratch owner issue。`std/stdio/read` の read_all / read_line buffer と fd_read scratch が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。read buffer / scratch を `RegionToken<u8>` owner に移し、finish/discard helper は owner token を消費する。text conversion 側の string owner transfer は `READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO` で同日に解決した。 |
| `READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO` | Stage 6 の Resource IR / string owner transfer issue。stdio read owner 移行後も ByteBuf-to-str が `string_from_mem_unchecked_result` の owner leak で止まっていた。 | 2026-05-16 に fixed。`string_finish` が `RegionToken.raw` を直接取り出し、同じ raw owner に長さを書いた後で `string_from_addr_unchecked` へ渡すため、`RegionToken` owner から `str` owner への確定境界を Resource IR が追える。 |
| `RESOURCE-OWNER-SUMMARY-LOSES-NESTED` | Stage 6 の Resource IR owner summary issue。callee summary が参照する nested `Result` payload owner を caller 側で materialize できず、owner-preserving helper が false leak になっていた。 | 2026-05-15 に fixed。`PendingVariantOwnerEffects` が helper summary の target place を基準に pending variant owner return を materialize し、stale pending consumption / return を除去する。 |
| `OWNER-AGGREGATE-FIELD-EVIDENCE-IGNOR` | Stage 6 の compiler source capability issue。`get_field_ref` / `get_field` intrinsic と same-module struct constructor evidence を source scanner が落としていた。 | 2026-05-15 に fixed。builtin field accessor intrinsic を `OwnerAggregateFieldBoundary` evidence として収集し、top-level type definition を value shadow として扱わない。 |
| `CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE` | Stage 5/6 の stdlib public API issue。 | 2026-05-13 に fixed。safe `core/mem` import から raw address escape は呼べない。direct internal/raw module と raw-memory-backed stdlib 全体の discipline は Stage 6 parent で継続する。 |
| `CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE` | Stage 4 の compiler-core drop obligation issue。 | 2026-05-13 に core 側は fixed / resolved。initialized payload を残した storage-only free は拒否され、collection element cleanup と raw-memory-backed public API migration は Stage 6 の stdlib issue へ分離する。 |
| `STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR` | Stage 6 の stdlib migration parent。 | raw-memory-backed implementation が safe public discipline を漏らさない。 |
| `VECDATALEN-CARRIES-RAW-VEC-STORAGE` | Stage 6 の Vec raw storage view issue。 | 2026-05-14 に fixed。`VecDataLen` / `data_len` を削除し、`MemPtr` owner-like field baseline を 5 件へ下げる。 |
| `STRINGBUILDER-DUPLICATES-BYTEBUILDER` | Stage 6 の StringBuilder owner boundary issue。 | 2026-05-14 に fixed。StringBuilder を ByteBuilder wrapper にし、`MemPtr` owner-like field baseline を 4 件へ下げる。 |
| `BYTEBUF-STORES-OWNED-BYTES-AS-OPTION` | Stage 6 の byte buffer owner boundary issue。 | 2026-05-14 に fixed。ByteBuf / ByteBuilder を RegionToken owner boundary へ移し、`MemPtr` owner-like field baseline を 2 件へ下げる。 |
| `VEC-STORES-BACKING-STORAGE-AS-MEMPTR` | Stage 6 の Vec owner boundary issue。 | 2026-05-14 に fixed。Vec storage owner を RegionToken field へ移し、`MemPtr` owner-like field baseline を `RegionToken.ptr` だけの 1 件へ下げる。 |
| `BYTEBUILDER-GROW-CLEANUP-STILL-USES` | Stage 6 の byte buffer cleanup issue。 | 2026-05-14 に fixed。grow failure cleanup を `dealloc_region` owner-token consumption へ揃え、通常 stdlib 実装の `unreachable` を排除する。 |
| `RV-STDLIB-004` | Stage 6 の collection API issue。 | collection drop / remove / borrowed read / Copy read の責務が分離される。 |
| `HASHMAP-AND-HASHSET-CLEANUP-CONTRACT` | Stage 6 の hash collection cleanup contract child issue。 | 2026-05-15 に fixed。HashMap / HashSet の `free` が key / value / hasher Copy-only contract を維持することを doctest と source policy で固定した。 |
| `HASHMAP-AND-HASHSET-ROOT-FACADES-RE` | Stage 6 の hash collection facade child issue。 | 2026-05-15 に fixed。HashMap / HashSet root facade が internal storage/probe/rehash helper を再公開しないことを source policy と compile-fail doctest で固定した。 |
| `VEC-STORAGE-FACADE-RE-EXPORTS-ALLOCA` | Stage 6 の Vec storage facade child issue。 | 2026-05-15 に fixed。Vec root/storage facade が allocation / storage-only cleanup helper を再公開しないことを source policy と compile-fail doctest で固定した。 |
| `BLOOM-FILTER-FREE-DROPS-HASHER-WITHO` | Stage 6 の BloomFilter cleanup child issue。 | 2026-05-15 に fixed。`BloomFilter.free` / `CountingBloomFilter.free` を `.T: HashKey&Copy,.H: Hasher<.T>&Copy` に揃え、Drop traversal なしに non-Copy hasher を破棄する経路を閉じる。 |
| `BLOOMFILTER-CLEAR-ACCEPTS-UNCONSTRAI` | Stage 6 の BloomFilter mutating API child issue。 | 2026-05-15 に fixed。`BloomFilter.clear` / `CountingBloomFilter.clear` を `.T: HashKey&Copy,.H: Hasher<.T>&Copy` に揃え、forged non-Copy hasher aggregate を clear だけ通す経路を閉じる。 |
| `BYTEBUF-AND-BYTEBUILDER-EXPOSE-EMPTY` | Stage 6 の byte buffer empty sentinel issue。 | 2026-05-15 に fixed。zero-size `RegionToken<u8>` sentinel helper を private にし、公開 API を typed empty constructor へ限定する。 |
| `FS-DIR-READER-STILL-DEPENDS-ON-RAW-V` | Stage 6 の fs dir reader migration issue。 | 2026-05-15 に fixed。`fs_sort_strings` を `&Vec<str>` public boundary に移し、`std/fs/dir/read_fd.nepl` から旧 `Vec.data` raw storage 依存を削除した。 |
| `VEC-IN-PLACE-SORT-APIS-KEEP-PURE-EFF` | Stage 6 の Vec sort effect contract issue。 | 2026-05-15 に fixed。backing storage を書き換える sort helper / public in-place sort / raw slice sort / owner-returning sort wrapper を impure `*>` signature へ揃え、observer helper は pure のまま分離した。 |
| `VEC-ROOT-FACADE-RE-EXPORTS-RAW-ELEME` | Stage 6 の Vec public/raw facade split issue。 | 2026-05-15 に fixed。root `alloc/collections/vec` から unchecked `vec/raw` re-export を外し、raw element helper は explicit `alloc/collections/vec/raw` import 境界へ閉じた。 |
| `VEC-SORT-FACADE-RE-EXPORTS-RAW-MEMPT` | Stage 6 の Vec sort public/raw facade split issue。 | 2026-05-15 に fixed。canonical `alloc/collections/vec/sort` から raw `MemPtr` sort helper と `sort_i32` raw slice adapter を外し、unchecked traversal は explicit `sort/raw/*` import 境界へ閉じた。 |
| `VEC-SORT-MERGE-SOURCE-POLICY-STILL-E` | Stage 6 の Vec sort/merge source policy issue。 | 2026-05-15 に fixed。`sort/merge` facade は `merge/api` だけを再公開し、raw `merge/buffer` / `merge/range` helper を root facade へ戻さないことを固定した。 |
| `VEC-EMPTY-CONSTRUCTOR-ACCEPTS-NON-CO` | Stage 6 の Vec empty constructor contract issue。 | 2026-05-15 に fixed。`vec_empty<T>` を `.T: Copy` に限定し、allocation なしの Empty state でも unsupported `Vec<NonCopyPayload>` owner aggregate を safe surface へ出さない。 |
| `VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE` | Stage 6 の Vec / ByteBuilder RegionToken realloc boundary issue。 | 2026-05-15 に fixed。`core/mem` に owner-preserving `realloc_region_bytes_keep<T>` を追加し、`Vec.push` の grow capacity を payload 上限で証明してから realloc するようにした。同時に Resource IR owner checker が非所有 `MemPtr` 集約値の raw-address alias を let/read/assign 越しに保持し、Ok / Err 両 variant の free obligation owner を証明できるようにした。 |
| `STD-FS-AND-STDIO-ROOT-FACADES-RE-EXP` | Stage 6 の std fs/stdio public/raw facade split issue。 | 2026-05-15 に fixed。root `std/fs` と `std/stdio` から raw ABI submodule re-export を外し、WASI / LLVM syscall helper は explicit raw submodule import 境界へ閉じた。 |
| `STD-FS-FD-WRITE-SCRATCH-STILL-USES-M` | Stage 6 の std/fs fd_write scratch owner issue。`std/fs/write/fd.nepl` の iovec / nwritten scratch が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。fd_write scratch を `RegionToken<u8>` owner に移し、`fs_fd_write_from_result` へ渡す値は non-owning view だけにした。raw ABI layout は `std/fs/raw/fd_io.nepl` に閉じる。 |
| `STD-FS-OPEN-FD-OUT-SCRATCH-STILL-USE` | Stage 6 の std/fs fd lifecycle scratch owner issue。`std/fs/fd.nepl` の path_open fd_out scratch が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。fd_out scratch を `RegionToken<u8>` owner に移し、raw fd_out address は `fs_open_with_flags` 内の non-owning view からだけ得る。 |
| `STD-FS-STAT-BUFFER-STILL-USES-MEMPTR` | Stage 6 の std/fs stat scratch owner issue。`std/fs/stat.nepl` の path_filestat_get out-buffer が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。filestat scratch を `RegionToken<u8>` owner に移し、raw filestat address は `fs_path_filetype` 内の non-owning view からだけ得る。依存先の `Result<Vec<i32>, i32>` raw identity false positive は `VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID` に分離した。 |
| `STD-FS-FD-READ-SCRATCH-STILL-USES-ME` | Stage 6 の std/fs fd_read scratch owner issue。`std/fs/read/fd.nepl` の growable read buffer / iovec / nread scratch が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。read buffer と scratch を `RegionToken<u8>` owner に移し、raw ABI helper へ渡す値は `region_ptr` 由来の non-owning view だけにした。finish/discard helper も `RegionToken<u8>` を消費する。 |
| `STD-FS-DIR-READ-SCRATCH-STILL-USES-M` | Stage 6 の std/fs fd_readdir scratch owner issue。`std/fs/dir/read_fd.nepl` の dirent buffer / used out-pointer scratch が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。fd_readdir buffer と scratch を `RegionToken<u8>` owner に移し、raw ABI address は `region_ptr` 由来の non-owning view だけにした。 |
| `STD-FS-LLVM-CSTR-SCRATCH-STILL-RETUR` | Stage 6 の std/fs LLVM fallback C string owner issue。`std/fs/raw/llvm.nepl` の C string scratch helper が `Result<MemPtr<u8>, i32>` を返していた。 | 2026-05-16 に fixed。C string scratch を `RegionToken<u8>` owner に移し、syscall address は `region_ptr` 由来の non-owning view だけにした。 |
| `STD-ENV-CLIARG-RAW-SCRATCH-STILL-USE` | Stage 6 の std/env cliarg raw argv scratch owner issue。`std/env/cliarg/raw.nepl` の metadata / argv / LLVM cmdline scratch が `MemPtr<u8>` owner API に依存していた。 | 2026-05-16 に fixed。argv raw boundary scratch を `RegionToken<u8>` owner に移し、raw ABI と checked byte access は `region_ptr` 由来の non-owning view だけにした。 |
| `STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N` | Stage 6 の std/env cliarg memory-safety issue。`cliarg_get_checked` が負 index を拒否せず raw slot 計算へ進む。 | 2026-05-16 に fixed。raw helper 自体にも `idx < 0` gate を追加し、public facade を迂回しても argv slot address を負 offset で計算しない。 |
| `STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM` | Stage 6 の std/env cliarg doctest issue。`cstr.nepl` doctest が ordinary source から `mem_ptr_add` / `store_u8` を使う。 | 2026-05-16 に fixed。NUL を含む string literal の `string_data_ptr` を使う fixture に直し、ordinary doctest から raw memory write を行わない。 |
| `VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID` | Stage 6 の Resource IR raw identity false positive。`Result<Vec<i32>, i32>` owner return が `resource.raw.identity_escape` になる。 | 2026-05-16 に fixed。`RawIdentityTable` が descendant projection を aggregate root へ粗く持ち上げていたため、`RegionToken.raw` identity が `Vec.len` など public scalar field へ混入していた。raw identity transfer を projection 精度へ戻し、`i32` / `MemPtr` raw address leaf の拒否は維持した。 |
| `FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE` | Stage 6 の Resource IR owner summary 残件。raw identity false positive 修正後、`std/fs/path/normalize` の `Result<Vec<T>, E>` / `Result<StringBuilder, E>` owner return が `resource.owner.maybe_leak` になっていた。 | 2026-05-16 に fixed。callee owner return summary を raw owner alias walk へ反映し、wrapper call 経由の `StringBuilder -> ByteBuilder -> RegionToken` 消費と `Result` payload owner return を source-level proof として接続した。 |
| `RESOURCE-PROOF-PRIMITIVE-CLASSIFICAT` | Stage 6 の compiler-core registry issue。memory primitive role 判定が複数 checker の string match に分散していた。 | 2026-05-16 に fixed。`resource_primitives` に compiler memory type / memory helper primitive registry を置き、Resource IR lowering / owner summary / source capability evidence は typed enum / query を通す。stdlib module allowlist は追加していない。 |
| `VEC-STORES-BACKING-STORAGE-DIRECTLY` | Stage 6 の Vec facade / backing storage owner issue。`Vec<T>` が `len/cap/storage` を直接持っていた。 | 2026-05-16 に fixed。`Vec<T>` を `buffer: OwnedBuffer<T>` facade にし、`OwnedBuffer<T>` が `len/cap/storage` と `VecStorage<T>::Empty/Owned(RegionToken<T>)` を保持する。source policy は旧 direct field と旧 constructor を拒否する。 |
| `STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B` | Stage 6 の std env cliarg public/raw facade split issue。 | 2026-05-15 に fixed。root `std/env/cliarg` から raw argv scratch / out pointer 実装を外し、explicit `std/env/cliarg/raw` と `std/env/cliarg/cstr` 境界へ分離した。 |
| `SELF-HOST-CLI-ARGS-PARSER-READS-VEC` | Stage 6 の self-host CLI args parser / `Vec<str>` observer issue。 | 2026-05-15 に fixed。parser は raw `Vec` storage を走査せず、borrowed `Vec<str>` と public `v::get` / `v::len` だけで CLI token を読む。 |
| `KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX` | Stage 6 の KP prefix raw owner boundary issue。 | 2026-05-15 に fixed。`PrefixI32` を `Vec<i32>` owner handle にし、copyable raw pointer owner と public raw address prefix API を削除した。 |
| `KPFENWICK-AND-KPDSU-EXPOSE-RAW-I32-O` | Stage 6 の KP Fenwick / DSU raw owner handle issue。 | 2026-05-15 に fixed。`Fenwick` / `DisjointSet` owner API に移し、public raw `i32` handle と raw memory helper import を削除した。 |
| `KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32` | Stage 6 の KP graph dense matrix raw owner issue。 | 2026-05-15 に fixed。`DenseGraph` を `AdjacencyMatrix` owner wrapper にし、raw matrix pointer API と raw BFS API を削除した。 |
| `KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E` | Stage 6 の KP graph source policy issue。 | 2026-05-15 に fixed。旧 raw BFS helper の存在を要求する stale policy を廃止し、typed owner BFS API の `Result` / `Vec` / cleanup contract を検査する。 |
| `KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC` | Stage 6 の KP search raw storage view issue。 | 2026-05-15 に fixed。query API を borrowed `&Vec<i32>` にし、raw memory import/helper と `mem_ptr_addr data_mem_ptr` 依存を削除した。 |
| `DIAG-RENDERER-READS-DIAGS-VEC-STORAG` | Stage 6 の diagnostic renderer / Diags storage boundary issue。 | 2026-05-15 に fixed。`alloc/diag/diag.nepl` の `diags_to_string` を raw `Vec` storage scan から `v::len` / `v::get` の Copy-safe borrowed observer へ移し、renderer file から raw memory import を削除した。 |
| `DIAGS-ERROR-OBSERVER-SCANS-VEC-STORA` | Stage 6 の Diags read-only observer / storage boundary issue。 | 2026-05-15 に fixed。`alloc/diag/error/diags.nepl` の `diags_has_errors` を raw `Vec` storage scan から `v::len` / `v::get` の Copy-safe borrowed observer へ移し、Diags owner helper から raw memory import を削除した。 |
| `ALLOC-STRING-ROOT-RE-EXPORTS-RAW-STR` | Stage 6 の string public/raw facade split issue。 | 2026-05-15 に fixed。root `alloc/string` から raw storage / UTF-8 memory helper re-export を外し、raw helper 利用は explicit `alloc/string/storage` / `alloc/string/utf8` import 境界へ閉じた。 |
| `ALLOC-STRING-FACADE-SOURCE-POLICY-ST` | Stage 6 の string facade source policy issue。 | 2026-05-15 に fixed。`alloc/string` root policy を safe submodule 再公開だけに更新し、`storage` / `utf8` raw helper が root から再公開されないことを固定した。 |
| `STD-TEXT-ROOT-RE-EXPORTS-RAW-UTF-8-M` | Stage 6 の text public/raw facade split issue。 | 2026-05-15 に fixed。root `std/text` から raw UTF-8 validation / decode helper re-export を外し、checked conversion と raw `MemPtr` helper 境界を分離した。 |
| `STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG` | stdlib facade target contract issue。 | 2026-05-15 に fixed。`std/io` は public API signature に出る `ReadStream` / `WriteStream` を `std/iotarget` から再公開し、target enum 定義は `std/iotarget` に集約する。 |

新しい個別 bug は、次の基準で追加する。

- 現行 checker の false negative / false positive が明確なら、既存 regression child issue として追加する。
- Resource IR 導入でまとめて直すべき構造問題なら、`RV-CORE-009` の子として追加する。
- stdlib API 移行が必要な場合は、compiler issue と混ぜず `STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR` または該当 stdlib issue へ分ける。

## 検証計画

### focused local tests

大規模修正中は、変更箇所に応じて focused test を選ぶ。

| 変更 | local test |
|---|---|
| issue metadata / docs | `node nodesrc/issues.js check` |
| Resource IR 型定義 / lowering | `cargo test -p nepl-core --test move_check`、Resource IR snapshot test |
| move/borrow/lifetime | `cargo test -p nepl-core --test move_check`、`tests/compiler/move_check.n.md` focused run |
| effect / raw memory | `tests/compiler/move_effect.n.md` focused run |
| stdlib memory API | 該当 `tests/stdlib/*.n.md` focused run |

全体 test は GitHub Actions を主に使い、local では変更に関係する範囲に絞る。

### regression 必須カテゴリ

- same-place raw load の二重 move。
- `MemPtr` copy は許可されるが free obligation は複製されない。
- live non-Copy payload を含む storage dealloc / realloc / bulk copy / byte overwrite の拒否。
- enum payload / aggregate field / function return / callback 経由の resource effect 伝播。
- branch / loop merge 後の maybe-moved / maybe-borrowed の保守的検査。
- unique borrow 中の write / move / dealloc 拒否。
- shared borrow 中の mutation 拒否。
- internal allocation が public raw identity を漏らさない場合だけ surface pure へ fold されること。

## self-host への影響

NEPLg2 self-host compiler は、S1/S2 の lexer/parser/module loader など pure data model から進められる。ただし、S3 以降の resource checker、diagnostic buffer、AST arena、token buffer、byte/string builder は、この文書の memory model を前提にする。

self-host 実装側の禁止事項:

- `MemPtr` を owner として保持する新規 public API を増やさない。
- raw address `i32` を compiler data structure の通常 field に持ち込まない。
- drop obligation を stdlib の手作業 cleanup だけで完結させる設計にしない。

許容される移行措置:

- 既存 `Vec` / `StringBuilder` を使った S1/S2 実装。
- raw-backed implementation を internal module に閉じた wrapper として使う。
- Resource IR 導入前の暫定 compiler regression を維持するための保守的 `resource.cell.*` / `resource.owner.*`。

## 2026-04-30 設計確認

[静的検査設計確認 2026-04-30](./static_check_design_verification_20260430.md) で、現行 Rust 実装、self-host 計画、stdlib memory model の整合を再確認した。

[静的検査 soundness review 2026-04-30](./static_check_soundness_review_20260430.md) では、pass 順序、現在の authority、Resource IR gate の hard-error 範囲、旧 HIR checker / shadow-only behavior に残る未完了点を追加で確認した。

判定は次の通り。

- Resource IR の data model、coverage gate、CellState / OwnerState / BorrowState gate、enum-first diagnostic の方向性は妥当である。
- 現行 pipeline は drop 未挿入 source semantics を monomorphize した reachable HIR を Resource IR check に渡し、checked `ResourceDropElaborationPlan` を生成する。実 drop call 生成は `passes::insert_resource_drops` がこの plan を消費して行うため、旧 `passes::move_check::run` fallback と旧 HIR `passes::insert_drops` 呼び出しはいずれも 2026-05-06 に削除済みである。生成 drop が source violation を隠さないよう、Resource IR gate は drop 挿入前の HIR に対して実行する。
- `ResourceCheckDiagnostic::CellUnavailable` と `ResourceOwnerDiagnostic::*` は compiler diagnostic で `resource.cell.*` と `resource.owner.*` に分離済みである。今後も D3100 相当の粗い raw bucket に戻さず、原因分類を enum-first で維持する。
- `UnsafeMemoryInPureFunction` は 2026-05-06 時点で Resource IR gate から `effect.pure.calls_impure` へ error 化済みである。ただし、configured stdlib の `core/mem.nepl`、`alloc/string.nepl`、`alloc/string/storage.nepl` など compiler-owned raw-memory-boundary capability を持つ source では、Stage 6 の stdlib migration が完了するまで移行中許可を維持する。この許可は loader の configured `stdlib_root` から計算した exact path に限定し、任意の同名 suffix path は許可しない。
- self-host の S1/S2 は進められるが、S3 以降の typecheck / Resource IR / diagnostic aggregation では raw header collection や `MemPtr` owner discipline を中核に持ち込まない。

追加精査で、`ResourceDiagnosticCode` 自体は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分離済みであることを確認した。2026-05-06 時点で drop elaboration authority も checked Resource IR plan に統合済みであり、旧 HIR scope walker を維持する方針は残さない。設計上の未完了点は、raw-memory-boundary capability による stdlib 移行中許可が残っていること、stdlib の owner token / collection storage state が compiler-issued capability に揃い切っていないこと、Stage 4 authority path の full review / regression を継続することである。

2026-05-06 の Stage 5 追記として、host effect operation と raw/host effect count は enum-first の Resource IR 表現へ移行済みである。`ExternalIo` / `Nondet` / `UnsafeMemory` は pure function 境界で Resource IR diagnostic から compiler error へ接続される。残件は、raw-memory-backed stdlib の public API を Stage 6 で internal/public 境界へ分け、raw identity と owner token を safe surface へ漏らさない形へ移行することである。

2026-05-16 の Stage 6 追記として、`core/mem` / `mem/pointer` safe facade から `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` の re-export を外した。`mem_ptr_add` は low-level alloc owner wrapper から分離して `pointer/view` に移し、non-owning pointer view helper と storage owner API の責務を分けた。`#import "core/mem" as *` だけでは `MemPtr<T>` owner API へ到達できないことを compile_fail と source policy で固定した。続く `ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686` の解決で direct `core/mem/pointer/alloc` module 自体も削除し、public / direct import 可能な `MemPtr<T>` allocation owner API は残っていない。

同日の追加対応として、`std/stdio/write/byte.nepl` の `print_byte` は 1 byte scratch buffer を `alloc_region<u8>` / `region_ptr` / checked `store_u8` / `dealloc_region<u8>` へ移行した。これは syscall iovec のような ABI raw boundary ではなく、単なる stdout byte adapter の private buffer なので、`MemPtr<u8>` を free owner として扱う必要はない。Stage 6 の残件は `std/stdio/write/fd.nepl` など raw ABI scratch を持つ箇所であり、そこでは owner token と raw ABI view を分けて移行する。

さらに `std/stdio/write/fd.nepl` の fd_write loop も `RegionToken<u8>` owner 境界へ移行した。iovec / nwritten scratch の free obligation は `iov_region` / `nwritten_region` が持ち、raw ABI layout の store/load は `std/stdio/raw.nepl` の `stdio_fd_write_from_result` に閉じる。これにより `std/stdio/write` 経由の stdout/stderr write path は direct `alloc_ptr` owner API を必要としない状態になった。

同日の read 側対応として、`std/stdio/read` の read_all/read_line buffer と fd_read scratch も `RegionToken<u8>` owner 境界へ移行した。続いて `read/text` の `ByteBuf` から `str` への変換で残っていた `resource.owner.maybe_leak` は、`string_finish` が `RegionToken.raw` を `MemPtr` owner wrapper に戻していた stale 境界を削除することで解消した。`string_finish` は raw owner を直接消費してヘッダ長を書き、同じ raw owner を `str` に移すため、Stage 6 の「MemPtr は non-owning pointer、free obligation owner は token/storage 側」という責務分割に沿う。

さらに `std/fs/write/fd.nepl` の fd_write loop も `RegionToken<u8>` owner 境界へ移行した。`fs_write_fd_mem_result` は iovec / nwritten scratch の free obligation を `iov_region` / `nwritten_region` に持たせ、raw ABI layout store/load は `std/fs/raw/fd_io.nepl` の `fs_fd_write_from_result` に閉じた。これにより `std/fs/write` の fd write path は direct `alloc_ptr` / `dealloc_ptr` scratch owner API を必要としない。

同じ std/fs の fd lifecycle 境界として、`std/fs/fd.nepl` の `fs_open_with_flags` も `RegionToken<u8>` owner 境界へ移行した。`path_open` の fd_out scratch は `fd_out_region` が所有し、raw fd_out address の store/load は関数内の `region_ptr` view からだけ行う。これにより `std/fs` facade から再公開される open/close 経路に direct `alloc_ptr` / `dealloc_ptr` scratch owner API を残さない。

`std/fs/stat.nepl` の `fs_path_filetype` も `RegionToken<u8>` owner 境界へ移行した。`path_filestat_get` の 64 byte out-buffer は `stat_region` が所有し、filetype byte の raw store/load は `fs_path_filetype` 内の `region_ptr` view からだけ行う。focused doctest は依存先 `fs_normalize_range_push` の `Result<Vec<i32>, i32>` owner return が `resource.raw.identity_escape` になる core false positive に到達したため、これは `ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD` として別 issue に分離し、Resource IR 側で正確に修正する。

同じく Stage 6 の fd read 境界として、`std/fs/read/fd.nepl` の `fs_read_fd_bytes` も `RegionToken<u8>` owner 境界へ移行した。growable read buffer、iovec、nread はそれぞれ `buf_region` / `iov_region` / `nread_region` が free obligation を持ち、raw ABI helper へ渡す `MemPtr<u8>` は `region_ptr` / `mem_ptr_add` から得る non-owning view に限定した。grow は `realloc_region_bytes_keep<u8>` で旧 owner を失わずに行い、`fs_finish_read_buffer` / `fs_discard_read_buffer` は `RegionToken<u8>` owner を消費する signature に揃えた。これにより `std/fs/read` の fd read path は direct `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` scratch owner API を必要としない。

同じく Stage 6 の directory read 境界として、`std/fs/dir/read_fd.nepl` の `fs_read_dir_fd` も `RegionToken<u8>` owner 境界へ移行した。`fd_readdir` の dirent data buffer と `used` out-pointer scratch は `buf_region` / `used_region` が free obligation を持ち、raw ABI へ渡す address は `region_ptr` から得る non-owning `MemPtr<u8>` view だけにした。entry name の構築・蓄積・sort は `Vec<str>` public API 境界を通すため、directory listing path でも `MemPtr<u8>` を buffer owner として扱わない。

LLVM fallback の C string 境界も同じ Stage 6 方針へ揃えた。`std/fs/raw/llvm.nepl` の `__fs_copy_to_cstr` は `Result<RegionToken<u8>, i32>` を返し、path byte copy と `openat` syscall address 取得は `region_ptr` 由来の non-owning view だけで行う。`wasi_path_open` は syscall 後に `cpath_region` を `dealloc_region<u8>` で閉じるため、`std/fs` 配下に direct `alloc_ptr` / `dealloc_ptr` scratch owner API を残さない。

`std/env/cliarg/raw.nepl` も Stage 6 の同じ境界へ移した。argc metadata、argv pointer array、argv byte buffer、LLVM `/proc/self/cmdline` C string、cmdline temporary buffer は `RegionToken<u8>` owner が free obligation を持ち、`args_sizes_get` / `args_get` / checked byte access へ渡す `MemPtr<u8>` は `region_ptr` 由来の non-owning view に限定する。これで `std/fs` / `std/stdio` / `std/env/cliarg` の主要 raw-backed scratch path は direct low-level allocation owner API を使わない。

同じ確認で `cliarg_get_checked` が負 index を拒否せず raw slot 計算へ進む memory-safety issue と、`std/env/cliarg/cstr.nepl` doctest が ordinary source から `mem_ptr_add` / `store_u8` を使う stale fixture issue を発見した。前者は raw helper 自体に `idx < 0` gate を追加して修正済みであり、public facade を迂回しても argv slot address を負 offset で計算しない。後者も NUL を含む string literal の `string_data_ptr` を渡す fixture に直し、ordinary doctest から raw memory write を行わない形で修正済みである。

`ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD` は 2026-05-16 に修正した。根本原因は、`RawIdentityTable` の prefix replacement が descendant projection の raw identity を返却 aggregate root へ追加していたことである。これにより `RegionToken.raw` / owner descendant の identity が `Result::Ok(Vec).field0` など通常 scalar field に混入し、`Vec` owner return を public raw address escape と誤診断していた。修正後の `std/fs/stat.nepl` focused doctest は raw identity では止まらず、次の `resource.owner.maybe_leak` に進んだため、`Vec` / `StringBuilder` owner payload の return summary 証明残件を `ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4` として分離した。

`ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4` は 2026-05-16 に修正した。根本原因は、Resource IR owner summary の raw owner alias walk が callee の owner return summary を call output へ反映せず、`string_builder_into_byte_builder` の parameter-derived projection return と `byte_builder_free` の consumed projection が wrapper 関数内で接続されなかったことである。修正では direct call の root/projection/variant payload owner return を summary-aware raw alias walk に伝播し、`function_returns_raw_owner_from` も summary-aware にした。`Result::Ok` payload が parameter owner と fresh owner のどちらでも所有者を返す場合は、`Maybe` で leak 扱いにせず `UnknownSource { extent }` として所有者存在を保持する。これは stdlib 関数名 whitelist ではなく、source-level owner transfer と callee summary から導出される proof である。`std/fs/stat.nepl` focused doctest は total=1, passed=1 になった。

`ISS-20260516T005642964Z-VEC-STORES-BACKING-STORAGE-DIRECTLY--2407B1D0` は 2026-05-16 に修正した。Stage 6 / Stage D の進捗として、`Vec<T>` は `buffer: OwnedBuffer<T>` だけを持つ facade になり、`OwnedBuffer<T>` が `len/cap/storage` と `VecStorage<T>::Empty/Owned(RegionToken<T>)` を保持する。observer / mutation / transform / sort wrapper はすべて `OwnedBuffer<T>` 経由で storage owner を参照または消費し、source policy は旧 `Vec<T> len cap storage` constructor と `Vec<T>` 直下の direct field を退行として拒否する。これは stdlib module 名 whitelist ではなく、source 型定義と constructor 形状そのものを検査する退行防止である。残件は `OwnedBuffer<T>` に initialized prefix / moved slot / drop traversal と compiler-issued owner token を接続し、non-Copy payload collection を Copy-only raw helper から分離することである。

`ISS-20260516T020917148Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-ACCE-5E35D33F` は 2026-05-16 に修正した。owner aggregate source capability は、修正前は expression 内の任意の未修飾大文字 symbol を constructor boundary evidence として扱っていた。修正後は evidence 判定を `source_capability/owner_aggregate/evidence.rs` に分け、constructor evidence は prefix expression の call-head に現れた symbol からだけ導出する。同一 module の enum variant 名は `OwnerAggregateEvidenceContext` で除外し、`consume Diag` のような引数位置の大文字値や `Ok 1` のような enum variant call では owner aggregate constructor authority を得られない。これは stdlib module ごとの許可ではなく、compiler-owned source に現れた構文証拠をより正確に抽出する authority gate であり、owner-backed aggregate かどうかの semantic proof は typecheck の構造的 owner token 判定と Resource IR 側に残す。

`ISS-20260516T021926423Z-RAW-MEMORY-SOURCE-EVIDENCE-ACCEPTS-N-88427FD2` は 2026-05-16 に修正した。raw memory source capability も同様に、修正前は expression 内の任意の raw helper symbol を evidence として扱っていた。修正後は prefix call-position scanner を通し、expression 先頭、および `let` / `set` / `if` / `while` / address/reference introducer / type annotation / pipe の直後に現れる call head だけを raw helper symbol evidence として扱う。これにより `let cur <i32> load_i32 0` のような正当な raw helper implementation evidence は維持しつつ、`consume load_i32` や `consume mem_ptr_addr` のような値・引数位置の raw helper 名から raw operation / structural boundary authority が出ない。raw body / intrinsic evidence は explicit low-level evidence として維持する。

`ISS-20260516T022823182Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-MISS-2CBBEB43` は 2026-05-16 に修正した。owner aggregate 側も raw memory 側と同じ `PrefixCallHead` tracker を使い、source capability が見る call-head 位置を共通化した。これにより `let boxed <OwnerBox<i32>> OwnerBox<i32> region` や `let owner <i32> field::get v "owner"` のように、`let` / type annotation の後ろへ constructor / field accessor call が来る正当な compiler-owned source でも必要な boundary evidence を得られる。一方で `consume Diag` のような非 call-head 大文字値は引き続き evidence にならない。これは module allowlist ではなく、NEPL prefix 構文上の call-position を source proof primitive として共通化する修正であり、owner-backed aggregate かどうかの semantic proof は typecheck / Resource IR 側に残す。

`ISS-20260516T024129752Z-OWNER-AGGREGATE-FIELD-EVIDENCE-ACCEP-22239551` は 2026-05-16 に修正した。owner aggregate field evidence は、修正前は call-head が `get` / `get_ref` / `put` という名前なら `core/field` 由来でなくても field boundary evidence として扱っていた。修正後は `owner_aggregate/field_imports.rs` で `#import "core/field"` の default alias / explicit alias / open import / merge import / selective import を `ImportClause` の網羅 `match` で証明し、その import provenance と call-head symbol が一致する場合だけ field accessor evidence を得る。`alloc/collections/vec/query/get` など unrelated module の `get` は evidence にならず、`#intrinsic "get_field"` / `get_field_ref` は explicit compiler primitive evidence として別経路に残す。これは stdlib module ごとの allowlist ではなく、source AST の import provenance と prefix call-head を組み合わせた汎用 source proof gate であり、field projection の型・owner 正当性は引き続き typecheck / Resource IR 側で検査する。

`ISS-20260516T025931471Z-WINDOWS-STDLIB-PATH-CANONICALIZATION-5C6E2D4E` は 2026-05-16 に修正した。Windows では既存 stdlib root の `canonicalize()` が `\\?\C:\...` 形式を返す一方、存在しない仮想 stdlib child は通常の `C:\...` 形式で lexical normalize され、`configured_stdlib_source_path` の `starts_with` が false になり得た。修正後は canonicalize 成功時と失敗時の両方で Windows verbatim prefix を通常 prefix に戻してから lexical normalization する。これにより provider / inline test など仮想 stdlib source でも SourceCapabilities の configured stdlib 判定が同じ path 表現で行われ、source capability regression が `SourceCapabilities::none` を誤って検査する状態を防ぐ。

Resource checker の責務分割 policy も確認し、`initialized_summary_variant_build.rs` が監視対象から漏れていたため `ISS-20260430T062912063Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-CC55287A` で修正した。2026-05-13 には `ISS-20260512T201359246Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-E382D3AB` として、後続 Stage 4/5 で追加された Resource IR module も全て行数上限と宣言検査の対象へ入れた。今後 Resource IR module を分割した場合は、実装だけでなく `nodesrc/test_resource_checker_responsibility.js` の責務上限も同時に更新しなければ source policy が失敗する。

したがって、この計画の完了条件は変更しない。旧 checker の special-case や旧 drop walker を戻して現状維持するのではなく、残る raw-memory-backed stdlib public API、owner token、collection storage state を Resource IR / enum / match の設計へ移す。

## 完了条件

この計画は次を満たした時点で完了とする。

1. `typecheck.rs` と `move_check.rs` の主要責務が module 境界へ分離されている。
2. Resource IR が typed HIR 後の正式な検査入力になっている。
3. move / borrow / lifetime / initialized / drop obligation / raw provenance が Resource IR 上で共有状態として検査される。
4. `MemPtr` は non-owning pointer に限定され、owner token と initialized cell state が別表現になっている。
5. raw memory primitive は public pure surface から閉じられ、必要な内部効果だけが surface pure へ fold される。
6. stdlib collection / string / self-host buffer が safe public discipline と compiler Resource IR の責務分割に従う。
7. 旧 HIR 個別 summary を削除しても、既存 memory safety / type safety / effect safety regression が通る。
