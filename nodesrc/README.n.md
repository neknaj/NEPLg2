# nodesrc

NEPLg2 の Node.js [系/けい]ツールを[目的別/もくてきべつ]にまとめた[案内/あんない]です。

## [全体/ぜんたい][方針/ほうしん]

- `nodesrc/` は compiler / stdlib / doctest / focused test の[検証/けんしょう]を[素早/すばや]く[回/まわ]すための[補助/ほじょ]ツール[群/ぐん]です。
- reboot [中/ちゅう]は `nodesrc/tests.js` で[範囲/はんい]を[絞/しぼ]った[実行/じっこう]と、`run_doctest.js` による 1 [件/けん]の[直接/ちょくせつ][再現/さいげん]を[使/つか]い[分/わ]けます。

## `tests.js`

[通常/つうじょう]の[回帰/かいき]テストと stdlib doctest の[両方/りょうほう]を[走査/そうさ]して[実行/じっこう]します。

### [注意/ちゅうい]

- `stdout:` / `stderr:` を[書/か]いた doctest は、`tests.js` でも[既定/きてい]で I/O [一致/いっち]を[検証/けんしょう]します。
- `--assert-io` は[明示的/めいじてき]に I/O [厳格/げんかく][確認/かくにん]を[示/しめ]したいときの補助で、I/O [期待値/きたいち]が[書/か]かれた case を[有効化/ゆうこうか]するための必須 flag ではありません。

### [主/おも]な[用途/ようと]

- `tests/compiler/*` / `tests/stdlib/*` の[通常/つうじょう]テスト
- `stdlib/**/*.nepl` や tutorials の `//:` doctest
- `-i` で[範囲/はんい]を[絞/しぼ]った focused [実行/じっこう]

### [例/れい]

```bash
node nodesrc/tests.js -i tests/compiler -i tests/stdlib --no-tree -o /tmp/tests.json -j 15
node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o /tmp/vec-doctest.json -j 15
```

## `run_doctest.js`

1 [件/けん]の doctest を[直接/ちょくせつ][指定/してい]して[再現/さいげん]します。

### [主/おも]な[用途/ようと]

- stdlib の `//:` doctest が 1 [件/けん]だけ[失敗/しっぱい]したときの[最短/さいたん][再現/さいげん]
- `tests.js` を[回/まわ]すには[重/おも]いが、1 [件/けん]だけ[見/み]たいとき

### [例/れい]

```bash
node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 9
node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 3
```

## `run_test.js`

1 [件/けん]の[構造化/こうぞうか]テスト JSON を[直接/ちょくせつ][実行/じっこう]します。

### [主/おも]な[用途/ようと]

- `tests.js` を[介/かい]さず、1 [件/けん]の[内容/ないよう]を[明示的/めいじてき]に[渡/わた]したいとき
- nodesrc [側/がわ]の test harness [自体/じたい]を[調/しら]べたいとき

### [注意/ちゅうい]

- `#target wasix` の case は Node.js [内蔵/ないぞう] WASI ではなく `wasmer run` で[実行/じっこう]します。
- `WASMER_BIN` を[設定/せってい]すると、`wasmer` [以外/いがい]の[実行/じっこう][バイナリ/ばいなり]を[指定/してい]できます。

## `cli.js`

Node.js から compiler/runtime を[呼/よ]び[出/だ]す CLI です。

### [主/おも]な[用途/ようと]

- `.nepl` を[単発/たんぱつ]で[実行/じっこう]
- tests.js / run_doctest.js [内部/ないぶ]の[共通/きょうつう][経路/けいろ]

## `compiler_loader.js`

`nepl-web` の build [成果物/せいかぶつ]を[読/よ]み[込/こ]み、Node.js から compiler を[使/つか]えるようにする helper です。

### [注意/ちゅうい]

- `trunk build` [後/ご]の `web/dist` を[前提/ぜんてい]にします。
- build [成果物/せいかぶつ]が[古/ふる]いと、Node.js [側/がわ]の[実行/じっこう]と[実装/じっそう]が[食/く]い[違/ちが]います。

## reboot [中/ちゅう]の[使/つか]い[分/わ]け

1. [広/ひろ]い[回帰/かいき][確認/かくにん]
   - `tests.js`
2. stdlib doctest 1 [件/けん]の[再現/さいげん]
   - `run_doctest.js`
3. harness [自体/じたい]や JSON [入力/にゅうりょく]の[調査/ちょうさ]
   - `run_test.js`

## [補足/ほそく]

- reboot [中/ちゅう]は[失敗/しっぱい]の[原因/げんいん]を
  - compiler
  - stdlib
  - tests 移行
  に[切/き]り[分/わ]けることが[重要/じゅうよう]です。
- そのため、`tests.js` の[範囲指定/はんいしてい]と `run_doctest.js` の 1 [件/けん][再現/さいげん]を[優先/ゆうせん]して[使/つか]います。
