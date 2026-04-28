# Advanced: sort / search / prefix sum

sort や prefix sum は、`Vec<i32>` の所有権と読み取り API を理解してから使います。現在の推奨は、mutable な内部表現へ直接触れず、`alloc/collections/vec` と `alloc/collections/vec/sort` の public API を通すことです。

設計時の注意点:

- sort は `Ord` bound を持つ値だけに使います。
- 読み取りは `get_ref` で Copy 要素を確認します。
- prefix sum は入力 `Vec` と出力 `Vec` の owner を混同しないよう、構築関数を分けます。
- 失敗しうる確保や grow は `Result` として呼び出し側へ返します。

競技向け catalog は、古い 20 サンプル集のように多数を並べるより、sort、binary search、prefix sum の 3 系統を current ownership model で確実に動かす形へ縮小してから追加します。
