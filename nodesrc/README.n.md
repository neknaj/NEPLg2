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
- timeout [調査/ちょうさ]では JSON の `timing.compile_ms` / `timing.run_ms` と `timeout.last_phase` を[見/み]て、compiler [側/がわ]の[遅/おそ]さと runtime [側/がわ]の[遅/おそ]さを[分/わ]けて[扱/あつか]います。

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

- `#target wasix` の case はまず `wasmer run` で[実行/じっこう]します。`wasmer` が[無/な]い[環境/かんきょう]や、`wasix_32v1.tty_get` / `tty_set` が[未対応/みたいおう]の Wasmer では、Node.js [内蔵/ないぞう] WASI に WASIX TTY host import を[足/た]した fallback で[実行/じっこう]します。
- `WASMER_BIN` を[設定/せってい]すると、`wasmer` [以外/いがい]の[実行/じっこう][バイナリ/ばいなり]を[指定/してい]できます。
- [結果/けっか] JSON には `timing.load_ms` / `timing.compile_ms` / `timing.run_ms` / `timing.total_ms` を[入/い]れます。compile_fail case など run phase に[進/すす]まない case の `run_ms` は `null` です。
- `CompilerSession` が[有効/ゆうこう]な compile では、`timing.compiler_session_stats` に materialized compile counter の before / after / delta を[入/い]れます。[累積/るいせき] counter をそのまま[集計/しゅうけい]せず、1 compile で[増/ふ]えた fallback [数/すう]を[性能/せいのう] report で[読/よ]むためです。

## `cli.js`

Node.js から compiler/runtime を[呼/よ]び[出/だ]す CLI です。

### [主/おも]な[用途/ようと]

- `.nepl` を[単発/たんぱつ]で[実行/じっこう]
- tests.js / run_doctest.js [内部/ないぶ]の[共通/きょうつう][経路/けいろ]

## `bench_materialized_compile_fallbacks.js`

同じ `CompilerSession` で `.neplmeta` store を[温/あたた]めながら cold / warm edit compile を[順番/じゅんばん]に[実行/じっこう]し、materialized compile fallback と `.neplobj` candidate surface [数/すう]を JSON で[出力/しゅつりょく]します。

### [主/おも]な[用途/ようと]

- `body_missing_candidate_surfaces_delta_sum` が[増/ふ]える compile を[探/さが]し、`.neplobj` の[最初/さいしょ]の[対象/たいしょう]を[決/き]める
- `materialized_fallback_diagnostic_code_counts` で `.neplobj` body missing ではない source fallback の[原因/げんいん]を typed diagnostic code ごとに[分解/ぶんかい]する
- `compile_ms` と `resource_static_check` / `resource_typecheck` / `wasm_codegen` の stage timing を、同じ session の artifact [温度/おんど]と[一緒/いっしょ]に[確認/かくにん]する

## `compiler_loader.js`

`nepl-web` の build [成果物/せいかぶつ]を[読/よ]み[込/こ]み、Node.js から compiler を[使/つか]えるようにする helper です。

### [注意/ちゅうい]

- `trunk build` [後/ご]の `web/dist` を[前提/ぜんてい]にします。
- build [成果物/せいかぶつ]が[古/ふる]いと、Node.js [側/がわ]の[実行/じっこう]と[実装/じっそう]が[食/く]い[違/ちが]います。

## `issues.js`

`issues/items/*.md` の Issue を[検証/けんしょう]し、`issues/index.json` / `issues/index.md` を[生成/せいせい]します。

### [主/おも]な[用途/ようと]

- 旧 `doc/review20260425` の Issue を `issues/` へ[移行/いこう]
- [衝突/しょうとつ]しにくい `ISS-...` ID の[新規/しんき] Issue [作成/さくせい]
- commit [前/まえ]の Issue metadata [検証/けんしょう]

### [例/れい]

```bash
node nodesrc/issues.js migrate-review20260425
node nodesrc/issues.js new --area selfhost --title "stdlib fs write API is missing" --priority P1 --type architecture
node nodesrc/issues.js index
node nodesrc/issues.js check
```

## `compare_git_versions.js`

git commit / ref [単位/たんい]で、doctest [通過率/つうかりつ]、compile/run timing、`repo_metrics.ts` の[規模/きぼ]指標を[比較/ひかく]します。

### [主/おも]な[用途/ようと]

- 静的検査や Resource IR の[大規模/だいきぼ][修正/しゅうせい]で、前後の[通過率/つうかりつ]と[速度/そくど]を[同/おな]じ[入力/にゅうりょく]で[比較/ひかく]する。
- `repo_metrics.ts` の files / lines / source / doc_comment / testCases を commit [単位/たんい]で[一覧/いちらん]する。
- Discord や issue に[貼/は]る Markdown [要約/ようやく]を[作/つく]る。

### [例/れい]

```bash
node nodesrc/compare_git_versions.js --rev HEAD~1 --rev HEAD -i tests/compiler/typeannot.n.md --dist-current web/dist --no-tree -o tmp/version_compare/typeannot.json --markdown tmp/version_compare/typeannot.md
node nodesrc/compare_git_versions.js --rev HEAD~10 --rev HEAD --metrics-only -o tmp/version_compare/metrics.json --markdown tmp/version_compare/metrics.md
```

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
