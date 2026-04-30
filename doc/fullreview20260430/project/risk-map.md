# リスクマップ

対象 commit: `f108cebd`

## 最重要リスク

### Resource IR final authority が未完

Resource IR は大きく進んだが、親 issue `ISS-20260425T000000Z-RV-CORE-009-58589A3F` は open のままである。旧 `passes::move_check` と HIR drop insertion が残る限り、selfhost がコピーすべき最終設計はまだ確定していない。

判断:

- 現行の二重防壁は開発中の安全策として妥当。
- selfhost の final checker に旧 HIR special-case を移植するのは不適切。
- Resource IR 上で move / borrow / initialized cell / owner / effect / drop obligation を統合する方向を維持する。

### `MemPtr` / `RegionToken` が compiler-issued capability ではない

`MemPtr<T>` が non-owning pointer と storage owner の両方に見える設計は、memory safety の根本リスクである。直近 main で Result variant / value condition の伝播は進んだが、`tests/stdlib/memory_safety.n.md` の残失敗はこの設計 issue に残っている。

判断:

- stdlib に cleanup を足して残失敗を隠すべきではない。
- `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell = Resource IR state` へ分ける設計が必要。
- selfhost の buffer / token stream / diagnostic storage は、この分離前提で API を選ぶ。

### collection element Drop が未完

`Vec<T>` や `HashMap<K,V>` の storage dealloc と element Drop はまだ根本完了していない。owning element を格納する collection を selfhost で多用すると、drop obligation の曖昧さが広がる。

判断:

- selfhost 初期実装では、owning element を含む長寿命 collection を不用意に増やさない。
- `Copy` read、borrowed read、owned remove/pop、container Drop を API と型制約で分ける設計が必要。
- sentinel state ではなく enum state へ寄せる。

### `.n.md` test output policy が移行中

assertion suite は stdout report と exit code を分ける方向だが、open issue が残る。Rust/selfhost 共通テスト運用の基盤として重要である。

判断:

- return value だけに依存する assertion suite は減らす。
- stdout report の deterministic format と exit code 0/1 を固定する。
- selfhost compiler でも同じ `.n.md` を読む前提で runner contract を設計する。

## selfhost 開始可否

### すぐ開始できる領域

- SourceText / SourceMap / line map。
- lexer tokenization と Rust lexer parity fixture。
- parser AST subset と Rust AST JSON parity。
- in-memory VFS と module graph。
- diagnostic enum / stable string boundary。
- CLI args / file_io / reporter / driver の I/O 境界。
- small helper stdlib: string find/slice/compare、hash、byte scanner、fs/stdout/stderr result boundary。

これらは raw memory owner model に深く依存せず、Rust 側の現行方針を参考に進められる。

### まだ慎重に進める領域

- full typecheck / overload / trait capability。
- Resource IR checker。
- drop insertion。
- owned collection を多用する compiler arena。
- selfhost codegen。

これらは Rust 側の Resource IR final authority と stdlib memory model の影響を強く受ける。先に独自設計で固定すると、後で Rust 側とずれる。

## 技術的負債として残してはいけないもの

- diagnostic code を raw string で持ち回る設計。
- token kind / AST kind / type kind / resource state を raw number sentinel で分岐する設計。
- `if` の深いネストで本来 `match` にすべき有限分岐を表す設計。
- raw memory operation を stdlib safe API として広げる設計。
- collection cleanup を caller convention に任せる設計。
- HIR direct traversal special-case を selfhost checker の正規実装にする設計。

## 既存防壁

- `node nodesrc/run_source_policy_regressions.js` は stdlib / selfhost / Resource IR の source policy を集約している。
- `cargo test -p nepl-core --test resource_ir` は Resource IR regression の中心になっている。
- `issues/index.md` と per-issue docs は open blocker を比較的よく追跡している。
- `note.n.md` には各 agent の同期・検証結果が詳細に残っている。

## リスク低減の優先順

1. Resource IR を final authority にするための残差分を明文化し、旧 move_check / HIR drop insertion の削除条件を固定する。
2. `MemPtr` / `RegionToken` / owner token / non-owning pointer の設計を stdlib と compiler で合わせる。
3. collection element Drop と owned remove/pop API を設計し、selfhost が安全に arena / list / table を使えるようにする。
4. `.n.md` stdout report policy を固定し、Rust/selfhost 共通 test の基盤にする。
5. README と doc の NEPLg2 / NEPLg3 / selfhost の説明を現行方針へ合わせる。
