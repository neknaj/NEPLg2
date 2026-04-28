# Advanced: graph / BFS / DP

graph と DP は大きな collection を扱うため、入門本文の後に置きます。古い tutorial では dense graph の内部 layout や raw allocation に近い書き方が混ざっていましたが、この track では public collection API だけを使います。

追加時の方針:

- graph の表現は `Vec`、matrix、queue などの owner を明確に分けます。
- BFS の queue は `alloc/collections/queue` の public API を使います。
- DP table は `Vec` の作成、更新、解放が 1 つの関数内で閉じるようにします。
- non-Copy payload を raw storage に置く必要が出た場合は、tutorial で回避せず stdlib issue として分けます。

この章は、collection API と静的検査の更新に追従しながら少数の runnable example を追加する場所です。
