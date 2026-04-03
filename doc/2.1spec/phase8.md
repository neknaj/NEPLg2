# NEPLg2.1 Phase 8: 依存型・形式証明（将来仕様）

最終更新: 2026-03-27

---

## 1. 目的

Phase 8 では、Phase 0-7 で確立した前置記法・所有権・効果分離を土台として、依存型と証明オブジェクトを導入する。

表層構文は Zenn #1 / #2 で確定したコア記法に揃える。

---

## 2. CTFE

型の位置に現れた式は CTFE で評価する。

```nepl
let Vec struct .T add .n 1:
    ...
```

型文脈に現れる式は次を満たさなければならない。

- Pure
- Total
- 結果が Pure persistent value

---

## 3. 証明オブジェクト

現段階のコードスケッチでは、証明オブジェクトを明示渡しする形を基本例とする。制約解決や証拠の自動導出を将来どこまで許すかは未確定であり、本章では固定しない。

```nepl
let proof decide_less idx len
match proof:
    IsLess::Yes p core::vec::get vec p
    IsLess::No Option::None
```

証明オブジェクトは `Copy` 型として扱う。

---

## 4. Core 層と Casual 層

### 4.1 Core

```nepl
use core::collections::vec as *
get vec proof
```

### 4.2 Casual

```nepl
use std::collections::vec as *
get vec 3
```

---

## 5. コードスケッチ

### 5.1 Core

```nepl
let Vec struct .T .len:
    ...

let push .T .n %fn Vec .T .n fn .T Vec .T add .n 1 \vec \item ...

let get .T .len .idx %fn Vec .T .len fn .idx .T
    where %IsLess .idx .len
    \vec \index ...
```

### 5.2 Casual

```nepl
let Vec struct .T:
    ...

let get .T %fn Vec .T fn i32 Option .T \vec \index:
    match decide_less index len:
        IsLess::Yes proof Option::Some core::vec::get vec proof
        IsLess::No Option::None

let push .T %fn Vec .T fn .T Vec .T \vec \item ...
```

---

## 6. 方針

依存型・証明・CTFE 自体は将来仕様だが、そこで使う表層記法は現行コアと一致させる。旧来の `fn A -> B` / `%fn ... -> ...` 形式は使わない。
