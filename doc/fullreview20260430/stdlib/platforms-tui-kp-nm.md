# stdlib platforms tui kp nm review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/platforms/wasix/tui/**`
- `stdlib/features/tui.nepl`
- `stdlib/kp/**`
- `stdlib/nm/**`
- `stdlib/nm/README.n.md`

## 良い点

TUI は `ansi/box/buffer/style/text/tty` に分割され、色 code は `std/stdio/ansi` の typed style に寄せている。TUI 固有の raw i32 color API を増やさない方向は正しい。

NM は parser と html generator が分割され、scanner、document、json_inline、json_section、html_escape、html_heading、html_inline、html_section へ責務が移っている。以前の raw aggregate detour や inline/block unwrap 問題を source policy で監視している。

KP は競技プログラミング用途の graph/search/fenwick/dsu/prefix helper を持ち、tutorial と integration test の入力として機能している。performance 寄り helper を一般 stdlib と分けている点はよい。

## 問題とリスク

TUI `tty` や buffer は terminal raw mode と scratch buffer を扱うため、raw allocation/deallocation の責務が残る。raw mode restore は failure path で漏れると利用体験だけでなく resource safety も壊す。

NM parser の scanner には nested boolean / numeric level helper が残る。これは markdown-like grammar の小さい finite state として現時点では許容できるが、状態が増えるなら enum stack / section state に移すべきである。

KP helper は `alloc_raw` / `mem_ptr_addr` を多く使う。これは performance用途として隔離する価値はあるが、compiler/static check が弱いから使う抜け道にしてはいけない。ResourceIR proof と source policy が必須である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| TUI style/text/buffer | module split + typed ANSI style。 | 良い。raw tty境界は監視。 |
| NM parser/htmlgen | scanner/document/html/json split。 | 良い。state増加時はenum化。 |
| KP helpers | graph/search/fenwick/dsu/prefix。 | raw-heavy。performance隔離として扱う。 |

## 推奨対応

- TUI raw mode は RAII/owner-token 相当の restore contract を明文化し、failure path の source policy を追加する。
- NM の section/open state が増える場合は bool列や numeric levelではなく typed enum/stackへ移す。
- KP raw memory helper は tutorial/selfhost の通常 API と混ぜず、ResourceIR regression の対象として維持する。
