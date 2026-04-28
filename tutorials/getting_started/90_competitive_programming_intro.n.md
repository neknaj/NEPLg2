# Advanced: 競技プログラミング導入

競技プログラミング向けの内容は、入門本文とは別 track に分けます。通常の getting started では、まず `Result`、`Option`、`match`、`Vec` の所有権、byte と text の区別を固定します。

競プロ track で使う方針は次の通りです。

- 入力は `std/streamio` や `std/io` の public API で扱い、raw memory に依存しません。
- 失敗しうる I/O は `Result` として扱い、panic helper を前提にしません。
- 大きな配列や graph は、所有者が明確な collection と cleanup の規則を決めてから使います。
- tutorial 本文の API と同じ `std/test` 形式で最小例を固定します。

この track は、標準 collection と I/O facade の静的検査が安定した範囲から順に拡張します。
