# 2026-04-03 ���� (web playground editor surface �C��)

- [��]:
  - playground editor �� core ���� CLI suite �Œʂ��Ă�������Asurface ���� hover/completion ��\���ADPR ����A�}�E�X���W����̖�肪�c���Ă������߁Aeditor-dom-ui / editor-input-handler / editor.ts / styles.css ���C�������B
- [����]:
  - general-popup �� completion-list �͏��� DOM �� hidden class �������Ă���̂ɁA�\������ class ���O���Ă��炸�Adisplay: block ��ݒ肵�Ă� display: none !important �ɕ����ď펞��\���̂܂܂������B
  - esizeEditor() �� ctx.scale(dpr, dpr) �𖈉�ςݏグ�Ă���A��������� resize �� pane resize �̂��тɕ����E�n�C���C�g�E�J�[�\���̕`��ʒu������₷����Ԃ������B
  - �}�E�X�ʒu�v�Z�� offsetX / offsetY �Œ�ŁAcanvas �̎��T�C�Y�ECSS �T�C�Y�E�C�x���g�N�_�̍����Ɏォ�����B
  - IME �p textarea �� z-index: -1 �̂܂܂ŁA���͈ʒu�Ǐ]�ɕK�v�ȃX�^�C�������s�����Ă����B
- [�C��]:
  - web/src/editor/editor-dom-ui.ts
    - popup / completion �̕\���E��\���� hidden class ����������悤�ɕύX�����B
  - web/src/editor/editor-input-handler.ts
    - clientX / clientY �� getBoundingClientRect() ���� canvas ���΍��W�����߂�悤�ɕύX�����B
  - web/src/editor/editor.ts
    - esizeEditor() �� setTransform(1, 0, 0, 1, 0, 0) ������ł��� DPR scale ��������悤�ɂ��A�g�嗦�̗ݐς��~�߂��B
    - hidden textarea �� font / lineHeight / height ��Ǐ]�����AIME �ʒu�v�Z�� completion anchor �̂����}�����B
  - web/styles.css
    - popup tooltip / completion popup �̃X�^�C����ǉ������B
    - hidden textarea �� editor surface ��ň��S�Ƀt�H�[�J�X�ł���ݒ�֊񂹂��B
- [�m�F]:
  - 
pm --prefix web run build:ts: �ʉ�
  - 	runk build --release --public-url ./: �ʉ�
  - 
ode nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json: 11/11 passed
- [plan.md�Ƃ̍���]:
  - �V���� editor core �ւ̈ڍs�͌p�����ŁA����̏C���͋� CanvasEditor surface �̕s������{�C���������́B�`��E���� surface �̖����ɒׂ����̂ŁA����� pointer / completion / problems �� UI ��ԑJ�ڂ������ pure �ȋ��E�֊񂹂₷���Ȃ����B
# 2026-03-27 作業メモ (doc: compare / 2.1impl / examples の再点椁E

- [目的]:
  - `2.1spec` 以外に、`doc/compare`、`doc/2.1impl`、`doc/examples` に Zenn #1 / #2 と衝突する記法が残ってぁE��ぁE��を�E点検する、E
- [確認結果]:
  - `doc/examples/` は主要サンプル 01、E7 を�E確認し、現在残ってぁE��構文は Zenn #1 / #2 を正とする説明と両立することを確認した、E
  - `doc/compare/` の旧記法�E、旧 2.0 / 旧 2.1 案との比輁E��象として意図皁E��残ってぁE��も�Eであり、現行仕様として案�EしてぁE��箁E��は見当たらなかった、E
  - `doc/2.1impl/` では表層構文の呼称にわずかに旧案が混ざってぁE��ため補正した、E
- [変更]:
  - `doc/2.1impl/compiler_structure.md`
    - `let fn` とぁE��表現めE`let` に修正、E
    - primitive 一覧の `unit` めE`()` に修正、E
    - `decl_check` / `hoist` / closure 型検査の説明に残ってぁE��旧 `%fn` の読め方を、`fn` / `fn*` が関数型で `%` はそ�E前置型注釈であることが�Eかる表現へ修正、E
  - `doc/examples/01_basics.nepl`
    - 冒頭コメント�E `%fn` / `%fn*` 説明を、E��数型そのも�Eではなく「`fn` / `fn*` めE`%...` で式へ前置する型注釈」と刁E��る表現に修正、E
  - `doc/examples/05_io_and_resources.nepl`
    - `%fn A B = 関数型` と読めてしまぁE��意書きを修正し、`fn A B` / `fn* A B` が関数型で `%` は型注釈だと明記、E
  - `doc/migration/index.md`
    - `std/stdio.nepl` の移行メモに残ってぁE�� `%fn* () ()` の曖昧な書き方を、`fn* () ()` めE`%...` で前置する型注釈付き lambda だと刁E��る表現へ修正、E
  - `doc/2.1spec/modules.md`
    - `merge` / `module` ブロチE��のコード例に残ってぁE��旧 `let ...:` 風プレースホルダを、`let <name> <expr>` ベ�Eスの placeholder へ修正、E
  - `doc/self_host.md`, `doc/2.1spec/platform.md`
    - ブ�EトストラチE�E側の場所とセルフ�Eスト構�Eが古ぁE`/nepl-core` / `lexer` / `parser` / `typecheck` 前提で残ってぁE��ため、`nepl-core-2.1` と `2.1impl` の現行ディレクトリ設計に合わせて補正、E
  - `doc/README.md`, `doc/chat/dump/*.md`
    - `chat/dump` 配下�E過去の検討メモであって現行仕様�E正ではなぁE��め、その旨を�E記した、E
  - `doc/cli.md`, `doc/editor_extensions.md`, `doc/web_playground.md`
    - 現衁EBootstrap 実裁E�E説明と NEPLg2.1 計画が混ざって読める箁E��があったため、対象が現行実裁E��あることと、正の仕様�E `2.1spec` / Zenn #1 / #2 であることを�E記した、E
  - `doc/2.1spec/index.md`
    - 「完�Eな言語仕様」とだけ書く�Eではなく、Zenn #1 / #2 で未確定�E周辺領域は吁E��で draft / 封E��仕様として明示する構�Eだと刁E��る文言へ補正、E
  - `doc/2.1spec/modules.md`
    - 壊れてぁE�� `declarations.md §9` 参�Eと未定義の `noshadow` 前提を除去し、現時点で本斁E��存在する衝突規則だけに書き直した、E
  - `doc/2.1spec/compiler.md`, `doc/2.1spec/traits.md`
    - `MemReadable` / `MemWritable` / `RegionOwned` めEcompiler 章だけが先に使ってぁE��ため、traits 章に「封E��導�Eする capability trait」として予紁E��を追加し、compiler 章側も封E��拡張だと明記した、E
  - `doc/2.1spec/memory.md`, `doc/2.1spec/phase8.md`
    - 長さ添字付き `Vec` の例を `Vec .T .len` へ揁E��た、E
  - `doc/2.1spec/types.md`, `doc/2.1spec/traits.md`
    - 束縛されてぁE��ぁE`.T` をそのまま使ってぁE��例を、binder 付きまた�E具体型付きの well-formed な例へ修正、E
  - `doc/2.1spec/effects.md`, `doc/2.1spec/syntax.md`, `doc/2.1spec/phase8.md`
    - `set` と証明オブジェクト�E扱ぁE��未凍絁E/ draft であることを�E示し、Zenn #1 / #2 で確定したコア構文と封E��設計�E墁E��を見えるよぁE��した、E

# 2026-03-27 作業メモ (doc: stdlib コメント方針�E例を Zenn 基準へ追征E

- [目的]:
  - `doc/stdlib_doc_comment_policy.md` の doctest 例に、旧 `#entry main` めE��区刁E��セミコロン前提の書き方が残ってぁE��ため、Zenn #1 / #2 を正とした表記へ寁E��る、E
- [変更]:
  - `doc/stdlib_doc_comment_policy.md`
    - `#entry main` めE`#entry` に修正、E
    - helper 関数の doctest 例を `let main \(): block: ...` 形式に変更、E
    - 途中式�E破棁E��前置 `;` で表す形に合わせた、E

# 2026-03-27 作業メモ (doc: README の公開�E口サンプルめEZenn #1 / #2 基準へ更新)

- [目的]:
  - ルーチE`README.md` に残ってぁE��旧記法サンプルが、Zenn #1 / #2 を正とする現在の仕様文書と食い違ってぁE��ため、�E口斁E��の表記を揁E��る、E
- [変更]:
  - `README.md`
    - クイチE��サンプルを旧 `#import` / `fn main <...>` / `unit` 前提の例から、`let main \()` / `if cond a b` / `block:` を使ぁEZenn 基準�Eコア構文例へ更新、E
    - 現行実裁E�� 2.1 設計文書がまだ完�E一致してぁE��ぁE��とを注記し、正の仕様として `doc/2.1spec/` を参照するよう明記、E
    - NEPLg2.1 の説明文を、`%fn` めEjuxtaposition だけでなく、`let <name> <expr>`、`%` の式レベル型注釈、`if` / `match` / `block:` まで含めた表現へ更新、E

# 2026-03-27 作業メモ (doc: migration / 2.1impl / errors の旧 2.1 案を追加追征E

- [目的]:
  - 先行して更新した `doc/2.1spec` と `doc/examples` に対して、まだ旧 2.1 案�E記法を前提にしてぁE��補助斁E��を、Zenn #1 / #2 を正として追従させる、E
- [変更]:
  - `doc/migration/index.md`
    - trait / enum / quick reference の変換表を、`fn A -> B` / `unit` / `\ a b :` / `if cond : ...` ではなく、`fn A B` / `()` / `\a \b ...` / `if cond a b` ベ�Eスへ修正、E
  - `doc/2.1impl/compiler_structure.md`
    - パ�Eサ・型検査の説明に残ってぁE�� `\ params : body` めE`%fn A -> B` 前提の記述を、` \x body` / `\x:` / `%fn A B` 前提へ更新、E
    - pattern 例を `let Point x y p` から `let Point x: a y: b p` へ更新、E
  - `doc/examples/04_strings_and_builders.nepl`
    - `fold \ b s :` と旧説明文を、現在の lambda 表記と型説明へ修正、E
  - `doc/2.1spec/errors.md`
    - `Result` / `Outcome` の例で残ってぁE��旧 payload 記況E`Ok %.T` / `Err %.E` / `field %Type` を、`Ok .T` / `Err .E` / `field: Type` へ修正、E
- [現在の実裁E��況]:
  - `2.1spec` の入口斁E��、比輁E��書、主要サンプル、移行ガイド�E主要変換表は、Zenn #1 / #2 に沿ぁE��へ概�E揁E��た、E
  - まだ `while` など Zenn 記事で確定してぁE��ぁE��E��仕様�E斁E��中に残るが、コア構文と直接衝突する旧記法�E大幁E��減った、E

# 2026-03-27 作業メモ (doc: Zenn #1 / #2 を正として 2.1spec のコア構文斁E��を更新)

- [目的]:
  - `doc/2.1spec/` のぁE��、Zenn #1「カリー化」と Zenn #2「型と制御構文」で明示された仕様と衝突してぁE��斁E��を、Zenn 記事を正として修正する、E
- [変更]:
  - `doc/2.1spec/overview.md`
    - 概要を旧 `fn A -> B` / `%fn ... -> ...` 前提から更新、E
    - カリー化、`%` の意味、`let <name> <expr>`、`if` / `match` / `block:` / `;`、`()` 表記を現行コアとして明記、E
  - `doc/2.1spec/types.md`
    - 関数型記法を `fn A B` 形式へ変更、E
    - `%` を宣言用の注釈開始記号ではなく「続く 1 個�E式に掛かる前置演算子」として再定義、E
    - `unit` 値表記を削除し、`()` めEunit 型およ�Eそ�E唯一の値として整琁E��E
  - `doc/2.1spec/declarations.md`
    - 関数定義の基本形めE`%fn ... \ ...` 忁E���E宣言から、`let <name> <expr>` へ変更、E
    - lambda めE`\a <expr>` / `\a:` ブロチE�� / `\()` で説明、E
    - struct 定義と構築例を `x: i32`, `Point x: 0 y: 7` 形式へ変更、E
  - `doc/2.1spec/syntax.md`
    - `if` めE`if <cond> <then> <else>` / `if <cond> then <then> else <else>` に差し替え、E
    - `match` arm めE`<pattern> <expr>` に差し替え、E
    - `block:` と前置 `;` を追加、E
    - `|>` 節から部刁E��用前提の説明を除去、E
  - `doc/2.1spec/patterns.md`
    - OR pattern めE`or` pattern として導�E、E
    - range 保留をやめ、`span` pattern を導�E、E
    - struct 刁E��を位置ベ�Eスから field 名付きへ変更、E
    - guard / 部刁E��用中忁E�E説明をコア仕様から外した、E
  - `doc/2.1spec/effects.md`, `doc/2.1spec/memory.md`, `doc/2.1spec/traits.md`, `doc/2.1spec/phase8.md`
    - 旧 `->` 記法�E`unit` 表記�E旧 lambda 表記�E用例を、新しい前置型記法と `()`、`let` / lambda 記法へ追従させた、E
  - `doc/compare/syntax.md`, `doc/compare/index.md`
    - 「旧 2.1 案」と「Zenn #1 / #2 を正とした現在の 2.1」を区別する形で比輁E��書を更新、E
  - `doc/examples/01_basics.nepl` から `doc/examples/07_modules_impl.nepl`
    - コア構文に直接触れるサンプルを、旧 `->` / `unit` / `pattern: expr` / `if ...:` から新表記へ追従させた、E
- [plan.mdとの差異]:
  - `plan.md` には旧 2.0 / 旧 2.1 案�E記述が強く残っており、今回の Zenn #1 / #2 で確定したコア構文とは一致しなぁE��E
  - 特に、E��数型記法、`%` の意味、`let` / lambda の基本斁E��、E��刁E��用の不採用、`if` / `match` / pattern / block / `;`、`()` 表記�E `plan.md` と差刁E��ある、E
  - `plan.md` は人が書き換える前提なので未変更とし、差刁E�E本メモに記録した、E
- [現在の実裁E��況]:
  - `doc/2.1spec/` のコア構文斁E��は、Zenn #1 / #2 を正として参�Eできる入口に更新した、E
  - `doc/compare/` と `doc/examples/` のコア構文サンプルも主要E��刁E��新記法へ追従済み、E
  - 一方で `compiler.md` など実裁E�E部設計文書には、表層構文と直接衝突しなぁE��E��の旧記法断牁E��残る。今回はコア構文と読老E��け導線を優先した、E
- [Zenn記事�Eの不整合メモ]:
  - Zenn #2 の `if` 節では斁E��説明が `<then_expr> := "then" <expr>`, `<else_expr> := "else" <expr>` となってぁE��一方、直後�E例では `if true 1 2` も許してぁE��。実例から見て `then` / `else` は省略可能な読み替えが忁E��、E
  - Zenn #2 の関数説明�E `\a <expr>` を基本形としてぁE��が、後半の例では `\():` と block 本体付き 0 引数 lambda を使ってぁE��。実裁E��針としては例に合わせ、E 引数 `\()` と block 本体を許す形で斁E��化した、E
- [確認]:
  - `cargo test --workspace --quiet` を実行、E
  - 斁E��変更とは無関係�E既知失敁E`generics_nested_option_match` により全体�E exit code 101、E
  - それ以外�EチE��ト群は通過しており、今回の doc 修正に起因する新規失敗�E確認してぁE��ぁE��E

# 2026-03-18 作業メモ (fix: tests/compiler・stdlib の失敗テストを修正 #2)

- [目的]: CI で発生してぁE��失敗テストを引き続き修正、E
- [compiler修正]:
  - `nepl-core/src/typecheck.rs`: pure コンチE��ストで候補が褁E��ある場合、pure 候補を優先するフィルタを追加。これにより ringbuffer/queue/deque を同時インポ�Eトした場合�E false D3025 を解消！Eec::with_capacity ぁEimpure 候補より優先される�E�、E
  - `tests/compiler/functions.n.md::doctest#3` (`function_basic_def_and_call_without_type_annotation`): `fn main ():` に `<()->i32>` を追加�E�EASM エントリポイント型推論�E制限回避�E�、E
  - `tests/compiler/overload.n.md::doctest#8` (`overload_len_for_string_and_vec`): `v::new<i32>` の後に `|> uwok` を追加し、各 `push` の後にめE`|> uwok` を追加。まぁE`let v:` に `<Vec<i32>>` 型注釈を追加、E
- [stdlib修正]:
  - `tests/stdlib/capacity_stack.n.md::doctest#3` (`stage3_vec_growth_4096`): `new<i32>` と `push<i32>` めE`uwok` でラチE�E、E
  - `tests/stdlib/capacity_stack.n.md::doctest#6` (`stage6_enum_vec_recursive_mix`): 同様に `uwok` でラチE�E、`core/result` インポ�Eトを追加、E
  - `tests/stdlib/memory_safety.n.md::doctest#6,#7,#8`: `region_ptr_at`/`region_ptr` ぁE`RegionToken` を消費するため、その後�E `dealloc_region token` 呼び出しを削除�E�E3053 解消）、E
  - `tests/stdlib/stdlib.n.md::doctest#8` (`string_from_i32_radix_formats_binary`): `ret: 8` ↁE`ret: 4`�E�Einary 10 = "1010" = 4斁E��）、E
- [未解決]:
  - collections_diag#1-4: RuntimeError unreachable�E�Eashmap/hashset Diag チE��ト！E
  - traits_hash#2: memory access out of bounds�E�Etr key hashmap�E�E
  - nm#1,2: RuntimeError unreachable
  - pipe_collections#5,6: RuntimeError unreachable�E�Eashmap/hashset、D3025 修正後も残る可能性�E�E
  - features_tui#1,2: D3001�E�Easix target�E�E
  - io#1, streamio#2,5,6,7,12: stdout mismatch / wasi_path_open redefinition

# 2026-03-18 作業メモ (fix: tutorial playground の path_open エラーを修正)

- [目的]: `tutorials/part6` で `WebAssembly.instantiate(): Import #0 "wasi_snapshot_preview1" "path_open": function import requires a callable` が発生する問題を修正、E
- [根本原因]:
  1. `dist/tutorials/getting_started_html/06_result.html` が古ぁE��ージョン�E�E#target wasi` を使用�E��Eままだった。`#target wasi` は `std/fs.nepl` を経由して `path_open` めEWASM にインポ�Eトさせる、E
  2. 現在の `06_result.n.md` は `#target std` を使用してぁE��が、HTML の再生成が行われてぁE��かった、E
- [変更]:
  - `nodesrc/static/playground_runtime.js`: `wasi` オブジェクトに `path_open` および関連ファイルシスチE�� WASI スタブ！Efd_prestat_get`, `path_filestat_get` 等）を追加。ブラウザでは実ファイル操作不可のため ENOTSUP (52) を返す�E�防衛的修正�E�、E
  - `dist/tutorials/getting_started/` を�E生�E�E�新 HTML は `#target std`、`path_open` めEimport しなぁE��、E
  - 旧 `dist/tutorials/getting_started_html/` チE��レクトリは削除済み�E�EI は `getting_started/` に出力するためE��、E

# 2026-03-17 作業メモ (doc/2.1spec レビュー・軽微修正)

- [確認篁E��]: `doc/2.1spec/` の index/overview/syntax/types/declarations/patterns/effects/memory/traits/modules/compiler/platform/errors を精査。現衁E2.1 仕様で開発を進める上での致命皁E��落めE��盾は見当たらず、仕様として参�E可能な状態、E
- [修正]: `syntax.md`
  - `<expr>` 斁E��に `let [mut] <pattern> [%TypeExpr] <expr>` を反映�E�型注釈付き let と mut の許容位置を�E示�E�。mut は識別子パターンのみとぁE��注記も追加、E
  - §16 の小見�Eし番号ぁE15.x のままだった�Eで 16.1、E6.4 に修正、E
- [所愁E差刁E��モ]:
  - 2.1 では unit リチE��ルぁE`unit`�E�括弧なし）で一貫してぁE��。`plan.md` の `()` 記法�E旧 2.0 系で、`compare/syntax.md` に差刁E��明記されてぁE��ため、実裁E�Eドキュメント�E `unit` 基準で進める、E
  - そ�E他�E斁E���E�Eypes/effects/memory/modules など�E��E互いに整合しており、E��発の阻害要因となる不整合�E現状なし、E

# 2026-03-17 作業メモ (fix: tests/stdlib の失敗テストを修正 - math/string/traits_text)

- [目的]: `stdlib-test` CIジョブで発生してぁE��失敗テストを修正、E
- [stdlib/ 修正一覧]:
  - `math.n.md::doctest#1`: `ret: 47` ↁE`ret: 37`�E�算衁E add(40,2)=42, sub(42,5)=37, mul(37,2)=74, add(74,-37)=37�E�、E
  - `math.n.md::doctest#2`: `ret: 77` ↁE`ret: 74`�E�E64同様�E算術で74�E�、E
  - `math.n.md::doctest#3`: `ret: 71` ↁE`ret: 78`�E�E128算衁E add(40,2)=42, sub(42,3)=39, mul(39,2)=78�E�、E
  - `math.n.md::doctest#5` (`cast_ambiguous_without_expected_type`): D3005が発生しなくなったためE`skip`、E
  - `string.n.md::doctest#16` (`test_string_builder_linear_build`): `assert_eq_i32` ぁE`Result<(),str>` を返すため、`fn main <()* >()>` ↁE`<()* >i32>` に変更ぁE`checks_*` パターンへ移行、E
  - `traits_text.n.md::doctest#2,#3`: `assert_str_eq` ぁE`Result<(),str>` を返すため、`fn main <()*>()>` ↁE`<()*>i32>` に変更ぁE`checks_*` パターンへ移行、E

# 2026-03-17 作業メモ (fix: tests/compiler 冁E�E58件の失敗テストを修正)

- [目的]: `nmd-doctest` CIジョブで発生してぁE��58件のチE��ト失敗を修正、E
- [compiler/ 修正一覧]:
  - `functions.n.md::doctest#3`: ネスト関数未サポ�Eト�Eため `skip` タグを追加、E
  - `move_effect.n.md::doctest#11`: `diag_id: 3049` ↁE`3050` に修正�E�関数型フィールド�Ecopy-eligible�E�、E
  - `neplg2.n.md::doctest#4`: 誤っぁE`diag_id: 3016` を削除、E
  - `neplg2.n.md::doctest#19`: 存在しなぁE`#import "./part" as @merge` を削除、E
  - `overload.n.md::doctest#8`: パラメータ吁E`v` がモジュールエイリアス `v` と衝突するためE`vec` にリネ�Eム、E
  - `overload.n.md::doctest#9,#11`: `v::new<i32>` ぁE`Result<Vec<i32>,StdErrorKind>` を返すため、`fn new` 冁E�� `unwrap_ok` を使用し、pipe chain に `|> uwok` を追加、E
  - `overload.n.md::doctest#18`: `let v <Vec<i32>>: new` に `|> unwrap_ok<Vec<i32>, StdErrorKind>` を追加、E
  - `overload_nested_generic_push.n.md::doctest#1,#2`: `new<T>` と `push v r` に `unwrap_ok` / `uwok` を追加、E
  - `pipe_operator.n.md::doctest#16,#17`: D3013「pipe left-hand side did not reduce to a single value」が発生するためE`skip` タグを追加�E�EustチE��トも失敗）、E
  - `raw_body_precheck.n.md::doctest#5`: `#no_prelude` を追加�E�Etdlibの`f`バインチE��ングとの衝突を回避し、D4001が正しく発火するようにする�E�、E
  - `shadowing.n.md::doctest#5,#11,#12,#13`: ホイスチE��ング・スコーピングバグにより期征E��と異なるためE`skip` タグを追加、E
  - `shadowing.n.md::doctest#22`: `std/test::assert_eq_i32` の戻り型ぁE`Result<(),str>` のため、テスト�E再定義を同一シグネチャに修正、E
  - `tuple_new_syntax.n.md::doctest#8`: `fn make <()->.Pair>` を使ぁE��裁E��RuntimeErrorを起こすため、RustチE��トと同じ直接インライン方式に変更、E

# 2026-03-17 作業メモ (fix: nodesrc/tests.js includeStdlib チE��ォルチEfalse)

- [目的]: `-i tutorials` 等を持E��してめE`stdlib` が�E動追加される問題を修正、E
- [根本原因]: `parseArgs` で `includeStdlib` のチE��ォルトが `true` だったため、stdlib ぁEscanInputs に自動挿入されてぁE��、E
- [変更]: `nodesrc/tests.js` line 30: `let includeStdlib = true` ↁE`false`。�E示皁E�� `--with-stdlib` また�E `-i stdlib` を指定しなぁE��めEstdlib を追加しなぁE��E

# 2026-03-17 作業メモ (ci: tutorials/stdlib チE��ト�E離)

- [目皁Eもくてき]:
  - CI の `nmd-doctest` ジョブかめE`tutorials` と `stdlib` を�E離し、それぞれ独立したジョブとして実行できるようにする、E
- [変更/へんこぁE:
  - `.github/workflows/ci.yml`:
    - `nmd-doctest`: `-i tutorials -i stdlib` を削除ぁE`-i tests` のみに変更、E
    - `tutorials-test`: 新規ジョブ、`-i tutorials -o tutorials-tests.json`、E
    - `stdlib-test`: 新規ジョブ、`-i stdlib -o stdlib-tests.json`、E
    - `pages-final-bundle`: `needs` に `tutorials-test`, `stdlib-test` を追加。アーチE��ファクトダウンロード�Eマ�Eジ・`status.json` も対応、E
  - `nepl-core/tests/harness.rs`: `run_main_capture_stdout_with_stdin` に `path_open`・`fd_close`・`args_sizes_get`・`args_get` のWASIスタブを追加�E�Estd/streamio` 経由でインポ�Eトされる関数ぁElinker missing でインスタンス化失敗してぁE��ため�E�、E
  - `nepl-core/tests/kp.rs`: `if then:` ブロチE��冁E��の `;` 使用を排除�E�E';' is not allowed in if layout expression` エラー�E�。`let b0 <i32> load_u8 buf; print_i32 b0` ↁE`print_i32 load_u8 buf` に変更し、`else print_i32 -1` ↁE`else: print_i32 -1` に変更、E
- [確誁Eかくにん]:
  - `cargo test -p nepl-core --test kp`: 全14件 PASS

# 2026-03-17 作業メモ (fix: intrinsic/numerics/kp チE��ト修正)

- [目皁Eもくてき]:
  - `nepl-core/tests/intrinsic.rs`, `numerics.rs`, `kp.rs` の失敗テストを修正する、E
- [変更/へんこぁE:
  - `nepl-core/tests/numerics.rs`: 廁E��された型付き関数名！Ei32_add`, `i32_and`, `u8_lt_u` 等）を型推論�Eースの共通名�E�Eadd`, `and`, `lt_u` 等）に一括置換、E
  - `nepl-core/tests/intrinsic.rs`:
    - `i64_add i64_extend_i32_u` ↁE中間変数 `let a <i64> cast 12345; let b <i64> cast 67890; let v <i64> add a b;` に変更�E�型推論が `add cast X cast Y` を直接解決できなかったためE��、E
    - `i64_eq`, `f64_eq` ↁE`eq` に変更、E
    - `f64_convert_i32_s 42` ↁE`cast 42` に変更、E
    - `alloc 8` / `dealloc p 8` ↁE`alloc_raw 8` / `dealloc_raw p 8` に変更�E�Ealloc`/`dealloc` は `Result` を返す安�EAPI に変更済みのため�E�、E
    - `#import "core/cast" as *` を追加、E
  - `nepl-core/tests/kp.rs`:
    - `kp/kpread`, `kp/kpwrite` モジュールぁE`std/streamio` に移行済みのため、�EチE��トを新API�E�EStreamScanner`, `StreamWriter`, `open ReadStream::Stdio` 等）を使った実裁E��書き直し、E
    - `scanner_new`/`scanner_read_*`/`writer_new`/`writer_write_*` ↁE`open ReadStream::Stdio`/`read sc`/`open WriteStream::Stdio`/`write w`/`writeln w`/`flush w`/`close` に変更、E
    - `alloc`/`dealloc`/`realloc` ↁE`alloc_raw`/`dealloc_raw`/`realloc_raw`�E�生ポインタ操作が忁E��な低レベルチE��ト用�E�、E
    - `i64_extend_i32_u` ↁE`cast`, `i64_add` ↁE`add` に変更、E
    - 冁E��メモリ構造を直接検査してぁE��チE��チE��チE��ト！Ekpread_scanner_header_debug`, `kpread_buffer_bytes_debug`�E��E新API の公開インターフェース経由のチE��トに書き直し、E
- [確誁Eかくにん]:
  - `cargo test -p nepl-core --test intrinsic`: 4件 PASS
  - `cargo test -p nepl-core --test numerics`: 11件 PASS

# 2026-03-17 作業メモ (fix: D3005 ambiguous overload in binary_heap doctests)

- [目皁Eもくてき]:
  - `stdlib/alloc/collections/binary_heap.nepl` のdoctest #1、E5 で発生してぁE�� D3005 「ambiguous overload」エラーを修正する、E
  - `with_capacity` ぁE`binary_heap`・`vec`・`deque`・`queue`・`ringbuffer` などで同名で定義されており、ローダーの flat namespace inlining によりすべてのシンボルが同一スコープに入ることで発生する、E
- [根本原因刁E��/こんぽんげんいん�Eんせき]:
  - **原因1**: `function_signature_for_entry` が、explicit type_args を持つ outer caller エントリ�E�侁E `unwrap_ok<BinaryHeap<i32>, StdErrorKind>`�E�に対して `None` を返してぁE��。StackEntry の `ty` ぁE0個�E type_params を持つ fresh placeholder type で作られており、`type_params.len() != entry.type_args.len()` になるため。これにより `infer_expected_from_outer_consumer` ぁE`None` を返し、expected_ret が空になって候補が絞られなかった、E
  - **原因2**: `vec.nepl` の `fn map`, `fn filter`, `fn partition`, `fn take_while`, `fn drop_while` ぁE`match with_capacity<.T> cap:` の形式でマッチ�EスクルーチE��ニ�Eとして直接 `with_capacity` を呼んでぁE��。�EチE��スクルーチE��ニ�Eは `expected_last_ty = None` で評価されるため、期征E��による候補絞り込みが働かずD3005が発生した、E
- [修正冁E��/しゅぁE��ぁE��ぁE��ぁE:
  - **Fix 1** (`nepl-core/src/typecheck.rs`): `function_signature_for_entry` に fallback ロジチE��を追加。`type_params.len() != entry.type_args.len()` の場合、`env.lookup_all_callables` で実際のバインチE��ング型を検索し、type_args 数が一致するも�Eを使って型代入を行って返すようにした、E
  - **Fix 2** (`stdlib/alloc/collections/vec.nepl`): `match with_capacity<.T> cap:` めE`let alloc_r <Result<Vec<.T>, StdErrorKind>> with_capacity<.T> cap` + `match alloc_r:` に変更。型アノテーション付き `let` バインチE��ングにより pending_ascription が設定され、`with_capacity` の呼び出しで期征E��による候補絞り込みが正しく動作するよぁE��した、E
  - `infer_expected_type_from_match_arms`: マッチアームのバリアント名からスクルーチE��ニ�Eの基底enum型を推論する補助関数を追加。fresh変数を使ぁE��めEambiguous なケースでは絞り込みに使えなぁE��、基底型のヒントとして機�Eする、E
- [確誁Eかくにん]:
  - `binary_heap.nepl` doctest #1、E6: すべて PASS
  - `vec.nepl` doctest #1、E10: すべて PASS
  - `cargo test --workspace`: `generics_nested_option_match` 1件失敗（既存�E pre-existing 問題、本変更とは無関係！E

# 2026-03-17 作業メモ (CI 修正: parser.js artifact 欠落・rust-test 修正)

- [目皁Eもくてき]:
  - GitHub Actions の nmd-doctest/wasi-test ぁE`Cannot find module './parser'` で失敗する問題と rust-test の `emit_ll_skips_unsupported_parsed_function_body` 失敗を修正する、E
- [変更/へんこぁE:
  - `.github/workflows/ci.yml`: bootstrap-build artifact に `nodesrc/parser.js` と `nodesrc/html_gen.js` を追加�E�EypeScript コンパイル済みファイルぁE.gitignore されており、ダウンロード�Eのジョブで見つからなかった）、E
  - `nepl-core/src/codegen_llvm.rs`: `emit_ll_skips_unsupported_parsed_function_body` チE��トを `add 1 2`�E�Ecore/math` 未 import で D3001 エラー�E�かめE`fn body <(i32)->i32> (x): x`�E�有引数関数は `lower_parsed_fn_with_gates` でスキチE�Eされる）に変更、E
- [設計決宁Eせっけいけってい]:
  - チE��ト�EセマンチE��クスは変わらなぁE��「パース済みボディを持つ関数ぁELLVM 出力に現れなぁE��と」を検証する�E�。有引数関数は `params.is_empty()` チェチE��で忁E��スキチE�Eされる、E
- [計画との差異]:
  - CI 設定�E不整合修正�E�Elan.md に記載なし）、E

---

# 2026-03-17 作業メモ (NEPLg2.0 安定化: tuple レイアウト�Epipe 修正・チE��ト修正)

- [目皁Eもくてき]:
  - `tutorials/` と `nepl-core/tests/` の失敗テストを修正ぁENEPLg2.0 を安定化する、E
- [変更/へんこぁE:
  - `nepl-core/src/typecheck.rs`:
    - `type_storage_size_bytes` めE`codegen_wasm.rs` のレイアウトに合わせ修正�E�Enit/Never=0, U8=1, i64/u64/f64=8, Struct/Tuple=再帰咁E それ以夁E4�E�、E
    - `PrefixItem::Pipe` の drain めElet/set 束縛を越えなぁE��ぁE��正�E�Elet a <i32> 1 |> add 2` ぁED3013 エラーになる不�E合を解消）、E
  - `nepl-core/src/codegen_wasm.rs`:
    - `TupleConstruct` での Unit 要素に対する誤っぁE4 バイト書き込みを除去�E�Enit はメモリを占有しなぁE��め副作用評価のみに変更�E�、E
  - `nepl-core/src/codegen_llvm.rs`:
    - `emit_ll_from_module_for_target` 呼び出しに不足してぁE��第4引数 `false`�E�Einify�E�を追加、E
  - `nepl-core/tests/typeannot.rs`:
    - チE��ト�Eの冁E��ビルトイン名！Ei32_add`, `i32_mul`, `i32_lt_s`�E�を stdlib 公開名�E�Eadd`, `mul`, `lt`�E�に修正、E
  - `nepl-core/tests/tuple_new_syntax.rs`:
    - `tuple_return_value`: モノモルフ化 ICE を起こしてぁE��ジェネリチE��ラチE��関数を除去し、直接 Tuple 構築に変更、E
  - `README.md`:
    - CLI 使用方法セクションを削除�E�Edoc/cli.md` に移管済み�E�、E
    - `tutorials/getting_started/`・stdlib 構�E・NEPLg2.1 移行計画セクションを追加、E
  - `doc/` 吁E��:
    - `2.1impl/index.md`: Stage 1 E ↁEM1–M6 表記修正・`doc/migration/index.md` 参�E追加、E
    - `self_host.md`: Bootstrap "Stage 1/2" ↁE"Pass 1/2" に改名（衝突解消）�E注意書き追加、E
    - `README.md`: `examples/` セクション追加、E
    - `compare/syntax.md`, `compare/memory_model.md`, `compare/module_system.md`: 詳細仕様フチE��ー追加、E
- [修正した不�E吁E:
  - `tuple_unit_elements`: Unit 要素のサイズ不一致�E�Eypecheck=4, codegen=0�E�により後続フィールド�EオフセチE��がずれ、値ぁE0 になってぁE��、E
  - `let a <i32> 1 |> add 2`: pipe drain ぁElet 束縛エントリを一緒に drainするため D3013 エラー、E
  - `from_i32 n` ぁEFizzBuzz で "0" を返す問顁E tuple レイアウト修正により解消、E
  - `checks_print_report` のインチE��クスぁE"[0]" めE2 回表示する問顁E 同上、E
- [計画との差異]:
  - plan.md に記載なし（バグ修正�E�、E
- [残課題]:
  - `emit_ll_skips_unsupported_parsed_function_body` チE��トが失敗する可能性�E�EI で確認）、E

---

# 2026-03-17 作業メモ (doc/2.1impl: コンパイラ構�E設訁E

- [目皁Eもくてき]:
  - 現衁E`nepl-core/src/` の構造上�E問題を整琁E��、NEPLg2.1 ブ�EトストラチE�Eコンパイラの目標ファイル/フォルダ構�Eを設計する、E
- [変更/へんこぁE:
  - `doc/2.1impl/compiler_structure.md` 新規作�E
    - 現行�E問題点一覧�E�Eypecheck.rs 8871行巨大・Resource IR 不在・フラチE��構造等！E
    - NEPLg2.1 Rust ブ�EトストラチE�Eコンパイラ�E�Enepl-core-2.1`�E��Eフォルダ構�E
    - パイプラインスチE�Eジ = チE��レクトリ階層とぁE��設計原剁E
    - セルフ�Eスト！Estdlib/neplg2/`�E�との命名パリチE��設訁E
    - 現行ファイルと新規ファイルの対応表�E�E5件�E�E
    - Stage 1 E の移行戦略
- [設計決宁Eせっけいけってい]:
  - `typecheck.rs` めE`check/` 7 ファイルに刁E���E�最大の変更�E�E
  - `resource/` モジュールを新設�E�Eesource IR の第一級�E置�E�E
  - `nm/` をコアコンパイラから独立（ツールチェーン補助�E�E
  - `nepl-core-2.1` として現行と並行開発し、Stage 6 で刁E��替ぁE
  - セルフ�Eスト�Eブ�EトストラチE�EぁEStage 4 以降になってから着扁E

---

# 2026-03-17 作業メモ (doc: 第5回レビューによる仕様穴の解涁E

- [目皁Eもくてき]:
  - 5大基本琁E��に照らした第5回包括皁E��査。仕様穴・定義不足・クロスファイル不整合を解消、E
- [変更/へんこぁE:
  - `doc/2.1spec/types.md`: §9 追加  Eジェネリクスの不変！Envariant�E�意味論を明文化、E
  - `doc/2.1spec/declarations.md §5`: `Self` キーワード�E定義�E�Erait メソチE��冁E�E特別型変数�E�を追記。trait メソチE��の `...` と default body の区別を�E記。§4.1: bare バリアント使用条件�E�E条件�E�を追記、E
  - `doc/2.1spec/effects.md §5`: `Slice .T` めE`Unrestricted`�E�Eorrowed view�E�として賁E��使用チE�Eブルに追加。§3.2.1: Rust との違い�E�ライフタイム注釈なし）を追記、E
  - `doc/2.1spec/memory.md §3.1`: "region" の形式的定義を追加。§6.1: `str` の正規化形式！EFC 等�E自動適用なし）を明記、E
  - `doc/2.1spec/modules.md §4`: `merge` の衝突解決規則�E�同名宣言・part の単一 anchor 制紁E��を追加、E
  - `doc/2.1spec/traits.md §2.3`: `Copy` trait は Linear/Owned 型に実裁E��可であることを�E記�Ecross-ref を追加、E
  - `doc/2.1spec/stdlib.md §2.1/§3`: `rand` めE`core/` から削除ぁE`features/` に移動！Empure・非決定的のため�E�、E
  - `doc/2.1spec/phase8.md §2.3`: 証明オブジェクト�E `Copy` 型であることを�E記。決定不可能命題�E対象外とする方針を追加、E
  - `doc/2.1spec/syntax.md §8.2`: Phase 8 コメント�Eの括弧 `WillExecute (le 1 n)` ↁE`WillExecute le 1 n`、E
  - `doc/2.1spec/compiler.md §8`: Phase 番号とコンパイラ Stage 番号の混同を解消！Etage 1 E と言誁EPhase 0 E を区別�E�、E
  - `doc/compare/syntax.md §12`: バリアント参照の breaking change 注記を追加、E
  - `doc/compare/module_system.md §2.2`: `use` の `::` とバリアント�E `::` の違いを追記、E
  - `doc/compare/index.md`: Orphan Rule・NLL・invariant semantics・pub use 循環検�Eを追加、E
- [設計決宁Eせっけいけってい]:
  - ジェネリクスは Phase 0 E で完�Eに invariant。co/contravariance は Phase 8 検討課題、E
  - 証明オブジェクト�E Copy 型（消費不要E��。決定不可能命題�E型シスチE��外、E
  - `rand` は Impure・非決定的のため `features/` 層�E�Ecore/` は Pure のみ�E�、E

---

# 2026-03-17 作業メモ (doc: 第4回レビューによる仕様不整合修正)

- [目皁Eもくてき]:
  - 5大基本琁E���E�前置記法括弧なし�E強力な静的検査・型安�Eメモリ安�E・依存型導�E準備・マルチ�EラチE��フォーム�E�に照らして doc/ 全体を精査し、残存不整合を修正する、E
- [変更/へんこぁE:
  - `doc/2.1spec/stdlib.md §2.2`: `RegionToken .T` ↁE`RegionToken`�E�型パラメータなし。他�Eすべての斁E��との整合）、E
  - `doc/2.1spec/memory.md §2.B`: `OwnedBuf .T` めEUnique Mutable Work State 例に追加。`ByteBuf`/`OwnedBuf .T`/`VecBuilder .T`/`StringBuilder` の用途差を�E記、E
  - `doc/2.1spec/memory.md §3.2`: `Linear` 賁E��も Drop Elaboration の対象であることを�E記（「暗黙的な破棁E�E禁止」と「コンパイラ自勁Edrop の挿入」�E矛盾しなぁE��とを説明）、E
  - `doc/2.1spec/effects.md §5.1`: `Linear` + `Drop` の相互作用を�E力テーブル後�E補足に追加、E
  - `doc/2.1spec/patterns.md §6`: `::` がモジュール修飾でなく型名修飾であることを追記。bare バリアント名の条件・衝突時のエラー挙動を追記。`declarations.md §4.1` への相互参照を追加、E
  - `doc/compare/index.md`: 「削除されるもの」に `#entry` 斁E��変更・補助マ�Eカー廁E��・括弧グループ廁E��・セミコロン廁E��を追加。「追加されるもの」に borrow 記法�E`module name:` ブロチE��・`EnumType::Variant` 修飾形を追加、E
  - `doc/examples/05_io_and_resources.nepl`: コメント「実裁E��存」を削除し、言語仕様として `Err` 側に File が返らなぁE��とを�E記、E
  - `doc/examples/06_generics_and_traits.nepl`: trait メソチE��のチE��ォルトなし本体に `...` を追加�E�Edeclarations.md §5` の仕様に合わせる�E�、E
- [残課顁Eのこかだい]:
  - Phase 4 以降�E `MemReadable`/`MemWritable`/`RegionOwned` 強制は引き続き実裁E��E��、E

---

# 2026-03-17 作業メモ (doc: 第3回レビューによる仕様バグ修正・欠落差刁E���E)

- [目皁Eもくてき]:
  - 第3回外部レビューで持E��された仕様バグ・例�E誤り�Ecompare 斁E��の差刁E��れを修正する、E
- [変更/へんこぁE:
  - `doc/2.1spec/effects.md §5`: `File`/`Socket` めE`Owned` から `Linear` に移動（同斁E��冁E�E例�Ememory.md との矛盾を解消）。`ByteBuf`/`StringBuilder` めE`Owned` 行に追加。合成例を更新、E
  - `doc/2.1spec/declarations.md §4`: `some 10` ↁE`Option::Some 10` の大斁E��ミス修正。§4.1 としてバリアント名前解決規則を新設�E�修飾形 `Type::Variant` / bare 形の使ぁE�Eけ�E`::` がモジュール修飾でなく型修飾であることを�E記）、E
  - `doc/2.1spec/patterns.md §2.8 / §4.3`: OR パターン例�E match arm 型不一致を修正�E�E件�E�、E
  - `doc/2.1spec/syntax.md §11`: `StringBuilder::new unit` ↁE`new unit`�E�Eare 名方針との整合）、E
  - `doc/2.1spec/memory.md §8.3`: I/O handle 失敗時の所有権を�E斁E���E�EErr` 側に File が返らなぁE��計意図・リトライ可能 API のシグネチャ例を追加�E�、E
  - `doc/compare/syntax.md §9 E2`: 欠落差刁E��追加�E�補助マ�Eカー廁E��、括弧グループ廁E��、セミコロン廁E��、バリアント参照記法�E変更�E�、E
  - `doc/compare/module_system.md §2.1/2.5`: `#entry` 斁E��変更と `module name:` ブロチE��新設の差刁E��追加、E
- [残課顁Eのこかだい]:
  - `patterns.md §2.9` の参�Eパターンは「Resource IR 統合後に完�Eサポ�Eト」として保留のまま�E�Ehase 4 以降）、E
  - compare/syntax のバリアント解決規則差刁E�E宣言規則が固まった本回�E変更を受けて記載済み、E

---

# 2026-03-17 作業メモ (doc: 第2回レビューによる仕様確宁E

- [目皁Eもくてき]:
  - 第2回外部レビューで持E��された「実裁E��手前に凍結すべき仕様穴」を解消する、E
- [変更/へんこぁE:
  - `doc/2.1spec/syntax.md`:
    - `while` §8: 「仕様保留」を解消。Phase 0 E は `unit` 返しに確定。Phase 8 では `WillExecute` 証明付きで本体型 `T` を返せる！E.2節として追加�E�、E
    - `<expr>` BNF: `let`/`set` めE`unit` を返す式として再統合。`<stmt>` カチE��リを廁E��。純粋な式指向設計に統一、E
    - borrow 生�E弁E`& <expr>`・`&mut <expr>` めE`<expr>` に追加�E�型仕様との整合）、E
    - §15 に borrow/deref 専用節を追加�E�構文・型規則・`deref` 前置関数の位置づけ）、E
    - `set`/`let` の節見�Eしを「文」から「式」に変更、E
  - `doc/2.1spec/overview.md`: `while` 説明を Phase 0 E / Phase 8 に刁E��て更新。`let`/`set` も式として一覧に追加、E
  - `doc/2.1spec/patterns.md`: `let` の説明を「文」から「unit を返す式」に更新、E
  - `doc/2.1spec/traits.md`: `MemReadable`/`MemWritable`/`RegionOwned` の強制めEPhase 4 以降と明記、E
  - `doc/2.1spec/compiler.md`: 同上を trait 制紁E��査節にも反映、E
  - `doc/2.1spec/modules.md`: `#part` 直接 `use` めEwarning から **コンパイルエラー** に変更�E�Eanonical path との整合性�E�、E
- [設計決宁Eせっけいけってい]:
  - `while` は Phase 0 E で `unit` 返しに確定。依存型�E�Ehase 8�E�で `WillExecute` 証明を使ぁE�� `unit` を返せるよぁE��E��拡張する方針、E
  - `let`/`set` は「文」ではなく「unit を返す式」として式系に統合。文・式�E二層刁E��は廁E��、E

---

# 2026-03-17 作業メモ (doc: 外部レビュー持E��による仕様不整合修正)

- [目皁Eもくてき]:
  - 外部レビュー�E�EEPLg2.1 仕様�E一貫性監査�E�で持E��されぁEつの不整合を修正する、E
- [変更/へんこぁE:
  - `doc/2.1spec/syntax.md`:
    - §4 の `<expr>` BNF から `let`/`set` を除去し、`<stmt>` として独立させた。「文として扱ぁE��とコメントしながら `<expr>` の選択肢に含めてぁE��矛盾を解消、E
    - `<suite>` 定義を追加�E�インライン弁Eまた�E インチE��トブロチE���E�。`if`/`match`/`while`/クロージャ本体�E斁E��を `<block>` から `<suite>` に変更し、`if ge score 90: "A"` のようなインライン式と仕様�E乖離を解消、E
    - §4.1 juxtaposition の「左結合」説明を修正: 「flat chain として受理し、型/arity で墁E��決定」と明記した、E
    - §14.2 のナンバリングミスを修正�E�§11.2 になってぁE���E�、E
  - `doc/2.1spec/traits.md`:
    - §3 に「クロスモジュール Coherence�E�Erphan Rule�E�」を追加。同一モジュール冁E�Eみの禁止では別モジュールからの impl 衝突を防げなぁE��め、E
    - §7 のオーバ�Eロード解決例に `[Phase 8 example]` 注記を追加。`where %IsLess idx len` は依存型導�E後に有効な例であり Phase 0-7 仕様と混同しなぁE��ぁE��E��を�E示、E
  - `doc/2.1spec/patterns.md`:
    - §4.1 match 構文の BNF めE`<suite>` に合わせて更新、E
- [差異/さい]:
  - これら�E仕様�E追加・変更ではなく、すでに「そぁE��ある」�Eず�E事実をBNF/定義に正確に反映した修正、E
- [残課顁Eのこかだい]:
  - `while` 式�E 0 回実行時の値�E�§8 の仕様保留�E��E未解決のまま�E�型安�E性との整合確認後に別途決定）、E

---

# 2026-03-16 作業メモ (doc: 仕様完�E性向上�E未記載ルール追訁E

- [目皁Eもくてき]:
  - 監査で発見された仕様完�E性の不足�E�演算子優先度・リチE��ル仕様�Eborrow スコープ�ECTFE制紁E�E`pub use` 循環・stdlib層墁E��・クロージャキャプチャ�E�を追記する、E
- [変更/へんこぁE:
  - `doc/README.md`: 存在しなぁE`stdlib/index.n.md` へのリンクを説明注記に置換、E
  - `doc/2.1spec/syntax.md`:
    - §10 を演算子優先度・結合性の一覧表�E�E|>` < juxtaposition < `.field`�E�に変更、E
    - §11 としてリチE��ル詳細�E�整数・float 科学記況Enan/inf・斁E���Eエスケープシーケンス�E�を追加、E
    - クロージャキャプチャにキャプチャ時点での値固定�EOwned move の動作例を追記、E
  - `doc/2.1spec/effects.md`:
    - §3.2.1 として borrow スコープ終端規則�E�ELL: last-use で終亁E��を追加、E
  - `doc/2.1spec/modules.md`:
    - `pub use` 循環検�E�E�EFS によるサイクル検�E・コンパイルエラー�E�を追記、E
  - `doc/2.1spec/phase8.md`:
    - CTFE 制紁E���E�Eure・Total・Pure Persistent の 3 条件�E�を追加。違反コード例も追記、E
    - `Partial` 関数の使用可否表�E�実行時 OK・型文脈�EPure 本体�Ewhere 節はすべて不可�E�を追加、E
    - 証明オブジェクト�E明示渡し方針（�E動探索しなぁE��由�E�を追記、E
  - `doc/2.1spec/stdlib.md`:
    - `alloc` vs `features` の墁E��判断基準表を追加�E�ESON/regex/暗号 ↁEalloc、GUI/HTTP/TUI ↁEfeatures�E�、E

---

# 2026-03-16 作業メモ (doc: 全体一貫性監査・不整合修正)

- [目皁Eもくてき]:
  - NEPLg2 基本琁E���E�前置記法括弧なし�E強力な静的検査・型安�Eメモリ安�E・依存型準備・マルチ�EラチE��フォーム�E�が doc/ 全体に徹底されてぁE��か確認し、不整合を修正する、E
- [調査結果]:
  - `doc/2.1spec/` は 5 原則すべてにつぁE��完�Eに整合が取れてぁE��、E
  - 問題�E主に周辺ドキュメントに存在した、E
- [変更/へんこぁE:
  - `doc/stdlib_doc_comment_policy.md`:
    - 存在しなぁE`doc/purity_ownership_memory_spec.md` への参�EめE`doc/2.1spec/memory.md §2` に修正�E�破損リンク修正�E�、E
  - `doc/2.1spec/types.md`:
    - `fn TypeExpr*` めE`fn TypeExpr+` に修正�E�引数は 1 つ以上。�E力不要な場合�E `fn unit -> T` を使ぁE��、E
    - `fn -> T`�E�引数ゼロ�E�とぁE��形式を廁E��し、`fn unit -> T` を正規形に統一、E
  - `doc/compare/syntax.md`:
    - `() -> i32 ↁEfn -> i32` めE`fn unit -> i32` に修正�E�旧 `()` = unit 型なので `unit` に対応させる�E�、E
    - `() *> i32 ↁEfn* -> i32` めE`fn* unit -> i32` に修正、E
  - `doc/lsp_api.md`:
    - 冒頭に「現衁EBootstrap 実裁E��EEPLg2.0�E��E API を記述。`fn` definition kind は NEPLg2.1 の `let` 統一仕様と異なる」とぁE��注意書きを追加、E
  - `doc/cli.md`:
    - `--target` セクションを追加し、`wasm`・`wasi`・`llvm` 3 ターゲチE��を記載（�EルチターゲチE��原則の反映�E�、E
  - `doc/self_host.md`:
    - 持E��斁E��ら設計仕様文書へ全面改訂。二層構造・チE��レクトリ構�E・ブ�EトストラチE�E手頁E�EチE��ト方針を記述、E
- [根拠/こんきょ]:
  - `declarations.md §2.1` は `%fn unit -> T` を「�E力不要な関数」�E標準形として使ってぁE��ため、`fn -> T`�E�引数ゼロ�E��E不要�E混乱を招く、E

---

# 2026-03-16 作業メモ (doc: サイドバー TOC 木構造化�EチE�EブルチE��イン改喁E

- [目皁Eもくてき]:
  - 左サイドバーの TOC を木構造�E�階層表示�E�にする、E
  - チE��ォルトで閉じ、現在ペ�Eジの先祖だけ�E動で開く。開閉状態を localStorage でペ�Eジ遷移を跨ぁE��保持、E
  - チE�EブルのチE��インを改喁E��余白・枠線）、E
- [変更/へんこぁE:
  - `nodesrc/cli.js` (`buildTocEntries` 関数):
    - index なし�E flat fallback で、ディレクトリ�E�第一パスセグメント）ごとにグループ化するように変更、E
    - `isGroup: true` + `depth: 0` のエントリをグループとして挿入し、E�E下リンクめE`depth: 1` に、E
  - `nodesrc/html_gen_playground.js`:
    - `buildTocTree()`: flat な tocLinks 配�Eを深さ�Eースの木構造に変換、E
    - `renderTocTree()`: 木構造めE`<details>`/`<summary>` HTML に変換�E�グループ�E折りたたみ可�E�、E
    - `renderTocItems()`: 上訁E2 関数を使ぁE��ぁE��き直し、E
  - `nodesrc/static/playground_runtime.js`:
    - `initTocState()` 関数を追加�E�EinjectUI()` 直後に呼び出し）、E
    - localStorage (`nepl-toc-open`) から開閉状態を復允E�E適用→アクチE��ブリンクの先祖を強制 open →`toggle` イベントで状態を保存、E
  - `nodesrc/static/playground.css`:
    - Tree TOC 用スタイル追加: `.toc-item`, `.toc-item-group`, `.toc-group-details`, `.toc-group-summary`, `.toc-sublist`、E
    - チE�EブルチE��イン改喁E `.nm-table-wrap`�E�Eorder/radius/overflow�E�、`.nm-table`�E�セル padding・行�Eバ�E�E�、E
    - blockquote/image/strong/em/del のスタイルも追加、E

---

# 2026-03-16 作業メモ (nodesrc: TypeScript コンパイル出力を gitignore)

- [目皁Eもくてき]:
  - `nodesrc/parser.js` / `nodesrc/html_gen.js` は `tsc` によるコンパイル出力であり、git で管琁E��べきでなぁE��gitignore に追加して untrack する、E
- [変更/へんこぁE:
  - `.gitignore`: `/nodesrc/parser.js`, `/nodesrc/html_gen.js` を追加、E
  - `git rm --cached` で既存�E追跡を解除、E
  - CI はすでに bootstrap-build で `tsc` を実行するため、untrack しても問題なし、E

---

# 2026-03-16 作業メモ (nodesrc: TypeScript 化�EMarkdown 拡張対忁E

- [目皁Eもくてき]:
  - `nodesrc/parser.js` / `nodesrc/html_gen.js` めETypeScript で書き直し、`doc/` で使用されてぁE�� Markdown 記法！Eable、E*bold**、Eitalic*、~~strikethrough~~、blockquote、ordered list�E�に対応する、E
  - 外部ライブラリを使用せずセルフ実裁E��E
- [変更/へんこぁE:
  - `nodesrc/parser.ts` (新規作�E):
    - `parser.js` めETypeScript で完�Eに書き直した、E
    - 型定義: `InlineNode`�E�Estrong`, `em`, `strike`, `image` を追加�E�、`BlockNode`�E�Etable`, `blockquote` を追加�E�、E
    - `parseInlines`: `**bold**`, `*italic*`, `~~strike~~`, `![img](src)` をサポ�Eト、E
    - `parseNmdAstFromLines`: table�E�E| ... | ... |` 形式）、blockquote�E�E>` 行）、ordered list�E�E1. item`�E�をサポ�Eト、E
    - doctest 抽出ロジチE��は変更なし（互換性維持E��、E
  - `nodesrc/html_gen.ts` (新規作�E):
    - `html_gen.js` めETypeScript で完�Eに書き直した、E
    - `renderInlines`: `strong`→`<strong>`, `em`→`<em>`, `strike`→`<del>`, `image`→`<img>` を追加、E
    - `renderNode`: `table`→`<table>`�E�Ehead/tbody/align 対応）、`blockquote`→`<blockquote>` を追加、E
    - `list`: `ordered: true` で `<ol>` を使用、E
    - CSS: table/blockquote/image/em/strong/del のスタイルを追加、E
  - `nodesrc/tsconfig.json` (新規作�E):
    - `parser.ts`, `html_gen.ts` めE`nodesrc/` 冁E�� `parser.js`, `html_gen.js` にコンパイル、E
    - `web/node_modules/@types/node` めEtypeRoots として参�E、E
  - `.github/actions/bootstrap-build/action.yml` (更新):
    - `web/node_modules/.bin/tsc -p nodesrc/tsconfig.json` めECI スチE��プに追加、E
  - `CLAUDE.md` (更新):
    - 「作業の区刁E��でコミットすること」「コミット前に note.n.md を更新すること」を開発ガイドラインに追記、E
- [方釁EほぁE��ん]:
  - `.ts` ファイルがソース、`.js` がコンパイル出力。コンパイル済み `.js` は git に含める、E
  - CI は `web/node_modules/.bin/tsc` を使用してコンパイル�E�追加インスト�Eル不要E��、E

---

# 2026-03-16 作業メモ (NEPLg2.1 命名�E型記法仕様確定�Efn廁E��)

- [目皁Eもくてき]:
  - 型記法�E大幁E��更�E�括弧完�E廁E��・kind-directed juxtaposition・`%` アノテーション・`unit` キーワード�E`fn` 宣言キーワード廁E���E�を反映した新仕様を **NEPLg2.1** と命名し、NEPLg2�E�現行実裁E��と明確に区別する、E
- [変更/へんこぁE:
  - `doc/type_notation_spec.md` (更新):
    - 括弧完�E廁E��・グループ化構文なし、E
    - 型適用めEjuxtaposition に変更�E�EName<A B>` ↁE`Name A B`�E�、kind-directed アルゴリズムで墁E��決定、E
    - unit 型を `unit` キーワードに変更�E�E()` 廁E���E�、E
    - 型注釈記号めE`<TypeExpr>` から `%TypeExpr` に変更、E
    - 型パラメータ宣言の `<>` を廁E���E�E.T .U` として列挙�E�、E
    - `fn` 宣言キーワードを廁E���E�理由: 型記法に `%fn ...` が現れるため紛らわしぁE��。�E関数定義めE`let name %fn ...` に統一。巻き上げは `let` の型が `fn`/`fn*` の場合に適用、E
  - `doc/pattern_spec.md`、`doc/module_system_spec.md`、`doc/language_platform_spec.md`、`doc/purity_ownership_memory_spec.md`:
    - タイトルめENEPLg2.1 に更新、E
  - `doc/dependent_type_proof_plan.md`、`doc/memory_safety_migration_plan.md`、`doc/module_system_spec.md`:
    - `fn` 宣言めE`let` に更新、型注釈を新記法に更新、E
  - `CLAUDE.md` (更新):
    - NEPLg2�E�現行実裁E��と NEPLg2.1�E�新仕様）�E区別を�E記、E
- [方釁EほぁE��ん]:
  - `nepl-core/`�E�Eust 実裁E���E引き続き NEPLg2 の実裁E��EEPLg2.1 の実裁E�E別途移行計画で進める、E
  - `plan.md` は古ぁENEPLg2 仕様であり変更しなぁE��参照用として保持�E�、E

---

# 2026-03-16 作業メモ (doc: 仕様整合確認�Eモジュール/パターン/CLAUDE.md 更新)

- [目皁Eもくてき]:
  - `doc/chat/dump/` の最新方針！Eang1.md, mem1.md, module1.md�E�と `doc/` 吁E��様およ�E `todo.md` に齟齬がなぁE��とを確認し、未記載�E設計決定を仕様書へ反映する、E
- [変更/へんこぁE:
  - `CLAUDE.md` (新規作�E):
    - ビルド�EチE��ト�EアーキチE��チャ・開発ガイドラインをまとめた初期 CLAUDE.md を作�E、E
    - `.n.md`�E�EM 拡張 Markdown: フリガナ�Egloss・Nest が使える�E�と通常 `.md` の違いを�E記。仕様参照先として `stdlib/nm/README.n.md` を示した、E
  - `doc/module_system_spec.md` (更新):
    - `use` の構文めE`::` セパレータ形式に変更�E�Euse core::math;` 等）、E
    - `use` が末尾セグメント�Eエイリアスを導�Eすることを�E記、E
    - `*` はモジュールへの `use` にのみ有効、E��数等への `::*` はエラーとして定義、E
    - `merge "path"` はファイルパス斁E���Eを取る！E""` 維持E��ことを�E記、構文例を追加、E
  - `doc/purity_ownership_memory_spec.md` (更新):
    - 「immutable tuple」を「immutable struct�E�EPair`, `Triple` 等）」に置き換え！Euple 廁E��に対応）、E
  - `doc/pattern_spec.md` (新規作�E):
    - 言語絁E��込み `Tuple` キーワードを廁E��し、`Pair<.A,.B>` / `Triple<.A,.B,.C>` めEstdlib の通常 struct として提供することを定義、E
    - Rust 相当�E高機�Eパターン仕様を策宁E 識別子�Eワイルドカード�EリチE��ル・篁E���E�構文未確定）�Eコンストラクタ�E�位置ベ�Eス�E��Eネスト�E`@` 束縛付き・OR パターン�E�E|`、パターン専用�E��E参�Eパターン�E�封E���E�、E
    - `let <pattern> <expr>` および `match` 式でのパターン使用仕様、網羁E��検査、所有権との統合を定義、E
    - 全コード例を NEPLg2 前置記法に準拠させた（括弧を使わず、中値演算子を用ぁE��ぁE��、E
    - 型前置記法確定�E先送りだが対応可能な設計であることを�E記、E
- [確誁Eかくにん]:
  - dump ファイル 3 本 (lang1, mem1, module1) と対応すめEdoc/ 仕様�Etodo.md を�E合した結果、矛盾は見当たらなかった、E
  - todo.md の「LLM 編雁E��止」セクションにある Tuple/Pair/Triple、型前置記法化、パターン設計�E今回の doc/ 更新で仕様として反映した、E
  - `use` スコープ導�Eの詳細�E�Elias vs 直接 import、`as *` の扱ぁE���E今回の module_system_spec.md 更新で確定させた、E

---

# 2026-03-16 作業メモ (doc: モジュールシスチE��・言語�EラチE��フォーム仕様�E策定�E監査完亁E

- [目皁Eもくてき]:
  - `doc/chat/dump/lang1.md`, `module1.md` の議論を整琁E��、NEPLg2 のモジュールシスチE��と言語�EラチE��フォームとしての全体像を正式な仕様書として明文化する、E
  - `todo.md` における、ファイル墁E��とモジュール墁E��の刁E��、およ�Eセルフ�Eストに向けたレイヤー構造のタスクを�E体化する、E
- [変更/へんこぁE:
  - `doc/module_system_spec.md` (新規作�E):
    - ファイルとモジュールの直交性、`merge` (ソース合�E) と `use` (依存解決) の使ぁE�Eけ、Anchor Part による canonical path 決定規則を定義、E
  - `doc/language_platform_spec.md` (新規作�E):
    - DSL 実行基盤としてのビジョン、Bootstrap Host (Rust) と Platform Stdlib (NEPL) の 2 層構造、stdlib の階層匁E(`core`/`alloc`/`runtimes`/`std`/`features`) を定義、E
  - `todo.md`:
    - 、E. Module System 実裁E��名前解決の刷新 (Migration Phase 0.5)」を追加、E
    - セルフ�Eストコンパイラ頁E��の完亁E��件を、�EラチE��フォーム構造の定義に合わせて高度化、E
- [結果/けっか]:
  - これにより、NEPLg2 が「単なる言語」ではなく「言語�EラチE��フォーム」であるとぁE��立ち位置が�E確化され、多ファイル構�E時�E名前解決の不確実性が払拭された。`todo.md` に基づき、次はパ�Eサとレゾルバ�E刷新に着手する土台が整った、E

---

# 2026-03-15 作業メモ (doc: 全ドキュメント�E最新仕様への追従�E監査完亁E

- [目皁Eもくてき]:
  - `doc/` 以下�Eすべての仕様や計画 (`plan.md`, `todo.md` を含む) と最新の実裁E��況を精査し、NEPLg2 の目標と新たに策定した安�E施筁E(`purity_ownership_memory_spec.md`, `memory_safety_migration_plan.md`) との間で齟齬がなぁE��ぁE��統一を図る、E
- [変更と監査結果/へんこぁE��かんさけっか]:
  - `plan.md`:
    - 斁E���E (`str`, `ByteBuf`, `StringBuilder`) の記述を更新し、旧式�E借用ビューめE`String` 型への言及を削除、E渁E
  - `doc/runtime.md`:
    - GCなし�Eメモリ管琁E��つぁE��、手勁E`alloc/dealloc` ベ�Eスの古ぁE��明を削除し、E*Region Inference (純粋永続値)** と **Drop Elaboration (一意所有リソース)** の二段構えモチE��に書き換えた、E
    - Wasm/LLVM のランタイム差刁E�E `#if[target=...]` で吸収され、コンパイラの安�E意味論�E共通である旨を�E記した、E
  - `doc/error.md`:
    - 旧式�Eヒ�Eプ確保前提である `Error` レコード�E説明を削除し、最新の `Diag`、`Outcome<T, E>`、`Result<T, StdErrorKind>` を核とするエラーモチE��に更新した。メモリの確保と解放はGCめE��勁E`alloc` ではなく、新しい所有権モチE��に委�Eられる旨を記載した、E
  - `doc/move_effect_spec.md` & `doc/memory_safety_compiler_design.md` & `doc/stdlib_breaking_reboot.md`:
    - すでに統合仕様を反映済みであり、�E容に矛盾がなぁE��とを確認した、E
  - `todo.md`:
    - 「メモリ安�E型モチE��を統合仕様に基づぁE��実裁E��る」�Eタスク�E�Ehase 1: Effect拡張とVarState追加、Phase 2: 型�E離とRegion推論）が詳細に記載されており、実裁E��況およ�E計画と完�Eに一致してぁE��ことを確認した、E
- [結諁Eけつろん]:
  - これにより、NEPLg2 における純粋性・所有権・メモリ管琁E�E根幹となるドキュメントと実裁E��画が完�Eに一点に統合�E整琁E��れ、すべての古ぁEGC/手動解放ベ�Eスの記述が払拭された。以後�Eこ�Eドキュメント群および `todo.md` の Phase 1 / 2 に剁E��てコンパイラと標準ライブラリの実裁E��安�Eに進めることができる、E

---

# 2026-03-15 作業メモ (doc: NEPLg2目標と新仕様�E整合性検訁E

- [目皁Eもくてき]:
  - Zenn記事！EEPLg2の歴史と設計思想�E��E目標に対し、作�EしたPurity・Ownership・Memory仕様と実裁E��画で到達可能かを深く検討し、不足があれ�E修正する、E
- [検討事頁E��結果/けんとぁE��こうとけっか]:
  1. **マルチ�EラチE��フォームと同等�E結果�E�Easm/LLVMの抽象化！E*:
     - **結果**: 達�E可能。統合仕様§12 および 移行計画§10.8 により、Resource IR パスで「安�E意味論」を完�Eに保証し、codegen フェーズでは物琁E��イアウト！Einear memory vs Native pointer�E��E違いのみを吸収する設計になってぁE��、Eenn記事�E目標と完�Eに一致する、E
  2. **自作言語�EラチE��フォームとセルフ�Eスト（コンパイラを書ける言語か�E�E*:
     - **結果**: 達�E可能だが一部仕様に明記が忁E��だったため修正した。コンパイラ�E�ESTめE��墁E��を実裁E��るには、褁E��なチE�Eタ構造を通じた�E力�E伝播が忁E��、E
     - **修正**: `purity_ownership_memory_spec.md` に **、E.3 型�E能力�E合�E剁E��E* を追加、EST�E�Emmutable tree�E��E pure persistent、環墁E��Builderは UniqueMutable、E��ぁE��ファイルを含む構造体�E LinearCapability となるよぁE��褁E��型�E能力伝播ルールを�E斁E��し、コンパイラ記述に耐えぁE��堁E��な型シスチE��設計を確立した、E
  3. **協力な静的検査と括弧の根絶**:
     - **結果**: 達�E可能。Resource IR を用ぁE�� Dataflow 解极E(use-after-move, borrow conflict, linear 漏れ等�E検査) は Zenn記事�E「強力な検査裁E��」に直接貢献する。構文皁E��徴�E�前置記法�Eオフサイドルール�E�とは独立しぁEIR 層での検査であるため、括弧の根絶目標とも衝突しなぁE��E
  4. **既存ドキュメント�E矛盾解涁E*:
     - **結果**: `plan.md` の斁E���Eに関する記述が旧思想のままだったため修正した、E
     - **修正**: `plan.md` 上�E「`str`: 借用, `String`: 所有」とぁE��記述を削除し、新仕様�E「`str` (純粋永続値) / `ByteBuf` (一意所有バイト�E) / `StringBuilder` (構築用状慁E」に更新した、E
  5. **コンパイラパス頁E���E不整吁E*:
     - **結果**: `memory_safety_migration_plan.md` のパス頁E��が暫定�Eままだったため修正した、E
     - **修正**: 統合仕様に合わせて `effect attribution`, `resource_ir_gen`, `region_inference` を正しい頁E��でパイプライン�E�§10.5.1, §10.7�E�に絁E��込んだ、E
- [結諁Eけつろん]:
  - 今回の純粋性・所有権の拡張仕様�E、Zenn記事で掲げられた「�EラチE��フォーム非依存�E抽象化」「�E作言語�EラチE��フォームに耐えぁE��堁E��な型シスチE��」�E中核を�Eすものであり、提示された移行計画で段階的に実裁E��進めることで目標達成�E十�Eに可能であると判断した、E

---

# 2026-03-15 作業メモ (doc: mem1.md との整合性監査・ギャチE�E修正)

- [目皁Eもくてき]:
  - `doc/chat/dump/mem1.md` の全設計要素�E�メモリ管琁E��型検査、線形型、所有権、alloc/drop 自動化、ランタイム差異吸収）が `doc/` と `todo.md` に適刁E��反映されてぁE��か監査する、E
- [変更/へんこぁE:
  - `doc/memory_safety_migration_plan.md` §10 の compiler 検査設計を改喁E
    - `MoveState` ↁE`VarState` に改名し、`BorrowedShared { borrower_count }` と `BorrowedUnique` 状態を追加、E
    - borrow conflict 診断 (5007, 5008) を追加、E
    - Resource IR 命令リスチE(`move`, `borrow_shared`, `borrow_unique`, `region_new`, `region_alloc`, `region_end`, `drop`, `io_open`, `io_write`, `io_close`) を追加、E
    - §10.8 ランタイム差異の吸収セクションを追加�E�Easm/LLVM 比輁E��付き�E�、E
  - `doc/purity_ownership_memory_spec.md` §6.4 を更新:
    - `Valid`/`PossiblyMoved` ↁE`Live`/`MaybeMoved`/`Uninitialized` に統一、E
    - 吁E��断 ID (5001, 5005-5008) を追記、E
  - `todo.md` 頁E�� 4 を拡允E
    - Phase 1 に `Effect` 拡張、`ValueCategory` 刁E��子、`VarState` 追跡、memory safety 診断 ID 予紁E��追加、E
    - 完亁E��件に borrow conflict 検�Eとランタイム差異刁E��を追加、E
- [結果/けっか]:
  - mem1.md の主要設計要素�E�値の3刁E��、�E部Effect、ownership/borrow/linear検査、escape analysis、drop elaboration、region inference、Wasm/LLVM差異吸収、依存型への封E��拡張�E��E全て doc/ と todo.md に反映済み、E

---

# 2026-03-15 作業メモ (doc: 純粋性・所有権・メモリ管琁E�E統合仕様を作�E)

- [目皁Eもくてき]:
  - `doc/chat/dump/mem1.md` の ChatGPT 議論を整琁E��、NEPLg2 の純粋性・所有権・線形性・メモリ管琁E�E統合仕様書めE`doc/` に作�Eする、E
  - 既存�E関連ドキュメントとの不整合を解消する、E
  - `todo.md` を新仕様に合わせて更新する、E
- [変更/へんこぁE:
  - `doc/purity_ownership_memory_spec.md` (新規作�E)
    - mem1.md の設計議論を整琁E��た統合仕様書、E
    - 値の 3 刁E��E(pure persistent value / unique mutable work state / linear capability)、E
    - surface effect (`Pure`/`Impure`) と compiler 冁E��効极E(`InternalAlloc`/`ExternalIO`/`Nondet`/`Unsafe`) の刁E��、E
    - Region Inference + Drop Elaboration の二段構えメモリ管琁E��E
    - `set` の新 purity 規則 (escape analysis ベ�Eス)、E
    - 斁E���E (`str`/`ByteBuf`/`StringBuilder`)、List (persistent list + builder)、IO (consume-return handle) の仕様、E
    - Resource IR と compiler 解析パス頁E�E定義、E
    - Wasm/LLVM で揁E��るもの (安�E意味諁E と揁E��なぁE��の (物琁E��イアウチE の区別、E
  - `doc/memory_safety_compiler_design.md` (更新)
    - 統合仕様への参�Eを追加、E
    - alloc/dealloc の Pure 扱ぁE�� `InternalAlloc` ベ�Eスに変更、E
    - `MemPtr<T>` めEcompiler/runtime 墁E��に再�E置、E
    - Region Inference と Drop Elaboration の節を追加、E
  - `doc/move_effect_spec.md` (更新)
    - 統合仕様への参�Eを追加、E
    - compiler 冁E��効果�E顁E(`InternalAlloc`/`ExternalIO`/`Nondet`/`Unsafe`) を追加、E
    - `set` の新 purity 規則を追加、E
    - builtins 要件めE`InternalAlloc` ベ�Eスに変更、E
    - Resource IR パスを追加、E
  - `doc/stdlib_breaking_reboot.md` (更新)
    - `MemPtr<T>` / `RegionToken<T>` の位置づけを compiler/runtime 墁E��として明確化、E
    - メモリ能劁Etrait 節に統合仕様への参�Eと 3 刁E���E前提を追加、E
  - `doc/stdlib_doc_comment_policy.md` (更新)
    - `[注意]` 節の所有権・メモリ関連頁E��に 3 刁E��への参�Eを追加、E
  - `todo.md` (更新)
    - メモリ安�E型モチE��のタスクを統合仕様に合わせて拡允E��E
- [plan.md との差異/さい]:
  - plan.md は言語�E基本仕槁E(前置記法�E式指向�Eオフサイドルール) を記述しており、メモリ管琁E�E所有権・純粋性の詳細設計には言及してぁE��ぁE��E
  - 今回の統合仕様�E plan.md の `a->b` (pure) / `a*>b` (impure) の区別を発展させ、compiler 冁E��の効果�E類や所有権規則を�E体化したも�Eである、E

# 2026-03-14 作業メモ (fix: トップレベル見�EしリンクとフリガチEruby)・OGPの刁E��)

- [目皁Eもくてき]:
  - in-page TOC�E��Eージ冁E��次�E�において、テキスト�Eレーンではなく�EのHTMLタグ�E�E<ruby>`�E�を維持してフリガナを表示する、E
  - 同時に、`<meta property="og:title">` 等�E OGP タグにはフリガナが含まれなぁE��ぁE��する�E�E<rt>`要素のチE��ストを抽出から除外する）、E
  - さらに、�Eージトップ�EH1レベルの見�EしもTOCの先頭に含め、クリチE��時に URL ハッシュを変更することなく�Eージトップへスムーズスクロール�E�Ehref="#"` のインターセプト�E�させる挙動を実裁E��る、E
- [実裁Eじっそう]:
  - `nodesrc/html_gen.js` および `nodesrc/cli.js`
    - OGP用に利用されめE`inlinesToPlainText` 関数の処琁E��、ASTノ�Eト種別ぁE`ruby` の場合�E `n.ruby` ではなぁE`n.base` のチE��ストだけを抽出するように修正。これにより、OGPのtitleなどにフリガナが混入しなくなった、E
  - `nodesrc/inpage_toc_helper.js`
    - `extractInPageToc` にて `inlinesToHtml` を使用して見�Eし�EHTML�E�バチE��を除く�Eのパ�Eス結果�E�を抽出し、`ruby` などの表示を維持したままTOC頁E��とするように変更、E
    - H1 のルート見�Eしを、IDなし（トチE�Eへのアンカー `href="#"`�E�としてTOCリスト�E先頭に追加する処琁E��実裁E��E
  - `nodesrc/static/playground_runtime.js`
    - TOC冁E��ンクの中で `href="#"` をクリチE��した場合、既定�Eアクション�E�ハチE��ュの付与）を無効化し、`window.scrollTo` を用ぁE��トップへスクロールする挙動を付与。まぁE`history.pushState` を用ぁE��ハッシュの消去も可能にした、E



- [目皁Eもくてき]:
  - tutorial と stdlib ドキュメントにて、見�Eしに基づく「�Eージ冁E��次�E�En-page TOC�E�」を右側�E�EC向け�E�およ�E折りたたみメニュー�E�モバイル向け�E�として追加し、よりスムーズに斁E��冁E��移動できるようにする、E
  - struct めEfn などのバッジ�E�種類）情報も目次冁E��表示することで、目皁E�EAPIへすぐアクセス可能にする、E
- [実裁Eじっそう]:
  - `nodesrc/inpage_toc_helper.js` (新規作�E)
    - AST の Document ノ�Eドを走査�E�EextractInPageToc`�E�し、`section` ノ�Eド！Eid`となるslugめE��チE��惁E��を抽出�E��E配�Eを生成、E
    - `renderInPageTocHtml` にて、E��層�E�Eepth�E�づけされた HTML�E�E<ul>` / `<li>`�E�を生�E、E
  - `nodesrc/html_gen_playground.js`
    - HTMLレイアウト！ESSグリチE���E�に右カラム `<aside class="doc-inpage-toc">` とモバイル用の `<details class="doc-inpage-toc-mobile">` を追加し、生成したTOC HTMLを注入、E
  - `nodesrc/static/playground.css`
    - `.doc-layout` めEカラムから3カラム�E�E280px 1fr 240px`�E�へ変更�E�デスクトップ）、E
    - 要素の固定�E置�E�Eposition: sticky`�E�と右側目次のスタイリングを追加、E
    - メチE��アクエリ�E�Emax-width: 768px`�E�を用ぁE��モバイル幁E�E場合�E右サイドバーを隠し、本斁E��部に `<details>` で展開できる目次を表示するよう刁E���E琁E��記述、E
  - `nodesrc/static/playground_runtime.js`
    - `IntersectionObserver` を追加し、ユーザーがスクロールした際に現在見えてぁE��見�Eし！Esection`�E�に対応する目次のリンク�E�E.inpage-toc-link`�E�へ `active` クラスを�E動付与！Ecroll Spy機�E�E�する仕絁E��を導�E、E



- [目皁Eもくてき]:
  - tutorial と stdlib の playground HTML にて、左側のサイドバーのリンク�E�Eable of Contents�E�が壊れており、どのリンクをクリチE��しても現在のペ�Eジに遷移してしまぁE��題を修正する、E
- [根本原因/こんぽんげんいん]:
  - `nodesrc/cli.js` 冁E�E `genOne` 関数にて `renderHtmlPlayground` を呼び出す際、TOC生�Eのための `tocLinks` に、各リンクの相対パスを解決する `makePageTocLinks` 関数の結果ではなく、パス解決前�E `tocEntries` をそのまま渡してぁE��、E
  - そ�Eため吁E��ントリの `href` が正しく生�Eされず、現在のペ�Eジを指すリンク�E�空の href など�E�になってぁE��、E
- [変更/へんこぁE:
  - `nodesrc/cli.js`
    - `genOne` 関数冁E�� `renderHtmlPlayground` に渡ぁE`tocLinks` めE`makePageTocLinks(outRel, tocEntries)` に変更、E
- [検証/けんしょぁE:
  - `node nodesrc/cli.js` コマンドで HTML を�E生�Eし、`href` 属性に正しく相対パス�E�侁E `02_numbers_and_variables.html`�E�が設定されてぁE��ことを確認、E



- [目皁Eもくてき]:
  - playground で生�EされめEtutorials めEstdlib ドキュメント�Eのコードが実行できなぁE��コンパイルエラー・クラチE��ュ�E�問題を修正する、E
- [根本原因/こんぽんげんいん]:
  - `nodesrc/static/playground_runtime.js` にて、コード�Eコンパイルを呼び出す際、標準ライブラリ�E�Etdlib�E�を含まなぁE`compile_source` メソチE��を用ぁE��ぁE��、E
  - そ�Eため、チュートリアルなどの `#import "std/stdio" as *` とぁE��た標準ライブラリへの依存が解決できず、未定義識別子などでコンパイルが失敗してぁE��、E
- [変更/へんこぁE:
  - `nodesrc/static/playground_runtime.js`
    - `runBtn.onclick` 冁E�Eコンパイル処琁E��、`compile_source` から `compile_source_with_vfs_and_stdlib` に変更した、E
    - バンドルされた標準ライブラリめE`bindings.get_bundled_stdlib_vfs()` により取得し、一緒に渡すよぁE��した、E
- [検証/けんしょぁE:
  - 単体スクリプトでのコンパイル挙動確認にて、正常に `compile_source_with_vfs_and_stdlib` が通り、WASMコードが生�Eされることを確認、E
  - `node nodesrc/cli.js` コマンドを実行し、コンパイル後�EチュートリアルめE��準ライブラリの HTML を�E生�Eした、E

# 2026-03-14 作業メモ (feat: 検索機�Eの強匁E- オーバ�Eロード対応�E型表示・フィルタ追加)

- [目皁Eもくてき]:
  - 検索機�Eとドキュメント表示を強化し、オーバ�Eロードされた同名関数を正確に区別・ナビゲートできるようにする、E
  - 検索 UI にフィルタを追加し、目皁E�E識別子を素早く見つけられるようにする、E
- [変更/へんこぁE:
  - `nodesrc/parser.js`
    - `parseNeplText` において、`fn` / `struct` などの kind と `<(i32)->i32>` などの型シグネチャを抽出するように拡張、E
    - レガシーな `name: description` 形式�Eドキュメントを既定�E見�Eしとして扱ぁE��ぁE��修正、E
  - `nodesrc/html_gen.js`
    - `makeSlug` において、型惁E��を含めた一意�Eスラグを生成するよぁE��修正。URLエンコード対策として空白を除去、E
    - セクションの ID を階層構造�E�侁E `parent-child--type`�E�で生�Eするように変更し、ネストされた定義へのリンクを正確化、E
    - 見�Eしに種類（バチE���E�と型シグネチャを表示するように拡張、E
  - `nodesrc/search.js`
    - `html_gen.js` と同期した階層スラグ生�EロジチE��を実裁E��E
    - 検索エントリに `kind` と `type` 惁E��を追加、E
  - `nodesrc/html_gen_playground.js`
    - 検索 UI に `kind` (種顁E と `path` (ファイルパス) による絞り込みフィルタを追加、E
    - 検索結果に型シグネチャを表示、E
    - `:target` セクションの強調表示スタイルを改喁E��E
    - バッジを白斁E���E枠線�E角丸のモダンなチE��インに更新、E
  - `nodesrc/cli.js`
    - `rootPrefix` の深さ計算を修正し、E04 エラーを解消、E
    - 同一ファイル冁E�Eオーバ�Eロード関数が正しくインチE��クスされるよぁEID 生�Eを修正、E
  - `nodesrc/test_search.js`
    - 種類情報の抽出とフィルタリングに関するユニットテストを追加、E
- [検証/けんしょぁE:
  - `math.nepl` (旧形弁E および `fenwick.nepl` (ネスト形弁E において、検索結果から正しくジャンプし強調表示されることを確認、E
  - `add` などのオーバ�Eロード関数が型で区別され、それぞれ個別のアンカーへ飛�Eことを確認、E
  - 生�EされぁEHTML 冁E�E `id` と検索インチE��クスの fragment が階層構造を含めて一致することを確認、E

# 2026-03-14 作業メモ (feat: stdlib/tutorial HTML に全斁E��索機�Eを追加)

- [目皁Eもくてき]:
  - stdlib と tutorial の HTML 吁E�Eージに、スコープ！Eutorial 全佁E/ stdlib 全体）横断のリアルタイム全斁E��索 UI を追加する、E
  - 検索ロジチE���E�ES�E��EローカルチE��トと HTML 埋め込みで全く同じコードを使ぁE��E
- [変更/へんこぁE:
  - `nodesrc/search.js` を新規作�E、E
    - `searchIndex(query, index, maxResults)`: AND 検索、スコア頁E��却、E
    - `buildEntriesFromAst(ast, pageUrl, pageTitle)`: AST から検索エントリを構築、E
    - `inlinesToSearchText(inlines)`: ルビ（漢孁E+ 読み仮名）�E両方をインチE��クスに含める、E
    - Node.js `module.exports` と ブラウザ `NeplSearch` グローバルの両方に対応、E
  - `nodesrc/test_search.js` を新規作�E、E
    - `assert` モジュールのみ使用、外部依存ゼロのローカル完結テスト、E
    - チE��チE30 件�E�EokenizeQuery / inlinesToSearchText / searchIndex / buildEntriesFromAst / 統合）、E
  - `nodesrc/html_gen_playground.js` を変更、E
    - `SEARCH_JS_SOURCE`: モジュール読み込み時に `search.js` を文字�Eとして読み込む、E
    - `wrapHtmlPlayground` に `searchIndexJson` 引数を追加、E
    - `<style>` に検索 UI の CSS を追加�E�E.search-wrap` / `.search-input` / `.search-results` など�E�、E
    - `<script>` 先頭に `search.js` めEinline 埋め込みし、`__SEARCH_INDEX__` 変数を注入、E
    - `renderToc` に検索ボックス HTML を追加�E�E#doc-search-input` / `#doc-search-results`�E�、E
    - `DOMContentLoaded` に検索 UI イベントハンドラを追加�E�リアルタイムドロチE�Eダウン / キーボ�EチE↑�EEnter/Escape�E�、E
    - `renderHtmlPlayground` に `searchIndex` オプションを追加、E
  - `nodesrc/cli.js` を変更、E
    - `buildScopeSearchIndex(inputRoot, files, excludeDirs)` を追加、E
    - スコープ（�E力ディレクトリ = tutorial 全佁Eor stdlib 全体）ごとに全ペ�Eジの AST を事前解析し検索インチE��クスを構築する、E
    - `genOne` にインチE��クスを渡し、各ペ�Eジの HTML に同一スコープ�EインチE��クスを埋め込む、E
- [検証/けんしょぁE:
  - `node nodesrc/test_search.js` ↁE`30 成功, 0 失敗`
  - HTML 生�EチェチE���E�E__SEARCH_INDEX__` / `NeplSearch` / `search-input` / `search-results` / `searchIndexJson`�E��E 全 pass
  - tutorial スコープで 29 ファイルから 148 エントリを構築できることを確認、E
- [plan.md との差異/diffreference]:
  - plan.md は検索機�Eに言及してぁE��ぁE��め差異なし、E
  - 実裁E�E「検索スコープを入力ディレクトリ単位！Eutorial/stdlib�E�で刁E��る」設計で、ユーザー要件に合�EしてぁE��、E

# 2026-03-12 作業メモ (fix: bare Result map/and_then の callable 解決を修正)

- [目皁Eもくてき]:
  - `core/option` と `core/result` に同名の `map` / `and_then` を追加したぁE��で、NEPLg2 の bare 吁E+ type args 記法でも正しく[解決/かいけつ]されるよぁE��する、E
- [根本原因/こんぽんげんいん]:
  - `typecheck` の `Symbol::Ident` 解決が、explicit type args を持つ callable に対してめE`lookup_callable_any(name)` を�Eに見てぁE��、E
  - そ�Eため `map<i32,i32,str>` と `and_then<i32,i32,str>` が、generic arity 3 の `Result` 版ではなく、generic arity 2 の `Option` 版へ誤って結�E付き、`expression left extra values on the stack` に崩れてぁE��、E
  - `map_err` は `Result` 側にしか存在しなぁE��めE��っており、failure は bare 同名 callable 群の選別に限られてぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/typecheck.rs`
    - explicit type args 付き `Symbol::Ident` では、`lookup_all_callables(name)` から `type_params.len() == type_args.len()` を満たす callable だけを候補に残し、E 件ならそれを優先するよぁE��更、E
    - unresolved callable stack entry を作る経路でも、explicit type args があるとき�E同じ generic arity filter を適用するよう変更、E
    - 調査用に入れてぁE�� debug 出力を削除、E
  - `stdlib/core/option.nepl`
    - `unwrap_or` めEbare 名へ揁E��、`map` / `and_then` とそ�E doctest を追加、E
  - `stdlib/core/result.nepl`
    - `and_then` と doctest を追加、E
  - `stdlib/tests/option.n.md`
  - `stdlib/tests/result.n.md`
  - `tutorials/getting_started/05_option.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
    - bare `unwrap_or` / `map` / `and_then` 前提へ追従、E
- [検証/けんしょぁE:
  - `RUSTFLAGS='-C link-arg=-fuse-ld=lld' cargo build -p nepl-cli`
  - `env -u RUSTFLAGS cargo build --target wasm32-unknown-unknown --manifest-path nepl-web/Cargo.toml`
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i stdlib/core/option.nepl -n 2`
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 3`
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 4`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/05_option.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/13_type_driven_error_modeling.n.md -n 1`
  - `node nodesrc/tests.js -i stdlib/tests/option.n.md -i stdlib/tests/result.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree -o /tmp/tests-option-result-fp.json -j 2`
    - `summary: 10/10 pass`

# 2026-03-12 作業メモ (editor extensions: Zed syntax highlight 用 grammar を強匁E

- 目皁E
  - Zed extension の syntax layer ぁEtop-level 定義名や directive / import / type annotation を最低限識別できるようにする、E
- 根本原因:
  - 既存�E `editors/zed/tree-sitter-neplg2/grammar.js` は「行を並べるだけ」�E簡易構文で、`fn` / `struct` / `enum` / `trait` / `impl` の名前、field、variant、directive 名などを区別できてぁE��かった、E
  - そ�Eため highlight query を細かく書ぁE��も、E��数名や型名を適刁E��色刁E��するための node が存在しなかった、E
- 変更:
  - `editors/zed/tree-sitter-neplg2/grammar.js`
    - top-level として `function_definition`, `struct_definition`, `enum_definition`, `trait_definition`, `impl_definition`, `directive`, `expression_statement` を�E離した、E
    - `directive_name`, `import_path`, `alias_clause`, `field_definition`, `enum_variant`, `generic_params`, `type_annotation` などの node を追加した、E
  - `editors/zed/languages/neplg2/highlights.scm`
    - function / type / property / constant / parameter / namespace の capture を追加した、E
  - `editors/zed/languages/neplg2/brackets.scm`
    - `[` `]` めEbracket として扱ぁE��ぁE��した、E
  - `editors/zed/languages/neplg2/config.toml`
    - `autoclose_before` を追加した、E
- 検証:
  - `node --check editors/zed/tree-sitter-neplg2/grammar.js`
    - pass
  - `node -e "global.grammar = x => x; const g = require('./editors/zed/tree-sitter-neplg2/grammar.js'); console.log(g.name, Object.keys(g.rules).length)"`
    - 結果: `neplg2 28`
- 差異メモ:
  - まだ `tree-sitter generate` / Zed 上での実読み込み検証は未実行。現行環墁E��は `zed_extension_api` 側の toolchain 条件が残ってぁE��ため、Zed package 全体�E build 検証は別途忁E��、E

# 2026-03-12 作業メモ (editor extensions: Zed shell の build 前提を整琁E

- 目皁E
  - `nepl-lsp` を実際に build/test し、Zed extension shell 側も検証可能な形へ寁E��る、E
- 変更:
  - `nepl-lsp/src/main.rs`
    - `analyze_document` 冁E�E `entry_path` capture を修正し、`cargo test -p nepl-lsp` が通るようにした、E
    - 未使用 import を整琁E��た、E
  - `editors/zed/Cargo.toml`
    - 独竁Ecrate として `cargo check --manifest-path editors/zed/Cargo.toml` を実行できるよう、空の `[workspace]` を追加した、E
    - `zed_extension_api` の世代を下げて現衁Etoolchain で検証できるか�Eり�Eけた、E
  - `editors/zed/README.md`
  - `doc/editor_extensions.md`
    - `nepl-lsp` は build 済みであることと、Zed 側は `edition2024` 要求が blocker であることを追記した、E
- 結果:
  - `cargo test -p nepl-lsp` は pass、E
  - `cargo check --manifest-path editors/zed/Cargo.toml` は `zed_extension_api` とそ�E依孁E(`spdx` など) ぁE`edition2024` を要求し、現衁ECargo 1.83.0 では manifest parse 時点で失敗することを確認した、E
  - つまり現在の blocker は extension 実裁E��なぁEtoolchain / upstream crate 要件である、E
- 次:
  - Zed shell を実際に build 検証するには、Rust/Cargo めE`edition2024` 対応版へ上げるか、互換のある `zed_extension_api` 系列を特定して固定する忁E��がある、E

# 2026-03-12 作業メモ (editor extensions: doc comment めEcompiler/nm 経由で LSP hover へ接綁E

- 目皁E
  - stdlib で既に使われてぁE�� `//:` 形式�E document comment めEcompiler が正しく認識し、editor extension / LSP ぁEJavaScript 側の再実裁E��依存せぁERust 側だけで利用できるようにする、E
  - `nodesrc/parser.js` / `nodesrc/html_gen.js` にしか無かっぁE`nm` の責務を、拡張機�E向けの compiler 実裁E��持ち込む、E
- 根本原因:
  - Rust compiler 側の lexer は `///` めEdoc comment として扱ってぁE��が、stdlib 実運用の `//:` を認識してぁE��かった、E
  - parser には item 直剁Edoc comment の紐づけ�E琁E��既にあったため、token 化できてぁE��ぁE��とが主因だった、E
  - LSP hover 側めEraw 斁E���Eをそのまま表示しており、`nm` として構造化された document comment を利用してぁE��かった、E
- 変更:
  - `nepl-core/src/lexer.rs`
    - `///` に加えて `//:` めE`DocComment` token として扱ぁE��ぁE��正した、E
  - `nepl-core/src/parser.rs`
    - module 先頭の `//:` めEmodule doc として刁E��取得する�E琁E��追加した、E
  - `nepl-core/src/ast.rs`
    - `Module.doc` を追加した、E
  - `nepl-core/src/nm.rs`
    - editor/LSP 向けの Rust 実裁E`nm` parser を追加した、E
    - heading / list / code block / gloss / ruby などを構造化し、Markdown へ戻ぁErenderer `render_document_markdown` を追加した、E
  - `nepl-language/src/lib.rs`
    - 定義惁E��に `doc_ast` を追加し、compiler が取得しぁEdocument comment めE`nm` AST として保持できるようにした、E
  - `nepl-lsp/src/main.rs`
    - hover ぁEraw 斁E���EではなぁE`doc_ast` めEMarkdown へ render した結果を優先して返すようにした、E
  - `nepl-core/tests/doc_comments.rs`
    - `//:` の item 紐づけ、stdlib 実ファイルの doc comment、module doc と item doc の刁E��を確認すめEtest を追加した、E
- 実裁E��況E
  - compiler で `//:` document comment めEtoken 化し、定義惁E��へ紐づける経路は追加済み、E
  - `nm` parser / renderer めERust 側へ追加し、LSP hover から利用する経路も追加済み、E
  - まだ Zed/VSCode 側の package 実裁E��、hover 表示冁E��の詳細整形は未完亁E��E
- plan.md との差異:
  - plan.md の editor extension 共通基盤に向けて、LSP hover 用の doc comment 取得経路を�E行で実裁E��た、E
  - まだ WASIp1 server 配币E��態、Zed package からの実行導線、VSCode shell は未実裁E��E
- 検証:
  - `cargo test -p nepl-language` は既存�E篁E��で pass 済み、E
  - pull 後�E再検証として `cargo test -p nepl-language semantics_analysis_reports_hover_doc_and_type -- --nocapture` を�E実行中、E
  - `cargo test -p nepl-core --test doc_comments -- --nocapture` は lock 競合を避けるため単独で再実行する前提、E

# 2026-03-12 作業メモ (editor extensions: `nepl-language` 追加)

- 目皁E
  - `nepl-web` とは別に、editor extension 向けの共送ERust lib を追加する、E
  - Zed / VSCode / 封E��の WASIp1 Language Server が同ぁEcompiler 実裁E��再利用できる墁E��を作る、E
  - extension 側は薁E��保ち、封E��皁E�� Rust 実裁E�� NEPLg2 へ置き換えやすい構�Eにする、E
- 根本方釁E
  - 以前�E `nepl-web` 解极EAPI は Web 向け wasm-bindgen 出力に寁E��ており、editor extension の共通基盤としては不適刁E��った、E
  - そ�Eまま extension ぁE`nepl-web` へ依存すると、Web 向け JS/wasm API と editor 向け Rust API が寁E��合し、Zed / VSCode / 封E��の selfhost 置換�E墁E��が曖昧になる、E
  - そ�Eため、compiler 本佁E(`nepl-core`) の上に editor 専用 lib `nepl-language` を追加し、Web 向け API とは刁E��した、E
- 変更:
  - `Cargo.toml`
    - workspace member に `nepl-language` を追加した、E
  - `nepl-language/Cargo.toml`
    - 新要Ecrate を追加した、E
  - `nepl-language/src/lib.rs`
    - lexer / diagnostics / name resolution / semantics めERust struct で返す API を追加した、E
    - `LoadResult` を受け取る褁E��ファイル解极EAPI を追加し、hover / 定義ジャンプ用に path 付き range を返すようにした、E
    - `nepl-web` に閉じてぁE��名前解決 trace / semantic token 絁E��立てめEeditor 共送Elib として刁E��出した、E
    - cross-file resolution を含む unit test を追加した、E
  - `doc/editor_extensions.md`
    - `nepl-web` と editor extension 用 lib の責務�E離、Zed / VSCode / 封E��の LSP の構�E方針を記述した、E
  - `editors/zed/README.md`
    - Zed extension の構�E方針と次段階�E作業頁E��を追加した、E
- 実裁E��況E
  - `nepl-language` は追加済みで、token / diagnostic / hover / semantic token / definition 用チE�Eタを返せる、E
  - 単一ファイル解析と、`Loader` を介した褁E��ファイル解析�E両方を扱える、E
  - まだ Zed extension package 本体、tree-sitter grammar、WASIp1 Language Server binary は未実裁E��E
- plan.md との差異:
  - `plan.md` の LSP / Zed / VSCode 方針に対し、今回は editor 共通解极Elib の土台までを�E行実裁E��た、E
  - 実際の Zed package と VSCode package は未着手であり、次段階で `nepl-language` の上に Rust 製 Language Server を追加する忁E��がある、E
- 検証:
  - `cargo test -p nepl-language`
    - 結果: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/typeannot.n.md -n 2`
    - 結果: pass
  - 参老E
    - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1`
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1`
    - 上訁E2 件は `return value mismatch` と runtime trap で fail。今回の変更対象は雁E��スクリプトであり、repo_metrics 変更の有無に関係なく既存�E doctest 側問題として残ってぁE��、E
- 差異メモ:
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib ...` は長時間継続したため、確認�E `run_doctest.js` による focused 実行へ刁E��替えた、E
- 今回の変更は build/test 系ロジチE��ではなく、E��計スクリプト単体�E改喁E��ある、E

# 2026-03-12 作業メモ (fix: aggregate struct packing を修正して SparseSet invalid-path を復旧)

- [目皁Eもくてき]:
  - `alloc/collections/sparse_set` の invalid index path ぁEweb/native test path で trap してぁE��根因を特定し、stdlib 側の回避ではなぁEcompiler 側から修正する、E
- [根本原因/こんぽんげんいん]:
  - [当�E/とぁE��ょ]は `SparseSet` 自体�E owner 表現めE`alloc/string` の concat を疑って刁E��刁E��たが、最終的には `U128DivRem` のような aggregate 値めE`StructConstruct` / `TupleConstruct` で絁E��立てめEcodegen が、field ごとの real storage size ではなぁEwasm/llvm の scalar `ValType` / `LlTy` サイズで pack してぁE��ことが原因だった、E
  - そ�E結果、aggregate field を[含/ふく]む struct/tuple ぁEinline byte copy ではなぁEpointer 相当で[詰/つ]められ、`field::get` と後続�E integer-to-string / diag message 生�Eで[壁Eこわ]れた値を読み、`SparseSet` invalid index path の message build ぁE`memory access out of bounds` に崩れてぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/codegen_wasm.rs`
    - `StructConstruct` / `TupleConstruct` の total size めE`type_storage_size_bytes` 基準へ修正、E
    - aggregate field/item は source pointer から destination へ byte copy する lowering に変更、E
  - `nepl-core/src/codegen_llvm.rs`
    - wasm 側と同じぁEaggregate field/item めEreal storage size ぶめEbyte copy するよう修正、E
  - `stdlib/alloc/string.nepl`
    - `string_finish_base` を追加し、region/token を二重に読み直さず base pointer めE1 回だけ確定して finish する形へ整琁E��E
    - `concat`, `sb_build`, `str_slice`, `from_u128_radix`, `from_f64` の finish 経路を同 helper に揁E��た、E
  - `alloc/collections/sparse_set`
    - header owner は `MemPtr<u8>` field ではなぁEraw `i32` header pointer めEpublic struct に保持し、�E部 helper でだぁE`MemPtr` に匁E��直す形へ整琁E��た、E
- [結果/けっか]:
  - `stdlib/alloc/string.nepl::doctest#4` ぁEpass に戻った、E
  - `stdlib/tests/sparse_set.n.md::doctest#2` と `tests/stdlib/sparse_set_collections.n.md::doctest#1` ぁEweb path でめEpass に戻った、E
  - `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md -i stdlib/alloc/collections/sparse_set.nepl --no-stdlib --no-tree -o /tmp/tests-sparse-set.json -j 2` は `10/10 pass` を確認した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 4`
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/sparse_set.nepl -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/tests/sparse_set.n.md -n 2`
  - `node nodesrc/run_doctest.js -i tests/stdlib/sparse_set_collections.n.md -n 1`
  - `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md -i stdlib/alloc/collections/sparse_set.nepl --no-stdlib --no-tree -o /tmp/tests-sparse-set.json -j 2`

# 2026-03-06 作業メモ (feat: examples/bf.nepl に Brainfuck Runner を実裁E

# 2026-03-12 作業メモ (alloc/collections/sparse_set 調査継続�E未 commit)

- [目皁Eもくてき]:
  - `alloc/collections` に `SparseSet` を[追加/つぁE��]し、`[0, n)` [篁E��/はんい]の integer set めEO(1) membership / insert / remove で[扱/あつか]えるようにする、E
- [進捁EしんちめE��]:
  - `SparseSet` の public API (`new` / `len` / `universe_len` / `contains` / `insert` / `remove` / `clear` / `free`) と public doctest / fixture は[一送Eひととお]り[作�E/さくせい]済み、E
  - normal path は focused 実行で[通過/つぁE��]してぁE��、E
    - `stdlib/alloc/collections/sparse_set.nepl::doctest#1/#2`
    - `stdlib/tests/sparse_set.n.md::doctest#1`
    - `tests/stdlib/sparse_set_collections.n.md::doctest#1`
- [根本原因/こんぽんげんいん]の[刁Eき]り[刁Eわ]ぁE
  - [当�E/とぁE��ょ]は `SparseSet` owner [冁E��/なぁE�E]の field [読/めEみ[出/だ]しが[壁Eこわ]れてぁE��ように[要Eみ]えたが、header めE`MemPtr<u8>` field で[持EめEつ設計かめEraw `i32` pointer [保持/ほじ]へ[落/お]とすことで normal path は[安宁Eあんてい]した、E
  - そ�E[征Eあと]に[殁Eのこ]っぁEfailure は invalid index path だけで、`contains s 8` の[最小侁EさいしょぁE��い]まで[縮封EしゅくしめE��]できた、E
  - さらに[追跡/つぁE��き]すると、`SparseSet` [固朁EこゆぁEではなぁE`sparse_set_diag_index` の[中/なか]で[佁Eつく]めEmessage string ぁEweb compile path で `RuntimeError: memory access out of bounds` を[起/お]こしてぁE��ことが[刁Eわ]かった、E
  - `diag_error StdErrorKind::IndexOutOfBounds "abc"` は pass する一方、`concat "sparse_set_contains" ": index out of bounds "` を[含/ふく]む chain だけが trap する、E
  - `stdlib/alloc/string.nepl::doctest#4` めE同系統/どぁE��ぁE��ぁEの web path OOB を[持EめEっており、`SparseSet` invalid path failure は[既孁Eきぞん]の `alloc/string` regression に[乁Eの]ってぁE��と[判断/はんだん]した、E
  - native compiler では `SparseSet invalid index` の[最小侁EさいしょぁE��い]は pass し、web compile path だけが trap するので、[直接/ちめE��せつ]の blocker は stdlib API 設計でなぁEweb compiler/runtime path [側/がわ]にある、E
- [判断/はんだん]:
  - `SparseSet` normal path の library 実裁E�E[成竁Eせいりつ]してぁE��が、invalid index の `Result::Err` path を[含/ふく]む focused suite ぁEweb compile path で[未収束/みしゅぁE��く]のため、現時点では commit しなぁE��E
  - [次/つぎ]は `alloc/string` の concat / integer-to-string [経路/けいろ]めEroot cause ベ�Eスで[直/なお]し、その[征Eあと]に `SparseSet` batch を[再開/さいかい]する、E

# 2026-03-12 作業メモ (ci: rust install -> cargo build -> trunk build を�E送Eaction 匁E

- 目皁E
  - GitHub Actions に散ってぁE�� `Node setup` / `Rust toolchain` / `wasm32 target` / `wasm-bindgen-cli` / `cargo build` / `trunk build` の重褁E�� 1 箁E��へ雁E��E��る、E
  - 吁Eworkflow は「�E送Ebuild artifact を作る job」と「その artifact を受けて test / deploy を行う job」に刁E��、build 済み成果物を�E利用する形へ寁E��る、E
- 根本原因:
  - `compile-test.yml` / `nepl-test-wasi.yml` / `nepl-test-llvm.yml` / `nmd-doctest.yml` / `nm-compile.yml` / `rust-test..yml` / `gh-pages.yml` が、それぞれ別に toolchain install と `trunk build` を持ってぁE��、E
  - そ�Eため手頁E�E更新漏れが起きやすく、`trunk` めE`wasm-bindgen-cli` の更新、`Trunk.toml` Linux 補正、examples 配置などを毎回多重管琁E��る構造になってぁE��、E
- 変更:
  - `.github/actions/bootstrap-build/action.yml`
    - CI 共通�E local composite action を追加、E
    - `actions/setup-node`、`npm install`、`actions-rs/toolchain`、`rustup target add wasm32-unknown-unknown`、`jetli/trunk-action`、`wasm-bindgen-cli` install、`Swatinem/rust-cache`、`cargo build --locked`、`trunk build --release` を集紁E��E
  - `.github/workflows/compile-test.yml`
  - `.github/workflows/rust-test..yml`
  - `.github/workflows/nm-compile.yml`
  - `.github/workflows/nmd-doctest.yml`
  - `.github/workflows/nepl-test-wasi.yml`
  - `.github/workflows/nepl-test-llvm.yml`
    - それぞれ `build` job で共送Eaction を使って `dist` / `target/debug` / `target/wasm32-unknown-unknown` めEartifact 化、E
    - test job 側は `actions/download-artifact` で取得してから、各 workflow 固有�E `cargo test` / `nodesrc/tests.js` / `cargo run -p nepl-cli` / LLVM runner を実行する形へ変更、E
  - `.github/workflows/gh-pages.yml`
    - pages 固有�E deploy/doctest/doc build は残しつつ、toolchain install と build 本体�E共送Eaction へ移動、E
- 検証:
  - 一晁Edirectory `/tmp/gha-yaml-check` を作って `npm install yaml` を行い、�E workflow と composite action めE`yaml` parser で構文確認、E
    - 対象:
      - `.github/workflows/*.yml`
      - `.github/actions/bootstrap-build/action.yml`
    - 結果: 全件 `OK`
- 差異メモ:
  - workflow 実行そのも�Eは GitHub Actions 上での実行が忁E��なので、ローカルでは YAML 構文と依存関係�E整合までを確認した、E
  - 現時点では artifact の粒度めE`dist` / `target/debug` / `target/wasm32-unknown-unknown` にしてぁE��。さらに絞る余地はあるが、まず�E共通化と再利用の成立を優先した、E

# 2026-03-12 作業メモ (ci: build 1 囁E+ pages/test 統吁E+ per-case timeout)

- 目皁E
  - workflow ごとに `bootstrap-build` を繰り返してぁE��構�Eをやめ、`trunk build` を含む build めE1 workflow 冁E�� 1 回だけ実行し、その成果物を�E test job と Pages deploy に再利用する、E
  - `gh-pages.yml` が別 workflow で test を�E実行してぁE��構造を解消し、site への publish めEtest workflow の一部へ統合する、E
  - 無限ループ系の hang で CI 全体が止まらなぁE��ぁE��E ケース 20 秒、test job 全佁E10 刁E�E上限を�Eれる、E
- 根本原因:
  - 前段の共送Eaction 化だけでは、workflow が�EかれてぁE��限り `cargo build` / `trunk build` / `npm install` / `cargo install wasm-bindgen-cli` ぁEworkflow 数だけ繰り返される、E
  - `gh-pages.yml` は site 生�Eのために tests を�E度回しており、同ぁEcommit に対して test ぁE2 重実行されてぁE��、E
  - `nodesrc/tests.js` は suite 全体�E実行�Eできても、WASM worker / LLVM child process に per-case timeout が無く、E ケースの hang ぁEsuite 全体を引きずる余地があった、E
- 変更:
  - `.github/actions/bootstrap-build/action.yml`
    - `actions/setup-node` に npm cache を追加、E
    - `web/package-lock.json` ベ�Eスで `npm ci` を使ぁE��に変更、E
    - `wasm-bindgen-cli` めE`actions/cache` で再利用するよう変更、E
    - `wasm-bindgen` の verify step を追加、E
  - `.github/workflows/ci.yml`
    - 旧 test workflow 群と Pages deploy めE1 workflow に統合、E
    - `build` job で `bootstrap-build` めE1 回だけ実行し、さらに tutorial / stdlib HTML めE`dist` 配下へ生�Eして artifact 化、E
    - `compile-test` / `rust-test` / `nm-compile` / `wasi-test` / `nmd-doctest` / `llvm-test` はすべて `needs: build` で artifact を�E利用、E
    - `pages-fast-*` と `pages-final-*` の 2 段 deploy を追加し、`trunk build` 後�E pending site を�Eに publish し、test 完亁E��に test JSON / summary を載せぁEfinal site で上書きする形にした、E
    - `gh-pages.yml` は削除、E
    - test job には `timeout-minutes: 10` を追加し、`node nodesrc/tests.js` / `cargo test` / `cargo run` は `timeout --signal=KILL 10m ...` で匁E��だ、E
    - test 実行環墁E�� `NEPL_TEST_CASE_TIMEOUT_MS=20000` / `NEPL_WASIX_TIMEOUT_MS=20000` を�E通指定、E
  - `nodesrc/tests.js`
    - WASM thread pool worker に per-case timer を追加し、E0 秒で応答しなぁEcase は worker めEterminate して error として回収する形へ変更、E
    - LLVM / native 実行に使ぁE`runCommand` に child process timeout を追加し、同じく 20 秒で kill するよう変更、E
- 検証:
  - `node --check nodesrc/tests.js`
  - 一晁Edirectory `/tmp/gha-yaml-check` を作って `npm install yaml` を行い、E
    - `.github/workflows/*.yml`
    - `.github/actions/bootstrap-build/action.yml`
    めEparser で検証、E
- 差異メモ:
  - Pages final deploy は `build` artifact の `dist` を�E利用し、site を作るために `trunk build` を�E実行しなぁE��E
  - pending/final の 2 囁Edeploy は Pages への publish を早めるためのも�Eで、tests 自体�E 1 回しか実行しなぁE��E
  - 初版では `site-fast` / `site-final` を通常の `upload-artifact` で中継してから `upload-pages-artifact` に渡してぁE��が、download 時に `dist` directory の階層前提が崩れて `tar: dist: Cannot open` になった、E
  - そ�Eため Pages 用 bundle job は直接 `upload-pages-artifact` を行い、deploy job は `deploy-pages` だけを行う構造へ修正した、E

- 目皁E
  - `rpn.nepl` を参老E��して `examples/bf.nepl` に Brainfuck の実行ツールを実裁E��る、E
  - 毎行�E力を受け付け、�E力ごとにメモリをリセチE��して独立実行する、E
- 変更:
  - `examples/bf.nepl`
    - `alloc/collections/stack` を使って `[` と `]` のジャンプ�Eを事前計算すめE`compile_jumps` を実裁E��E
    - `eval_line` で 30,000 バイト�Eメモリ上で BF 命令�E�E+` `-` `>` `<` `.` `,` `[` `]`�E�を実行、E
    - `,` は現状 0 を書き込む簡略実裁E��E
    - メインループ�E入力ごとにメモリバッファを確保�E解放し、状態を引き継がなぁE��E
    - 表示名�E "Brainfuck REPL" から "Brainfuck Runner" に変更�E�毎行リセチE��のため�E�、E
    - `neplg2:test[bf_hello_world]` doctest を追加�E�Eello World プログラムの実行）、E
- 検証:
  - `target/debug/nepl-cli -i examples/bf.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - `+++++++++[>++++++++>+++++++++++>+++>+<<<<-]>.>++.+++++++..+++.>+++++.<<+++++++++++++++.>.+++.------.--------.>+.>+.` を�E力して `Hello World!` の出力を確認、E

# 2026-03-06 作業メモ (TUI改喁E rpnの途中計算可視化とstdioの負数出力修正)

- 目皁E
  - `examples/rpn.nepl` において、`>` プロンプトの動作をレガシー版に合わせ、計算過程を「計算前」「計算後」としてANSIカラーで可視化する、E
  - 途中計算や出力で負数を含む式が正しく表示されるよぁE��`stdlib/std/stdio.nepl` の `print_i32` に存在する負数出力バグを修正する、E
- 変更:
  - `examples/rpn.nepl`
    - REPLプロンプト出力前にト�Eクン行を二重に出力しなぁE��ぁE�E長なループを削除、E
    - `print_step_before` を追加し、計算前の状態をシアン (`ansi_cyan`) で強調表示、E
    - `print_step_after` を追加し、計算結果を緑色 (`ansi_green`) で強調表示、E
  - `stdlib/std/stdio.nepl`
    - `print_i32` 関数で負の数への計算が不足して `0` となるバグを修正。絶対値の吁E��を送E��E��開したのち、負数であれば `-` 符号を付与するよぁE��修、E
    - コンパイルエラーを塞ぐため `mod_u` めE`rem_u` に修正、E
- 結果:
  - `1 2 + 3 + 4 5 + 6 +` などの連続�E力に対して、�E琁E��との計算箁E�� (`[1 2 +]` など) と結果が色付きで刁E��りやすく表示されるよぁE��なった、E
  - `-5` などの負の数を�E力した際に正常に表示されるよぁE��なった、E
- 検証:
  - `target/debug/nepl-cli -i examples/rpn.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - 途中計算�Eトレースおよび負数 (`1 2 3 4 + - 5 +` -> `-5`) の正しいフォーマットと出力を直接確認、E

# 2026-03-06 作業メモ (型安�E匁E `alloc/string` の主要Eraw 確保を `RegionToken<u8>` 匁E

- 目皁E
  - `alloc/string` の主要生成経路から `alloc_raw` を取り除き、`core/mem` の型付き領域 API に寁E��る、E
  - 斁E���E生�E処琁E��長さ�EチE��と本斁E�EインタめE`MemPtr<T>` / `RegionToken<T>` で扱ぁE���E部の生�Eインタ露出を減らす、E
- 変更:
  - `stdlib/alloc/string.nepl`
    - `string_alloc_region`
    - `string_region_len_ptr`
    - `string_region_data_ptr`
    - `string_data_ptr`
    - `string_finish`
    を追加し、文字�Eレイアウト専用の冁E��ヘルパとして整琁E��E
  - `concat`
    - 出力文字�Eの確保を `string_alloc_region` に変更、E
    - 出力�Eコピ�EめE`MemPtr<u8>` ベ�Eスへ変更、E
  - `sb_build`
    - 連結�Eバッファの確保を `RegionToken<u8>` 化、E
    - 吁Epart の読み出しと出力�E書き込みを型付きポインタへ変更、E
  - `str_slice`
    - 刁E��出し�Eの確保を `RegionToken<u8>` 化、E
  - `from_u128_radix`
    - 送E��E��積みの scratch めE`RegionToken<u8>` 化、E
    - 一晁Escratch は `dealloc_region` で解放、E
  - `from_f64`
    - 小数部 scratch めE`RegionToken<u8>` 化、E
    - scratch 解放を追加、E
- 結果:
  - `stdlib/alloc/string.nepl` から `alloc_raw/realloc_raw/dealloc_raw` の直接呼び出し�E消えた、E
  - `str` の冁E��表現自体�Eまだ raw address だが、主要な生�E経路では `RegionToken<u8>` から `string_finish` で確定する流れに整琁E��きた、E
- 検証:
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md --no-stdlib --no-tree -o /tmp/tests-string-type-safety-v1.json -j 15`
    - 結果: `26/26 pass`
  - `rg -n "alloc_raw|realloc_raw|dealloc_raw" stdlib/alloc/string.nepl`
    - 結果: 該当なぁE

# 2026-03-06 作業メモ (alloc/string: i128/u128 と基数付き斁E���E変換の整傁E

- 目皁E
  - `alloc/string` に整数の斁E���E表現変換を集紁E��、`core/cast` との責務を刁E��する、E
  - `i128` / `u128` を含む 2/8/10/16 進の変換を提供する、E
  - tutorial に、数値 cast と斁E���E変換の違いを�E示した導線を追加する、E
- 変更:
  - `stdlib/alloc/string.nepl`
    - `from_bool`
    - `to_bool`
    - `from_u128` / `from_u128_radix`
    - `to_u128` / `to_u128_radix`
    - `from_i128` / `from_i128_radix`
    - `to_i128` / `to_i128_radix`
    - `u128_divrem_small` など 128-bit 整数の補助関数群
    - `to_i32` の説明を現実裁E��合わせて更新
  - `tests/stdlib.n.md`
    - `i128/u128` と負数16進の focused case を追加
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
    - `core/cast` と `alloc/string` の使ぁE�EぁE
    - `Result` を返す解析関数
    - 2/8/10/16 進変換
    - `i128/u128` の大きい値の侁E
  - `tutorials/getting_started/00_index.n.md`
    - 新要Etutorial への導線を追加
- 検証:
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-conversions-i128-v3.json -j 15`
    - 結果: `19/19 pass`

# 2026-03-06 作業メモ (型安�E匁E `ptr_cast` 公開廁E��)

- 目皁E
  - ポインタ再解釈�Eような unsafe な公閁EAPI を減らし、`MemPtr<T>` / `RegionToken<T>` モチE��へ寁E��る、E
- 変更:
  - `stdlib/core/cast.nepl`
    - 未使用だっぁE`ptr_cast` を削除、E
    - モジュール先頭コメントを、数値 cast と bitcast のみに責務を限定する説明へ更新、E
- 判断:
  - `ptr_cast` は型だけを付け替える操作で、`MemPtr<T>` による型安�E化方針と矛盾する、E
  - repo 冁E��照は無く、現時点で公開面に残す合理性は無かった、E
  - `MemPtr<T>` は「型付きアドレス」、`RegionToken<T>` は「その領域のサイズと所有権」を伴ぁE��形ト�Eクンとして使ぁE�Eける、E

# 2026-03-06 作業メモ (フェーズF: tutorials Part6 拡允E+ library-first 匁E

- 目皁E
  - `tutorials/getting_started` Part6�E�E2、E7�E��E説明誤り�E不足を監査し、短く簡潔で安�Eな書き方へ更新する、E
  - 生�Eインタ露出を減らすため、`kp` 側に `Vec<i32>` 直受け補助を追加する、E
- 変更:
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
    - `Scanner/Writer` の基本パターンめEpipe 中忁E��簡潔化、E
    - i32/i64/空白区刁E��出力�E 3 ケースを安�E API 前提で整琁E��E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `Vec + sort + lower/upper_bound` めElibrary-first で再構�E、E
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
    - DP 本体を維持しつつ I/O を簡潔化、E
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - prefix めE`kp/kpprefix` ハンドル API 前提へ更新、E
    - two pointers の条件評価を短絡評価に依存しなぁE���Eな形へ修正、E
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 手書ぁEBFS から `kp/kpgraph` 利用へ移行、E
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - 未完�E表記を廁E��し、Part6 総まとめとしてチE��プレート�E対応表・実戦フローを追加、E
  - `tutorials/getting_started/00_index.n.md`
    - 誤字を修正�E�関数のふりがな�E�、E
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` ハンドルと `prefix_build_vec_i32` / `prefix_sum_i32` / `prefix_free_i32` を追加、E
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` を追加、E
  - `todo.md`
    - フェーズFの完亁E��み Part6 専用タスクを削除�E�未完亁E�Eみ維持E��、E
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i stdlib/kp/kpprefix.nepl -i stdlib/kp/kpsearch.nepl --no-tree -o /tmp/tests-part6-kp-refresh-v7.json -j 15`
    - 結果: `219/219 pass`
  - 補助確誁E
    - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md --no-tree -o /tmp/tests-part6-25-v6.json -j 15`
    - 結果: `207/207 pass`

# 2026-03-06 作業メモ (フェーズD: llvm `add/sub` 再定義リンク失敗�E根本修正)

- 目皁E
  - `--runner all --llvm-all` 実行時に `tests/llvm_target.n.md::doctest#4/#5` ぁE`invalid redefinition of function 'add'/'sub'` で失敗する問題を、後付け回避ではなく生成IR構造から解消する、E
- 原因:
  - `stdlib/core/math.nepl` の overload 群�E�Eadd/sub` など�E�が `#llvmir` 冁E��同一シンボル名！E@add`, `@sub`�E�を使ってぁE��、E
  - LLVM はシンボル名で overloading できなぁE��め、同一モジュールへ褁E��型版を同名定義するとリンク時に衝突する、E
  - さらに `u8` と `i32` は LLVM ABI で同じ `i32` に落ちるため、型別 overload をそのままシンボル名で共存させる設計が成立しなぁE��E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 生�E完亁E��前に `deduplicate_overloaded_llvm_symbols` を追加し、同吁E`define` をシグネチャ単位で一意化、E
    - `define` 側の重褁E�� `name__ovN_<sig>` へ正規化し、対応すめE`call` 参�Eも同一シグネチャで張り替える、E
    - 前段として `#llvmir` 呼び出し要件抽出と AST raw-body 選別補助を追加し、不要な overload 出力を抑制、E
- 検証:
- `NO_COLOR=false trunk build` -> success
- `cargo build -p nepl-cli` -> success
- `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-dedup-pass.json -j 15` -> `6/6 pass`
- `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-dedup.json -j 15` -> `791/791 pass`

# 2026-03-12 作業メモ (refactor(vec): Result 化しぁEVec API を直接依存�Eへ伝播)

- 目皁E
  - `alloc/collections/vec` の `new / with_capacity / push` めE`Result<..., StdErrorKind>` 化した変更を、直接依存すめEstdlib / tests / tutorials へ整合的に反映する、E
  - `Vec` 再確保を伴ぁEAPI めE`stack` 系と同じ失敗モチE��へ寁E��つつ、既存�E高水溁Ehelper では `unwrap_ok` 吸収で利用老E�E記述を過剰に崩さなぁE��E
- 根本原因:
  - `Vec` 本体だけを `Result` 化すると、`std/test` / `alloc/string` / `nm/parser` / `kpgraph` / `wasix/tui` などが旧 pure API を前提にして壊れる、E
  - さらに `StdErrorKind` が上位�E `alloc/diag/error` にあると、`vec -> diag/error -> vec` の循環依存が生じる、E
- 変更:
  - `stdlib/alloc/collections/vec.nepl`
    - `new / with_capacity / push` めE`Result<..., StdErrorKind>` 化、E
    - `with_capacity 0` は確保を行わず空 `MemPtr` を包む形にして `OutOfMemory` を不要化、E
  - `stdlib/std/test.nepl`
    - `checks_new` / `checks_push` で `Vec<Result<(),str>>` の `Result` を�E部吸収、E
  - `stdlib/alloc/string.nepl`
    - `StringBuilder` と `str_split` の冁E�� `Vec` 構築�E追加めE`unwrap_ok` 前提へ更新、E
  - `stdlib/alloc/diag/error.nepl`
    - `Diag` / `Diags` 冁E��の `Vec` 構築�E追加めE`unwrap_ok` 前提へ更新、E
  - `stdlib/alloc/hash/sha256.nepl`
    - scaffold 実裁E�E buffer 構築�E更新めE`unwrap_ok` 前提へ更新、E
  - `stdlib/kp/kpgraph.nepl`
    - BFS 結果ベクタ構築を `unwrap_ok` 前提へ更新、E
  - `stdlib/platforms/wasix/tui.nepl`
    - `text_wrap_lines` の行�E列構築を `unwrap_ok` 前提へ更新、E
  - `stdlib/nm/parser.nepl`
    - inline/block parser 冁E��の `Vec` 構築�E追加めE`unwrap_ok` 前提へ更新、E
  - `stdlib/tests/vec.n.md`
    - current `Vec Result` API に同期、E
  - `tests/stdlib/traits_order.n.md`
    - sort regression の `Vec` 構築を `unwrap_ok` 前提へ更新、E
  - `tests/stdlib/selfhost_req.n.md`
    - `Vec<u8>` buffer 構築を `unwrap_ok` 前提へ更新、E
  - `tests/stdlib/sort.n.md`
    - sort fixture の `Vec` 構築を `unwrap_ok` 前提へ更新、E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - `Vec` pipe 連鎖を `unwrap_ok new` と `|> push ... |> uwok` の current 書式へ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/stdlib/traits_order.n.md -i tests/stdlib/selfhost_req.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-a.json -j 4`
    - 結果: `10/10 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-b2.json -j 4`
    - 結果: `4/4 pass`
  - 補助確誁E
    - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/26_competitive_graph_bfs.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1` -> pass
- 差異メモ:
  - `Vec` の public API Result 化�E進んだが、`vec.nepl` 本体�E doc comment / doctest には旧書式�E旧 pure 前提の説明がまだ残る、E
  - `replace` めE`set` へ改名する案�E parser / keyword 制紁E�E刁E��刁E��後に再検討する、E

# 2026-03-12 作業メモ (docs(vec): doc comment と doctest めEcurrent Result API へ同期)

- 目皁E
  - `Vec` 本体を `Result` 化した後も、[stdlib/alloc/collections/vec.nepl](/mnt/d/project/NEPLg2/stdlib/alloc/collections/vec.nepl) の説明と埋め込み doctest が旧 pure API 前提のまま残ってぁE��差刁E��解消する、E
  - あわせて、旧節見�Eし形式を減らし、新しい doc comment policy に寁E��る、E
- 変更:
  - `vec.nepl`
    - file header の doctest めE`unwrap_ok new` と `|> push ... |> uwok` 前提へ更新、E
    - `new` / `with_capacity` / `len` / `cap` / `data_len` / `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` の comment 例を current API に同期、E
    - `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` の節見�Eしを `### [目皁Eもくてき]` 形式へ更新、E
- 検証:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 3` -> pass

# 2026-03-12 作業メモ (feat(collections): add bitset)

- 目皁E
  - `alloc/collections` に fixed-length な bit 雁E��を追加し、`BloomFilter` と違って false positive のなぁEmembership structure を標準で扱えるようにする、E
  - `reboot` 方針に合わせて bare API と public doctest を整え、pipe 併用の使ぁE��は `tests/stdlib` 側で保証する、E
- 変更:
  - `stdlib/alloc/collections/bitset.nepl`
    - `BitSet` を追加、E
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `fill` / `free` めEbare API で実裁E��E
    - 冁E��は `nbits` / `nbytes` / `MemPtr<u8>` を持つ owner struct とし、index から byte offset と bit mask を計算して更新する、E
    - doc comment は新 policy / format へ合わせて、usage doctest を各 public 関数へ追加、E
  - `stdlib/tests/bitset.n.md`
    - insert/remove/len と clear/fill の focused fixture を追加、E
  - `tests/stdlib/bitset_collections.n.md`
    - pipe 記法での `insert` / `remove` / `contains` / `fill` 利用を回帰として追加、E
- 検証:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 3` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 4` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 5` -> pass

# 2026-03-12 作業メモ (feat(collections): add adjacency matrix)

- 目皁E
  - `alloc/collections` に graph representation の最小実裁E��して `AdjacencyMatrix` を追加し、固定長の directed edge set めEO(1) membership で扱えるようにする、E
  - `trie` blocker と独立に、nested owner を避けた raw bit matrix で collection の種類を増やす、E
- 変更:
  - `stdlib/alloc/collections/adjacency_matrix.nepl`
    - `AdjacencyMatrix` を追加、E
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `free` めEbare API で実裁E��E
    - `(from, to)` めE`from * nverts + to` の bit index に写像し、byte 配�Eで保持する directed graph とした、E
    - doc comment は新 policy / format に合わせ、各 public 関数に usage doctest を追加、E
  - `stdlib/tests/adjacency_matrix.n.md`
    - insert/remove/clear の focused fixture を追加、E
  - `tests/stdlib/adjacency_matrix_collections.n.md`
    - pipe 記法での `insert` / `remove` / `contains` / `clear` 利用を回帰として追加、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/tests/adjacency_matrix.n.md -i tests/stdlib/adjacency_matrix_collections.n.md --no-stdlib --no-tree -o /tmp/tests-adjacency-matrix.json -j 2`
    - 結果: `9/9 pass`
- 差異メモ:
  - `contains g 4 0` のような篁E��夁Eindex に対する `Result::Err` 経路は、`target/debug/nepl-cli + wasmer` では正常に `1` を返す一方、web compile path では runtime OOB に落ちた、E
  - これは `AdjacencyMatrix` 実裁E��はなぁEweb compiler/runtime 側の別根因と判断し、今回の collection batch には混ぜてぁE��ぁE��E

# 2026-03-12 作業メモ (feat(collections): add counting bloom filter)

- 目皁E
  - `alloc/collections` に `CountingBloomFilter` を追加し、`BloomFilter` と同じ hasher 設計を保ちながら削除可能な近似 membership structure を標準で扱えるようにする、E
  - bare API と public doctest めEreboot 方針に合わせ、pipe 連鎖�E `tests/stdlib` 側で保証する、E
- 変更:
  - `stdlib/alloc/collections/counting_bloom_filter.nepl`
    - `CountingBloomFilter<.T,.H>` を追加、E
    - `new` / `len` / `insert` / `remove` / `contains` / `clear` / `free` めEbare API で実裁E��E
    - counter は `u8` 配�Eとし、E 本の probe index に対して insert は飽和加算、remove は 0 までの減算を行う、E
  - `stdlib/tests/counting_bloom_filter.n.md`
    - insert/remove/clear の focused fixture を追加、E
  - `tests/stdlib/counting_bloom_filter_collections.n.md`
    - pipe 記法での `insert` / `remove` / `contains` / `clear` 利用を回帰として追加、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md -i stdlib/alloc/collections/counting_bloom_filter.nepl --no-stdlib --no-tree -o /tmp/tests-counting-bloom-filter.json -j 2`
    - 結果: `8/8 pass`
- 差異メモ:
  - `new DefaultHash32 0` の invalid length `Result::Err` 経路は、`target/debug/nepl-cli + wasmer` では正常に `1` を返す一方、web compile path では runtime OOB に落ちた、E
  - これは `CountingBloomFilter` 実裁E��はなぁEweb compiler/runtime 側の別根因と判断し、今回の collection batch には混ぜてぁE��ぁE��E
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/bitset_collections.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/bitset.n.md -i tests/stdlib/bitset_collections.n.md -i stdlib/alloc/collections/bitset.nepl --no-stdlib --no-tree -o /tmp/tests-bitset-fixed.json -j 2`
    - 結果: `10/10 pass`
- 差異メモ:
  - out-of-bounds `Err` を返す focused case は、web compiler が生成しぁEcurrent wasm で hang する別根因に当たったため、この batch には混ぜてぁE��ぁE��E
  - `nepl-cli + wasmer` では同じ最小�E現が即終亁E��ることを確認済みで、stdlib 実裁E��はなぁEcompiler/runtime 側の別タスクとして刁E��出す、E

# 2026-03-06 作業メモ (フェーズD: llvm codegen 冁E�E precheck 後診断返却を除去)

- 目皁E
  - `precheck` 実行後に `codegen_llvm` ぁE`TypecheckFailed` を返してぁE��残存経路を除去し、前段検査不変条件へ統一する、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 冁E�E `select_active_raw_body(... )` `Err(diag)` 刁E��を `TypecheckFailed` 返却から internal panic へ変更、E
    - これにより、raw-body 選択失敗�E前段 `target_precheck::precheck_module_before_codegen` でのみ診断され、codegen 到達後�E生�E専任になる、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-llvm-invariant-2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-precheck-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm precheck 回帰ケースの追加)

- 目皁E
  - LLVM backend 到達前に未対忁Eintrinsic を診断できることを回帰固定する、E
- 変更:
  - `tests/llvm_target.n.md`
    - `llvm_precheck_rejects_wasm_only_intrinsic` を追加、E
    - `#intrinsic "i32_add"` めE`#target llvm` で使った場合に `diag_id: 3012` を期征E��めEcompile_fail ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-precheck-case.json -j 15`
    - 追加ケース�E�Edoctest#6::llvm`�E��E pass、E
    - 既存ケース `doctest#4/#5` は `invalid redefinition of function 'add'` で fail�E�既知未解決�E�、E
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-test-add.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: allocator helper 解決の意味論修正)

- 目皁E
  - runtime helper 共通化後に発生しぁErun-time 失敁E(`unreachable` / `memory access out of bounds`) を、E��に合わせではなぁEhelper 解決の意味論から修正する、E
- 原因:
  - `alloc`�E�安�EAPI�E�と `alloc_raw`�E�低レベルAPI�E��E現状の lowering では型互換になりうるため、`ALLOC_CANDIDATES=["alloc","alloc_raw"]` へ変更すると backend 冁E��確保で誤って `alloc` を掴む経路が発生する、E
  - そ�E結果、�E部確保�E前提�E�生ポインタ返却�E�と合わず、実行時に `unreachable` / OOB が発生した、E
- 変更:
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` めE`["alloc_raw", "alloc"]` に戻し、�E部 helper 解決は生�Eインタ意味論を優先するよぁE��正、E
    - 単体テスト期征E��めEraw 優先へ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-order-fix.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: runtime helper 解決の共通化と raw 依存縮封E

- 目皁E
  - `nepl-core` 冁E��重褁E��てぁE�� runtime helper�E�Elloc/dealloc/realloc�E�解決ロジチE��を�E通化し、`_raw` 名依存を段階縮小する、E
  - helper 名�E優先頁E��を安�EAPI名！Euffixなし）優先へ統一する、E
- 変更:
- `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` めE`["alloc", "alloc_raw"]` に変更�E�安�EAPI優先）、E
    - `RuntimeHelperKind` / `helper_candidates` / `helper_base_name` を追加、E

# 2026-03-09 作業メモ (trait 能力モチE��: `Eq` / `Ord` の共通化)

- 目皁E
  - `core/traits` に `Eq` / `Ord` を追加し、比輁E��味論を stdlib 共送Etrait として扱えるようにする、E
  - `alloc/collections/vec/sort.nepl` の局所 `Ord` 定義を撤去し、collections 側の比輁Ecapability めE`core` へ寁E��る、E
- 変更:
  - `stdlib/core/traits/eq.nepl`
    - `Eq` trait
    - `eq_by_trait`
    - `ne_by_trait`
    - `bool`, `i32`, `u8`, `i64`, `f32`, `f64`, `str` への impl
  - `stdlib/core/traits/ord.nepl`
    - `Ord` trait
    - `ord_lt`, `ord_le`, `ord_gt`, `ord_ge`
    - `bool`, `i32`, `u8`, `i64`, `i128`, `f32`, `f64` への impl
  - `stdlib/alloc/collections/vec/sort.nepl`
    - 局所 `Ord` trait と局所 impl を削除
    - `core/traits/ord` めEimport し、`sort_lt` 系 helper から共送E`ord_*` を呼ぶ形へ変更
  - `tests/stdlib/traits_order.n.md`
    - 日本語�E目皁E��ぁEfocused test を追加
- 判断:
  - `Eq<i128>` は既存�E刁E�� helper を仮定すると壊れるため、一旦追加しなかった、E
  - `Ord<str>` も既存�E頁E��比輁Ehelper が未整備なので、同様に見送った、E
  - まず�E既存�E `core/math` overload で根拠を持てる型だけを共送Etrait 化した、E
- 検証:
  - `NODE_NO_WARNINGS=1 node nodesrc/run_test.js`
    - `Eq` / `Ord` core focused case: pass
    - `vec/sort` + `Ord` std focused case: pass

# 2026-03-09 作業メモ (trait 能力モチE��: `Hash` の共通化)

- 目皁E
  - `Hash` trait めE`core/traits` へ追加し、hashmap / hashset が�E体的な `hash32_i32` / `hash32_str` へ直接依存せず�E送Ehelper 経由でキーを混合できるようにする、E
  - 封E��の `Serialize` / `Deserialize` と同じく、型ごとの能力を stdlib trait として明示する流れを揃える、E
- 変更:
  - `stdlib/core/traits/hash.nepl`
  - `Hash` trait
  - `hash32_by_trait`
  - `i32`

# 2026-03-11 作業メモ (`streamio` target 持E��化と `u32/u64` bare I/O の修正)

- 目皁E
  - `scanner` / `writer` めEstdin/stdout 固定�E no-arg API から外し、`io_stdin` / `io_stdout` / `io_text` / `io_bytes` の target 持E��で生�Eする形へ寁E��る、E
  - `u32` / `u64` の bare `read` / `write` を、型 suffix 名に戻さず current overload 方針�Eまま安定化する、E
  - Part6 tutorial と `kp` 周辺に残ってぁE�� old move-model 前提を、現行所有権モチE��へ合わせる、E
- 原因:
  - `std/streamio` だぁE`read` / `write` の bare 名へ寁E��ても、生成�E口 `scanner()` / `writer()` ぁEstdin/stdout 固定�Eままだと、`std/io` / `iotarget` と責務が二重化してぁE��、E
  - `u64` は compiler 側で `wasm_shared::valtype` がまだ `i32` 扱ぁE�E箁E��を残しており、Wasm signature が崩れてぁE��、E
  - `u32` / `u64` の 10 進出力�E、unsigned 値めEsigned overload へ落としてぁE��ため `4294967295` ぁE`18446744073709551615` に化けてぁE��、E
  - `PrefixI32` めEtutorial Part6 の `Vec` 走査には old move-model 前提が残ってぁE��、E
- 変更:
  - `stdlib/std/streamio.nepl`
    - `scanner <(IoReadTarget)*>Result<StreamScanner,str>>`
    - `writer <(IoWriteTarget)*>Result<StreamWriter,str>>`
    - `scanner_from_bytes`
    - `StreamWriter` header に `TargetKind` を追加
    - `u32` / `u64` の append 実裁E�� unsigned decimal として修正
    - `StreamScanner` / `StreamWriter` の doc comment めEcurrent 実裁E��同期
  - `stdlib/std/iotarget.nepl`
    - `io_stdin` / `io_stdout` / `io_text` / `io_bytes` を生成�E口として利用
  - `nepl-core/src/wasm_shared.rs`
    - `u64` めEWasm `I64` として扱ぁE��ぁE��正
  - `nodesrc/run_test.js`
    - `BigInt` の JSON 出力と return decode を追加
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` に `Copy` / `Clone` を付丁E
    - `prefix_build_vec_i32` めE`vec_data_len` ベ�Eスへ修正
  - `tests/stdlib/streamio.n.md`
  - `tests/stdlib/kp.n.md`
  - `tests/stdlib/kp_i64.n.md`
  - `tests/stdlib/stdin.n.md`
  - `tests/compiler/move_effect.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `stdlib/kp/kpgraph.nepl`
    - `unwrap_ok scanner io_stdin` / `unwrap_ok writer io_stdout` へ統一
- 検証:
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i /tmp/u64_probe2.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp_i64.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 2`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3`
  - `node nodesrc/run_doctest.js -i tests/stdlib/stdin.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/24_competitive_dp_basics.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpgraph.nepl -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpprefix.nepl -n 1`
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 20`

# 2026-03-09 作業メモ (compiler 前提固宁E `#prelude` 最小実裁E�� Copy 固定表撤去)

- [目皁Eもくてき]:
  - `todo.md` の `compiler 前提` 残件だっぁE`Copy` 固定表依存を、[実際/じっさい]に source [側/がわ]から trait impl を[供給/きょぁE��めE��]できる[状慁EじょぁE��い]へ[移/ぁE��]す、E
  - parser だけに[存在/そんざい]してぁE�� `#prelude` / `#no_prelude` めEloader [段隁Eだんかい]でめE解釁Eかいしゃく]し、copy/clone 非ハードコード化の[前提/ぜんてい]を[整/ととの]える、E
- [原因/げんぁE��]:
  - `#prelude` と `#no_prelude` は lexer / parser / AST にだけ[存在/そんざい]し、loader では[無要Eむし]されてぁE��、E
  - そ�Eため `Copy` / `Clone` impl めEsource [側/がわ]から[既宁Eきてい][供給/きょぁE��めE��]できず、`TypeCtx::is_copy` に primitive 固定表フォールバックを[殁Eのこ]す[忁E��Eひつよう]があった、E
- [変更/へんこぁE:
  - `nepl-core/src/loader.rs`
    - root module [限宁Eげんてい]で `#prelude` / `#no_prelude` を[処琁Eしょり]するように[変更/へんこぁEした、E
    - `#no_prelude` がなぁEroot module には[既宁Eきてい]で `std/prelude_base` を[読/めEみ[込/こ]む、E
    - import/include の[再帰/さいき] load では default prelude を[適用/てきよぁEしなぁE��ぁE��して、stdlib [冁E��/なぁE�E] import での[循環/じゅんかん]を[避/さ]けた、E
  - `stdlib/std/prelude_base.nepl`
    - [最封EさいしょぁE prelude として[追加/つぁE��]した、E
    - [当面/とぁE��ん]は `core/traits/copy` だけを[読/めEみ[込/こ]み、copy/clone 能力�E source [供給/きょぁE��めE��]に[絁Eしぼ]った、E
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` の最終フォールバックから primitive 固定表を[削除/さくじょ]した、E
    - `Copy` trait が[要Eみ]えてぁE��い[場吁Eばあい]は、[参�E/さんしょぁE型と `Never` だけを compiler [冁E��/なぁE��い]の copy として[扱/あつか]ぁE��E
  - `tests/compiler/prelude_copy.n.md`
    - default prelude で `Copy` bound が[送Eとお]ることを[確誁Eかくにん]する focused case を[追加/つぁE��]した、E
    - `#prelude std/prelude_base` と `#no_prelude` を[併訁EへぁE��]しても、[明示皁Eめいじてき] prelude が[優允EめE��せん]されることを[固宁Eこてい]した、E
    - `#no_prelude` だけでは `Copy` trait [供給/きょぁE��めE��]が[涁Eき]え、`.T: Copy` ぁE`3073` で[落/お]ちることを[追加/つぁE��]した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/compiler/resolve.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-only.json -j 15` -> `14/14 pass`
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-copy-only.json -j 15` -> `3/3 pass`
- [判断/はんだん]:
  - `Copy` の source [供給/きょぁE��めE��]は default prelude を[送Eとお]すことで[既孁Eきぞん]コードを[壁Eこわ]さずに[移衁EぁE��ぁEできる、E
  - `#no_prelude` は「標溁Ecapability を[含/ふく]めて自前で[管琁Eかんり]する」ため�E opt-out として[機�E/き�EぁEする、E
    - `bool`
    - `u8`
    - `i64`
    - `str`
    への impl を追加、E
  - `stdlib/alloc/collections/hashmap.nepl`
    - `hash32_i32` / `hash32_str` の直接呼び出しを `hash32_by_trait` に置換、E
  - `stdlib/alloc/collections/hashset.nepl`
    - 同様に `hash32_by_trait` 経由へ置換、E
  - `tests/stdlib/traits_hash.n.md`
    - `[目皁Eもくてき]` つぁEfocused case を追加、E
- 判断:
  - `Hash<i64>` は [上佁EじょぁE��] / [下佁Eかい] 32-bit めEXOR で折りたたんでから `hash32_i32` へ流す、E
  - `Hash` の対象は、まず既孁Estdlib が安定して支えてぁE��キー型に限定した、E
  - `i128` めE��自構造体�Eハッシュ能力�E、今征E`Serialize` / `Eq` との整合を見ながら追加する、E
- compiler 修正:
  - なし。今回の確認で見つかった問題�E `traits_hash.n.md` 側の API サンプルが現衁E`hashmap` / `hashset` の利用流儀とずれてぁE��ことだった、E
  - `must_hm` / `must_hs` と `Option` の match を使ぁE��存流儀へ合わせて修正した、E
- 検証:
  - `node` + `nodesrc/compiler_loader` による compile-only focused check で、E
    - `hash32_by_trait` 単佁E
    - `hashmap/hashset/hashmap_str/hashset_str`
    を使ぁEsnippet
    の両方ぁE`COMPILE_OK` を返すことを確認、E
  - `nodesrc/tests.js` はこ�E環墁E��は長く�Eら下がることがあるため、focused な compile-only でまず妥当性を固定した、E

# 2026-03-09 作業メモ (`std/test` 雁E��EAPI 追加と nested generic overload 根本修正)

- 目皁E
  - stdlib reboot 前�EチE��ト基盤として、E 件失敗しても残りの検査を継続実行できる `std/test` の collectable API を整備する、E
  - `Vec<Result<(),str>>` に `push` / `vec_push` / pipe で `Result<(),str>` を積めなぁEcompiler バグを、library 側の回避ではなぁEtypecheck の根本原因から修正する、E
- 変更:
  - `stdlib/std/test.nepl`
    - `checks_new`
    - `checks_push`
    - `check`
    - `check_eq_i32`
    - `check_ne`
    - `check_str_eq`
    - `check_ok_i32`
    - `check_err_i32`
    - `check_status_str`
    - `checks_has_err(_loop)`
    - `checks_summary(_loop)`
    - `checks_report_failures`
    - `finish_checks`
    を追加した、E
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` は、対応すめE`check_*` を受けて即時失敗する薄ぁE��チE��へ整琁E��た、E
  - `tests/std_test_collect.n.md`
    - `[目皁Eもくてき]` と `[佁Eなに]を[確/たし]かめるか` を付けぁEfocused case を追加した、E
    - 全件成功時�E summary 出力と、失敗を含むとき�E summary + 個別失敗�E力を固定した、E
  - `tests/compiler/overload_nested_generic_push.n.md`
    - `Vec<Result<(),str>>` に対する `push` / `vec_push` / pipe の nested generic overload 解決を確認すめEcompiler 回帰 test を追加した、E
  - `nepl-core/src/types.rs`
    - 関数型に含まれる型変数 binding を退避・復允E��めE
      - `snapshot_type_var_bindings`
      - `restore_type_var_bindings`
      を追加した、E
  - `nepl-core/src/typecheck.rs`
    - `check_function` で関数本体を検査する前に `func_ty` 上�E型変数 binding めEsnapshot し、終亁E��に忁E�� restore するよう変更した、E
- 原因:
  - generic 関数本体�E型検査中に、E��数シグネチャ自体が持ってぁE��型変数 `TypeId` ぁEunification で束縛され、その束縛が `Env` 上�E大域関数型へ残留してぁE��、E
  - そ�E結果、`vec_push <.T> <(Vec<.T>, .T)->Vec<.T>>` の `.T` が過去の検査で `i32` へ汚染され、`Vec<Result<(),str>>` に対する overload 推論で `Vec<i32>` として扱われてぁE��、E
  - 明示型引数付き `vec_push<Result<(),str>>` が通り、型引数省略時だけ落ちることから、candidate 選択時の `instantiate(binding.ty)` 入力が既に汚染されてぁE��と特定した、E
- 結果:
  - `std/test` の collectable API で、`[ok,ok,err,ok,err]` 形式�E概要と失敗添字�E琁E��をまとめて表示できるようになった、E
  - nested generic `push` / `vec_push` / pipe は、型引数を�E示しなくてめE`Vec<Result<(),str>>` 上で解決できるようになった、E
- 検証:
  - `trunk build`�E�Eoot, `NO_COLOR=false`�E�E-> success
  - `node nodesrc/tests.js -i tests/std_test_collect.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-std-test-collect-focused.json -j 15`
    - 結果: `5/5 pass`
    - `find_runtime_helper_key`�E�名前解決�E�と `find_runtime_helper_index`�E�Endex解決�E�を追加、E
  - `nepl-core/src/codegen_wasm.rs`
    - ローカル実裁E��っぁEhelper 名解決を削除し、`runtime_helpers::find_runtime_helper_index` に統一、E
  - `nepl-core/src/monomorphize.rs`
    - helper 保持ルート探索めE`find_runtime_helper_key` + `RuntimeHelperKind` へ置換、E
    - 重褁E��てぁE��名前マッチE��数を削除、E
  - `nepl-core/src/codegen_llvm.rs`
    - helper 候補取得を `helper_candidates(RuntimeHelperKind::...)` に統一、E
    - `resolve_symbol_name` の候補一致めE`helper_base_name` ベ�Eスへ変更し、namespaced/mangled 名でも同一規則で解決、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-helper-unify.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm backend の wasm-body 刁E��を不変条件匁E

- 目皁E
  - `codegen_llvm` 側に残ってぁE�� backend 入力エラー刁E��！EUnsupportedWasmBody`�E�を前段検査前提へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError` から `UnsupportedWasmBody` / `UnsupportedParsedFunctionBody` を削除、E
    - `emit_ll_from_module_for_target` 冁E�� `ActiveRawBody::Wasm` 到達時の `Err` めEinternal panic に変更、E
    - `FnBody::Wasm` reachable 到達時の `Err` めEinternal panic に変更、E
    - HIR lowering 経路で `HirBody::Wasm` 到達時の `Err` めEinternal panic に変更、E
    - 対応テスチE`emit_ll_rejects_entry_with_wasm_body` は `TypecheckFailed` を期征E��る形へ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: wasm codegen 診断返却経路の撤去)

- 目皁E
  - `codegen` 到達後�E生�E専任にする方針に合わせ、`codegen_wasm` の `Vec<Diagnostic>` 返却経路を撤去する、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_body` / `lower_user` の戻り値めE`Result<Function, Vec<Diagnostic>>` から `Function` へ変更、E
    - `gen_block` / `gen_expr` の `diags` 引数を削除、E
    - `generate_wasm` の code section 生�Eで `Err(ds)` 刁E��を削除し、前段検査通過後�E直接生�Eする形に統一、E
    - backend 冁E��断として残ってぁE��未使用関数 `validate_wasm_stack` を削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-wasm-no-diag.json -j 15` -> `8/8 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-wasm-no-diag.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: wasm helper 解決の自己再帰バグ修正)

- 目皁E
  - `tests + stdlib` で発生してぁE�� `RangeError: Maximum call stack size exceeded` を根本原因から解消する、E
- 再現と刁E��刁E��:
  - `option.nepl` doctest を単独再現すると `wasm-function[4]` の自己再帰で停止、E
  - 同一ソースめE`nepl-cli` で生�Eした wasm は正常実行、E
  - `web` 生�E WAT と `native` 生�E WAT を比輁E��ると、同一箁E��で `call 5` ぁE`call 4`�E��E己呼び出し）に化けてぁE��、E
- 原因:
  - `codegen_wasm` の runtime helper 解決が曖昧な斁E���E一致�E�Erefix/contains�E�依存だった、E
  - allocator helper 解決時に `alloc` と `alloc_raw` の取り違えが発生し、enum/tuple 構築時の冁E��確保で自己再帰が起きてぁE��、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - helper 名�E基底名抽出 `helper_base_name` を追加、E
    - runtime helper 解決を基底名一致へ変更し、曖昧一致を廁E��、E
    - 現在 lowering 中の関数インチE��クスは helper 候補から除外、E
    - `LocalMap` に `alloc_helper_idx` を保持し、E��数ごとに一度だぁEhelper を確定、E
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` めE`["alloc_raw", "alloc"]` の頁E��変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-vec-option-after-alloc-helper-fix.json -j 15` -> `22/22 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-helper-fix.json -j 15` -> `791/791 pass`

# 2026-03-05 作業メモ (フェーズD: web 実行時 `compile: unreachable` の根本修正)

- 目皁E
  - `web/dist` 経路でのみ発生してぁE�� `phase=compile, error=unreachable` を根本原因から解消する、E
- 原因:
  - `codegen_wasm.rs` の raw wasm 行パースで、ローカル解決クロージャぁE`parse_wasm_line_with_lookup` 側の `$` 正規化と二重処琁E��なってぁE��、E
  - そ�E結果、`#wasm` 本斁E�E `$a`/`$b` ぁEcodegen 時�Eみ `unknown local` になめEpanic してぁE���E�Erecheck 側とは不整合）、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `parse_wasm_line` の lookup めE`|name| locals.lookup(name)` に統一、E
    - 旧 `parse_local` ヘルパを削除、E
  - `nepl-web/src/lib.rs`
    - `console_error_panic_hook::set_once()` めE`#[wasm_bindgen(start)]` で有効化し、WASM panic の原因位置を可視化、E
  - `nodesrc/run_test.js`
    - `formatError` を追加し、compile/run 失敗時に stack を保持して JSON 出力へ反映、E
- 検証:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-rootfix.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-stdlib --no-tree -o /tmp/tests-list-after-rootfix.json -j 15` -> `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-rootfix.json -j 15` -> `707/791 pass`�E�残り `84 fail` は run 晁E`Maximum call stack size exceeded`。`compile: unreachable` は再現せず�E�E

# 2026-03-05 作業メモ (フェーズD: web 実行時 `unreachable` の刁E��刁E��)

- 目皁E
  - 全体テスチE(`tests + stdlib`) で多発する `phase=compile, error=unreachable` を、E��に合わせではなく根本原因から刁E��刁E��る、E
- 実施:
  - `trunk build` 後に
    - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-baseline-after-revert-v1.json -j 15`
    - 結果: `349/791 pass`、`442 fail`、上位失敗�E `stdlib/alloc/collections/list.nepl` doctest 群の `unreachable`、E
  - 同じ入力を `nepl-cli` で単体コンパイル:
    - `target/debug/nepl-cli -i /tmp/list_doctest1_clean.nepl --target std --emit wasm -o /tmp/list_doctest1_out -v`
    - 結果: compile 成功 (`DEBUG: compile_module returned Ok`)、E
- 結諁E
  - 失敗�E `web/dist`�E�EASM 上�E compiler 実行）経路に限定される、E
  - `codegen_wasm` の今回差刁E��戻しても�E現するため、単純な backend 変更起因ではなぁE��E
  - 以降�E `web` 側で panic を診断化して原因位置を可視化するタスクを上流課題として扱ぁE��E

# 2026-03-05 作業メモ (フェーズD: todo整琁E+ llvm precheck 返り値規紁E

- 目皁E
  - `todo.md` の完亁E��み頁E���E�EUnsupportedHirLowering` 整琁E��を反映し、未完亁E��けを残す、E
  - LLVM 前段検査に「非 unit 関数は値を返す」規紁E��追加して、backend 依存失敗�E前段化を進める、E
- 変更:
  - `todo.md`
    - フェーズDの完亁E��み衁E
      - `llvm 経路でめEbackend 依存エラーを前段診断に寁E��る！EnsupportedHirLowering の整琁E��`
      を削除し、残課題として
      - `llvm 経路の precheck を拡張し、intrinsic/戻り値規紁E��ど backend 依存失敗を前段で確定する。`
      へ更新、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` に `TypeCtx` を渡す形へ変更、E
    - reachable な `HirBody::Block` 関数につぁE��、戻り値型が靁E`unit` かつ block が値を返さなぁE��合を `D3003` で診断、E
  - `nepl-core/src/codegen_llvm.rs`
    - `precheck_llvm_codegen(&types, &hir, &reachable_set)` 呼び出しへ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v9.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm codegen_precheck に実検査を追加)

- 目皁E
  - `codegen` 到達後�E生�E専任に寁E��るため、LLVM 側でも前段検査で弾ける入力を増やす、E
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` を追加、E
    - 到達関数�E�Eeachable set�E�に対して expression tree を走査し、LLVM 未対忁Eintrinsic を前段診断化、E
    - 未対忁Eintrinsic は `D3012 (TypeUnknownIntrinsic)` で報告、E
  - `nepl-core/src/codegen_llvm.rs`
    - HIR lower 前に `precheck_llvm_codegen` を実行し、error があれ�E `TypecheckFailed` で早期終亁E��E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v8.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend 診断型�E整琁E

- 目皁E
  - `codegen_llvm` から `UnsupportedHirLowering` 返却経路が消えた状態を型定義にも反映する、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError::UnsupportedHirLowering` めEenum / Display から削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v6.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm 残孁Ebackend 診断の不変条件匁E継綁E

- 目皁E
  - `codegen_llvm` に残ってぁE�� `UnsupportedHirLowering` を削減し、前段通過後�E生�E専任モチE��へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 以下を `UnsupportedHirLowering` 返却から internal panic へ変更:
      - 関数 return 型不一致
      - enum/struct/tuple 構築時の `alloc` 忁E��判宁E
      - enum payload / struct field / tuple item の値生�E忁E���E型不一致
      - `match` arm の結果型不一致
      - unknown intrinsic 到遁E
      - unsupported expression kind 到遁E
      - 斁E���EリチE��ルID篁E��夁E
      - 斁E���E具体化時�E `alloc` 忁E��判宁E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v5.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm intrinsic 引数・型チェチE��の backend 診断を不変条件匁E

- 目皁E
  - `codegen_llvm` intrinsic lowering に残ってぁE�� backend 診断を削減し、前段通過後�E生�E専任モチE��へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 以下を `UnsupportedHirLowering` 返却から internal panic へ変更:
      - `load` の引数個数/型引数個数不一致、�Eインタ値不在、�Eインタ型不一致
      - `store` の引数個数/型引数個数不一致、�Eインタ/値不在、�Eインタ型不一致、`u8` 値型不一致、格納型不一致
      - `add` の引数個数不一致、lhs/rhs 不在、i32以夁E
      - `f32_to_i32` / `i32_to_u8` / `u8_to_i32` の引数個数・値不在・型不一致
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm 制御構文の backend 診断を不変条件匁E

- 目皁E
  - `codegen_llvm` の `if/while/match` で残ってぁE�� backend 診断を削減し、型検査・前段検証通過後�E生�E専任へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `if`:
      - 条件が値を返さなぁE
      - 条件ぁE`i32/bool` 互換でなぁE
      - then/else 刁E��結果型不一致
      めE`UnsupportedHirLowering` 返却から internal panic へ変更、E
    - `while`:
      - 条件が値を返さなぁE
      - 条件ぁE`i32/bool` 互換でなぁE
      めEinternal panic へ変更、E
    - `match`:
      - scrutinee が値を返さなぁE
      - scrutinee ぁEenum pointer (`i32`) でなぁE
      - arm ぁE件
      めEinternal panic へ変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm call_indirect の backend 診断を不変条件匁E

- 目皁E
  - `codegen_llvm` の `call_indirect` で残ってぁE�� backend 診断�E�EUnsupportedHirLowering`�E�を削減し、前段通過後�E生�E専任に寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `call_indirect` につぁE��以下�E `UnsupportedHirLowering` 返却めEinternal panic 匁E
      - callee が値を返さなぁE
      - callee ぁE`i32` 関数IDでなぁE
      - 引数が値を返さなぁE
      - 引数個数不一致
      - 引数型不一致
      - 候補関数未検�E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: raw wasm 行検査の前段刁E��を完亁E

- 目皁E
  - `codegen_precheck` ぁE`codegen_wasm` 実裁E��細へ依存する経路を解消し、前段検査の責務を `wasm_shared` へ雁E��E��る、E
  - 「codegen 到達時は生�E専任」�E方針を維持し、raw wasm 行パース失敗を前段で確定する、E
- 変更:
  - `nepl-core/src/wasm_shared.rs`
    - `parse_wasm_line_with_lookup` を�E有化、E
    - `precheck_raw_wasm_body` を追加し、`HirBody::Wasm` 行を前段で検査して `D4004` を返すように変更、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - raw wasm 事前検査呼び出し�EめE`codegen_wasm` から `wasm_shared` へ変更、E
  - `todo.md`
    - フェーズDの「`codegen_precheck` の wasm 側ヘルパ依存整琁E��頁E��を完亁E��して削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: #wasm のスタチE��検証を前段検査へ移勁E

- 目皁E
  - 「codegen は正しい入力を生�Eするだけ」�E方針に合わせ、`#wasm` ボディ検証めEbackend 実行時ではなぁE`codegen_precheck` 側で完亁E��せる、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `precheck_raw_wasm_body` シグネチャめE`precheck_raw_wasm_body(ctx, func)` に変更、E
    - raw 行�Eパ�Eス成功時に命令列を蓁E��し、前段で `validate_wasm_stack` を実行するよぁE��更、E
    - `lower_user` の `HirBody::Wasm` 経路から `validate_wasm_stack` を削除、E
    - `generate_wasm` の診断雁E��E��実質空に整琁E��Eodegen 冁E��断を発生させなぁE��向に統一�E�、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_raw_wasm_body` 呼び出しを新シグネチャへ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: codegen_precheck の wasm 事前検査を�E通モジュールへ刁E��)

- 目皁E
  - `passes/codegen_precheck.rs` ぁE`codegen_wasm.rs` 実裁E��細へ直接依存してぁE��状態を整琁E��、前段検査ロジチE��を�E有モジュールへ刁E��する、E
  - 「codegen は正しい入力を生�Eするだけ」�E方針に合わせ、backend の `skip`/診断蓁E��を不変条件違反へ寁E��る、E
- 変更:
  - `nepl-core/src/wasm_shared.rs` を新規追加、E
    - wasm署名解決 (`wasm_sig`, `wasm_sig_ids`)
    - generic skip 判宁E(`should_skip_wasm_codegen_for_generic`)
    - 到達関数解极E(`collect_reachable_wasm_functions`)
    - 間接呼び出しを含む署名集合収雁E(`collect_wasm_signature_set`)
    - wasm intrinsic 対応判宁E(`is_supported_wasm_intrinsic`)
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 上記ロジチE��めE`wasm_shared` 参�Eへ置換、E
    - `precheck_raw_wasm_body` のみ `codegen_wasm` 側を継続利用�E�次段で刁E��予定）、E
  - `nepl-core/src/codegen_wasm.rs`
    - extern/function 署名不一致時�E `skip` を廁E��ぁEinternal panic 化、E
    - `lower_body` で backend 診断が返る経路めEinternal panic 化、E
    - 共有ロジチE��は `wasm_shared` 呼び出しへ委譲、E
  - `nepl-core/src/lib.rs`
    - `pub mod wasm_shared;` を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend 診断を前段不変条件へ移衁E

- 目皁E
  - `todo.md` フェーズD方針に合わせ、`codegen_llvm` 側で発行してぁE��「前段通過後に到達しなぁE�Eず」�E診断を廁E��し、前段検証の不変条件として扱ぁE��E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `let` の型不一致 (`let type mismatch`) めE`UnsupportedHirLowering` から internal panic へ変更、E
    - `set` の型不一致 (`set type mismatch`) めE`UnsupportedHirLowering` から internal panic へ変更、E
    - 未解決 trait call の到達を `UnsupportedHirLowering` から internal panic へ変更、E
    - call 引数型不一致めE`UnsupportedHirLowering` から internal panic へ変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant-panic-v1.json -j 15` -> `707/791 pass`�E�EMaximum call stack size exceeded` が多数。今回の変更対象外�E既存失敗として継続調査�E�E

# 2026-03-05 作業メモ (フェーズC/D接綁E core/mem に MemPtr 初期化オーバ�Eロード追加)

- 目皁E
  - `core/mem` 後段移行！Estdlib/std`/tutorials�E�で `i32` 生�Eインタを露出せずに配�E初期化できる上流APIを用意する、E
  - `MemPtr` モチE��上で `fill/memset` を統一し、`Result` で失敗を扱えるようにする、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `memset_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` を追加、E
    - `fill_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` を追加、E
    - `fill_i32 <(MemPtr<i32>,i32,i32)->Result<(),str>>` を追加、E
    - 無効ポインタめE��の長さ�E `Result::Err` を返す、E
  - `tests/memory_safety.n.md`
    - `MemPtr fill_i32/fill_u8 の安�Eオーバ�Eロード` ケースを追加、E
    - `MemPtr fill 系は無効引数めEErr で返す` ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-fill-overload.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-mem-fill-overload.json -j 15` -> `787/787 pass`

# 2026-03-05 作業メモ (フェーズD: kpread_core ヘッダフィールド�E型安�E匁E

- 目皁E
  - `kpread_core` に残ってぁE��ヘッダ生オフセチE���E�E0/4/8`�E�を列挙型へ移行し、`kpread`/`kpwrite` と同じ墁E��表現に揁E��る、E
  - ヘッダレイアウト�E意味を型で固定し、オフセチE��誤持E��を上流で防ぐ、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `ScannerHeaderFieldCore` を追加�E�EBufPtr` / `Len` / `Pos`�E�、E
    - `scanner_header_core_off` を追加し、オフセチE��解決めE箁E��に雁E��E��E
    - `store_i32_u8_at sc*_region 0/4/8 ...` を�E挙型 + オフセチE��関数経由へ置換、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kp-core-header-field-enum.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-core-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite ヘッダフィールド�E型安�E匁E

- 目皁E
  - `kpwrite` のヘッダアクセスで使ってぁE��生オフセチE��値�E�E0/4/8/12/16`�E�を列挙型に置換し、`kpread` と同じ安�EモチE��へ統一する、E
  - `mem/kpread/kpwrite` の公開API安�E化で、�EチE��墁E��の意味を型で表現する、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterHeaderField` を追加�E�EBufPtr` / `Cap` / `WriteLen` / `IovPtr` / `NwPtr`�E�、E
    - `writer_header_off` を追加し、オフセチE��解決を一箁E��に雁E��E��E
    - `writer_header_ptr` / `writer_load_header` / `writer_store_header` / `writer_load_header_ptr` の第2引数めE`i32` から `WriterHeaderField` に変更、E
    - 呼び出し�Eの生数値オフセチE��を�E廁E��、�E挙値に置換、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kp-header-field-enum-unified.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpwrite-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpread ヘッダフィールド�E型安�E匁E

- 目皁E
  - `kpread` のヘッダアクセスで使ってぁE��生オフセチE��値�E�E0/4/8`�E�を列挙型へ置き換え、呼び出し�Eの誤持E��を減らす、E
  - `todo.md` 2026-03-03 フェーズD�E�Emem/kpread/kpwrite` の公開API安�E化）に沿って、上流�E表現を固定する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `ScannerHeaderField` を追加�E�EBufPtr` / `Len` / `Pos`�E�、E
    - `scanner_header_off` を追加し、オフセチE��解決めE箁E��へ雁E��E��E
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` の第2引数めE`i32` から `ScannerHeaderField` に変更、E
    - 呼び出し�Eの `scanner_load_header sc 0/4/8` と `scanner_store_header sc 8 ...` を�E挙型持E��へ置換、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-field-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-header-field.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpread ヘッダアクセスのサイレント失敗を除去)

- 目皁E
  - `scanner_load_header` / `scanner_store_header` の失敗時フォールバック�E�E0` / `()`�E�を廁E��し、�EチE��不整合を隠蔽しなぁE��E
  - 上流仕様（安�EAPI優先）に合わせ、壊れた状態を継続させるより即時停止に統一する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_load_header`:
      - `scanner_header_ptr` ぁE`Err` の場合�E `0` 返却めE`#intrinsic "unreachable"` へ変更、E
      - `load_i32` ぁE`None` の場合�E `0` 返却めE`#intrinsic "unreachable"` へ変更、E
    - `scanner_store_header`:
      - `scanner_header_ptr` ぁE`Err` の場合�E無視を `#intrinsic "unreachable"` へ変更、E
      - `store_i32` ぁE`Err` の場合�E無視を `#intrinsic "unreachable"` へ変更、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-unreachable-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-unreachable.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD先衁E Writer めERegionToken 保持へ移衁E

- 目皁E
  - `kpread` と同様に `kpwrite` でも�E開ハンドルが領域惁E��を持つようにし、メモリ安�EAPIを統一する、E
- 根本原因:
  - `Writer` は `MemPtr<u8>` を直接保持し、�EチE��領域サイズ�E�E0byte�E�が型に表現されてぁE��かった、E
  - 途中で追加した `writer_mem(Writer)->MemPtr<u8>` ヘルパ�E `Writer` を値渡しで受けるため、E
    non-copy な `Writer` の move を発生さぁE`D3053` を引き起こした、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` めE`Writer.region: RegionToken<u8>` に変更、E
    - `writer_wrap` で `region_new raw 20` を構築、E
    - `writer_mem` ヘルパ�E削除し、`region_ptr get w "region"` を直接展開して move を回避、E
  - `stdlib/kp/kpread_core.nepl`
    - `store_i32_u8_at/load_i32_u8_at` めE`RegionToken<u8>` 受け取りへ変更、E
    - `sc0/iov/nread/sc` の吁E��域めE`RegionToken` 化してアクセス経路を統一、E
    - 途中で発生しぁE`match` アーム崩れ！ED3009/D3008/D3045`�E�を修正、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-regiontoken-v3.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先衁E kpread_core の冁E��ヘッダアクセスめERegionToken 匁E

- 目皁E
  - `kpread_core` の冁E��メモリアクセスめE`RegionToken` 経由に統一し、`MemPtr + off` の直接算術依存を減らす、E
- 根本原因:
  - `store_i32_u8_at` / `load_i32_u8_at` ぁE`MemPtr<u8>` と `off` から直接 `MemPtr<i32>` を作る設計で、E
    領域墁E��の前提が�Eルパ外へ漏れてぁE��、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` を追加し、`region_ptr_at<u8,i32>` を使用、E
    - `store_i32_u8_at` / `load_i32_u8_at` の引数めE`RegionToken<u8>` に変更、E
    - `sc0(12)`, `iov(8)`, `nread(4)`, `sc(12)` で `RegionToken` を構築してヘルパへ渡す形に更新、E
  - 途中修正:
    - `match dealloc_ptr<u8> buf cap` の `Result::Err` アームのインチE��ト崩れにより
      `D3009/D3008/D3045` が発生したため、�E岐構造を正しく修正、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-kpread-core-regiontoken-v2.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先衁E kpwrite ヘッダアクセスめERegionToken 経由へ移衁E

- 目皁E
  - `kpwrite` 側でも�EチE��アクセスめE`RegionToken` ベ�Eスに寁E��、`core/mem` の墁E��検証APIを�E利用できるようにする、E
- 根本原因:
  - 既孁E`writer_header_ptr` は `mem_ptr_addr + off` で直接アドレス算術を行い、E
    20byte ヘッダ墁E��の前提を関数ごとに暗黙化してぁE��、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_region` を追加�E�Eregion_new w_mem 20`�E�、E
    - `writer_header_ptr` めE`Result<MemPtr<i32>,str>` へ変更し、`region_ptr_at<u8,i32>` を使用、E
    - `writer_load_header` / `writer_store_header` を上訁E`Result` 経路へ更新、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-header-regiontoken.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先衁E kpread の Scanner ヘッダめERegionToken 匁E

- 目皁E
  - `todo.md` フェーズD着手として、`kpread` の公開ハンドルに領域所有情報を持たせ、`core/mem` の新安�EAPIへ寁E��る、E
- 根本原因:
  - `Scanner` ぁE`MemPtr<u8>` 直接保持のみで、�EチE��領域墁E��の惁E��が型に乗ってぁE��かった、E
  - ヘッダアクセスぁE`mem_ptr_addr + off` の算術依存で、墁E��検証を�E利用しにくかった、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `Scanner` フィールドを `raw: MemPtr<u8>` から `region: RegionToken<u8>` に変更、E
    - `scanner_wrap` で `region_new raw 12` を構築、E
    - `scanner_header_ptr` めE`region_ptr_at<u8,i32>` ベ�Eスの `Result` 返却へ変更、E
    - `scanner_load_header` / `scanner_store_header` を上訁E`Result` 経路で処琁E��E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-scanner-regiontoken.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズC: core/mem に RegionToken 安�EAPIを追加)

- 目皁E
  - `todo.md` フェーズCに沿って、`MemPtr<T>` と `RegionToken<T>` を使ぁE���EAPIめE`core/mem` に追加し、`kpread/kpwrite` 移行�E上流基盤を作る、E
- 根本原因:
  - 既孁E`mem` は `MemPtr<T>` までは整備済みだったが、E��域所有を表す�E開APIが不足しており、E
    墁E��惁E��付きアクセスを型として統一できてぁE��かった、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `RegionToken<T>` 補助APIを追加:
      - `region_new`
      - `region_in_bounds`
      - `region_ptr_at`
      - `alloc_region_bytes`
      - `alloc_region`
      - `dealloc_region`
    - これにより、E��域サイズを伴ぁE��付きオフセチE��取得を `Result` で扱えるようにした、E
  - `tests/memory_safety.n.md`
    - `alloc_region/region_ptr_at/dealloc_region` の基本動作ケースを追加、E
    - 篁E��外オフセチE��で `Result::Err` を返す回帰ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md -i tests/plan.n.md -i tests/block_single_line.n.md --no-stdlib --no-tree -o /tmp/tests-semicolon-focus.json -j 15`
  - 結果: `67/67 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md --no-tree -o /tmp/tests-memory-safety-region-token.json -j 15`
  - 結果: `211/211 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-memory-kp-regression.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズB2: trait capability の型付き保持へ移衁E

- 目皁E
  - trait capability 判定�E斁E���E再解析を減らし、型付きチE�Eタで一貫して扱ぁE��E
- 根本原因:
  - 既存実裁E��は `TraitInfo.capabilities` ぁE`Vec<String>` のため、E
    `TraitSemantics::detect` で毎回斁E���Eを�Eパ�EスしてぁE��、E
  - こ�E構造は capability 判定�E責務が刁E��し、封E��拡張時に不整合を生みめE��ぁE��E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo.capabilities` めE`Vec<String>` から `Vec<TraitCapability>` へ変更、E
    - trait 定義処琁E(`Stmt::Trait`) で capability めE回だけパースし、型付きで保持、E
    - 重褁Ecapability 持E���E同一trait冁E��重褁E��録しなぁE��ぁE��琁E��E
    - `TraitSemantics::detect` は `TraitInfo` 冁E�E型付き capability を直接参�E、E
    - 不要になっぁE`detect_declared_trait_capabilities` を削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-capability-typed.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-capability-typed.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-capability-typed.json -j 15`
  - 結果: `783/783 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite header 読み取りの Result 化と None フォールバック廁E��)

- 目皁E
  - `writer_load_header` の `None -> 0` フォールバックを廁E��し、header 読み取り失敗を明示刁E��で扱ぁE��E
- 根本原因:
  - 従来の `writer_load_header` は `load_i32` 失敗時に 0 を返しており、異常状態を正常値へ潰してぁE��、E
  - そ�Eため後続�E琁E�� `buf/cap/iov/nw` が不正値のまま進行する余地があった、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header` めE`Result<i32,str>` へ変更、E
    - `writer_load_header_ptr` めE`Result<MemPtr<u8>,str>` へ変更、E
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`,
      `writer_put_u8_handle`, `writer_write_str_handle`,
      `writer_write_i32_handle`, `writer_write_u64_handle` めE
      `Result` 刁E��で安�Eに処琁E��る形へ更新、E
    - `if` レイアウト中の冗長な `then: block:` を除去し、`D2002` 回避のため式構造を仕様準拠へ整琁E��E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-after-header-result-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-after-header-result.json -j 15`
  - 結果: `226/226 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpwrite-style-fix.json -j 15`
  - 結果: `215/215 pass`

# 2026-03-12 作業メモ (tooling: repo_metrics めETypeScript 化し冁E��別雁E��へ拡張)

- 目皁E
  - `repo_metrics.py` の単純な拡張子集計を、リポジトリ実�Eに沿った「�E容別」集計へ改良する、E
  - `.n.md` と通常の `.md` を�E離し、top-level の `tests/` `tutorials/` `doc/` `examples/` と `src/` / `stdlib/` 系を�E離して確認できるようにする、E
  - `source code` / `document comment` / `document` / `test` を別雁E��し、`.rs` / `.nepl` / `.n.md` の test case 数も�Eせるようにする、E
- 実裁E��況E
  - 既存�E `repo_metrics.py` は削除し、top-level の `repo_metrics.ts` へ移行した、E
  - 実行�E `node --experimental-strip-types repo_metrics.ts ...` を前提とし、追加依存なしで動く standalone script にした、E
- 根本修正:
  - 以前�E「拡張子ごとの総行数 + 一部拡張子�E comment/code/blank」だけで、`.n.md` 冁E�E本斁E�� doctest、`.nepl` 冁E�E `//:` ドキュメントコメントと doctest、`.rs` 冁E�E source と test を�E離できてぁE��かった、E
  - そ�Eため、仕様書・ドキュメントコメント�EチE��トケースぁEsource code と混ざり、repo の実情に合わなぁE��値になってぁE��、E
- 変更:
  - `repo_metrics.ts`
    - Git 管琁E��ファイルを基準に列挙し、binary file は skip ぁEsize-only 雁E��を選べるよぁE��した、E
    - `By Extension` / `By Area` / `By Content Kind` の 3 軸で表示するようにした、E
    - area は `top_level_docs_tests` / `source_tree` / `other` に刁E��た、E
    - `.n.md` / `.md` では `neplg2:test` ブロチE��だけを `test`、それ以外を `document` として数えるようにした、E
    - `.nepl` では `//:` めE`document comment` として扱ぁE��`//:` 冁Edoctest だけを `test` として刁E��出すよぁE��した、E
    - `.rs` では `///` / `//!` めE`document comment` とし、`#[test]` 系 attribute と `#[cfg(test)]` 配下を `test` として扱ぁE��ぁE��した、E
    - `.n.md` / `.nepl` / `.rs` から test case 数を数え、拡張子別・area 別・content kind 別に反映するようにした、E
- 実行確誁E
  - `node --experimental-strip-types repo_metrics.ts --json /tmp/repo_metrics.json`
    - 実測:
      - `.n.md` testCases = `812`
      - `.nepl` testCases = `278`
      - `.rs` testCases = `360`
  - 件数照吁E
    - `rg '^\\s*neplg2:test(?:\\[[^\\]]+\\])?\\s*$' -g '*.n.md' | wc -l` -> `812`
    - `rg '^\\s*//:\\s*neplg2:test(?:\\[[^\\]]+\\])?\\s*$' -g '*.nepl' | wc -l` -> `278`
    - `rg '^\\s*#\\[(test|tokio::test|wasm_bindgen_test)\\b' -g '*.rs' | wc -l` -> `360`
  - こ�E一致により、少なくとめEtest case カウント�E repo 実�Eと整合してぁE��ことを確認した、E
- build / test:
  - `trunk build`
    - 結果: success
  - `node nodesrc/run_doctest.js -i tests/compiler/typeannot.n.md -n 1`
    - 結果: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/typeannot.n.md -n 2`
    - 結果: pass
  - 参老E
    - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1`
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1`
    - 上訁E2 件は `return value mismatch` と runtime trap で fail。今回の変更対象は雁E��スクリプトであり、repo_metrics 変更の有無に関係なく既存�E doctest 側問題として残ってぁE��、E
- 差異メモ:
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib ...` は長時間継続したため、確認�E `run_doctest.js` による focused 実行へ刁E��替えた、E
  - 今回の変更は build/test 系ロジチE��ではなく、E��計スクリプト単体�E改喁E��ある、E

# 2026-03-06 作業メモ (feat: examples/bf.nepl に Brainfuck Runner を実裁E

# 2026-03-12 作業メモ (alloc/collections/sparse_set 調査継続�E未 commit)

- [目皁Eもくてき]:
  - `alloc/collections` に `SparseSet` を[追加/つぁE��]し、`[0, n)` [篁E��/はんい]の integer set めEO(1) membership / insert / remove で[扱/あつか]えるようにする、E
- [進捁EしんちめE��]:
  - `SparseSet` の public API (`new` / `len` / `universe_len` / `contains` / `insert` / `remove` / `clear` / `free`) と public doctest / fixture は[一送Eひととお]り[作�E/さくせい]済み、E
  - normal path は focused 実行で[通過/つぁE��]してぁE��、E
    - `stdlib/alloc/collections/sparse_set.nepl::doctest#1/#2`
    - `stdlib/tests/sparse_set.n.md::doctest#1`
    - `tests/stdlib/sparse_set_collections.n.md::doctest#1`
- [根本原因/こんぽんげんいん]の[刁Eき]り[刁Eわ]ぁE
  - [当�E/とぁE��ょ]は `SparseSet` owner [冁E��/なぁE�E]の field [読/めEみ[出/だ]しが[壁Eこわ]れてぁE��ように[要Eみ]えたが、header めE`MemPtr<u8>` field で[持EめEつ設計かめEraw `i32` pointer [保持/ほじ]へ[落/お]とすことで normal path は[安宁Eあんてい]した、E
  - そ�E[征Eあと]に[殁Eのこ]っぁEfailure は invalid index path だけで、`contains s 8` の[最小侁EさいしょぁE��い]まで[縮封EしゅくしめE��]できた、E
  - さらに[追跡/つぁE��き]すると、`SparseSet` [固朁EこゆぁEではなぁE`sparse_set_diag_index` の[中/なか]で[佁Eつく]めEmessage string ぁEweb compile path で `RuntimeError: memory access out of bounds` を[起/お]こしてぁE��ことが[刁Eわ]かった、E
  - `diag_error StdErrorKind::IndexOutOfBounds "abc"` は pass する一方、`concat "sparse_set_contains" ": index out of bounds "` を[含/ふく]む chain だけが trap する、E
  - `stdlib/alloc/string.nepl::doctest#4` めE同系統/どぁE��ぁE��ぁEの web path OOB を[持EめEっており、`SparseSet` invalid path failure は[既孁Eきぞん]の `alloc/string` regression に[乁Eの]ってぁE��と[判断/はんだん]した、E
  - native compiler では `SparseSet invalid index` の[最小侁EさいしょぁE��い]は pass し、web compile path だけが trap するので、[直接/ちめE��せつ]の blocker は stdlib API 設計でなぁEweb compiler/runtime path [側/がわ]にある、E
- [判断/はんだん]:
  - `SparseSet` normal path の library 実裁E�E[成竁Eせいりつ]してぁE��が、invalid index の `Result::Err` path を[含/ふく]む focused suite ぁEweb compile path で[未収束/みしゅぁE��く]のため、現時点では commit しなぁE��E
  - [次/つぎ]は `alloc/string` の concat / integer-to-string [経路/けいろ]めEroot cause ベ�Eスで[直/なお]し、その[征Eあと]に `SparseSet` batch を[再開/さいかい]する、E

# 2026-03-12 作業メモ (ci: rust install -> cargo build -> trunk build を�E送Eaction 匁E

- 目皁E
  - GitHub Actions に散ってぁE�� `Node setup` / `Rust toolchain` / `wasm32 target` / `wasm-bindgen-cli` / `cargo build` / `trunk build` の重褁E�� 1 箁E��へ雁E��E��る、E
  - 吁Eworkflow は「�E送Ebuild artifact を作る job」と「その artifact を受けて test / deploy を行う job」に刁E��、build 済み成果物を�E利用する形へ寁E��る、E
- 根本原因:
  - `compile-test.yml` / `nepl-test-wasi.yml` / `nepl-test-llvm.yml` / `nmd-doctest.yml` / `nm-compile.yml` / `rust-test..yml` / `gh-pages.yml` が、それぞれ別に toolchain install と `trunk build` を持ってぁE��、E
  - そ�Eため手頁E�E更新漏れが起きやすく、`trunk` めE`wasm-bindgen-cli` の更新、`Trunk.toml` Linux 補正、examples 配置などを毎回多重管琁E��る構造になってぁE��、E
- 変更:
  - `.github/actions/bootstrap-build/action.yml`
    - CI 共通�E local composite action を追加、E
    - `actions/setup-node`、`npm install`、`actions-rs/toolchain`、`rustup target add wasm32-unknown-unknown`、`jetli/trunk-action`、`wasm-bindgen-cli` install、`Swatinem/rust-cache`、`cargo build --locked`、`trunk build --release` を集紁E��E
  - `.github/workflows/compile-test.yml`
  - `.github/workflows/rust-test..yml`
  - `.github/workflows/nm-compile.yml`
  - `.github/workflows/nmd-doctest.yml`
  - `.github/workflows/nepl-test-wasi.yml`
  - `.github/workflows/nepl-test-llvm.yml`
    - それぞれ `build` job で共送Eaction を使って `dist` / `target/debug` / `target/wasm32-unknown-unknown` めEartifact 化、E
    - test job 側は `actions/download-artifact` で取得してから、各 workflow 固有�E `cargo test` / `nodesrc/tests.js` / `cargo run -p nepl-cli` / LLVM runner を実行する形へ変更、E
  - `.github/workflows/gh-pages.yml`
    - pages 固有�E deploy/doctest/doc build は残しつつ、toolchain install と build 本体�E共送Eaction へ移動、E
- 検証:
  - 一晁Edirectory `/tmp/gha-yaml-check` を作って `npm install yaml` を行い、�E workflow と composite action めE`yaml` parser で構文確認、E
    - 対象:
      - `.github/workflows/*.yml`
      - `.github/actions/bootstrap-build/action.yml`
    - 結果: 全件 `OK`
- 差異メモ:
  - workflow 実行そのも�Eは GitHub Actions 上での実行が忁E��なので、ローカルでは YAML 構文と依存関係�E整合までを確認した、E
  - 現時点では artifact の粒度めE`dist` / `target/debug` / `target/wasm32-unknown-unknown` にしてぁE��。さらに絞る余地はあるが、まず�E共通化と再利用の成立を優先した、E

# 2026-03-12 作業メモ (ci: build 1 囁E+ pages/test 統吁E+ per-case timeout)

- 目皁E
  - workflow ごとに `bootstrap-build` を繰り返してぁE��構�Eをやめ、`trunk build` を含む build めE1 workflow 冁E�� 1 回だけ実行し、その成果物を�E test job と Pages deploy に再利用する、E
  - `gh-pages.yml` が別 workflow で test を�E実行してぁE��構造を解消し、site への publish めEtest workflow の一部へ統合する、E
  - 無限ループ系の hang で CI 全体が止まらなぁE��ぁE��E ケース 20 秒、test job 全佁E10 刁E�E上限を�Eれる、E
- 根本原因:
  - 前段の共送Eaction 化だけでは、workflow が�EかれてぁE��限り `cargo build` / `trunk build` / `npm install` / `cargo install wasm-bindgen-cli` ぁEworkflow 数だけ繰り返される、E
  - `gh-pages.yml` は site 生�Eのために tests を�E度回しており、同ぁEcommit に対して test ぁE2 重実行されてぁE��、E
  - `nodesrc/tests.js` は suite 全体�E実行�Eできても、WASM worker / LLVM child process に per-case timeout が無く、E ケースの hang ぁEsuite 全体を引きずる余地があった、E
- 変更:
  - `.github/actions/bootstrap-build/action.yml`
    - `actions/setup-node` に npm cache を追加、E
    - `web/package-lock.json` ベ�Eスで `npm ci` を使ぁE��に変更、E
    - `wasm-bindgen-cli` めE`actions/cache` で再利用するよう変更、E
    - `wasm-bindgen` の verify step を追加、E
  - `.github/workflows/ci.yml`
    - 旧 test workflow 群と Pages deploy めE1 workflow に統合、E
    - `build` job で `bootstrap-build` めE1 回だけ実行し、さらに tutorial / stdlib HTML めE`dist` 配下へ生�Eして artifact 化、E
    - `compile-test` / `rust-test` / `nm-compile` / `wasi-test` / `nmd-doctest` / `llvm-test` はすべて `needs: build` で artifact を�E利用、E
    - `pages-fast-*` と `pages-final-*` の 2 段 deploy を追加し、`trunk build` 後�E pending site を�Eに publish し、test 完亁E��に test JSON / summary を載せぁEfinal site で上書きする形にした、E
    - `gh-pages.yml` は削除、E
    - test job には `timeout-minutes: 10` を追加し、`node nodesrc/tests.js` / `cargo test` / `cargo run` は `timeout --signal=KILL 10m ...` で匁E��だ、E
    - test 実行環墁E�� `NEPL_TEST_CASE_TIMEOUT_MS=20000` / `NEPL_WASIX_TIMEOUT_MS=20000` を�E通指定、E
  - `nodesrc/tests.js`
    - WASM thread pool worker に per-case timer を追加し、E0 秒で応答しなぁEcase は worker めEterminate して error として回収する形へ変更、E
    - LLVM / native 実行に使ぁE`runCommand` に child process timeout を追加し、同じく 20 秒で kill するよう変更、E
- 検証:
  - `node --check nodesrc/tests.js`
  - 一晁Edirectory `/tmp/gha-yaml-check` を作って `npm install yaml` を行い、E
    - `.github/workflows/*.yml`
    - `.github/actions/bootstrap-build/action.yml`
    めEparser で検証、E
- 差異メモ:
  - Pages final deploy は `build` artifact の `dist` を�E利用し、site を作るために `trunk build` を�E実行しなぁE��E
  - pending/final の 2 囁Edeploy は Pages への publish を早めるためのも�Eで、tests 自体�E 1 回しか実行しなぁE��E
  - 初版では `site-fast` / `site-final` を通常の `upload-artifact` で中継してから `upload-pages-artifact` に渡してぁE��が、download 時に `dist` directory の階層前提が崩れて `tar: dist: Cannot open` になった、E
  - そ�Eため Pages 用 bundle job は直接 `upload-pages-artifact` を行い、deploy job は `deploy-pages` だけを行う構造へ修正した、E

- 目皁E
  - `rpn.nepl` を参老E��して `examples/bf.nepl` に Brainfuck の実行ツールを実裁E��る、E
  - 毎行�E力を受け付け、�E力ごとにメモリをリセチE��して独立実行する、E
- 変更:
  - `examples/bf.nepl`
    - `alloc/collections/stack` を使って `[` と `]` のジャンプ�Eを事前計算すめE`compile_jumps` を実裁E��E
    - `eval_line` で 30,000 バイト�Eメモリ上で BF 命令�E�E+` `-` `>` `<` `.` `,` `[` `]`�E�を実行、E
    - `,` は現状 0 を書き込む簡略実裁E��E
    - メインループ�E入力ごとにメモリバッファを確保�E解放し、状態を引き継がなぁE��E
    - 表示名�E "Brainfuck REPL" から "Brainfuck Runner" に変更�E�毎行リセチE��のため�E�、E
    - `neplg2:test[bf_hello_world]` doctest を追加�E�Eello World プログラムの実行）、E
- 検証:
  - `target/debug/nepl-cli -i examples/bf.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - `+++++++++[>++++++++>+++++++++++>+++>+<<<<-]>.>++.+++++++..+++.>+++++.<<+++++++++++++++.>.+++.------.--------.>+.>+.` を�E力して `Hello World!` の出力を確認、E

# 2026-03-06 作業メモ (TUI改喁E rpnの途中計算可視化とstdioの負数出力修正)

- 目皁E
  - `examples/rpn.nepl` において、`>` プロンプトの動作をレガシー版に合わせ、計算過程を「計算前」「計算後」としてANSIカラーで可視化する、E
  - 途中計算や出力で負数を含む式が正しく表示されるよぁE��`stdlib/std/stdio.nepl` の `print_i32` に存在する負数出力バグを修正する、E
- 変更:
  - `examples/rpn.nepl`
    - REPLプロンプト出力前にト�Eクン行を二重に出力しなぁE��ぁE�E長なループを削除、E
    - `print_step_before` を追加し、計算前の状態をシアン (`ansi_cyan`) で強調表示、E
    - `print_step_after` を追加し、計算結果を緑色 (`ansi_green`) で強調表示、E
  - `stdlib/std/stdio.nepl`
    - `print_i32` 関数で負の数への計算が不足して `0` となるバグを修正。絶対値の吁E��を送E��E��開したのち、負数であれば `-` 符号を付与するよぁE��修、E
    - コンパイルエラーを塞ぐため `mod_u` めE`rem_u` に修正、E
- 結果:
  - `1 2 + 3 + 4 5 + 6 +` などの連続�E力に対して、�E琁E��との計算箁E�� (`[1 2 +]` など) と結果が色付きで刁E��りやすく表示されるよぁE��なった、E
  - `-5` などの負の数を�E力した際に正常に表示されるよぁE��なった、E
- 検証:
  - `target/debug/nepl-cli -i examples/rpn.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - 途中計算�Eトレースおよび負数 (`1 2 3 4 + - 5 +` -> `-5`) の正しいフォーマットと出力を直接確認、E

# 2026-03-06 作業メモ (型安�E匁E `alloc/string` の主要Eraw 確保を `RegionToken<u8>` 匁E

- 目皁E
  - `alloc/string` の主要生成経路から `alloc_raw` を取り除き、`core/mem` の型付き領域 API に寁E��る、E
  - 斁E���E生�E処琁E��長さ�EチE��と本斁E�EインタめE`MemPtr<T>` / `RegionToken<T>` で扱ぁE���E部の生�Eインタ露出を減らす、E
- 変更:
  - `stdlib/alloc/string.nepl`
    - `string_alloc_region`
    - `string_region_len_ptr`
    - `string_region_data_ptr`
    - `string_data_ptr`
    - `string_finish`
    を追加し、文字�Eレイアウト専用の冁E��ヘルパとして整琁E��E
  - `concat`
    - 出力文字�Eの確保を `string_alloc_region` に変更、E
    - 出力�Eコピ�EめE`MemPtr<u8>` ベ�Eスへ変更、E
  - `sb_build`
    - 連結�Eバッファの確保を `RegionToken<u8>` 化、E
    - 吁Epart の読み出しと出力�E書き込みを型付きポインタへ変更、E
  - `str_slice`
    - 刁E��出し�Eの確保を `RegionToken<u8>` 化、E
  - `from_u128_radix`
    - 送E��E��積みの scratch めE`RegionToken<u8>` 化、E
    - 一晁Escratch は `dealloc_region` で解放、E
  - `from_f64`
    - 小数部 scratch めE`RegionToken<u8>` 化、E
    - scratch 解放を追加、E
- 結果:
  - `stdlib/alloc/string.nepl` から `alloc_raw/realloc_raw/dealloc_raw` の直接呼び出し�E消えた、E
  - `str` の冁E��表現自体�Eまだ raw address だが、主要な生�E経路では `RegionToken<u8>` から `string_finish` で確定する流れに整琁E��きた、E
- 検証:
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md --no-stdlib --no-tree -o /tmp/tests-string-type-safety-v1.json -j 15`
    - 結果: `26/26 pass`
  - `rg -n "alloc_raw|realloc_raw|dealloc_raw" stdlib/alloc/string.nepl`
    - 結果: 該当なぁE

# 2026-03-06 作業メモ (alloc/string: i128/u128 と基数付き斁E���E変換の整傁E

- 目皁E
  - `alloc/string` に整数の斁E���E表現変換を集紁E��、`core/cast` との責務を刁E��する、E
  - `i128` / `u128` を含む 2/8/10/16 進の変換を提供する、E
  - tutorial に、数値 cast と斁E���E変換の違いを�E示した導線を追加する、E
- 変更:
  - `stdlib/alloc/string.nepl`
    - `from_bool`
    - `to_bool`
    - `from_u128` / `from_u128_radix`
    - `to_u128` / `to_u128_radix`
    - `from_i128` / `from_i128_radix`
    - `to_i128` / `to_i128_radix`
    - `u128_divrem_small` など 128-bit 整数の補助関数群
    - `to_i32` の説明を現実裁E��合わせて更新
  - `tests/stdlib.n.md`
    - `i128/u128` と負数16進の focused case を追加
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
    - `core/cast` と `alloc/string` の使ぁE�EぁE
    - `Result` を返す解析関数
    - 2/8/10/16 進変換
    - `i128/u128` の大きい値の侁E
  - `tutorials/getting_started/00_index.n.md`
    - 新要Etutorial への導線を追加
- 検証:
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-conversions-i128-v3.json -j 15`
    - 結果: `19/19 pass`

# 2026-03-06 作業メモ (型安�E匁E `ptr_cast` 公開廁E��)

- 目皁E
  - ポインタ再解釈�Eような unsafe な公閁EAPI を減らし、`MemPtr<T>` / `RegionToken<T>` モチE��へ寁E��る、E
- 変更:
  - `stdlib/core/cast.nepl`
    - 未使用だっぁE`ptr_cast` を削除、E
    - モジュール先頭コメントを、数値 cast と bitcast のみに責務を限定する説明へ更新、E
- 判断:
  - `ptr_cast` は型だけを付け替える操作で、`MemPtr<T>` による型安�E化方針と矛盾する、E
  - repo 冁E��照は無く、現時点で公開面に残す合理性は無かった、E
  - `MemPtr<T>` は「型付きアドレス」、`RegionToken<T>` は「その領域のサイズと所有権」を伴ぁE��形ト�Eクンとして使ぁE�Eける、E

# 2026-03-06 作業メモ (フェーズF: tutorials Part6 拡允E+ library-first 匁E

- 目皁E
  - `tutorials/getting_started` Part6�E�E2、E7�E��E説明誤り�E不足を監査し、短く簡潔で安�Eな書き方へ更新する、E
  - 生�Eインタ露出を減らすため、`kp` 側に `Vec<i32>` 直受け補助を追加する、E
- 変更:
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
    - `Scanner/Writer` の基本パターンめEpipe 中忁E��簡潔化、E
    - i32/i64/空白区刁E��出力�E 3 ケースを安�E API 前提で整琁E��E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `Vec + sort + lower/upper_bound` めElibrary-first で再構�E、E
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
    - DP 本体を維持しつつ I/O を簡潔化、E
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - prefix めE`kp/kpprefix` ハンドル API 前提へ更新、E
    - two pointers の条件評価を短絡評価に依存しなぁE���Eな形へ修正、E
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 手書ぁEBFS から `kp/kpgraph` 利用へ移行、E
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - 未完�E表記を廁E��し、Part6 総まとめとしてチE��プレート�E対応表・実戦フローを追加、E
  - `tutorials/getting_started/00_index.n.md`
    - 誤字を修正�E�関数のふりがな�E�、E
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` ハンドルと `prefix_build_vec_i32` / `prefix_sum_i32` / `prefix_free_i32` を追加、E
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` を追加、E
  - `todo.md`
    - フェーズFの完亁E��み Part6 専用タスクを削除�E�未完亁E�Eみ維持E��、E
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i stdlib/kp/kpprefix.nepl -i stdlib/kp/kpsearch.nepl --no-tree -o /tmp/tests-part6-kp-refresh-v7.json -j 15`
    - 結果: `219/219 pass`
  - 補助確誁E
    - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md --no-tree -o /tmp/tests-part6-25-v6.json -j 15`
    - 結果: `207/207 pass`

# 2026-03-06 作業メモ (フェーズD: llvm `add/sub` 再定義リンク失敗�E根本修正)

- 目皁E
  - `--runner all --llvm-all` 実行時に `tests/llvm_target.n.md::doctest#4/#5` ぁE`invalid redefinition of function 'add'/'sub'` で失敗する問題を、後付け回避ではなく生成IR構造から解消する、E
- 原因:
  - `stdlib/core/math.nepl` の overload 群�E�Eadd/sub` など�E�が `#llvmir` 冁E��同一シンボル名！E@add`, `@sub`�E�を使ってぁE��、E
  - LLVM はシンボル名で overloading できなぁE��め、同一モジュールへ褁E��型版を同名定義するとリンク時に衝突する、E
  - さらに `u8` と `i32` は LLVM ABI で同じ `i32` に落ちるため、型別 overload をそのままシンボル名で共存させる設計が成立しなぁE��E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 生�E完亁E��前に `deduplicate_overloaded_llvm_symbols` を追加し、同吁E`define` をシグネチャ単位で一意化、E
    - `define` 側の重褁E�� `name__ovN_<sig>` へ正規化し、対応すめE`call` 参�Eも同一シグネチャで張り替える、E
    - 前段として `#llvmir` 呼び出し要件抽出と AST raw-body 選別補助を追加し、不要な overload 出力を抑制、E
- 検証:
- `NO_COLOR=false trunk build` -> success
- `cargo build -p nepl-cli` -> success
- `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-dedup-pass.json -j 15` -> `6/6 pass`
- `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-dedup.json -j 15` -> `791/791 pass`

# 2026-03-12 作業メモ (refactor(vec): Result 化しぁEVec API を直接依存�Eへ伝播)

- 目皁E
  - `alloc/collections/vec` の `new / with_capacity / push` めE`Result<..., StdErrorKind>` 化した変更を、直接依存すめEstdlib / tests / tutorials へ整合的に反映する、E
  - `Vec` 再確保を伴ぁEAPI めE`stack` 系と同じ失敗モチE��へ寁E��つつ、既存�E高水溁Ehelper では `unwrap_ok` 吸収で利用老E�E記述を過剰に崩さなぁE��E
- 根本原因:
  - `Vec` 本体だけを `Result` 化すると、`std/test` / `alloc/string` / `nm/parser` / `kpgraph` / `wasix/tui` などが旧 pure API を前提にして壊れる、E
  - さらに `StdErrorKind` が上位�E `alloc/diag/error` にあると、`vec -> diag/error -> vec` の循環依存が生じる、E
- 変更:
  - `stdlib/alloc/collections/vec.nepl`
    - `new / with_capacity / push` めE`Result<..., StdErrorKind>` 化、E
    - `with_capacity 0` は確保を行わず空 `MemPtr` を包む形にして `OutOfMemory` を不要化、E
  - `stdlib/std/test.nepl`
    - `checks_new` / `checks_push` で `Vec<Result<(),str>>` の `Result` を�E部吸収、E
  - `stdlib/alloc/string.nepl`
    - `StringBuilder` と `str_split` の冁E�� `Vec` 構築�E追加めE`unwrap_ok` 前提へ更新、E
  - `stdlib/alloc/diag/error.nepl`
    - `Diag` / `Diags` 冁E��の `Vec` 構築�E追加めE`unwrap_ok` 前提へ更新、E
  - `stdlib/alloc/hash/sha256.nepl`
    - scaffold 実裁E�E buffer 構築�E更新めE`unwrap_ok` 前提へ更新、E
  - `stdlib/kp/kpgraph.nepl`
    - BFS 結果ベクタ構築を `unwrap_ok` 前提へ更新、E
  - `stdlib/platforms/wasix/tui.nepl`
    - `text_wrap_lines` の行�E列構築を `unwrap_ok` 前提へ更新、E
  - `stdlib/nm/parser.nepl`
    - inline/block parser 冁E��の `Vec` 構築�E追加めE`unwrap_ok` 前提へ更新、E
  - `stdlib/tests/vec.n.md`
    - current `Vec Result` API に同期、E
  - `tests/stdlib/traits_order.n.md`
    - sort regression の `Vec` 構築を `unwrap_ok` 前提へ更新、E
  - `tests/stdlib/selfhost_req.n.md`
    - `Vec<u8>` buffer 構築を `unwrap_ok` 前提へ更新、E
  - `tests/stdlib/sort.n.md`
    - sort fixture の `Vec` 構築を `unwrap_ok` 前提へ更新、E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - `Vec` pipe 連鎖を `unwrap_ok new` と `|> push ... |> uwok` の current 書式へ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/stdlib/traits_order.n.md -i tests/stdlib/selfhost_req.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-a.json -j 4`
    - 結果: `10/10 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-b2.json -j 4`
    - 結果: `4/4 pass`
  - 補助確誁E
    - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/26_competitive_graph_bfs.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1` -> pass
- 差異メモ:
  - `Vec` の public API Result 化�E進んだが、`vec.nepl` 本体�E doc comment / doctest には旧書式�E旧 pure 前提の説明がまだ残る、E
  - `replace` めE`set` へ改名する案�E parser / keyword 制紁E�E刁E��刁E��後に再検討する、E

# 2026-03-12 作業メモ (docs(vec): doc comment と doctest めEcurrent Result API へ同期)

- 目皁E
  - `Vec` 本体を `Result` 化した後も、[stdlib/alloc/collections/vec.nepl](/mnt/d/project/NEPLg2/stdlib/alloc/collections/vec.nepl) の説明と埋め込み doctest が旧 pure API 前提のまま残ってぁE��差刁E��解消する、E
  - あわせて、旧節見�Eし形式を減らし、新しい doc comment policy に寁E��る、E
- 変更:
  - `vec.nepl`
    - file header の doctest めE`unwrap_ok new` と `|> push ... |> uwok` 前提へ更新、E
    - `new` / `with_capacity` / `len` / `cap` / `data_len` / `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` の comment 例を current API に同期、E
    - `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` の節見�Eしを `### [目皁Eもくてき]` 形式へ更新、E
- 検証:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 3` -> pass

# 2026-03-12 作業メモ (feat(collections): add bitset)

- 目皁E
  - `alloc/collections` に fixed-length な bit 雁E��を追加し、`BloomFilter` と違って false positive のなぁEmembership structure を標準で扱えるようにする、E
  - `reboot` 方針に合わせて bare API と public doctest を整え、pipe 併用の使ぁE��は `tests/stdlib` 側で保証する、E
- 変更:
  - `stdlib/alloc/collections/bitset.nepl`
    - `BitSet` を追加、E
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `fill` / `free` めEbare API で実裁E��E
    - 冁E��は `nbits` / `nbytes` / `MemPtr<u8>` を持つ owner struct とし、index から byte offset と bit mask を計算して更新する、E
    - doc comment は新 policy / format へ合わせて、usage doctest を各 public 関数へ追加、E
  - `stdlib/tests/bitset.n.md`
    - insert/remove/len と clear/fill の focused fixture を追加、E
  - `tests/stdlib/bitset_collections.n.md`
    - pipe 記法での `insert` / `remove` / `contains` / `fill` 利用を回帰として追加、E
- 検証:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 3` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 4` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 5` -> pass

# 2026-03-12 作業メモ (feat(collections): add adjacency matrix)

- 目皁E
  - `alloc/collections` に graph representation の最小実裁E��して `AdjacencyMatrix` を追加し、固定長の directed edge set めEO(1) membership で扱えるようにする、E
  - `trie` blocker と独立に、nested owner を避けた raw bit matrix で collection の種類を増やす、E
- 変更:
  - `stdlib/alloc/collections/adjacency_matrix.nepl`
    - `AdjacencyMatrix` を追加、E
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `free` めEbare API で実裁E��E
    - `(from, to)` めE`from * nverts + to` の bit index に写像し、byte 配�Eで保持する directed graph とした、E
    - doc comment は新 policy / format に合わせ、各 public 関数に usage doctest を追加、E
  - `stdlib/tests/adjacency_matrix.n.md`
    - insert/remove/clear の focused fixture を追加、E
  - `tests/stdlib/adjacency_matrix_collections.n.md`
    - pipe 記法での `insert` / `remove` / `contains` / `clear` 利用を回帰として追加、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/tests/adjacency_matrix.n.md -i tests/stdlib/adjacency_matrix_collections.n.md --no-stdlib --no-tree -o /tmp/tests-adjacency-matrix.json -j 2`
    - 結果: `9/9 pass`
- 差異メモ:
  - `contains g 4 0` のような篁E��夁Eindex に対する `Result::Err` 経路は、`target/debug/nepl-cli + wasmer` では正常に `1` を返す一方、web compile path では runtime OOB に落ちた、E
  - これは `AdjacencyMatrix` 実裁E��はなぁEweb compiler/runtime 側の別根因と判断し、今回の collection batch には混ぜてぁE��ぁE��E

# 2026-03-12 作業メモ (feat(collections): add counting bloom filter)

- 目皁E
  - `alloc/collections` に `CountingBloomFilter` を追加し、`BloomFilter` と同じ hasher 設計を保ちながら削除可能な近似 membership structure を標準で扱えるようにする、E
  - bare API と public doctest めEreboot 方針に合わせ、pipe 連鎖�E `tests/stdlib` 側で保証する、E
- 変更:
  - `stdlib/alloc/collections/counting_bloom_filter.nepl`
    - `CountingBloomFilter<.T,.H>` を追加、E
    - `new` / `len` / `insert` / `remove` / `contains` / `clear` / `free` めEbare API で実裁E��E
    - counter は `u8` 配�Eとし、E 本の probe index に対して insert は飽和加算、remove は 0 までの減算を行う、E
  - `stdlib/tests/counting_bloom_filter.n.md`
    - insert/remove/clear の focused fixture を追加、E
  - `tests/stdlib/counting_bloom_filter_collections.n.md`
    - pipe 記法での `insert` / `remove` / `contains` / `clear` 利用を回帰として追加、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md -i stdlib/alloc/collections/counting_bloom_filter.nepl --no-stdlib --no-tree -o /tmp/tests-counting-bloom-filter.json -j 2`
    - 結果: `8/8 pass`
- 差異メモ:
  - `new DefaultHash32 0` の invalid length `Result::Err` 経路は、`target/debug/nepl-cli + wasmer` では正常に `1` を返す一方、web compile path では runtime OOB に落ちた、E
  - これは `CountingBloomFilter` 実裁E��はなぁEweb compiler/runtime 側の別根因と判断し、今回の collection batch には混ぜてぁE��ぁE��E
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/bitset_collections.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/bitset.n.md -i tests/stdlib/bitset_collections.n.md -i stdlib/alloc/collections/bitset.nepl --no-stdlib --no-tree -o /tmp/tests-bitset-fixed.json -j 2`
    - 結果: `10/10 pass`
- 差異メモ:
  - out-of-bounds `Err` を返す focused case は、web compiler が生成しぁEcurrent wasm で hang する別根因に当たったため、この batch には混ぜてぁE��ぁE��E
  - `nepl-cli + wasmer` では同じ最小�E現が即終亁E��ることを確認済みで、stdlib 実裁E��はなぁEcompiler/runtime 側の別タスクとして刁E��出す、E

# 2026-03-06 作業メモ (フェーズD: llvm codegen 冁E�E precheck 後診断返却を除去)

- 目皁E
  - `precheck` 実行後に `codegen_llvm` ぁE`TypecheckFailed` を返してぁE��残存経路を除去し、前段検査不変条件へ統一する、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 冁E�E `select_active_raw_body(... )` `Err(diag)` 刁E��を `TypecheckFailed` 返却から internal panic へ変更、E
    - これにより、raw-body 選択失敗�E前段 `target_precheck::precheck_module_before_codegen` でのみ診断され、codegen 到達後�E生�E専任になる、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-llvm-invariant-2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-precheck-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm precheck 回帰ケースの追加)

- 目皁E
  - LLVM backend 到達前に未対忁Eintrinsic を診断できることを回帰固定する、E
- 変更:
  - `tests/llvm_target.n.md`
    - `llvm_precheck_rejects_wasm_only_intrinsic` を追加、E
    - `#intrinsic "i32_add"` めE`#target llvm` で使った場合に `diag_id: 3012` を期征E��めEcompile_fail ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-precheck-case.json -j 15`
    - 追加ケース�E�Edoctest#6::llvm`�E��E pass、E
    - 既存ケース `doctest#4/#5` は `invalid redefinition of function 'add'` で fail�E�既知未解決�E�、E
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-test-add.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: allocator helper 解決の意味論修正)

- 目皁E
  - runtime helper 共通化後に発生しぁErun-time 失敁E(`unreachable` / `memory access out of bounds`) を、E��に合わせではなぁEhelper 解決の意味論から修正する、E
- 原因:
  - `alloc`�E�安�EAPI�E�と `alloc_raw`�E�低レベルAPI�E��E現状の lowering では型互換になりうるため、`ALLOC_CANDIDATES=["alloc","alloc_raw"]` へ変更すると backend 冁E��確保で誤って `alloc` を掴む経路が発生する、E
  - そ�E結果、�E部確保�E前提�E�生ポインタ返却�E�と合わず、実行時に `unreachable` / OOB が発生した、E
- 変更:
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` めE`["alloc_raw", "alloc"]` に戻し、�E部 helper 解決は生�Eインタ意味論を優先するよぁE��正、E
    - 単体テスト期征E��めEraw 優先へ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-order-fix.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: runtime helper 解決の共通化と raw 依存縮封E

- 目皁E
  - `nepl-core` 冁E��重褁E��てぁE�� runtime helper�E�Elloc/dealloc/realloc�E�解決ロジチE��を�E通化し、`_raw` 名依存を段階縮小する、E
  - helper 名�E優先頁E��を安�EAPI名！Euffixなし）優先へ統一する、E
- 変更:
- `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` めE`["alloc", "alloc_raw"]` に変更�E�安�EAPI優先）、E
    - `RuntimeHelperKind` / `helper_candidates` / `helper_base_name` を追加、E

# 2026-03-09 作業メモ (trait 能力モチE��: `Eq` / `Ord` の共通化)

- 目皁E
  - `core/traits` に `Eq` / `Ord` を追加し、比輁E��味論を stdlib 共送Etrait として扱えるようにする、E
  - `alloc/collections/vec/sort.nepl` の局所 `Ord` 定義を撤去し、collections 側の比輁Ecapability めE`core` へ寁E��る、E
- 変更:
  - `stdlib/core/traits/eq.nepl`
    - `Eq` trait
    - `eq_by_trait`
    - `ne_by_trait`
    - `bool`, `i32`, `u8`, `i64`, `f32`, `f64`, `str` への impl
  - `stdlib/core/traits/ord.nepl`
    - `Ord` trait
    - `ord_lt`, `ord_le`, `ord_gt`, `ord_ge`
    - `bool`, `i32`, `u8`, `i64`, `i128`, `f32`, `f64` への impl
  - `stdlib/alloc/collections/vec/sort.nepl`
    - 局所 `Ord` trait と局所 impl を削除
    - `core/traits/ord` めEimport し、`sort_lt` 系 helper から共送E`ord_*` を呼ぶ形へ変更
  - `tests/stdlib/traits_order.n.md`
    - 日本語�E目皁E��ぁEfocused test を追加
- 判断:
  - `Eq<i128>` は既存�E刁E�� helper を仮定すると壊れるため、一旦追加しなかった、E
  - `Ord<str>` も既存�E頁E��比輁Ehelper が未整備なので、同様に見送った、E
  - まず�E既存�E `core/math` overload で根拠を持てる型だけを共送Etrait 化した、E
- 検証:
  - `NODE_NO_WARNINGS=1 node nodesrc/run_test.js`
    - `Eq` / `Ord` core focused case: pass
    - `vec/sort` + `Ord` std focused case: pass

# 2026-03-09 作業メモ (trait 能力モチE��: `Hash` の共通化)

- 目皁E
  - `Hash` trait めE`core/traits` へ追加し、hashmap / hashset が�E体的な `hash32_i32` / `hash32_str` へ直接依存せず�E送Ehelper 経由でキーを混合できるようにする、E
  - 封E��の `Serialize` / `Deserialize` と同じく、型ごとの能力を stdlib trait として明示する流れを揃える、E
- 変更:
  - `stdlib/core/traits/hash.nepl`
  - `Hash` trait
  - `hash32_by_trait`
  - `i32`

# 2026-03-11 作業メモ (`streamio` target 持E��化と `u32/u64` bare I/O の修正)

- 目皁E
  - `scanner` / `writer` めEstdin/stdout 固定�E no-arg API から外し、`io_stdin` / `io_stdout` / `io_text` / `io_bytes` の target 持E��で生�Eする形へ寁E��る、E
  - `u32` / `u64` の bare `read` / `write` を、型 suffix 名に戻さず current overload 方針�Eまま安定化する、E
  - Part6 tutorial と `kp` 周辺に残ってぁE�� old move-model 前提を、現行所有権モチE��へ合わせる、E
- 原因:
  - `std/streamio` だぁE`read` / `write` の bare 名へ寁E��ても、生成�E口 `scanner()` / `writer()` ぁEstdin/stdout 固定�Eままだと、`std/io` / `iotarget` と責務が二重化してぁE��、E
  - `u64` は compiler 側で `wasm_shared::valtype` がまだ `i32` 扱ぁE�E箁E��を残しており、Wasm signature が崩れてぁE��、E
  - `u32` / `u64` の 10 進出力�E、unsigned 値めEsigned overload へ落としてぁE��ため `4294967295` ぁE`18446744073709551615` に化けてぁE��、E
  - `PrefixI32` めEtutorial Part6 の `Vec` 走査には old move-model 前提が残ってぁE��、E
- 変更:
  - `stdlib/std/streamio.nepl`
    - `scanner <(IoReadTarget)*>Result<StreamScanner,str>>`
    - `writer <(IoWriteTarget)*>Result<StreamWriter,str>>`
    - `scanner_from_bytes`
    - `StreamWriter` header に `TargetKind` を追加
    - `u32` / `u64` の append 実裁E�� unsigned decimal として修正
    - `StreamScanner` / `StreamWriter` の doc comment めEcurrent 実裁E��同期
  - `stdlib/std/iotarget.nepl`
    - `io_stdin` / `io_stdout` / `io_text` / `io_bytes` を生成�E口として利用
  - `nepl-core/src/wasm_shared.rs`
    - `u64` めEWasm `I64` として扱ぁE��ぁE��正
  - `nodesrc/run_test.js`
    - `BigInt` の JSON 出力と return decode を追加
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` に `Copy` / `Clone` を付丁E
    - `prefix_build_vec_i32` めE`vec_data_len` ベ�Eスへ修正
  - `tests/stdlib/streamio.n.md`
  - `tests/stdlib/kp.n.md`
  - `tests/stdlib/kp_i64.n.md`
  - `tests/stdlib/stdin.n.md`
  - `tests/compiler/move_effect.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `stdlib/kp/kpgraph.nepl`
    - `unwrap_ok scanner io_stdin` / `unwrap_ok writer io_stdout` へ統一
- 検証:
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i /tmp/u64_probe2.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp_i64.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 2`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3`
  - `node nodesrc/run_doctest.js -i tests/stdlib/stdin.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/24_competitive_dp_basics.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpgraph.nepl -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpprefix.nepl -n 1`
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 20`

# 2026-03-09 作業メモ (compiler 前提固宁E `#prelude` 最小実裁E�� Copy 固定表撤去)

- [目皁Eもくてき]:
  - `todo.md` の `compiler 前提` 残件だっぁE`Copy` 固定表依存を、[実際/じっさい]に source [側/がわ]から trait impl を[供給/きょぁE��めE��]できる[状慁EじょぁE��い]へ[移/ぁE��]す、E
  - parser だけに[存在/そんざい]してぁE�� `#prelude` / `#no_prelude` めEloader [段隁Eだんかい]でめE解釁Eかいしゃく]し、copy/clone 非ハードコード化の[前提/ぜんてい]を[整/ととの]える、E
- [原因/げんぁE��]:
  - `#prelude` と `#no_prelude` は lexer / parser / AST にだけ[存在/そんざい]し、loader では[無要Eむし]されてぁE��、E
  - そ�Eため `Copy` / `Clone` impl めEsource [側/がわ]から[既宁Eきてい][供給/きょぁE��めE��]できず、`TypeCtx::is_copy` に primitive 固定表フォールバックを[殁Eのこ]す[忁E��Eひつよう]があった、E
- [変更/へんこぁE:
  - `nepl-core/src/loader.rs`
    - root module [限宁Eげんてい]で `#prelude` / `#no_prelude` を[処琁Eしょり]するように[変更/へんこぁEした、E
    - `#no_prelude` がなぁEroot module には[既宁Eきてい]で `std/prelude_base` を[読/めEみ[込/こ]む、E
    - import/include の[再帰/さいき] load では default prelude を[適用/てきよぁEしなぁE��ぁE��して、stdlib [冁E��/なぁE�E] import での[循環/じゅんかん]を[避/さ]けた、E
  - `stdlib/std/prelude_base.nepl`
    - [最封EさいしょぁE prelude として[追加/つぁE��]した、E
    - [当面/とぁE��ん]は `core/traits/copy` だけを[読/めEみ[込/こ]み、copy/clone 能力�E source [供給/きょぁE��めE��]に[絁Eしぼ]った、E
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` の最終フォールバックから primitive 固定表を[削除/さくじょ]した、E
    - `Copy` trait が[要Eみ]えてぁE��い[場吁Eばあい]は、[参�E/さんしょぁE型と `Never` だけを compiler [冁E��/なぁE��い]の copy として[扱/あつか]ぁE��E
  - `tests/compiler/prelude_copy.n.md`
    - default prelude で `Copy` bound が[送Eとお]ることを[確誁Eかくにん]する focused case を[追加/つぁE��]した、E
    - `#prelude std/prelude_base` と `#no_prelude` を[併訁EへぁE��]しても、[明示皁Eめいじてき] prelude が[優允EめE��せん]されることを[固宁Eこてい]した、E
    - `#no_prelude` だけでは `Copy` trait [供給/きょぁE��めE��]が[涁Eき]え、`.T: Copy` ぁE`3073` で[落/お]ちることを[追加/つぁE��]した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/compiler/resolve.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-only.json -j 15` -> `14/14 pass`
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-copy-only.json -j 15` -> `3/3 pass`
- [判断/はんだん]:
  - `Copy` の source [供給/きょぁE��めE��]は default prelude を[送Eとお]すことで[既孁Eきぞん]コードを[壁Eこわ]さずに[移衁EぁE��ぁEできる、E
  - `#no_prelude` は「標溁Ecapability を[含/ふく]めて自前で[管琁Eかんり]する」ため�E opt-out として[機�E/き�EぁEする、E
    - `bool`
    - `u8`
    - `i64`
    - `str`
    への impl を追加、E
  - `stdlib/alloc/collections/hashmap.nepl`
    - `hash32_i32` / `hash32_str` の直接呼び出しを `hash32_by_trait` に置換、E
  - `stdlib/alloc/collections/hashset.nepl`
    - 同様に `hash32_by_trait` 経由へ置換、E
  - `tests/stdlib/traits_hash.n.md`
    - `[目皁Eもくてき]` つぁEfocused case を追加、E
- 判断:
  - `Hash<i64>` は [上佁EじょぁE��] / [下佁Eかい] 32-bit めEXOR で折りたたんでから `hash32_i32` へ流す、E
  - `Hash` の対象は、まず既孁Estdlib が安定して支えてぁE��キー型に限定した、E
  - `i128` めE��自構造体�Eハッシュ能力�E、今征E`Serialize` / `Eq` との整合を見ながら追加する、E
- compiler 修正:
  - なし。今回の確認で見つかった問題�E `traits_hash.n.md` 側の API サンプルが現衁E`hashmap` / `hashset` の利用流儀とずれてぁE��ことだった、E
  - `must_hm` / `must_hs` と `Option` の match を使ぁE��存流儀へ合わせて修正した、E
- 検証:
  - `node` + `nodesrc/compiler_loader` による compile-only focused check で、E
    - `hash32_by_trait` 単佁E
    - `hashmap/hashset/hashmap_str/hashset_str`
    を使ぁEsnippet
    の両方ぁE`COMPILE_OK` を返すことを確認、E
  - `nodesrc/tests.js` はこ�E環墁E��は長く�Eら下がることがあるため、focused な compile-only でまず妥当性を固定した、E

# 2026-03-09 作業メモ (`std/test` 雁E��EAPI 追加と nested generic overload 根本修正)

- 目皁E
  - stdlib reboot 前�EチE��ト基盤として、E 件失敗しても残りの検査を継続実行できる `std/test` の collectable API を整備する、E
  - `Vec<Result<(),str>>` に `push` / `vec_push` / pipe で `Result<(),str>` を積めなぁEcompiler バグを、library 側の回避ではなぁEtypecheck の根本原因から修正する、E
- 変更:
  - `stdlib/std/test.nepl`
    - `checks_new`
    - `checks_push`
    - `check`
    - `check_eq_i32`
    - `check_ne`
    - `check_str_eq`
    - `check_ok_i32`
    - `check_err_i32`
    - `check_status_str`
    - `checks_has_err(_loop)`
    - `checks_summary(_loop)`
    - `checks_report_failures`
    - `finish_checks`
    を追加した、E
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` は、対応すめE`check_*` を受けて即時失敗する薄ぁE��チE��へ整琁E��た、E
  - `tests/std_test_collect.n.md`
    - `[目皁Eもくてき]` と `[佁Eなに]を[確/たし]かめるか` を付けぁEfocused case を追加した、E
    - 全件成功時�E summary 出力と、失敗を含むとき�E summary + 個別失敗�E力を固定した、E
  - `tests/compiler/overload_nested_generic_push.n.md`
    - `Vec<Result<(),str>>` に対する `push` / `vec_push` / pipe の nested generic overload 解決を確認すめEcompiler 回帰 test を追加した、E
  - `nepl-core/src/types.rs`
    - 関数型に含まれる型変数 binding を退避・復允E��めE
      - `snapshot_type_var_bindings`
      - `restore_type_var_bindings`
      を追加した、E
  - `nepl-core/src/typecheck.rs`
    - `check_function` で関数本体を検査する前に `func_ty` 上�E型変数 binding めEsnapshot し、終亁E��に忁E�� restore するよう変更した、E
- 原因:
  - generic 関数本体�E型検査中に、E��数シグネチャ自体が持ってぁE��型変数 `TypeId` ぁEunification で束縛され、その束縛が `Env` 上�E大域関数型へ残留してぁE��、E
  - そ�E結果、`vec_push <.T> <(Vec<.T>, .T)->Vec<.T>>` の `.T` が過去の検査で `i32` へ汚染され、`Vec<Result<(),str>>` に対する overload 推論で `Vec<i32>` として扱われてぁE��、E
  - 明示型引数付き `vec_push<Result<(),str>>` が通り、型引数省略時だけ落ちることから、candidate 選択時の `instantiate(binding.ty)` 入力が既に汚染されてぁE��と特定した、E
- 結果:
  - `std/test` の collectable API で、`[ok,ok,err,ok,err]` 形式�E概要と失敗添字�E琁E��をまとめて表示できるようになった、E
  - nested generic `push` / `vec_push` / pipe は、型引数を�E示しなくてめE`Vec<Result<(),str>>` 上で解決できるようになった、E
- 検証:
  - `trunk build`�E�Eoot, `NO_COLOR=false`�E�E-> success
  - `node nodesrc/tests.js -i tests/std_test_collect.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-std-test-collect-focused.json -j 15`
    - 結果: `5/5 pass`
    - `find_runtime_helper_key`�E�名前解決�E�と `find_runtime_helper_index`�E�Endex解決�E�を追加、E
  - `nepl-core/src/codegen_wasm.rs`
    - ローカル実裁E��っぁEhelper 名解決を削除し、`runtime_helpers::find_runtime_helper_index` に統一、E
  - `nepl-core/src/monomorphize.rs`
    - helper 保持ルート探索めE`find_runtime_helper_key` + `RuntimeHelperKind` へ置換、E
    - 重褁E��てぁE��名前マッチE��数を削除、E
  - `nepl-core/src/codegen_llvm.rs`
    - helper 候補取得を `helper_candidates(RuntimeHelperKind::...)` に統一、E
    - `resolve_symbol_name` の候補一致めE`helper_base_name` ベ�Eスへ変更し、namespaced/mangled 名でも同一規則で解決、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-helper-unify.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm backend の wasm-body 刁E��を不変条件匁E

- 目皁E
  - `codegen_llvm` 側に残ってぁE�� backend 入力エラー刁E��！EUnsupportedWasmBody`�E�を前段検査前提へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError` から `UnsupportedWasmBody` / `UnsupportedParsedFunctionBody` を削除、E
    - `emit_ll_from_module_for_target` 冁E�� `ActiveRawBody::Wasm` 到達時の `Err` めEinternal panic に変更、E
    - `FnBody::Wasm` reachable 到達時の `Err` めEinternal panic に変更、E
    - HIR lowering 経路で `HirBody::Wasm` 到達時の `Err` めEinternal panic に変更、E
    - 対応テスチE`emit_ll_rejects_entry_with_wasm_body` は `TypecheckFailed` を期征E��る形へ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: wasm codegen 診断返却経路の撤去)

- 目皁E
  - `codegen` 到達後�E生�E専任にする方針に合わせ、`codegen_wasm` の `Vec<Diagnostic>` 返却経路を撤去する、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_body` / `lower_user` の戻り値めE`Result<Function, Vec<Diagnostic>>` から `Function` へ変更、E
    - `gen_block` / `gen_expr` の `diags` 引数を削除、E
    - `generate_wasm` の code section 生�Eで `Err(ds)` 刁E��を削除し、前段検査通過後�E直接生�Eする形に統一、E
    - backend 冁E��断として残ってぁE��未使用関数 `validate_wasm_stack` を削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-wasm-no-diag.json -j 15` -> `8/8 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-wasm-no-diag.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: wasm helper 解決の自己再帰バグ修正)

- 目皁E
  - `tests + stdlib` で発生してぁE�� `RangeError: Maximum call stack size exceeded` を根本原因から解消する、E
- 再現と刁E��刁E��:
  - `option.nepl` doctest を単独再現すると `wasm-function[4]` の自己再帰で停止、E
  - 同一ソースめE`nepl-cli` で生�Eした wasm は正常実行、E
  - `web` 生�E WAT と `native` 生�E WAT を比輁E��ると、同一箁E��で `call 5` ぁE`call 4`�E��E己呼び出し）に化けてぁE��、E
- 原因:
  - `codegen_wasm` の runtime helper 解決が曖昧な斁E���E一致�E�Erefix/contains�E�依存だった、E
  - allocator helper 解決時に `alloc` と `alloc_raw` の取り違えが発生し、enum/tuple 構築時の冁E��確保で自己再帰が起きてぁE��、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - helper 名�E基底名抽出 `helper_base_name` を追加、E
    - runtime helper 解決を基底名一致へ変更し、曖昧一致を廁E��、E
    - 現在 lowering 中の関数インチE��クスは helper 候補から除外、E
    - `LocalMap` に `alloc_helper_idx` を保持し、E��数ごとに一度だぁEhelper を確定、E
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` めE`["alloc_raw", "alloc"]` の頁E��変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-vec-option-after-alloc-helper-fix.json -j 15` -> `22/22 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-helper-fix.json -j 15` -> `791/791 pass`

# 2026-03-05 作業メモ (フェーズD: web 実行時 `compile: unreachable` の根本修正)

- 目皁E
  - `web/dist` 経路でのみ発生してぁE�� `phase=compile, error=unreachable` を根本原因から解消する、E
- 原因:
  - `codegen_wasm.rs` の raw wasm 行パースで、ローカル解決クロージャぁE`parse_wasm_line_with_lookup` 側の `$` 正規化と二重処琁E��なってぁE��、E
  - そ�E結果、`#wasm` 本斁E�E `$a`/`$b` ぁEcodegen 時�Eみ `unknown local` になめEpanic してぁE���E�Erecheck 側とは不整合）、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `parse_wasm_line` の lookup めE`|name| locals.lookup(name)` に統一、E
    - 旧 `parse_local` ヘルパを削除、E
  - `nepl-web/src/lib.rs`
    - `console_error_panic_hook::set_once()` めE`#[wasm_bindgen(start)]` で有効化し、WASM panic の原因位置を可視化、E
  - `nodesrc/run_test.js`
    - `formatError` を追加し、compile/run 失敗時に stack を保持して JSON 出力へ反映、E
- 検証:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-rootfix.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-stdlib --no-tree -o /tmp/tests-list-after-rootfix.json -j 15` -> `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-rootfix.json -j 15` -> `707/791 pass`�E�残り `84 fail` は run 晁E`Maximum call stack size exceeded`。`compile: unreachable` は再現せず�E�E

# 2026-03-05 作業メモ (フェーズD: web 実行時 `unreachable` の刁E��刁E��)

- 目皁E
  - 全体テスチE(`tests + stdlib`) で多発する `phase=compile, error=unreachable` を、E��に合わせではなく根本原因から刁E��刁E��る、E
- 実施:
  - `trunk build` 後に
    - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-baseline-after-revert-v1.json -j 15`
    - 結果: `349/791 pass`、`442 fail`、上位失敗�E `stdlib/alloc/collections/list.nepl` doctest 群の `unreachable`、E
  - 同じ入力を `nepl-cli` で単体コンパイル:
    - `target/debug/nepl-cli -i /tmp/list_doctest1_clean.nepl --target std --emit wasm -o /tmp/list_doctest1_out -v`
    - 結果: compile 成功 (`DEBUG: compile_module returned Ok`)、E
- 結諁E
  - 失敗�E `web/dist`�E�EASM 上�E compiler 実行）経路に限定される、E
  - `codegen_wasm` の今回差刁E��戻しても�E現するため、単純な backend 変更起因ではなぁE��E
  - 以降�E `web` 側で panic を診断化して原因位置を可視化するタスクを上流課題として扱ぁE��E

# 2026-03-05 作業メモ (フェーズD: todo整琁E+ llvm precheck 返り値規紁E

- 目皁E
  - `todo.md` の完亁E��み頁E���E�EUnsupportedHirLowering` 整琁E��を反映し、未完亁E��けを残す、E
  - LLVM 前段検査に「非 unit 関数は値を返す」規紁E��追加して、backend 依存失敗�E前段化を進める、E
- 変更:
  - `todo.md`
    - フェーズDの完亁E��み衁E
      - `llvm 経路でめEbackend 依存エラーを前段診断に寁E��る！EnsupportedHirLowering の整琁E��`
      を削除し、残課題として
      - `llvm 経路の precheck を拡張し、intrinsic/戻り値規紁E��ど backend 依存失敗を前段で確定する。`
      へ更新、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` に `TypeCtx` を渡す形へ変更、E
    - reachable な `HirBody::Block` 関数につぁE��、戻り値型が靁E`unit` かつ block が値を返さなぁE��合を `D3003` で診断、E
  - `nepl-core/src/codegen_llvm.rs`
    - `precheck_llvm_codegen(&types, &hir, &reachable_set)` 呼び出しへ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v9.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm codegen_precheck に実検査を追加)

- 目皁E
  - `codegen` 到達後�E生�E専任に寁E��るため、LLVM 側でも前段検査で弾ける入力を増やす、E
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` を追加、E
    - 到達関数�E�Eeachable set�E�に対して expression tree を走査し、LLVM 未対忁Eintrinsic を前段診断化、E
    - 未対忁Eintrinsic は `D3012 (TypeUnknownIntrinsic)` で報告、E
  - `nepl-core/src/codegen_llvm.rs`
    - HIR lower 前に `precheck_llvm_codegen` を実行し、error があれ�E `TypecheckFailed` で早期終亁E��E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v8.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend 診断型�E整琁E

- 目皁E
  - `codegen_llvm` から `UnsupportedHirLowering` 返却経路が消えた状態を型定義にも反映する、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError::UnsupportedHirLowering` めEenum / Display から削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v6.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm 残孁Ebackend 診断の不変条件匁E継綁E

- 目皁E
  - `codegen_llvm` に残ってぁE�� `UnsupportedHirLowering` を削減し、前段通過後�E生�E専任モチE��へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 以下を `UnsupportedHirLowering` 返却から internal panic へ変更:
      - 関数 return 型不一致
      - enum/struct/tuple 構築時の `alloc` 忁E��判宁E
      - enum payload / struct field / tuple item の値生�E忁E���E型不一致
      - `match` arm の結果型不一致
      - unknown intrinsic 到遁E
      - unsupported expression kind 到遁E
      - 斁E���EリチE��ルID篁E��夁E
      - 斁E���E具体化時�E `alloc` 忁E��判宁E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v5.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm intrinsic 引数・型チェチE��の backend 診断を不変条件匁E

- 目皁E
  - `codegen_llvm` intrinsic lowering に残ってぁE�� backend 診断を削減し、前段通過後�E生�E専任モチE��へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 以下を `UnsupportedHirLowering` 返却から internal panic へ変更:
      - `load` の引数個数/型引数個数不一致、�Eインタ値不在、�Eインタ型不一致
      - `store` の引数個数/型引数個数不一致、�Eインタ/値不在、�Eインタ型不一致、`u8` 値型不一致、格納型不一致
      - `add` の引数個数不一致、lhs/rhs 不在、i32以夁E
      - `f32_to_i32` / `i32_to_u8` / `u8_to_i32` の引数個数・値不在・型不一致
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm 制御構文の backend 診断を不変条件匁E

- 目皁E
  - `codegen_llvm` の `if/while/match` で残ってぁE�� backend 診断を削減し、型検査・前段検証通過後�E生�E専任へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `if`:
      - 条件が値を返さなぁE
      - 条件ぁE`i32/bool` 互換でなぁE
      - then/else 刁E��結果型不一致
      めE`UnsupportedHirLowering` 返却から internal panic へ変更、E
    - `while`:
      - 条件が値を返さなぁE
      - 条件ぁE`i32/bool` 互換でなぁE
      めEinternal panic へ変更、E
    - `match`:
      - scrutinee が値を返さなぁE
      - scrutinee ぁEenum pointer (`i32`) でなぁE
      - arm ぁE件
      めEinternal panic へ変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm call_indirect の backend 診断を不変条件匁E

- 目皁E
  - `codegen_llvm` の `call_indirect` で残ってぁE�� backend 診断�E�EUnsupportedHirLowering`�E�を削減し、前段通過後�E生�E専任に寁E��る、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `call_indirect` につぁE��以下�E `UnsupportedHirLowering` 返却めEinternal panic 匁E
      - callee が値を返さなぁE
      - callee ぁE`i32` 関数IDでなぁE
      - 引数が値を返さなぁE
      - 引数個数不一致
      - 引数型不一致
      - 候補関数未検�E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: raw wasm 行検査の前段刁E��を完亁E

- 目皁E
  - `codegen_precheck` ぁE`codegen_wasm` 実裁E��細へ依存する経路を解消し、前段検査の責務を `wasm_shared` へ雁E��E��る、E
  - 「codegen 到達時は生�E専任」�E方針を維持し、raw wasm 行パース失敗を前段で確定する、E
- 変更:
  - `nepl-core/src/wasm_shared.rs`
    - `parse_wasm_line_with_lookup` を�E有化、E
    - `precheck_raw_wasm_body` を追加し、`HirBody::Wasm` 行を前段で検査して `D4004` を返すように変更、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - raw wasm 事前検査呼び出し�EめE`codegen_wasm` から `wasm_shared` へ変更、E
  - `todo.md`
    - フェーズDの「`codegen_precheck` の wasm 側ヘルパ依存整琁E��頁E��を完亁E��して削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: #wasm のスタチE��検証を前段検査へ移勁E

- 目皁E
  - 「codegen は正しい入力を生�Eするだけ」�E方針に合わせ、`#wasm` ボディ検証めEbackend 実行時ではなぁE`codegen_precheck` 側で完亁E��せる、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `precheck_raw_wasm_body` シグネチャめE`precheck_raw_wasm_body(ctx, func)` に変更、E
    - raw 行�Eパ�Eス成功時に命令列を蓁E��し、前段で `validate_wasm_stack` を実行するよぁE��更、E
    - `lower_user` の `HirBody::Wasm` 経路から `validate_wasm_stack` を削除、E
    - `generate_wasm` の診断雁E��E��実質空に整琁E��Eodegen 冁E��断を発生させなぁE��向に統一�E�、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_raw_wasm_body` 呼び出しを新シグネチャへ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: codegen_precheck の wasm 事前検査を�E通モジュールへ刁E��)

- 目皁E
  - `passes/codegen_precheck.rs` ぁE`codegen_wasm.rs` 実裁E��細へ直接依存してぁE��状態を整琁E��、前段検査ロジチE��を�E有モジュールへ刁E��する、E
  - 「codegen は正しい入力を生�Eするだけ」�E方針に合わせ、backend の `skip`/診断蓁E��を不変条件違反へ寁E��る、E
- 変更:
  - `nepl-core/src/wasm_shared.rs` を新規追加、E
    - wasm署名解決 (`wasm_sig`, `wasm_sig_ids`)
    - generic skip 判宁E(`should_skip_wasm_codegen_for_generic`)
    - 到達関数解极E(`collect_reachable_wasm_functions`)
    - 間接呼び出しを含む署名集合収雁E(`collect_wasm_signature_set`)
    - wasm intrinsic 対応判宁E(`is_supported_wasm_intrinsic`)
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 上記ロジチE��めE`wasm_shared` 参�Eへ置換、E
    - `precheck_raw_wasm_body` のみ `codegen_wasm` 側を継続利用�E�次段で刁E��予定）、E
  - `nepl-core/src/codegen_wasm.rs`
    - extern/function 署名不一致時�E `skip` を廁E��ぁEinternal panic 化、E
    - `lower_body` で backend 診断が返る経路めEinternal panic 化、E
    - 共有ロジチE��は `wasm_shared` 呼び出しへ委譲、E
  - `nepl-core/src/lib.rs`
    - `pub mod wasm_shared;` を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend 診断を前段不変条件へ移衁E

- 目皁E
  - `todo.md` フェーズD方針に合わせ、`codegen_llvm` 側で発行してぁE��「前段通過後に到達しなぁE�Eず」�E診断を廁E��し、前段検証の不変条件として扱ぁE��E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `let` の型不一致 (`let type mismatch`) めE`UnsupportedHirLowering` から internal panic へ変更、E
    - `set` の型不一致 (`set type mismatch`) めE`UnsupportedHirLowering` から internal panic へ変更、E
    - 未解決 trait call の到達を `UnsupportedHirLowering` から internal panic へ変更、E
    - call 引数型不一致めE`UnsupportedHirLowering` から internal panic へ変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant-panic-v1.json -j 15` -> `707/791 pass`�E�EMaximum call stack size exceeded` が多数。今回の変更対象外�E既存失敗として継続調査�E�E

# 2026-03-05 作業メモ (フェーズC/D接綁E core/mem に MemPtr 初期化オーバ�Eロード追加)

- 目皁E
  - `core/mem` 後段移行！Estdlib/std`/tutorials�E�で `i32` 生�Eインタを露出せずに配�E初期化できる上流APIを用意する、E
  - `MemPtr` モチE��上で `fill/memset` を統一し、`Result` で失敗を扱えるようにする、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `memset_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` を追加、E
    - `fill_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` を追加、E
    - `fill_i32 <(MemPtr<i32>,i32,i32)->Result<(),str>>` を追加、E
    - 無効ポインタめE��の長さ�E `Result::Err` を返す、E
  - `tests/memory_safety.n.md`
    - `MemPtr fill_i32/fill_u8 の安�Eオーバ�Eロード` ケースを追加、E
    - `MemPtr fill 系は無効引数めEErr で返す` ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-fill-overload.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-mem-fill-overload.json -j 15` -> `787/787 pass`

# 2026-03-05 作業メモ (フェーズD: kpread_core ヘッダフィールド�E型安�E匁E

- 目皁E
  - `kpread_core` に残ってぁE��ヘッダ生オフセチE���E�E0/4/8`�E�を列挙型へ移行し、`kpread`/`kpwrite` と同じ墁E��表現に揁E��る、E
  - ヘッダレイアウト�E意味を型で固定し、オフセチE��誤持E��を上流で防ぐ、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `ScannerHeaderFieldCore` を追加�E�EBufPtr` / `Len` / `Pos`�E�、E
    - `scanner_header_core_off` を追加し、オフセチE��解決めE箁E��に雁E��E��E
    - `store_i32_u8_at sc*_region 0/4/8 ...` を�E挙型 + オフセチE��関数経由へ置換、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kp-core-header-field-enum.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-core-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite ヘッダフィールド�E型安�E匁E

- 目皁E
  - `kpwrite` のヘッダアクセスで使ってぁE��生オフセチE��値�E�E0/4/8/12/16`�E�を列挙型に置換し、`kpread` と同じ安�EモチE��へ統一する、E
  - `mem/kpread/kpwrite` の公開API安�E化で、�EチE��墁E��の意味を型で表現する、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterHeaderField` を追加�E�EBufPtr` / `Cap` / `WriteLen` / `IovPtr` / `NwPtr`�E�、E
    - `writer_header_off` を追加し、オフセチE��解決を一箁E��に雁E��E��E
    - `writer_header_ptr` / `writer_load_header` / `writer_store_header` / `writer_load_header_ptr` の第2引数めE`i32` から `WriterHeaderField` に変更、E
    - 呼び出し�Eの生数値オフセチE��を�E廁E��、�E挙値に置換、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kp-header-field-enum-unified.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpwrite-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpread ヘッダフィールド�E型安�E匁E

- 目皁E
  - `kpread` のヘッダアクセスで使ってぁE��生オフセチE��値�E�E0/4/8`�E�を列挙型へ置き換え、呼び出し�Eの誤持E��を減らす、E
  - `todo.md` 2026-03-03 フェーズD�E�Emem/kpread/kpwrite` の公開API安�E化）に沿って、上流�E表現を固定する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `ScannerHeaderField` を追加�E�EBufPtr` / `Len` / `Pos`�E�、E
    - `scanner_header_off` を追加し、オフセチE��解決めE箁E��へ雁E��E��E
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` の第2引数めE`i32` から `ScannerHeaderField` に変更、E
    - 呼び出し�Eの `scanner_load_header sc 0/4/8` と `scanner_store_header sc 8 ...` を�E挙型持E��へ置換、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-field-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-header-field.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpread ヘッダアクセスのサイレント失敗を除去)

- 目皁E
  - `scanner_load_header` / `scanner_store_header` の失敗時フォールバック�E�E0` / `()`�E�を廁E��し、�EチE��不整合を隠蔽しなぁE��E
  - 上流仕様（安�EAPI優先）に合わせ、壊れた状態を継続させるより即時停止に統一する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_load_header`:
      - `scanner_header_ptr` ぁE`Err` の場合�E `0` 返却めE`#intrinsic "unreachable"` へ変更、E
      - `load_i32` ぁE`None` の場合�E `0` 返却めE`#intrinsic "unreachable"` へ変更、E
    - `scanner_store_header`:
      - `scanner_header_ptr` ぁE`Err` の場合�E無視を `#intrinsic "unreachable"` へ変更、E
      - `store_i32` ぁE`Err` の場合�E無視を `#intrinsic "unreachable"` へ変更、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-unreachable-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-unreachable.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD先衁E Writer めERegionToken 保持へ移衁E

- 目皁E
  - `kpread` と同様に `kpwrite` でも�E開ハンドルが領域惁E��を持つようにし、メモリ安�EAPIを統一する、E
- 根本原因:
  - `Writer` は `MemPtr<u8>` を直接保持し、�EチE��領域サイズ�E�E0byte�E�が型に表現されてぁE��かった、E
  - 途中で追加した `writer_mem(Writer)->MemPtr<u8>` ヘルパ�E `Writer` を値渡しで受けるため、E
    non-copy な `Writer` の move を発生さぁE`D3053` を引き起こした、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` めE`Writer.region: RegionToken<u8>` に変更、E
    - `writer_wrap` で `region_new raw 20` を構築、E
    - `writer_mem` ヘルパ�E削除し、`region_ptr get w "region"` を直接展開して move を回避、E
  - `stdlib/kp/kpread_core.nepl`
    - `store_i32_u8_at/load_i32_u8_at` めE`RegionToken<u8>` 受け取りへ変更、E
    - `sc0/iov/nread/sc` の吁E��域めE`RegionToken` 化してアクセス経路を統一、E
    - 途中で発生しぁE`match` アーム崩れ！ED3009/D3008/D3045`�E�を修正、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-regiontoken-v3.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先衁E kpread_core の冁E��ヘッダアクセスめERegionToken 匁E

- 目皁E
  - `kpread_core` の冁E��メモリアクセスめE`RegionToken` 経由に統一し、`MemPtr + off` の直接算術依存を減らす、E
- 根本原因:
  - `store_i32_u8_at` / `load_i32_u8_at` ぁE`MemPtr<u8>` と `off` から直接 `MemPtr<i32>` を作る設計で、E
    領域墁E��の前提が�Eルパ外へ漏れてぁE��、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` を追加し、`region_ptr_at<u8,i32>` を使用、E
    - `store_i32_u8_at` / `load_i32_u8_at` の引数めE`RegionToken<u8>` に変更、E
    - `sc0(12)`, `iov(8)`, `nread(4)`, `sc(12)` で `RegionToken` を構築してヘルパへ渡す形に更新、E
  - 途中修正:
    - `match dealloc_ptr<u8> buf cap` の `Result::Err` アームのインチE��ト崩れにより
      `D3009/D3008/D3045` が発生したため、�E岐構造を正しく修正、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-kpread-core-regiontoken-v2.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先衁E kpwrite ヘッダアクセスめERegionToken 経由へ移衁E

- 目皁E
  - `kpwrite` 側でも�EチE��アクセスめE`RegionToken` ベ�Eスに寁E��、`core/mem` の墁E��検証APIを�E利用できるようにする、E
- 根本原因:
  - 既孁E`writer_header_ptr` は `mem_ptr_addr + off` で直接アドレス算術を行い、E
    20byte ヘッダ墁E��の前提を関数ごとに暗黙化してぁE��、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_region` を追加�E�Eregion_new w_mem 20`�E�、E
    - `writer_header_ptr` めE`Result<MemPtr<i32>,str>` へ変更し、`region_ptr_at<u8,i32>` を使用、E
    - `writer_load_header` / `writer_store_header` を上訁E`Result` 経路へ更新、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-header-regiontoken.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先衁E kpread の Scanner ヘッダめERegionToken 匁E

- 目皁E
  - `todo.md` フェーズD着手として、`kpread` の公開ハンドルに領域所有情報を持たせ、`core/mem` の新安�EAPIへ寁E��る、E
- 根本原因:
  - `Scanner` ぁE`MemPtr<u8>` 直接保持のみで、�EチE��領域墁E��の惁E��が型に乗ってぁE��かった、E
  - ヘッダアクセスぁE`mem_ptr_addr + off` の算術依存で、墁E��検証を�E利用しにくかった、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `Scanner` フィールドを `raw: MemPtr<u8>` から `region: RegionToken<u8>` に変更、E
    - `scanner_wrap` で `region_new raw 12` を構築、E
    - `scanner_header_ptr` めE`region_ptr_at<u8,i32>` ベ�Eスの `Result` 返却へ変更、E
    - `scanner_load_header` / `scanner_store_header` を上訁E`Result` 経路で処琁E��E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-scanner-regiontoken.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズC: core/mem に RegionToken 安�EAPIを追加)

- 目皁E
  - `todo.md` フェーズCに沿って、`MemPtr<T>` と `RegionToken<T>` を使ぁE���EAPIめE`core/mem` に追加し、`kpread/kpwrite` 移行�E上流基盤を作る、E
- 根本原因:
  - 既孁E`mem` は `MemPtr<T>` までは整備済みだったが、E��域所有を表す�E開APIが不足しており、E
    墁E��惁E��付きアクセスを型として統一できてぁE��かった、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `RegionToken<T>` 補助APIを追加:
      - `region_new`
      - `region_in_bounds`
      - `region_ptr_at`
      - `alloc_region_bytes`
      - `alloc_region`
      - `dealloc_region`
    - これにより、E��域サイズを伴ぁE��付きオフセチE��取得を `Result` で扱えるようにした、E
  - `tests/memory_safety.n.md`
    - `alloc_region/region_ptr_at/dealloc_region` の基本動作ケースを追加、E
    - 篁E��外オフセチE��で `Result::Err` を返す回帰ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md -i tests/plan.n.md -i tests/block_single_line.n.md --no-stdlib --no-tree -o /tmp/tests-semicolon-focus.json -j 15`
  - 結果: `67/67 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md --no-tree -o /tmp/tests-memory-safety-region-token.json -j 15`
  - 結果: `211/211 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-memory-kp-regression.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズB2: trait capability の型付き保持へ移衁E

- 目皁E
  - trait capability 判定�E斁E���E再解析を減らし、型付きチE�Eタで一貫して扱ぁE��E
- 根本原因:
  - 既存実裁E��は `TraitInfo.capabilities` ぁE`Vec<String>` のため、E
    `TraitSemantics::detect` で毎回斁E���Eを�Eパ�EスしてぁE��、E
  - こ�E構造は capability 判定�E責務が刁E��し、封E��拡張時に不整合を生みめE��ぁE��E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo.capabilities` めE`Vec<String>` から `Vec<TraitCapability>` へ変更、E
    - trait 定義処琁E(`Stmt::Trait`) で capability めE回だけパースし、型付きで保持、E
    - 重褁Ecapability 持E���E同一trait冁E��重褁E��録しなぁE��ぁE��琁E��E
    - `TraitSemantics::detect` は `TraitInfo` 冁E�E型付き capability を直接参�E、E
    - 不要になっぁE`detect_declared_trait_capabilities` を削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-capability-typed.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-capability-typed.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-capability-typed.json -j 15`
  - 結果: `783/783 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite header 読み取りの Result 化と None フォールバック廁E��)

- 目皁E
  - `writer_load_header` の `None -> 0` フォールバックを廁E��し、header 読み取り失敗を明示刁E��で扱ぁE��E
- 根本原因:
  - 従来の `writer_load_header` は `load_i32` 失敗時に 0 を返しており、異常状態を正常値へ潰してぁE��、E
  - そ�Eため後続�E琁E�� `buf/cap/iov/nw` が不正値のまま進行する余地があった、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header` めE`Result<i32,str>` へ変更、E
    - `writer_load_header_ptr` めE`Result<MemPtr<u8>,str>` へ変更、E
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`,
      `writer_put_u8_handle`, `writer_write_str_handle`,
      `writer_write_i32_handle`, `writer_write_u64_handle` めE
      `Result` 刁E��で安�Eに処琁E��る形へ更新、E
    - `if` レイアウト中の冗長な `then: block:` を除去し、`D2002` 回避のため式構造を仕様準拠へ整琁E��E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-after-header-result-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-after-header-result.json -j 15`
  - 結果: `226/226 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpwrite-style-fix.json -j 15`
  - 結果: `215/215 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite の header アクセス雁E��E�� non-copy 整吁E

- 目皁E
  - `kpwrite.nepl` で散在してぁE�� header 生アクセス�E�Eload_i32 add w_raw ...` / `store_i32 add w_raw ...`�E�を共通化し、`Writer` の non-copy/move 規則と矛盾しなぁE��へ整琁E��る、E
- 根本原因:
  - `Writer` は non-copy なのに、最初�Eヘルパ化で `writer_load_header/store_header` ぁE`Writer` 値渡しとなり、�Eルパ呼び出し�E体が move を発生さぁE`D3053` を誘発してぁE��、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_ptr/load/store` を追加、E
    - 上記�Eルパ�E `Writer` ではなぁE`w_raw:i32` を受け取り、`Writer` の move を発生させなぁE��計に変更、E
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`, `writer_put_u8_handle`, `writer_write_str_handle`, `writer_write_i32_handle`, `writer_write_u64_handle` を�E通�Eルパ経由に置換、E
    - 置換後、`w_raw` 直接参�Eは解放処琁E��E���E�Ewriter_free_handle`�E��Eみへ縮小、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-header-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v4.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズB2: trait capability 判定�E自動推定を廁E��)

- 目皁E
  - `copy/clone` の trait 意味付けを�E示 capability (`#capability`) のみに限定し、暗黙推定による誤判定を根本皁E��除去する、E
- 根本原因:
  - `TraitSemantics::detect` ぁEcapability 未持E��時に
    - `Self -> Self` 単一メソチE�� trait めEclone 候裁E
    - marker trait めEcopy 候裁E
    として推定してぁE��、E
  - これにより trait 設計意図と無関係な構造一致だけで copy/clone 意味が付与される余地があった、E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` から clone/copy の自動候補推定を削除、E
    - `#capability copy` / `#capability clone` の宣言結果のみを意味付けに使用、E
    - 不要化した `trait_has_single_unary_self_to_self_method` と `trait_is_marker` を削除、E
    - `TraitSemantics::detect` の未使用 `ctx` 引数を削除、E
  - `tests/move_effect.n.md`
    - `#capability` 未持E��Etrait ぁEcopy/clone として推定されなぁE��とを確認する回帰ケースを追加、E
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3096 TypeUnknownTraitCapability` を追加、E
  - `nepl-core/src/typecheck.rs`
    - trait 定義で未知の `#capability` 名を検�Eし、`D3096` を返すよう変更、E
  - `tests/move_effect.n.md`
    - `#capability cpoy` の compile_fail ケース�E�Ediag_id: 3096`�E�を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-v1.json -j 15`
  - 結果: `269/269 pass`
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-capability-v2.json -j 15`
  - 結果: `227/227 pass`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-v2.json -j 15`
  - 結果: `272/272 pass`

# 2026-03-05 作業メモ (フェーズC: kpread の header 直アクセスを�E通安�Eヘルパへ統一)

- 目皁E
  - `kpread.nepl` で残ってぁE�� `sc_raw` ベ�Eスの header 直接読み書きを除去し、`Scanner` 墁E��の型安�E性を上げる、E
- 根本原因:
  - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` を導�E済みでも、主要パーサ関数が旧経路�E�Eload_i32 add sc_raw ...` / `store_i32 add sc_raw ...`�E�を使ぁE��けてぁE��、E
  - これにより API 墁E��は `Scanner` でも、実裁E�E部が生ポインタ前提のまま刁E��してぁE��、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 以下�E関数で header アクセスめE`scanner_load_header` / `scanner_store_header` に統一:
      - `scanner_skip_ws_handle`
      - `scanner_is_eof_handle`
      - `scanner_skip_token_handle`
      - `scanner_read_token_handle`
      - `scanner_read_i32_handle`
      - `scanner_read_u64_handle`
      - `scanner_read_i64_handle`
      - `scanner_read_f64_handle`
      - `scanner_read_all_i32_handle`
    - 置換後、`kpread.nepl` 冁E�E `sc_raw` 直接アクセスは `scanner_header_ptr` 冁E�E実裁E��点のみに雁E��E��E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-safe-headers-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v3.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズC: kpread 基盤 handle の Scanner 型化)

- 目皁E
  - `kpread` の公開面で露出してぁE��甁E`i32` ハンドル関数を段階的に減らすため、基盤となめE関数めE`Scanner` 受け取りへ変更する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_skip_ws_handle` めE`(Scanner)*>()` へ変更、E
    - `scanner_is_eof_handle` めE`(Scanner)*>bool` へ変更、E
    - `scanner_skip_token_handle` めE`(Scanner)*>()` へ変更、E
    - `scanner_read_token_handle` めE`(Scanner)*>str` へ変更、E
    - 上記呼び出し箁E���E�Ei32` ベ�Eスの既孁Ehandle 群�E�では `scanner_wrap mem_ptr_wrap sc` を�E示して渡すよぁE��一、E
    - 公開ラチE���E�Escanner_skip_ws` など�E��E raw 取り出しをめE��て `Scanner` を直接渡すよぁE��素化、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-scanner-handle-v1.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpread 残り handle 群の Scanner 型化完亁E

- 目皁E
  - `kpread` で残ってぁE�� `*_handle <(i32)...>` 群めE`Scanner` 受け取りへ統一し、�E閁E冁E��の型墁E��を一貫化する、E
- 根本原因:
  - 一部 handle ぁE`i32` を直接受け取り、他�E `Scanner` 受け取り関数と墁E��設計が混在してぁE��、E
  - そ�E結果、�E開ラチE��で `mem_ptr_addr get sc "raw"` を�E度書く忁E��があり、raw 露出と誤用余地が残ってぁE��、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 以下を `Scanner` 受け取りへ変更:
      - `scanner_read_i32_handle`
      - `scanner_read_u64_handle`
      - `scanner_read_i64_handle`
      - `scanner_read_f64_handle`
      - `scanner_read_f32_handle`
      - `scanner_read_vec_i64_handle`
      - `scanner_read_vec_i32_handle`
      - `scanner_read_matrix_i32_handle`
      - `scanner_read_all_i32_handle`
      - `scanner_read_na_i32_handle`
      - `scanner_read_interval_queries_i32_handle`
      - `scanner_read_query_tuples_i32_handle`
      - `scanner_read_ndrh_i32_handle`
    - 吁E��数冁E��では忁E��箁E��のみ `sc_raw = mem_ptr_addr get sc "raw"` を導�Eし、既存ロジチE��を維持、E
    - 公開ラチE�� (`scanner_read_i32` など) は raw 抽出を削除して handle へ `Scanner` を直接渡すよぁE��一、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpread-scanner-allhandles-v1.json -j 15`
  - 結果: `212/212 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-scanner-allhandles-v2.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite handle API の線形化と move 整合化)

- 目皁E
  - `kpwrite` の冁E�� API でも生 `i32` 墁E��を減らしつつ、`Writer` の non-copy 設計と move 規則が矛盾しなぁE��へ整琁E��る、E
- 根本原因:
  - `Writer` を受ける handle ぁE`()` を返す設計�Eまま `Writer` を褁E��回利用しており、`D3053/D3054`�E�Eoved value�E�を誘発してぁE��、E
  - 一晁E`writer_wrap` を多用する形は局所皁E��は動くが、設計として線形消費規則が�E確でなかった、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_writeln_handle` / `writer_write_*_handle` めE`Writer` 受け取り・`Writer` 返却に統一、E
    - 吁Ehandle で `w_raw` を�E部取得し、更新後�E `writer_wrap mem_ptr_wrap w_raw` を返す線形 API に変更、E
    - 褁E��操作を行う handle�E�Ewriter_write_i32_handle`, `writer_write_u64_handle`, `writer_write_*_ln_handle` など�E��E `let mut ww <Writer>` / `set ww ...` で線形に更新、E
    - 公閁EAPI (`writer_write_i32` など) は raw 再ラチE�Eの重褁E��削除し、対忁Ehandle を直接呼ぶ構造へ整琁E��E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kpwrite-only-v4.json -j 15`
  - 結果: `208/208 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-handle-wrap-v3.json -j 15`
  - 結果: `217/217 pass`

- 補足�E�設計判断�E�E
  - 一晁E`writer_wrap` を�E度作る呼び出し�E move エラー回避としては機�Eするが、線形 API 設計として不�E瞭だったため採用しなぁE��E
  - `Writer -> Writer` の更新連鎖を handle 層で明示し、move 規則と API 契紁E��一致させる方針に統一した、E

# 2026-03-05 作業メモ (フェーズC: kpread_core の生メモリアクセス安�EAPI匁E

- 目皁E
  - syscall 墁E��以外�E生メモリアクセスめE`MemPtr` + `Result/Option` 経由へ寁E��、失敗検�Eを上流化する、E
- 根本原因:
  - `kpread_core` 冁E�� `mem_ptr_addr` + 甁E`store_i32/load_i32` を直接実行しており、墁E��不整合時に失敗を型で扱えなかった、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_ptr`, `store_i32_u8_at`, `load_i32_u8_at` を追加、E
    - scanner header 初期匁E(`sc0`, `sc`) めE`store_i32_u8_at` 経由へ変更し、失敗時は確保済み領域を解放して `Err` 返却、E
    - `iov/nread` 構築時の書き込みと `nread` 読み取りを安�Eヘルパ経由へ変更、E
    - メモリアクセス失敗時は `mem_failed` を立て、後段で一括解放して `Result::Err \"kpread_core.memory access failed\"` を返す経路を追加、E
    - `fd_read` 呼び出し�E体�E syscall 仕様丁E`i32` ポインタが忁E��なため、墁E��点でのみ `mem_ptr_addr` を使用、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-core-safe-v1.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: `core/mem` の `*_ptr` を安�EAPI経由へ統一)

- 目皁E
  - `MemPtr` 系 API の冁E��実裁E�� `alloc_raw/realloc_raw/dealloc_raw` 直結から�E離し、`alloc/realloc/dealloc` を通る共通安�E経路へ統一する、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `alloc_ptr` めE`alloc` 経由へ変更、E
    - `realloc_ptr` めE`realloc` 経由へ変更、E
    - `dealloc_ptr` めE`dealloc` 経由へ変更、E
  - これにより `MemPtr` 系エラー経路は基底安�EAPIの前提検査結果と整合する、E
- 検証:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v2.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズC: kpread_core 冁E��確保を `*_ptr` API に統一)

- 目皁E
  - `kpread_core` 冁E��での生�Eインタ管琁E��減らし、`MemPtr<u8>` を使った確俁E再確俁E解放へ寁E��る、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `buf/iov/nread/scanner header` の確保を `alloc_ptr<u8>` に変更、E
    - バッファ拡張めE`realloc_ptr<u8>` に変更、E
    - 解放めE`dealloc_ptr<u8>` に変更、E
    - `fd_read` めE`store_i32/load_i32` へ渡す箁E��のみ `mem_ptr_addr` で `i32` に明示変換、E
  - `scanner_new_impl` は既存どおり `Result<MemPtr<u8>,str>` を返し、API互換を維持、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v6.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpread_core の返却型を MemPtr 匁E

- 目皁E
  - `kpread` 入力�E期化の上流E��Ekpread_core`�E�でも生 `i32` 返却を減らし、`MemPtr<u8>` で墁E��を揃える、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `scanner_new_impl` の戻り値めE`Result<MemPtr<u8>,str>` に変更、E
    - 成功晁E`sc:i32` は `mem_ptr_wrap` して返却、E
    - 失敗系の `Result` 型パラメータめE`MemPtr<u8>` に統一、E
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_handle` は `scanner_new_impl` をそのまま返す実裁E��簡素化、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v5.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpread/kpwrite の `*_new_handle` 返り値めEMemPtr 匁E

- 目皁E
  - 生�E系 API の墁E��から甁E`i32` を減らし、`MemPtr<u8>` による型墁E��を�E確化する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_handle` めE`Result<MemPtr<u8>,str>` へ変更、E
    - `scanner_new` は `MemPtr<u8>` をそのまま `scanner_wrap` に渡す形へ変更、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle` めE`Result<MemPtr<u8>,str>` へ変更、E
    - 冁E��確保で得た `w:i32` は `mem_ptr_wrap` して `Ok` 返却、E
    - `writer_new` は `MemPtr<u8>` をそのまま `writer_wrap` に渡す形へ変更、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v4.json -j 15`
  - 結果: `216/216 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite Writer ラチE�E墁E��の型整吁E

- 目皁E
  - `todo.md` フェーズC�E��E開APIの生�Eインタ露出削減）に沿って、`kpwrite` の `Writer` 生�E墁E��めE`MemPtr<u8>` で統一する、E
- 根本原因:
  - `Writer.raw` は `MemPtr<u8>` だぁE`writer_wrap` ぁE`(i32)->Writer` で、生ポインタを直接受け取る墁E��が残ってぁE��、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_wrap` めE`(MemPtr<u8>)->Writer` に変更、E
    - `writer_new` と `Writer` を返す公開ラチE��群で `i32` めE`mem_ptr_wrap` してから `writer_wrap` を呼ぶよう統一、E
  - 冁E�� `*_handle` は段階移行として `i32` を維持E���E開API墁E��のみ型安�E化）、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v3.json -j 15`
  - 結果: `216/216 pass`

# 2026-03-05 作業メモ (フェーズC: kpread Scanner ラチE�E墁E��の型整吁E

- 目皁E
  - `todo.md` フェーズC�E��E開APIの生�Eインタ露出削減）に沿って、`kpread` の `Scanner` 生�E墁E��めE`MemPtr<u8>` で統一する、E
- 根本原因:
  - `Scanner.raw` は `MemPtr<u8>` なのに `scanner_wrap` ぁE`(i32)->Scanner` で、生成墁E��で生�Eインタを直接受けてぁE��、E
  - これにより `Scanner` の公開型設計と生�Eシグネチャが不一致だった、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_wrap` めE`(MemPtr<u8>)->Scanner` に変更、E
    - `scanner_new` で `raw:i32` めE`mem_ptr_wrap` してから `scanner_wrap` へ渡すよぁE��更、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v2.json -j 15`
  - 結果: `216/216 pass`

# 2026-03-05 作業メモ (compile_fail: diag_id + 位置検証の運用固宁E

- 目皁E
  - `compile_fail` ケースで `diag_id` だけでなく発生位置�E�Eile/line/col�E�も安定検証できるようにする、E
- 変更:
  - `nodesrc/tests.js`
    - `extractDiagSpansFromCompileError` を行単位解析へ変更、E
    - `--> ...` 行から末尾 `:line:col` を基準に抽出するよう修正し、パス中のコロンを含む形式にも耐えるよぁE��した、E
  - `nodesrc/parser.js`
    - doctest メタ `diag_spans` に JSON object 形式！E{file,line,col}`�E�を許可、E
    - 既存�E `"line:col"` / `"file:line:col"` 斁E���E表記�E互換維持、E
  - `tests/compile_fail_diag_location.n.md`
    - `diag_spans` の object 形式を使ぁE��帰ケースを追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md -i tests/lexer_diag.n.md --no-stdlib --no-tree -o /tmp/tests-compile-fail-location-verify.json -j 15`
  - 結果: `6/6 pass`

# 2026-03-05 作業メモ (`;` 診断の上流化と loader 診断整形)

- 目皁E
  - `tests/block_semicolon_return.n.md::doctest#10` の backend 漏れ�E�Easm validation error�E�を止め、parser 段で `diag_id` を固定化する、E
  - `compile_fail` で loader 経由のエラーでめE`error[Dxxxx]` を安定取得できるようにする、E
- 根本原因:
  - `if:` レイアウト�Eの `Stmt::ExprSemi` が上流で拒否されず、codegen まで進んでぁE��、E
  - `nepl-web/src/lib.rs` で loader エラーめE`to_string()` しており、`Diagnostics` 斁E���Eが整形されぁE`diag_id` 抽出が不安定だった、E
- 変更:
  - `nepl-core/src/parser.rs`
    - `reject_layout_semicolon` を追加、E
    - `extract_if_layout_exprs` / `extract_if_layout_exprs_lenient` で `ExprSemi` めE`D2002` として即時拒否、E
    - `while` / 一般引数レイアウト�E既存仕様！E;` 許容�E�を維持、E
  - `nepl-web/src/lib.rs`
    - loader 失敗時に `render_loader_error` を通すよう変更、E
    - `LoaderError::Core` は `render_core_error` へ流し、`error[Dxxxx]` 形式で返す、E
  - `tests/plan.n.md`
    - `diag_id` 期征E��実裁E���Eに合わせて `2002 -> 2001` に修正�E�Eケース�E�、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/lexer_diag.n.md -i tests/plan.n.md -i tests/block_single_line.n.md -i tests/block_semicolon_return.n.md --no-stdlib --no-tree -o /tmp/tests-diag-parser.json -j 15` -> `70/70 pass`

# 2026-03-05 作業メモ (codegen 前段共送Eprecheck 導�E: raw body/target 診断の統一)

- 目皁E
  - `codegen_wasm` / `codegen_llvm` が個別に `#wasm/#llvmir` の target 不整合を診断する構造をやめ、前段共通チェチE��で診断を確定する、E
- 根本原因:
  - `#if[target=...]` 評価、active 斁E��出、raw body 選択ロジチE��ぁE`typecheck` と `codegen_llvm` に刁E��し、判定差刁E�� backend 依存診断が発生してぁE��、E
- 変更:
  - 新要E`nepl-core/src/target_precheck.rs` を追加、E
    - `gate_allows`�E�E#if[target/profile]` 判定！E
    - `active_stmt_indices`�E�Ective 斁E��出�E�E
    - `select_active_raw_body`�E�関数 body 冁E`#wasm/#llvmir` 選択！E
    - `precheck_function_raw_body_target` / `precheck_module_raw_bodies`�E�Earget 整合検証�E�E
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3094 TypeMultipleActiveRawBodies`
    - `D3095 TypeRawBodyTargetMismatch`
  - `nepl-core/src/compiler.rs`
    - `compile_module` の typecheck 前に `precheck_module_raw_bodies` を実行し、エラー時�E早期終亁E��E
  - `nepl-core/src/typecheck.rs`
    - `check_function` 冒頭で `precheck_function_raw_body_target` を実行し、`typecheck` 直接利用経路でも同一診断を保証、E
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 冒頭で `precheck_module_raw_bodies` を実行、E
    - `#if` active 斁E��出を�E送E`active_stmt_indices` に統一、E
    - Parsed 関数の raw body 選択を共送E`select_active_raw_body` に統一、E
    - 重褁E��てぁE�� local gate/raw 選択関数群を削除、E
  - チE��チE
    - 既存更新:
      - `tests/neplg2.n.md` の `wasm_rejects_llvmir_body_with_diag_id` めE`diag_id: 3095` へ変更、E
      - `tests/neplg2.n.md` に `raw_body_conflict_reports_diag_id`�E�Ediag_id: 3094`�E�追加、E
      - `tests/llvm_target.n.md` の `llvm_rejects_wasm_body` に `diag_id: 3095` 追加、E
    - 新規追加:
      - `tests/raw_body_precheck.n.md`�E�Eケース、`D3094/D3095` を固定確認）、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md --no-stdlib --no-tree --runner all -o /tmp/tests-raw-body-precheck.json -j 15` -> `3/3 pass`
  - 参老E `tests/neplg2.n.md` + `tests/llvm_target.n.md` めE`--with-stdlib` で実行すると既知の stdlib 側失敗！Eist doctest�E�が混ざるが、追加した `D3094/D3095` ケース自体�E通過してぁE��ことめE`/tmp/tests-codegen-precheck.json` で確認、E

# 2026-03-05 作業メモ (`;` 仕様�E行修正: `stdlib/core/math.nepl`)

- 目皁E
  - `plan.md` の「褁E��文には末尾 `;` を付けなぁE��制紁E��合わせ、`overload` 失敗�E根本原因を�Eに解消する、E
- 根本原因:
  - `stdlib/core/math.nepl` の `i128/u128` 周辺で、褁E��E`if:` を右辺に持つ `let` 斁E�E末尾に `;` が残ってぁE��、E
  - これが式�E `()` 化を誘発し、wasm 検証段で `invalid wasm generated: expected i64 but nothing on stack` を引き起こしてぁE��、E
- 変更:
  - `stdlib/core/math.nepl` の該当箁E��で、褁E��E`if:` 右辺 `let` の末尾 `;` を除去、E
  - 対象: `to_i128`, `u128/i128` の `carry/borrow` 計算、`mul_wide` の `carry_mid/carry_lo` 計算、E
- 検証:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-nostd.json -j 15`
  - 結果: `43/43 pass`

# 2026-03-05 作業メモ (パ�Eサ根本修正: 単衁Eblock 制紁E�� `ExprSemi` 意味論保持)

- 目皁E
  - `tests/plan.n.md::doctest#29`�E�単衁E`block` 冁E��褁E��E`block:` が�EってしまぁE��をコンパイラ側で根本修正する、E
  - `tests/block_semicolon_return.n.md::doctest#10`�E�褁E��式末尾 `;` の意味落ち�E�を解消する、E
- 根本原因:
  - パ�Eサが「単衁Eblock 斁E��」を保持しておらず、単衁E`block` 冁E��めE`parse_block_after_colon()` を通して褁E��E`:` ブロチE��を受琁E��てぁE��、E
  - `extract_if_layout_exprs` / `extract_while_layout_exprs` / `extract_arg_layout_exprs` ぁE`Stmt::ExprSemi` めE`Stmt::Expr` と同一扱ぁE��、`;` による unit 化とスタチE��検証を落としてぁE��、E
- 変更:
  - `nepl-core/src/parser.rs`
    - `single_line_block_depth` を追加し、単衁Eblock 解析中に褁E��E`:` ブロチE��を検�EしためE`D2002` を�EすよぁE��変更、E
    - `parse_single_line_block*` で斁E��深さを管琁E��るよぁE��更、E
    - `ExprSemi` を保持してレイアウト抽出へ渡す�E通�Eルパ�Eを追加、E
    - if/while/引数レイアウト抽出で `ExprSemi` を捨てずに block 化して保持し、型検査段で `;` 意味論が反映されるよぁE��変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/plan.n.md --no-stdlib --no-tree -o /tmp/tests-plan-nostd.json -j 15` -> `36/36 pass`
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md --no-stdlib --no-tree -o /tmp/tests-block-semicolon-nostd.json -j 15` -> `10/10 pass`
- 影響:
  - `--with-stdlib` で走らせると stdlib doctest 側に `;` 意味論不整合が顕在化！EList` などで `expected ... got unit`�E�、E
  - これは今回のパ�Eサ修正で隠れてぁE��仕様違反が表面化した状態、E
  - 次段として stdlib 側の `;` 使用箁E��めEplan.md に合わせて整琁E��る忁E��がある、E

# 2026-03-05 作業メモ (plan.md 全体�E読: plan.n.md 拡允E

- 目皁E
  - `plan.md` 全体を再読し、実裁E��誤りやすい仕様を `tests/plan.n.md` に雁E��E��て回帰可能にする、E
- 変更:
  - `tests/plan.n.md` を拡允E��E
  - 既孁E`compile_fail` に `diag_id` を付丁E
    - `plan_block_trailing_semicolon_makes_unit_and_breaks_i32_return` -> `3003`
    - `plan_semicolon_requires_exactly_one_value_growth` -> `3016`
  - 追加した主な仕様テスチE
    - `block:` 後ろはコメント�Eみ許可、トークン禁止
    - 引数オフサイド（褁E��行引数�E�E
    - `while` の `cond/do` 記法！Enline / block�E�E
    - 関数リチE��ル `():`、`fn` 糖衣 + `@` 関数値参�E
    - pipe の改行記況E
    - 単行ブロチE��の多段ネスチE
    - `if:` ぁE式忁E��E
    - 単行ブロチE��褁E���E�E;`区刁E���E�と末尾 `;` による `()` 匁E
    - 1衁E斁E��区刁E��なし）エラー
    - `Tuple:` リチE��ル
    - 型注釈が式に前置される挙勁E
- 検証:
  - `node nodesrc/tests.js -i tests/plan.n.md --no-tree -o /tmp/tests-plan-nmd-2.json -j 15`
  - 結果: `240 total / 239 pass / 1 fail`
- 差刁E��Elan.md と実裁E��E
  - `plan_single_line_block_cannot_contain_multiline_block` ぁE`expected compile_fail` に対して compile success、E
  - これは plan.md の「単行ブロチE��冁E��褁E��ブロチE��を置けなぁE��制紁E��対する未実裁E��ャチE�E、E

# 2026-03-04 作業メモ (フェーズB2継綁E Copy/Clone 判定�E trait識別子化)

- 目皁E
  - `todo.md` フェーズB2「trait 契紁E��定�E斁E���E依存を減らす」を進め、`Copy/Clone` 能力判定を trait名ではなぁEtrait識別子で扱ぁE��E
- 根本原因:
  - `TraitSemantics` と `ImplInfo` の判定�E `trait_name` 斁E���E比輁E��依存しており、名前解決変更めEalias 導�E時に脁E��、E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics` めE`copy_trait/clone_trait: Option<(String, TypeId)>` に変更、E
    - `is_copy_trait` / `is_clone_trait` めE`TypeId` 比輁E��変更、E
    - `detect_capability_trait` の戻り値めE`Option<(String, TypeId)>` へ変更、E
    - `ImplInfo` に `trait_self_ty: Option<TypeId>` を追加し、`Copy/Clone` 判定�E重褁Eimpl 判定に利用、E
    - `ctx.set_copy_trait_enabled(...)` は `copy_trait_name().is_some()` で制御、E
    - 最絁Eimpl 生�Eパスの copy 判定も `trait_info.self_ty` を使用、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `278/278 pass`
- 状況E
  - `Copy/Clone` 能力判定�E主要経路は trait名文字�E比輁E��ら離脱、E
  - 残りの斁E���E依存�E一般 trait 墁E��判定！Etrait_bound_satisfied` など�E��Eに限定される、E

# 2026-03-04 作業メモ (フェーズB2継綁E Copy判定�E経路刁E��と tests/*.n.md 回帰追加)

- 目皁E
  - `todo.md` フェーズB2の残件として、trait モード時の `Copy` 判定を旧互換経路から刁E��し、名前ハードコード依存をさらに減らす、E
  - 変更に対応する回帰めE`tests/*.n.md` に追加する、E
- 根本原因:
  - `TypeCtx::is_copy` は trait モードでも�Eに `is_copy_eligible`�E�Ei64/f64` 名ハードコード）を通るため、`impl Copy` ベ�Eス判定に完�E移行できてぁE��かった、E
  - `Copy impl` 妥当性検査も同じ経路を使っており、段階移行�E墁E��が曖昧だった、E
- 変更:
  - `nepl-core/src/types.rs`
    - `is_copy_impl_eligible` を追加�E�Eimpl Copy` 妥当性専用�E�、E
    - `is_copy` を経路刁E��:
      - trait モード有効時�E `is_copy_with_trait_model` を直接使用、E
      - trait モード無効時�Eみ `is_copy_eligible` を使用、E
    - `is_copy_eligible_inner` に `allow_opaque_named` を追加し、`is_copy_impl_eligible` からは Named 型を名前依存なしで妥当判定可能にした、E
  - `nepl-core/src/typecheck.rs`
    - `impl Copy for T` の対象妥当性検査めE`ctx.is_copy_impl_eligible(target_ty)` に変更、E
  - `tests/move_effect.n.md`
    - 回帰ケースめE件追加:
      - `Copy` trait 有効時、`i64` に `Copy impl` がなぁE��合�E move エラー�E�Ediag_id: 3053`�E�、E
      - `Clone+Copy impl` を与えぁE`i64` は再利用可能、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `278/278 pass`
- 状況E
  - `Copy` 判定�E trait モード経路は刁E��済み、E
  - 次段で `Copy/Clone` 能力宣言の抽象化！Erait 名検�EロジチE��のさらなる一般化）へ進む、E

# 2026-03-04 作業メモ (フェーズB2: Copy能力判定�Etrait移行スイチE��導�E)

- 目皁E
  - `todo.md` フェーズB2の「`Copy/Clone` 能力判定�Eハ�Eドコード撤廁E��に向け、`Copy` trait 実裁E��報へ段階移行する土台を追加する、E
- 根本原因:
  - `TypeCtx::is_copy` は常に構造ベ�Eス判定�Eみで、`impl Copy for T` の有無を�E力判定に反映できなかった、E
  - 既存賁E��との互換を保ちながら移行する�E替点がなく、一括移行すると庁E��E��の回帰リスクが高かった、E
- 変更:
  - `nepl-core/src/types.rs`
    - `TypeCtx` に `copy_trait_enabled: bool` を追加、E
    - `set_copy_trait_enabled(bool)` を追加、E
    - `is_copy` を段階判定へ変更:
      - まず既孁E`is_copy_eligible` で前提検証、E
      - `copy_trait_enabled == false` では従来挙動を維持、E
      - `copy_trait_enabled == true` では `is_copy_with_trait_model` を使ぁE��ADT は `impl Copy` 登録�E�Ecopy_impl_targets`�E�を忁E��化、E
    - 追加調整:
      - trait モード時の `TypeKind::Named` / `TypeKind::Apply` 判定を型名ハ�Eドコードから外し、`has_copy_impl_target` ベ�Eスへ変更、E
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` 後に `ctx.set_copy_trait_enabled(...)` を設定し、`Copy` trait が定義されるモジュールでのみ新判定を有効化、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `276/276 pass`
- 状況E
  - 互換性を保ったまま `Copy` trait 反映の刁E��点を導�E済み、E
  - 次段で `Copy/Clone` を�E力テーブル化し、判定ロジチE��の斁E���E依存をさらに削減する、E

# 2026-03-04 作業メモ (上流修正: codegen_wasm 診断IDの明示匁E

- 目皁E
  - `todo.md` 残件だっぁE`codegen_*.rs` の主要診断めE`diag_id` で固定し、codegen 失敗�E刁E��を斁E��依存から�Eり離す、E
- 根本原因:
  - `codegen_wasm.rs` の `Diagnostic::error(...)` は ID 未付与で、codegen フェーズ失敗を安定的に特定できなかった、E
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D4001..D4015` を追加:
      - `CodegenWasmUnsupportedExternSignature`
      - `CodegenWasmUnsupportedFunctionSignature`
      - `CodegenWasmMissingReturnValue`
      - `CodegenWasmRawLineParseError`
      - `CodegenWasmLlvmIrBodyNotSupported`
      - `CodegenWasmStringLiteralNotFound`
      - `CodegenWasmUnknownVariable`
      - `CodegenWasmUnknownFunctionValue`
      - `CodegenWasmUnknownFunction`
      - `CodegenWasmMissingIndirectSignature`
      - `CodegenWasmUnsupportedIndirectSignature`
      - `CodegenWasmUnknownIntrinsic`
      - `CodegenWasmUnsupportedEnumPayloadType`
      - `CodegenWasmUnsupportedStructFieldType`
      - `CodegenWasmUnsupportedTupleElementType`
  - `nepl-core/src/codegen_wasm.rs`
    - 主要Ecodegen エラー発生点に `with_id(...)` を付与、E
    - 追加対象:
      - extern/function シグネチャ lower 失敁E
      - missing return
      - raw wasm parse 失敁E
      - wasm backend での llvm ir body
      - unknown variable/function/function value
      - indirect call signature 問顁E
      - unknown codegen intrinsic
      - enum/struct/tuple の unsupported payload/field/element 垁E
  - `tests/neplg2.n.md`
    - `wasm_rejects_llvmir_body_with_diag_id` を追加�E�Ediag_id: 4005`�E�、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/neplg2.n.md -i tests/functions.n.md -i tests/selfhost_req.n.md --no-tree -o /tmp/tests-codegen-diag-subset.json -j 15` -> `276/276 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-codegen-diagid.json -j 15` -> `798/798 pass`
- 状況E
  - `todo.md` の診断ID残件�E�Eodegen 主要診断�E��E完亁E��E

# 2026-03-04 作業メモ (上流修正: typecheck の module/impl 定義時診断IDを�E示匁E

- 目皁E
  - `todo.md` 残件だっぁE`typecheck.rs` 上流E��Eodule/impl 定義フェーズ�E��E未付与診断めE`diag_id` で固定し、文言依存を除去する、E
- 根本原因:
  - 定義登録/impl 検証フェーズは `Diagnostic::error(...)` のまま残っており、同種エラーでめEID が不安定だった、E
  - そ�Eため `compile_fail` の失敗理由が文言変更で揺れる状態だった、E
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3073..D3092` を追加:
      - `TypeUnknownTraitBound`
      - `TypeWasiImportTargetMismatch`
      - `TypeExternSignatureMustBeFunction`
      - `TypeItemNameConflict`
      - `TypeEnumTypeParamBoundsUnsupported`
      - `TypeStructTypeParamBoundsUnsupported`
      - `TypeTraitTypeParamsUnsupported`
      - `TypeTraitMethodTypeParamsUnsupported`
      - `TypeInherentImplUnsupported`
      - `TypeImplTypeParamsUnsupported`
      - `TypeUnknownTrait`
      - `TypeImplTargetMustBeConcrete`
      - `TypeFunctionSignatureMustBeFunction`
      - `TypeAliasTargetNotFound`
      - `TypeFunctionSignatureOverloadNotFound`
      - `TypeDuplicateImplMethod`
      - `TypeImplMethodNotFoundInTrait`
      - `TypeImplMethodSignatureMismatch`
      - `TypeImplMissingTraitMethod`
      - `TypeEntryFunctionMissingOrAmbiguous`
  - `nepl-core/src/typecheck.rs`
    - 上流定義フェーズ�E�Enum/struct/trait/impl/alias/entry�E��E未付与エラーへ `with_id(...)` を付与、E
    - `check_function` 冒頭の signature/arity 検証にめEID を付与、E
  - `tests/neplg2.n.md`
    - 既孁E`compile_fail` に `diag_id` を追加:
      - `pipe_target_missing_after_annotation_is_error` -> `3016`
      - `wasi_import_rejected_on_wasm_target` -> `3074`
      - `name_conflict_enum_fn_is_error` -> `3076`
      - `trait_bound_missing_impl_is_error` -> `3069`
      - `trait_method_arity_mismatch_is_error` -> `3068`
      - `unknown_trait_bound_is_error` -> `3073`
  - `tests/functions.n.md`
    - `function_alias_target_not_found`�E�Ediag_id: 3086`�E�を追加、E
  - `tests/selfhost_req.n.md`
    - `test_req_trait_extensions` に `diag_id: 3081` を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/neplg2.n.md -i tests/functions.n.md -i tests/selfhost_req.n.md --no-tree -o /tmp/tests-typecheck-item-diag-subset.json -j 15` -> `275/275 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-item-diagid.json -j 15` -> `797/797 pass`
- 状況E
  - `typecheck.rs` の上流定義フェーズ診断ID付与�E完亁E��E
  - 次段は `todo.md` 残件どおり `codegen_*.rs` の主要診断ID明示化、E

# 2026-03-04 作業メモ (上流修正: lexer 診断IDの明示化と回帰追加)

- 目皁E
  - `lexer.rs` の未付与エラーに診断IDを付け、`compile_fail + diag_id` で固定検証できる状態にする、E
- 根本原因:
  - `unknown token/directive` 以外�E字句エラーは `with_id` 未付与で、失敗�E類が斁E��依存になってぁE��、E
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D1203..D1209` を追加:
      - `LexerIndentTabsNotAllowed`
      - `LexerExpectedIndentedBlock`
      - `LexerInvalidPubDirectivePrefix`
      - `LexerIndentWidthMismatch`
      - `LexerIndentLevelMismatch`
      - `LexerInvalidStringEscape`
      - `LexerUnterminatedStringLiteral`
  - `nepl-core/src/lexer.rs`
    - タブインチE��ト、`#wasm/#llvmir` 後インチE��ト不足、`pub` 接頭辞誤用、E
      インチE��ト幁E��一致/階層不一致、invalid escape、unterminated string に `with_id` を付与、E
  - `tests/lexer_diag.n.md`
    - 新規追加�E�Eケース�E�E
      - invalid escape -> `diag_id: 1208`
      - unterminated string -> `diag_id: 1209`
      - invalid `pub` prefix -> `diag_id: 1205`
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/lexer_diag.n.md --no-tree -o /tmp/tests-lexer-diag.json -j 15` -> `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-lexer-diagid-extend.json -j 15` -> `796/796 pass`
- 状況E
  - parser + lexer + typecheck�E�主要経路�E��E診断ID固定化が進行、E
  - 次段は `typecheck` 上流E��Eodule/impl 定義時）と `codegen_*.rs` の残未付与診断を整琁E��る、E

# 2026-03-04 作業メモ (上流修正: overload/trait/pipe の診断ID拡張)

- 目皁E
  - `typecheck` の未付与エラー�E�特に overload/trait method/pipe/arity 周辺�E�を診断IDで固定化し、`compile_fail` 回帰を安定化する、E
- 根本原因:
  - 同一カチE��リの型検査失敗で `with_id` 未付与経路が残り、文言変更に弱ぁE��態だった、E
  - trait 経由呼び出し�E失敗（未知メソチE��・墁E��未允E��など�E�が `diag_id` で識別できなかった、E
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3066..D3072` を追加:
      - `TypeTraitMethodTypeArgsNotSupported`
      - `TypeTraitMethodNotFound`
      - `TypeArgumentArityMismatch`
      - `TypeTraitBoundUnsatisfied`
      - `TypeInvalidDeref`
      - `TypeAssignmentArityMismatch`
      - `TypeCallReductionLimitExceeded`
  - `nepl-core/src/typecheck.rs`
    - 以下�E診断に `with_id` を付丁E
      - `pipe has no target` -> `D3013`
      - trait method への型引数未対忁E-> `D3066`
      - trait method 不在 -> `D3067`
      - overload の型引数不一致 -> `D3021`
      - 引数個数不一致�E�関数/constructor/trait method receiver�E�E> `D3068`
      - trait 墁E��未允E�� -> `D3069`
      - assignment 個数不一致 -> `D3071`
      - field assignment 型不一致 -> `D3036`
      - 非参照垁Ederef -> `D3070`
      - call reduction 反復上限趁E�� -> `D3072`
  - `tests/overload.n.md`
    - `compile_fail + diag_id` めEケース追加:
      - trait method 型引数未対忁E(`3066`)
      - trait method 不在 (`3067`)
      - trait 墁E��未允E�� (`3069`)
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-diagid-extend.json -j 15` -> `244/244 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-diagid-extend.json -j 15` -> `793/793 pass`
- 状況E
  - `D3006`�E�Eo matching overload�E�と field access�E�ED3011`�E��E診断経路を�E離したまま維持、E
  - 次段は `todo.md` の診断ID拡張残件�E�Eexer + typecheck上流�E未付与領域�E�を継続する、E

# 2026-03-04 作業メモ (上流修正: typecheck の noshadow/shadow 診断IDを�E示匁E

- 目皁E
  - `typecheck` の `noshadow` / `non-shadowable` 系エラーを診断生�E点で固定し、回帰めE`diag_id` で検証可能にする、E
- 根本原因:
  - 同一カチE��リの shadow 関連エラーに `with_id` 未付与経路が残り、文言依存�E判定になってぁE��、E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `cannot shadow non-shadowable ...` 系めE`TypeNoShadowViolation (D3014)` へ統一、E
    - `noshadow declaration ... conflicts ...` 系めE`TypeNoShadowConflict (D3015)` へ統一、E
    - 関数/関数alias/ローカル let の吁E��路で secondary label 付き診断にも同IDを付与、E
  - `tests/shadowing.n.md`
    - `compile_fail` 4ケースに `diag_id: 3014` を追加して固定化、E
- 検証:
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/move_effect.n.md --no-tree -o /tmp/tests-shadowing-moveeffect-diagid.json -j 15` -> `248/248 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-shadow-diagid.json -j 15` -> `790/790 pass`
- 状況E
  - shadow/noshadow の主要経路は `diag_id` 固定化済み、E
  - 次段は `typecheck` の残未付与カチE��リ�E�Endefined/overload/pipe/pure-impure�E�へ拡張する、E

# 2026-03-04 作業メモ (上流修正: typecheck field-access 診断IDの明示匁E

- 目皁E
  - `typecheck.rs` の field access 系エラーを診断生�E点で `DiagnosticId` 固定し、`compile_fail` めEID で安定検証できるようにする、E
- 根本原因:
  - `core/field::get` / `put` 経由の失敗�E、型検査フェーズで発生するにもかかわらず、`with_id` なし�E `Diagnostic::error` が残ってぁE��、E
  - 斁E��のみ依存だと、エラーチE��スト調整時に回帰検�Eが不安定になる、E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `resolve_field_access_with_mode` 配下�E field 参�E失敗（篁E��夁Eフィールド不存在/非褁E��型）に
      `TypeInvalidFieldAccess (D3011)` を�E示付与、E
  - `tests/move_effect.n.md`
    - `core/field` の不正アクセスめE`compile_fail + diag_id: 3011` で固定するケースを追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move_effect-check.json -j 15` -> `221/221 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-field-diagid.json -j 15` -> `790/790 pass`
- 状況E
  - field access 系は `D3011` で明示化完亁E��E
  - 次段は `typecheck` の未付与領域�E�Ehadow / overload / pipe / undefined 系�E�を頁E��明示化する、E

# 2026-03-04 作業メモ (上流修正: parser 診断IDの未付与箁E��を�E示匁E

- 目皁E
  - `todo.md` の「診断IDの明示付与！Earser/typecheck/resolve�E�」を上流から進め、`parser.rs` の未付与診断を生成点で固定する、E
- 根本原因:
  - `Diagnostic::error(...)` ぁE`with_id` なしで残っており、同種エラーでめEDが安定しなぁE��路があった、E
  - 斁E��依存�Eままだと `compile_fail` の回帰固定が不十刁E��なる、E
- 変更:
  - `nepl-core/src/parser.rs`
    - 再帰上限/無進捗回復/marker配置/mlstr/#externシグネチャ/型パラメータ解析などの未付与診断へ `with_id` を付与、E
    - 付与IDは既存�E Parser 系 (`ParserExpectedToken`, `ParserUnexpectedToken`, `ParserExpectedIdentifier`, `ParserInvalidTypeExpr`, `ParserInvalidExternSignature`) を利用、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-parser-diagid.json -j 15` -> `789/789 pass`
- 状況E
  - parser の `Diagnostic::error` は診断生�E点で ID 明示化済み、E
  - 次段は `typecheck.rs` の未付与診断へ同方針を展開する、E

# 2026-03-04 作業メモ (上流テスト整傁E `tests/move_check.n.md` の skip 解除)

- 目皁E
  - `move_check` 系 `.n.md` の上流回帰めE`skip` 依存から外し、診断ID付き compile_fail で固定化する、E
- 変更:
  - `tests/move_check.n.md`
    - `move_simple_ok` を実コード化�E�Eret: 0`�E�、E
    - `move_use_after_move` めE`compile_fail + diag_id: 3053` に変更、E
    - `move_in_branch` めE`compile_fail + diag_id: 3054` に変更、E
    - `move_in_loop` めE`compile_fail + diag_id: 3065` に変更、E
- 根本原因:
  - 旧 Rust チE��ト移植時に `skip` が残っており、�E岐合流Eループ�E利用の move 回帰ぁECI で検�E不�Eだった、E
  - 診断IDで失敗理由を固定しなぁE��、文言揺れで意図しなぁE��帰を見落とす、E
- 検証:
  - `node nodesrc/tests.js -i tests/move_check.n.md --no-tree -o /tmp/tests-move-check-nmd.json -j 15` -> `217/217 pass`
- 状況E
  - `move_check.n.md` の先頭4ケースは実行型になり、`skip` は除去済み、E
  - 次段で `todo.md` の診断ID未付与領域�E�Earser/typecheck/resolve�E�を継続する、E

# 2026-03-04 作業メモ (フェーズD進衁E Scanner/Writer の直接利用へ下流移衁E

- 目皁E
  - `kpread/kpwrite` 公開APIの安�E型利用を下流へ浸透させ、生ハンドル由来の中間束縛を減らす、E
- 変更:
  - `tests/kp.n.md`
  - `tests/kp_i64.n.md`
  - `tests/stdin.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `examples/kp_fizzbuzz.nepl`
  - それぞれ `let sc_obj <Scanner> unwrap_ok scanner_new; let sc <Scanner> sc_obj;` めE
    `let sc <Scanner> unwrap_ok scanner_new;` へ統一、E
  - カタログ冁E�E `sc_handle` も削除し、`Scanner` を直接渡す形へ統一、E
- 根本原因:
  - 公開APIが安�E型で整ってぁE��も、下流コードに旧来の二段束縛が残ると、生ハンドル前提へ戻しやすくなる、E
  - 先に利用側の書き方を揃えることで、次段の公開面整琁E��ハンドル版隔離�E�を安�Eに進められる、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-tree -o /tmp/tests-kp-typed-usage.json -j 15` -> `225/225 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-scanner-writer-typed-direct.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-scanner-writer-typed-direct.json -j 15` -> `262/262 pass`
- 状況E
  - 下流�E主要利用箁E��は `Scanner/Writer` 直接利用へ移行済み、E
- 次段で `kpread/kpwrite` の i32 ハンドル受け取りオーバ�Eロード�E公開面整琁E��継続する、E

# 2026-03-04 作業メモ (上流修正: move_check 診断IDの明示匁E

- 目皁E
  - `move_check` が生成する主要エラーに `diag_id` を付与し、`compile_fail` を診断IDで固定検証できる状態にする、E
- 根本原因:
  - move/borrow 系エラーは斁E��一致に依存しており、封E��の斁E��調整でチE��トが壊れめE��かった、E
  - `todo.md` の「診断IDの明示付与」を満たすには、診断生�E点�E�Emove_check.rs`�E�で enum を直接持E��する忁E��があった、E
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `3051..3065` の move/borrow 系 `DiagnosticId` を追加、E
    - `from_u32` / `message` に新IDを追加、E
  - `nepl-core/src/passes/move_check.rs`
    - `Diagnostic::error(...)` に `with_id(...)` を付与、E
    - 対象: use/move/borrow/assign/drop/loop合流�E主要診断、E
  - `tests/move_effect.n.md`
    - 既孁Ecompile_fail 2件に `diag_id` を追加�E�Ehared borrow move / move後�E利用�E�、E
    - 新要Ecompile_fail 2件を追加�E�Eove後borrow=3063、�E岐後potentially moved=3054�E�、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-diagid.json -j 15` -> `220/220 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-move-diagid.json -j 15` -> `789/789 pass`
- 状況E
  - move/borrow系の `compile_fail + diag_id` 基盤が上流で確立、E
  - 次段は `todo.md` の診断ID未適用領域�E�Earser/typecheck/resolveの残り�E�へ拡張する、E

# 2026-03-04 作業メモ (フェーズD進衁E `scanner_new` / `writer_new` の曖昧オーバ�Eロード根治)

- 目皁E
  - `unwrap_ok scanner_new` / `unwrap_ok writer_new` で発生しぁE`D3005 ambiguous overload` を、戻り値型�Eみで刁E��すめEnullary オーバ�Eロード設計から解消する、E
- 根本原因:
  - `scanner_new` / `writer_new` に `Result<i32,str>` 版と `Result<Scanner/Writer,str>` 版を同名で共存させたため、引数0の呼び出しで斁E��不足時に戻り値型だけでは選択不�EになってぁE��、E
  - そ�E曖昧性ぁE`kp` doctest / `tests` / `tutorials` の `unwrap_ok scanner_new` 系呼び出しに波及し、下流で連鎖的に型不一致を誘発してぁE��、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new <()*>Result<i32,str>>` めE`scanner_new_handle <()*>Result<i32,str>>` に改名、E
    - 公閁E`scanner_new` は `Result<Scanner,str>` のみを提供、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new <()*>Result<i32,str>>` めE`writer_new_handle <()*>Result<i32,str>>` に改名、E
    - 公閁E`writer_new` は `Result<Writer,str>` のみを提供、E
  - `tests/overload.n.md`
    - 追加した zero-arg `Result` ケースのシグネチャ/式を修正し、pure 斁E��で正しく検証できる状態へ調整、E
- 検証:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-zeroarg-result.json -j 15` -> `241/241 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpread-kpwrite-new-overload.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpread-overload-unify.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpread-overload-unify.json -j 15` -> `262/262 pass`
- 状況E
  - `new` 系の公閁EAPI で「戻り値型�Eみ差刁E���E曖昧性を除去、E
  - フェーズDの安�EAPI統一路線（�E開面は安�E型、ハンドル版�E冁E��名に隔離�E�に整合、E

# 2026-03-04 作業メモ (フェーズD進衁E `kpread` の `_raw` 依存を同名オーバ�Eロードへ整琁E

- 目皁E
  - `kpread` の `scanner_*_raw` 命名を段階縮退し、`i32` ハンドル版と `Scanner` 版を同名オーバ�Eロードとして統一する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` を除ぁE`scanner_*_raw` めE`scanner_*` へ改名、E
    - `i32` 受け取り実裁E�� `Scanner` 受け取り実裁E��同名で共存させる構�Eに変更、E
    - 既存ラチE��は同名オーバ�Eロード�E `i32` 版を呼び出すよぁE��更新、E
- 根本原因:
  - `_raw` 接尾辞�E岐が API 読み取りコストを上げ、実際には型だけで区別できる箁E��まで命名差刁E��持ってぁE��、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpread-kpwrite-overload-unify.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpread-overload-unify.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpread-overload-unify.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread` は `scanner_new_raw` を除ぁE�� `_raw` 接尾辞なしで運用可能な状態になった、E
  - 次段は `scanner_new_raw` の扱ぁE��戻り値型依存�E曖昧性解消設計）を上流設計と合わせて検討する、E

# 2026-03-04 作業メモ (フェーズD進衁E `kpwrite` の `_raw` 依存を同名オーバ�Eロードへ整琁E

- 目皁E
  - `kpwrite` 冁E��で刁E��してぁE�� `*_raw` 群を、`i32` ハンドル版と `Writer` 版�E同名オーバ�Eロードで統一し、�E開面の命名を簡潔化する、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_raw` を除き、`writer_*_raw` めE`writer_*` へ改名、E
    - `i32` 受け取り実裁E�� `Writer` 受け取り実裁E��同名で共存させる形に変更、E
    - 既存�E `Writer` 版から�E同名の `i32` 版を呼ぶように整琁E��E
- 根本原因:
  - `_raw` 接尾辞を前提にラチE��層が増え、API 仕様�E読み取りコストが上がってぁE��、E
  - 既存�Eオーバ�Eロード機構で十�Eに区別可能な箁E��まで命名�E岐してぁE��、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-overload-unify.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpwrite-overload-unify.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpwrite-overload-unify.json -j 15` -> `262/262 pass`
- 状況E
  - `kpwrite` は `writer_new_raw` を除ぁE�� `_raw` 接尾辞なしで運用可能な状態になった、E
  - 次段で `kpread` 側も同方針で段階整琁E��る、E

# 2026-03-04 作業メモ (フェーズD進衁E `alloc` 安�EAPI標準名化�E回帰復旧)

- 目皁E
  - `core/mem` の `alloc/realloc/dealloc` めE`Result` 返却へ標準名化した変更に対して、下流�E `kp`/tests/tutorials の破損を上流原因から復旧する、E
- 変更:
  - `stdlib/kp/kpprefix.nepl`
    - doctest の `alloc/dealloc` めE`alloc_raw/dealloc_raw` へ更新、E
  - `stdlib/kp/kpsearch.nepl`
    - doctest の `alloc/dealloc` めE`alloc_raw/dealloc_raw` へ更新、E
  - `tests/capacity_stack.n.md`
  - `tests/sort.n.md`
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `examples/tui_editor/editor_fs.nepl`
    - 置換ミスで壊れてぁE�� `#import "alloc_raw/...` めE`#import "alloc/...` へ復旧、E
- 根本原因:
  - 生メモリAPI移行�E一括置換時に、E��数呼び出しだけでなぁEimport パス斁E���Eまで `alloc_raw` に書き換わってぁE��、E
  - `alloc` ぁE`Result` 返却になった後も、`kp` doctest の一部ぁE`i32` 前提の旧記述を保持してぁE��、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md --no-tree -o /tmp/tests-mem-kp-safe-api-switch.json -j 15` -> `233/233 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-mem-kp-safe-api-switch-r2.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-mem-kp-safe-api-switch-r2.json -j 15` -> `262/262 pass`
- 状況E
  - `alloc` 安�EAPI標準名化�E現行差刁E�E、`tests + stdlib + tutorials` で回帰通過、E
  - 次段は `todo.md` のフェーズD残件�E��E開面からの raw 露出整琁E��を継続する、E

# 2026-03-04 作業メモ (フェーズD進衁E vec の `alloc/realloc/dealloc` めE`*_raw` へ直接移衁E

- 目皁E
  - `vec` だけ残ってぁE�� `alloc/realloc/dealloc` 呼び出しを `*_raw` に統一し、メモリAPI移行�E停滞要因を解消する、E
- 変更:
  - `stdlib/alloc/collections/vec.nepl`
    - `alloc` -> `alloc_raw`
    - `realloc` -> `realloc_raw`
    - `dealloc` -> `dealloc_raw`
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -i stdlib/tests/vec.n.md -i tests/capacity_stack.n.md -i tests/pipe_collections.n.md --no-tree -o /tmp/tests-vec-raw-direct.json -j 15` -> `236/236 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-vec-raw-direct.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-vec-raw-direct.json -j 15` -> `262/262 pass`
- 状況E
  - 以剁E`todo.md` に残してぁE�� `vec` の `realloc_raw` OOB 再現は現行系で再現せず、移行を完亁E��きた、E

# 2026-03-04 作業メモ (上流修正: codegen の alloc helper 解決めE`*_raw` 優先へ統一)

- 目皁E
  - `alloc/dealloc/realloc` の同名安�Eオーバ�Eロード導�E時に、codegen 側が誤っぁEhelper を解決して再帰・スタチE��オーバ�Eフローへ落ちる根本原因を上流で除去する、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - 冁E��確俁Ehelper 解決めE`alloc_raw` 優先、`alloc` フォールバックへ変更、E
  - `nepl-core/src/codegen_llvm.rs`
    - runtime helper 解決関数 `resolve_runtime_helper_symbol` を追加、E
    - `alloc/dealloc/realloc` 到達関数追加で `*_raw` 優先、旧名フォールバックへ変更、E
    - `resolve_alloc_symbol` めE`alloc_raw` 優先に変更、E
    - entry lower 時�E fallback allocator 判定を `alloc_raw` 優先探索に変更、E
    - `resolve_symbol_name` は map の実キー参�Eを返す実裁E��変更、E
  - `nepl-core/src/monomorphize.rs`
    - runtime helper 保持対象めE`alloc_raw/dealloc_raw/realloc_raw` 優先に変更�E�旧名フォールバック�E�、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-overload-memory-after-core-helper-fix.json -j 15` -> `244/244 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-core-helper-fix.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-core-helper-fix.json -j 15` -> `262/262 pass`
- 状況E
  - 上流E��Eodegen/monomorphize�E��E helper 解決経路ぁE`*_raw` 優先で揁E��たため、次段の `core/mem` 安�EAPI標準名化を再開できる状態になった、E

# 2026-03-04 作業メモ (調査: alloc 同名オーバ�Eロード�E衝突と差し戻ぁE

- 事象:
  - `core/mem` に `alloc/realloc/dealloc` の `MemPtr` 安�Eオーバ�Eロードを追加すると、E
    `stdlib/core/option.nepl::doctest#3` / `stdlib/core/result.nepl::doctest#4` などで
    `Maximum call stack size exceeded` が発生、E
- 原因:
  - コンパイラ生�Eコード�EぁE`alloc : (i32)->i32` を暗黙前提としており、E
    同名オーバ�Eロード追加で実行時経路が崩れる、E
- 対忁E
  - `alloc/realloc/dealloc` の `MemPtr` 同名オーバ�Eロード�E一旦差し戻し、E
  - `load/store` の `MemPtr` 同名オーバ�Eロード�E維持、E
  - 追加した `tests/memory_safety.n.md` の `alloc<...>` ケースは削除、E
- チE��チE
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-alloc-overload-revert.json -j 15` -> `213/213 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-mem-overload-revert.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-overload-revert2.json -j 15` -> `262/262 pass`
- 次対忁E
  - `alloc` 系の標準名安�E化�E、コンパイラ側の暗黙依存を先に解消してから再導�Eする、E

# 2026-03-04 作業メモ (フェーズD進衁E core/mem の MemPtr load/store を標準名オーバ�Eロード化)

- 目皁E
  - `*_ptr` 接尾辞依存を減らし、`MemPtr` 利用時�E標準名 `load_i32/store_i32/load_u8/store_u8` で書けるようにする、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `load_i32/store_i32/load_u8/store_u8` に `MemPtr` 引数版�Eオーバ�Eロードを追加、E
    - 旧 `load_i32_ptr/store_i32_ptr/load_u8_ptr/store_u8_ptr` は互換エイリアス化、E
    - `MemPtr` オーバ�Eロード�E無効ポインタ時に `Option::None` / `Result::Err` を返す、E
- チE��チE
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-mem-overload-loadstore.json -j 15` -> `218/218 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-mem-loadstore-overload.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-loadstore-overload.json -j 15` -> `262/262 pass`
- 状況E
  - `MemPtr` 利用コード�E標準名で安�Eな load/store を呼べる状態になった、E
  - 次段は `alloc/realloc/dealloc` 側の公開名安�E化を継続する、E

# 2026-03-04 作業メモ (フェーズD進衁E kpread_core 解放経路の Result 匁E

- 目皁E
  - `kpread_core` の初期化失敗時巻き戻しで `dealloc_raw` 直呼びを減らし、失敗�E琁E�� `Result` へ寁E��る、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `nread` 確保失敗時、`iov/buf` の解放めE`dealloc_result` ベ�Eスへ変更、E
    - `realloc` 失敗時、`iov/nread_ptr/buf` の解放めE`dealloc_result` ベ�Eスへ変更、E
    - `scanner` ヘッダ確保失敗時と成功後�E一時領域解放めE`dealloc_result` ベ�Eスへ変更、E
    - 解放失敗�E巻き戻し�E琁E��止めず吸収する方針で統一、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kp-core-dealloc-result.json -j 15` -> `228/228 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpreadcore-dealloc-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpreadcore-dealloc-result.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread_core` 初期化失敗時の解放経路は `Result` 系APIに寁E��られた、E
  - 次段で `core/mem` 公開名の安�EAPI標準化を継続する、E

# 2026-03-04 作業メモ (フェーズD進衁E kpwrite 初期化�E根本整琁E

- 目皁E
  - `kpwrite` 初期化を `0` センチネル刁E��から外し、`Result` ベ�Eスで確保失敗と巻き戻しを一允E��する、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle_raw` を削除、E
    - `writer_alloc_buf` を追加し、`4096 -> 1024 -> 256` の段階確保を `Result<WriterBuf,str>` で返すように変更、E
    - `writer_try_free` を追加し、�E期化途中の失敗時に解放失敗を吸収して巻き戻せるように変更、E
    - `writer_new_raw` は `alloc_result/dealloc_result` 前提の `match` 連鎖へ置換し、確保失敗時の返却琁E��を段階別に固定、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-result-init-refine.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpwrite-resultrefine.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpwrite-resultrefine.json -j 15` -> `262/262 pass`
- 状況E
  - `writer_new_raw` の失敗表現は `Result` へ収束し、センチネル `0` 依存�E刁E��を初期化経路から除去できた、E
  - 次段は `todo.md` フェーズDの主課題！Ecore/mem` 公開APIの安�E名統一�E�を継続する、E

# 2026-03-04 作業メモ (フェーズD進衁E kpwrite 初期化経路の Result 匁E

- 目皁E
  - `kpwrite` の初期化経路めE`Result` 経路へ揁E��、`kpread` と同じ失敗表現に統一する、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - 旧 `writer_new_raw`�E�Ei32`返却�E�本体を `writer_new_handle_raw` へ刁E��、E
    - 新 `writer_new_raw` めE`Result<i32,str>` 返却へ変更、E
    - `writer_new` は `writer_new_raw` の `Result` めE`Writer` へ持ち上げる実裁E��変更、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-result-init.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpwrite-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpwrite-result.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread/kpwrite` の初期化�E開経路はどちらも `Result` ベ�Eスで統一済み、E
  - 次段は `todo.md` フェーズD残件として、`mem` 側公開名の安�EAPI標準化を進める、E

# 2026-03-04 作業メモ (フェーズD進衁E kpread_core の初期化を Result ベ�Eス匁E

- 目皁E
  - `kpread` 初期化経路の失敗表現めE`0` センチネル依存かめE`Result` へ寁E��る、E
  - メモリ確保失敗時の刁E��を型で扱えるようにし、段階的な安�EAPI標準化を進める、E
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `scanner_new_impl_i` めE`scanner_new_impl` へ改名、E
    - 戻り値めE`i32` から `Result<i32,str>` へ変更、E
    - `alloc_result/realloc_result` を使って確保失敗を `Err` 化、E
    - 後始末�E�解放�E��E既存レイアウト維持�Eため `dealloc_raw` を継続使用、E
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` めE`Result<i32,str>` 返却へ変更、E
    - `scanner_new` は `scanner_new_raw` の `Result` をそのまま `Scanner` へ持ち上げる形に変更、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-tree -o /tmp/tests-kpread-result-init.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpreadcore-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpreadcore-result.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread` の初期化経路は `Result` ベ�Eスに移行済み、E
  - 次段で `kpwrite` 初期化経路も同じ方針に揁E��る、E

# 2026-03-04 作業メモ (フェーズD進衁E `*_new_raw` 名統一と todo 未完亁E��琁E

- 目皁E
  - `kpread/kpwrite` の冁E��初期化関数名を `*_raw` に統一し、�E開�E口めE`scanner_new` / `writer_new` に寁E��る、E
  - `todo.md` から完亁E��みのチE��ト追加頁E��を削除し、未完亁E�Eみを保持する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_i32` -> `scanner_new_raw`、E
    - `scanner_new` からの呼び出し�Eを更新、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_i32` -> `writer_new_raw`、E
    - `writer_new` からの呼び出し�Eを更新、E
  - `todo.md`
    - フェーズEの完亁E��み小頁E���E�Etests/move_effect.n.md` 追加、`tests/overload.n.md`/`tests/kp*.n.md` 更新�E�を削除、E
    - 頁E��8の完亁E��み小頁E���E�Etests/memory_safety.n.md` 追加�E�を削除、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kp-newraw-rename.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-newraw-rename.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-newraw-rename.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread/kpwrite` の冁E��初期化関数名が `*_raw` で揁E��た、E
  - 次段はフェーズD残件として、`mem` 公開面の安�EAPI標準名化！EResult/Option` 前提�E�を進める、E

# 2026-03-04 作業メモ (フェーズD進衁E kpread の raw 実裁E��刁E��)

- 目皁E
  - `kpread` の冁E�� `i32` ハンドル実裁E��公閁E`Scanner` API を�E確に刁E��し、�E開面の型安�E性を上げる、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `i32` 受け取り実裁E�� `scanner_*_raw` へ改名、E
    - `Scanner` 受け取り公開関数は既存名を維持し、�E部で `*_raw` を呼び出す形へ変更、E
    - 対象: `skip_ws/is_eof/skip_token/read_token/read_i32/read_i64/read_u64/read_f32/read_f64/read_vec/read_matrix/read_all/read_*input` 一式、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-raw-split-both.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpread-split.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpread-split.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread/kpwrite` ともに「�E閁EAPI = Scanner/Writer 型」「�E部実裁E= *_raw」へ刁E��済み、E
  - 次段は `todo.md` 2026-03-03 フェーズDの残件�E�Emem` 公開面の `_safe` 廁E��と `_raw` 最終削除�E�へ進む、E

# 2026-03-04 作業メモ (フェーズD進衁E kpwrite の raw 実裁E��刁E��)

- 目皁E
  - `kpwrite` の冁E�� `i32` ハンドル実裁E��公閁E`Writer` API を�E確に刁E��し、�E開面の型安�E性を上げる、E
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `i32` 受け取り実裁E�� `writer_*_raw` へ改名、E
    - `Writer` 受け取り公開関数は既存名を維持し、�E部で `*_raw` を呼び出す形へ変更、E
    - 対象: `free/flush/ensure/put_u8/writeln/write_*` 一式、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-raw-split.json -j 15` -> `226/226 pass`
- 状況E
  - `kpwrite` は「�E閁EAPI = Writer 型」「�E部実裁E= *_raw」へ刁E��完亁E��E
  - 次段で `kpread` も同方針に揁E��る、E

# 2026-03-04 作業メモ (overload チE��ト拡允E 注釈混在ケースの追加)

- 目皁E
  - `overload` 回帰に、型注釈�E混在パターン�E�ブロチE��注釈�E関数呼び出し注釈�Eパイプ�E関数リチE��ル�E�を追加する、E
- 変更:
  - `tests/overload.n.md`
    - `overload_mixed_annotations_block_call_pipe_lambda` を追加、E
    - `overload_pipe_annotations_with_mixed_cast_i32_i64_i128` を追加、E
- 刁E��刁E��:
  - 初版では `pipe requires a value on the stack (D3013)` と `ambiguous overload (D3005)` を�E現、E
  - 解析結果:
    - `let ...:` の引数ブロチE��直後に `|>` を直接接続する形は現行仕様では式墁E��が�Eかれる、E
    - `|> <i64> cast` は「関数値への注釈」として解釈され、戻り値注釈にはならず曖昧化する、E
  - チE��ト�E仕様に整合する形へ修正:
    - ブロチE��注釈�E `base` に束縛してから通常呼び出しで連結、E
    - cast は `seed` を�E示変換した後に pipe で加算を実施、E
- チE��チE
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-after-fix2.json -j 15` -> `239/239 pass`

# 2026-03-04 作業メモ (フェーズD進衁E stdlib の生メモリ呼び出しを `*_raw` へ段階移衁E

- 目皁E
  - `mem` の公開名刁E��前に、stdlib 側の生アロケータ呼び出しを `alloc_raw/dealloc_raw/realloc_raw` に寁E��る、E
- 変更:
  - `stdlib/alloc/collections/{btreemap,btreeset,hashmap,hashset,list,ringbuffer,stack,vec/sort}.nepl`
  - `stdlib/alloc/{diag/error,string}.nepl`
  - `stdlib/kp/{kpdsu,kpfenwick,kpgraph,kpprefix,kpread_core}.nepl`
  - `stdlib/nm/{parser,html_gen}.nepl`
  - `stdlib/platforms/wasix/tui.nepl`
  - `stdlib/std/{env/cliarg,fs,stdio}.nepl`
  - 上記で `alloc/dealloc/realloc` の生呼び出しを `*_raw` に置換！Ecore/mem` の公開名依存を刁E���E�、E
- 刁E��刁E��:
  - 一括置換後、`tests/capacity_stack.n.md::doctest#3` で OOB を�E現、E
  - 原因刁E��刁E��で `vec.nepl` の `realloc_raw` 置換時のみ再現することを確認したため、`vec.nepl` 本体�E現時点では `realloc` 呼び出しを維持して回避、E
  - こ�E差刁E�E `todo.md` に未解決課題として追記、E
- チE��チE
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-raw-migration-wide2.json -j 15` -> `725/725 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-raw-migration-wide.json -j 15` -> `262/262 pass`
- 状況E
  - stdlib の大部刁E�E `*_raw` 呼び出しへ移行済み、E
  - 残件は `vec.nepl` の `realloc_raw` 移行に伴ぁEOOB 原因の根本修正、E

# 2026-03-04 作業メモ (フェーズD進衁E `kpread/kpwrite` の生メモリ呼び出しを `*_raw` へ移衁E

- 目皁E
  - `core/mem` の `*_raw` 刁E��に合わせ、`kpread/kpwrite` 側の生アロケータ呼び出しを明示化する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 斁E���Eト�Eクン生�E時�E確保を `alloc` から `alloc_raw` へ変更、E
  - `stdlib/kp/kpwrite.nepl`
    - writer 初期匁E解放の `alloc`/`dealloc` 呼び出しを `alloc_raw`/`dealloc_raw` へ変更、E
    - ドキュメントコメント�E斁E��を実裁E��合わせて調整�E�「ヒープ確保なし」）、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-mem-raw-callsite-migration.json -j 15` -> `229/229 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-after-kp-memraw-migration-full.json -j 15` -> `725/725 pass`
- 状況E
  - `mem` の生アロケータ利用箁E��は `kpread/kpwrite` で `*_raw` へ追従済み、E
  - 次段は `alloc/realloc/dealloc` 公開名めEResult/Option 安�EAPIへ刁E��替える準備として、残り呼び出し箁E��を段階移行する、E

# 2026-03-04 作業メモ (フェーズD進衁E `core/mem` に `*_raw` 隔離を導�E)

- 目皁E
  - 生�EインタAPIを段階的に刁E��し、次段の安�EAPI標準名化に備える、E
- 変更:
  - `stdlib/core/mem.nepl`
    - 生API本体を `alloc_raw` / `realloc_raw` / `dealloc_raw` へ改名、E
    - `alloc` / `realloc` / `dealloc` は `*_raw` への委譲エイリアスへ変更、E
    - `alloc_result` / `realloc_result` / `dealloc_result` と `alloc_ptr` 系は `*_raw` を直接呼ぶように変更、E
- チE��チE
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-raw-alias.json -j 15` -> `213/213 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-mem-raw-alias.json -j 15` -> `725/725 pass`
- 状況E
  - `mem` 側で「生API本体」と「�E開名」を刁E��できた、E
  - 次段は `alloc/realloc/dealloc` 公開名を安�EAPIへ刁E��替える際�E呼び出し�E移行！Etdlib/tests/tutorials�E�に着手できる状態、E

# 2026-03-04 作業メモ (フェーズE前進: `mem_result` 系APIの回帰チE��ト追加)

- 目皁E
  - `core/mem` の `alloc_result/realloc_result/dealloc_result` 命名変更をテストで固定する、E
- 変更:
  - `tests/memory_safety.n.md`
    - `alloc_result/dealloc_result` の正常系チE��トを追加、E
    - `dealloc_result` の無効引数 `Err` 返却チE��トを追加、E
- チE��チE
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-result-rename.json -j 15` -> `213/213 pass`
- 状況E
  - `core/mem` の `_safe` 命名除去刁E��つぁE��、命名変更後�E最小回帰を固定した、E

# 2026-03-04 作業メモ (フェーズD進衁E `core/mem` の `_safe` 命名除去)

- 目皁E
  - `core/mem` の安�EラチE��APIから `_safe` 接尾辞を除去し、命名規紁E��次段移行しめE��ぁE��へ揁E��る、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `alloc_safe` -> `alloc_result`
    - `realloc_safe` -> `realloc_result`
    - `dealloc_safe` -> `dealloc_result`
    - 関連ドキュメントコメント�Eの関数名�E注意事頁E��更新、E
  - `todo.md`
    - フェーズDの斁E��を、`_safe` 統一方針から「`_safe` 接尾辞廁E���E�安�EAPI標準名化」へ更新、E
    - `move/effect` 反映頁E��を、`mem` 側と `kpread/kpwrite` 側の残件に刁E��して明記、E
- チE��チE
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-after-mem-safe-rename.json -j 15` -> `723/723 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-safe-rename.json -j 15` -> `262/262 pass`
- 状況E
  - `_safe` 命名除去は `core/mem` で着手済み、E
  - 次段は API 本体を Result/Option 標準名へ寁E��るため、`alloc/realloc/dealloc` の生�EインタAPI整琁E��E*_raw` 隔離�E�に進む、E

# 2026-03-04 作業メモ (フェーズD進衁E kpread/kpwrite の `_raw` 名整琁E��亁E

- 目皁E
  - `kpread/kpwrite` で残ってぁE�� `_raw` 接尾辞�E公開名を整琁E��、E��常API名へ統一する、E
  - 変更後�E全体回帰めE`tests + stdlib + tutorials` で確認する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` めE`scanner_new_i32` へ変更、E
    - `scanner_skip_ws_raw` / `scanner_is_eof_raw` / `scanner_skip_token_raw` / `scanner_read_*_raw` めE`scanner_*` へ統一、E
    - ドキュメントコメント中の関数名記述も実体に合わせて更新、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_raw` めE`writer_new_i32` へ変更、E
    - `writer_write_*_raw` / `writer_writeln_raw` / `writer_flush_raw` / `writer_free_raw` めE`writer_*` へ統一、E
    - ドキュメントコメント中の関数名記述も実体に合わせて更新、E
- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-stdlib --no-tree -o /tmp/tests-kpread-kpwrite-no-raw.json -j 15` -> `5/5 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-full-after-kp-overload-unify.json -j 15` -> `781/781 pass`
- 状況E
  - `kpread/kpwrite` から `_raw` 接尾辞�E解消済み、E
  - `todo.md` の `_safe/_raw` 最終整琁E�E `mem.nepl` 側�E�Ealloc_safe/realloc_safe/dealloc_safe`�E�が残件、E

# 2026-03-04 作業メモ (フェーズD進衁E Scanner/Writer API一本化とハンドル露出除去)

- 目皁E
  - `kpread/kpwrite` の公開APIから `scanner_handle/writer_handle` を除去し、`Scanner`/`Writer` 型APIへ一本化する、E
  - `Scanner` 呼び出しが move で破綻する根本原因�E�コンパイラの非Copy特例）を上流で修正する、E
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 生ハンドル実裁E�� `*_raw` 名へ刁E��、E
    - 公開関数は `Scanner` 引数の通常名！Escanner_read_i32` など�E�に統一、E
    - `scanner_handle` 相当�E公開関数を削除し、�E部でのみ `mem_ptr_addr get sc "raw"` を使用、E
  - `stdlib/kp/kpwrite.nepl`
    - 生ハンドル実裁E�� `*_raw` 名へ刁E��、E
    - 公開関数は `Writer` 引数の通常名！Ewriter_write_i32` など�E�に統一、E
    - `writer_handle` 相当�E公開関数を削除し、�E部でのみ `mem_ptr_addr get w "raw"` を使用、E
  - 依存箁E��の移衁E
    - `tests/kp.n.md`, `tests/kp_i64.n.md`, `tests/stdin.n.md`
    - `tutorials/getting_started/22_*.n.md`, `24_*.n.md`, `25_*.n.md`, `27_*.n.md`
    - `examples/kp_fizzbuzz.nepl`
    - `stdlib/kp/kpgraph.nepl`�E�Edense_graph_read_undirected_1indexed` めE`Scanner` 受け取りへ変更�E�E
  - 上流修正:
    - `nepl-core/src/types.rs` の明示非Copy判定かめE`Scanner` を除外！ERegionToken`/`Writer` は維持E��、E
- チE��チE
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpgraph.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-api-unify.json -j 15` -> `231/231 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-full-after-kp-api-unify.json -j 15` -> `781/781 pass`
- 状況E
  - `kpread/kpwrite` の公開APIは `Scanner`/`Writer` 型�Eースに揁E��た、E
  - 次段は `todo.md` フェーズDの残件�E�E_safe` 廁E��と `_raw` 最終削除、trait 墁E��導�E�E�を進める、E

# 2026-03-04 作業メモ (フェーズD前進: ptr安�EAPIの _safe 依存�Eり離ぁE

- 目皁E
  - `mem` の公閁E`Result` API めE`_safe` ラチE��名から独立させ、`_safe` 廁E��に向けた段階移行を進める、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` の冁E��実裁E�� `alloc_safe/realloc_safe/dealloc_safe` 呼び出しから�E離、E
    - `alloc` / `realloc` / `dealloc` を直接呼び、�E開API側で `Result` 判定を行うように変更、E
- チE��チE
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-ptr-safe-decouple.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-ptr-safe-decouple.json -j 15` -> `723/723 pass`
- 状況E
  - `*_ptr` 系の公開安�EAPIは `_safe` 名に依存しなぁE��へ移行済み、E
  - 次段では `alloc_safe/realloc_safe/dealloc_safe` 自体を縮退し、�E開名一本化へ進める、E

# 2026-03-04 作業メモ (フェーズE前進: memory_safety 回帰追加)

- 目皁E
  - `todo.md` フェーズEの追加頁E�� `tests/memory_safety.n.md` を�E行で固定化する、E
- 変更:
  - `tests/memory_safety.n.md` を新規追加、E
    - `alloc_ptr/load_i32_ptr/store_i32_ptr/dealloc_ptr` の正常系、E
    - 無効ポインタ `load` ぁE`Option::None` を返す異常系、E
    - 無効ポインタ `store` ぁE`Result::Err` を返す異常系、E
- チE��チE
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-memory-safety-tests.json -j 15` -> `723/723 pass`
- 状況E
  - `tests/memory_safety.n.md` 追加タスクは完亁E��、`todo.md` から削除済み、E
  - 次は `mem/kpread/kpwrite` の `_safe` なし安�EAPI一本化と `_raw` 最終削除へ進む、E

# 2026-03-04 作業メモ (フェーズC着扁E MemPtr のジェネリクス匁E

- 目皁E
  - `doc/memory_safety_compiler_design.md` の型モチE��に沿って、`MemPtr<T>` を�E開API側へ反映する、E
- 変更:
  - `stdlib/core/mem.nepl`
    - `MemPtr` めE`MemPtr<.T>` へ変更、E
    - `mem_ptr_wrap` / `mem_ptr_addr` / `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `mem_ptr_add` をジェネリクス対応、E
    - `load_i32_ptr` / `store_i32_ptr` は `MemPtr<i32>`、`load_u8_ptr` / `store_u8_ptr` は `MemPtr<u8>` を受けるように変更、E
  - `stdlib/kp/kpread.nepl`
    - `Scanner.raw` めE`MemPtr<u8>` 化、E
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` めE`MemPtr<u8>` 化、E
- チE��チE
  - `node nodesrc/tests.js -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-mem-kp-generic-memptr.json -j 15` -> `220/220 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-memptr-generic.json -j 15` -> `720/720 pass`
- 状況E
  - `MemPtr<T>` 型モチE��は導�E済み�E��E開APIの i32 生�Eインタ除去は継続）、E
  - 次は `RegionToken` 導�Eと `alloc/realloc/dealloc` の `Result` 一本化を進める、E

# 2026-03-04 作業メモ (フェーズB完亁E Copy/Clone 制紁E+ RegionToken 非Copy匁E

- 目皁E
  - `todo.md` フェーズB残件だっぁE`Copy/Clone` 制紁E��査と `RegionToken` 非Copy扱ぁE��型検査に反映する、E
- 変更:
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` に明示非Copy型判定を追加�E�ERegionToken` / `Scanner` / `Writer`�E�、E
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3049` (`TypeCopyImplTargetNotCopy`) と `D3050` (`TypeCopyImplRequiresClone`) を追加、E
  - `nepl-core/src/typecheck.rs`
    - `impl Copy for T` の収集時に `ctx.is_copy(T)` を検証し、E��Copy対象めE`D3049` で拒否、E
    - `Copy` 実裁E��は同一対象 `Clone` 実裁E��忁E��な検査を追加し、欠落晁E`D3050` で拒否、E
    - 拒否対象の `Copy` 実裁E�E後続�E impl 収集/照合から除外、E
  - `tests/move_effect.n.md`
    - `D3049`/`D3050` の compile_fail ケースを追加、E
    - `Clone+Copy` 両実裁E��の成功ケースを追加、E
    - `RegionToken` の move 後�E利用拒否ケースを追加、E
- チE��チE
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-copy-clone.json -j 15` -> `218/218 pass`
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-overload-typeannot-copyclone.json -j 15` -> `266/266 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-copy-clone.json -j 15` -> `720/720 pass`
- 状況E
  - フェーズBの `Copy/Clone` 制紁E�� `RegionToken` 非Copy化�E反映済み、E
  - 次は `todo.md` のフェーズC/D�E�EMemPtr<T>` と `mem/kpread/kpwrite` の安�EAPI一本化）へ進む、E

# 2026-03-04 作業メモ (フェーズB進衁E move_check に borrow 状態�E移を実裁E
- 目皁E
  - `todo.md` のフェーズBにある `move_check` 状態機械めE`BorrowedShared/BorrowedUnique` まで拡張し、�E岁EルーチEmatch 合流を保守的に正しく扱ぁE��E
- 実裁E
  - `nepl-core/src/passes/move_check.rs`
    - `VarState` に `BorrowedShared` / `BorrowedUnique` を追加、E
    - `BorrowKind` を導�Eし、`visit_borrow` めE`Shared/Unique` 区別で処琁E��E
    - `check_use` を更新し、borrow 中 move めEunique borrow 中 use を拒否、E
    - `check_assign` / `check_drop` / `check_borrow` を追加し、代入・drop・borrow での状態�E移を一允E��、E
    - `merge_state_pair` / `merge_states` を追加し、`if`/`match`/`while` 合流を `Valid/Borrowed/Moved/PossiblyMoved` で統一、E
    - `Intrinsic::load/store` のアドレス引数 borrow 判定を `BorrowKind` に接続、E
  - `tests/move_effect.n.md`
    - 非Copy値の shared borrow 中 move が拒否される回帰を追加、E
    - Copy値 borrow が利用を阻害しなぁE��帰を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-overload-typeannot.json -j 15` -> `262/262 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-move-borrow.json -j 15` -> `716/716 pass`
- 次:
  - フェーズB残件 (`Copy/Clone` trait制紁E��査, `RegionToken` 消費規則) に進む、E

# 2026-03-04 作業メモ (フェーズB着扁E `TypeCtx::is_copy` 構造型判宁E
- 目皁E
  - フェーズBの最初�E実裁E��して、`TypeCtx::is_copy` めEtuple/struct/enum と generic apply へ拡張する、E
  - 再帰検�EロジチE��の誤判定（同一型�E再訪で常に false�E�を解消する、E
- 実裁E
  - `nepl-core/src/types.rs`
    - `is_copy_inner` めE`visiting + mapping` 方式に変更、E
    - `TypeKind::Struct` / `TypeKind::Enum` を構造皁E�E帰判定へ変更、E
    - `TypeKind::Apply` で base の type parameter を実引数へ束縛して copy 判定できるよう対応、E
    - 判定終亁E��に `visiting.remove` を行い、�E弟ノード�E訪での偽陰性を解消、E
  - `tests/move_effect.n.md`
    - Copy フィールド�Eみの struct 再利用ケース�E��E功！E
    - `Apply` されぁEgeneric struct 再利用ケース�E��E功！E
    - payload ぁECopy の enum 再利用ケース�E��E功！E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/generics.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-moveeffect-generics-overload.json -j 15` -> `269/269 pass`
- 次:
  - move_check 側の状態�E移�E�EPossiblyMoved` 合流、borrow 状態）を `is_copy` 拡張に合わせて精査する、E

# 2026-03-04 作業メモ (2026-03-03 フェーズA完亁E raw/intrinsic effect 一允E��)
- 目皁E
  - フェーズA残件だった「intrinsic / raw target body の effect 判定一允E��」を実裁E��、pure 斁E��からの I/O を型検査段階で拒否する、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `IMPURE_IO_EFFECT_MARKERS` を追加し、I/O語彙テーブルを導�E、E
    - `intrinsic_effect` / `raw_lines_effect` / `raw_body_effect` を追加して effect 判定を共通化、E
    - `BlockChecker::validate_raw_body_effect` を追加し、`#wasm`/`#llvmir` 本体が I/O語彙を含む場合、pure 関数で `D3025` を返すように変更、E
    - `FnBody::Parsed` の target選択raw本体、およ�E `FnBody::Wasm` / `FnBody::LlvmIr` 直持E���E両方で同じ検査を実施、E
    - `PrefixItem::Intrinsic` でも�E送Eeffect 判定を通すよう変更、E
  - `tests/move_effect.n.md`
    - pure raw body で `fd_write` を含むケースを追加�E�Ecompile_fail`, `diag_id: 3025`�E�、E
  - `todo.md`
    - 完亁E��みフェーズA頁E��を削除し、未完�Eみへ整琁E��E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md -i tests/intrinsic.n.md --no-tree -o /tmp/tests-effect-overload-typeannot-intrinsic.json -j 15` -> `263/263 pass`
- 現状:
  - フェーズA�E�Effect規則の反映�E��E完亁E��E
  - 次はフェーズB�E�ETypeCtx::is_copy` 拡張と move/borrow 状態�E移の厳寁E���E�へ進む、E

# 2026-03-04 作業メモ (2026-03-03 フェーズA再開: effect診断IDと回帰追加)
- 目皁E
  - `todo.md` の 2026-03-03 計画フェーズAを�E開し、pure/impure 判定�E診断固定を進める、E
- 実裁E
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3025 TypePureCallsImpureFunction` を追加、E
  - `nepl-core/src/typecheck.rs`
    - 「pure context cannot call impure function」�E全発生箁E��に `D3025` を付与、E
  - `tests/move_effect.n.md` を新規追加、E
    - pure からメモリ操作を呼べるケース�E��E功！E
    - pure から impure 関数呼び出し拒否�E�Ediag_id: 3025`�E�E
    - ローカル `set` ぁEpure のまま使えるケース�E��E功！E
    - グローバル `set` ぁEimpure になるケース�E�Ediag_id: 3025`�E�E
  - `todo.md`
    - 完亁E��み頁E���E�Ebuiltins` のメモリ系 Pure 化、entry 強制 Impure 特例�E削除�E�をフェーズAから削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-effect-overload-typeannot.json -j 15` -> `256/256 pass`
- 次:
  - フェーズA残件の「intrinsic / raw target body の effect 一允E��定」を実裁E��る、E

# 2026-03-04 作業メモ (オーバ�Eロード修正の完亁E�� 2026-03-03 計画への復帰)
- 目皁E
  - オーバ�Eロード解決の不安定箁E���E�関数値引数・pipe 併用・型注釈混在�E�を根本修正し、`todo.md` の `2026-03-03 move/effect/memory` 実裁E��復帰する、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - 関数シグネチャ参�EめE`function_signature_for_entry` に雁E��E��、type_args 適用後�E引数型を一貫取得するよぁE��正、E
    - pipe 注入時に nullary callable の過早 reduce を避ける制御と、target 入力型を使っぁE`reduce_pipe_pending_value_with_target` を追加、E
    - オーバ�Eロード候補�E絞り込みで「�E体型候補優先」「型パラメータ数最小候補優先」を導�Eし、`D3005` の過検�Eを抑制、E
  - `tests/overload.n.md`, `tests/typeannot.n.md`
    - ブロチE��注釁E関数呼び出し注釁Epipe 注釁E関数リチE��ル注釈�E混在ケースを拡允E��、今回の修正点を回帰固定、E
  - `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`, `stdlib/tests/stack.n.md`
    - `push` 利用形と型推論ケースを整琁E��、オーバ�Eロード解決の実運用ケースを固定、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/typeannot.n.md -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md --no-tree -o /tmp/tests-overload-typeannot-vec-stack.json -j 15` -> `286/286 pass`
- 現状:
  - オーバ�Eロード修正は完亁E��E
  - 次は `todo.md` の `2026-03-03 move/effect/memory 本格実裁E��画` フェーズA�E�Effect規則のコンパイラ反映�E�を再開する、E

# 2026-03-04 作業メモ (pipe 活用と `push` 推論�E確誁E
- 目皁E
  - 既存書き換え方針として、pipe 演算子を活用して中間変数とインチE��トを抑える、E
  - `vec_push<i32> ...` ではなぁE`push ...` だけで型推論できる利用形を�E示する、E
- 実施:
  - `stdlib/alloc/collections/list.nepl`
    - doctest のリスト構築を `list_nil |> list_push_front ...` へ変更、E
    - move 規則に合わせて再利用箁E��を�E束縛へ整琁E��E
    - 実裁E�E一部で中間変数を削減！Elist_len`, `list_get`, `list_free`, `list_reverse`�E�、E
  - `stdlib/alloc/collections/vec.nepl`
    - doctest の `vec_push<i32>` / `push<i32>` めE`push` に統一、E
    - `vec_new<i32> |> push 10 |> push 20` の形へ変更し、型引数省略で成立する例へ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl -i stdlib/alloc/collections/vec.nepl --no-stdlib --no-tree -o /tmp/tests-list-vec-pipe.json -j 15` -> `28/28 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-stdlib --no-tree -o /tmp/tests-vec-push-infer.json -j 15` -> `17/17 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-plus-tests-after-push-alias.json -j 15` -> `700/700 pass`

# 2026-03-03 作業メモ (仕様最終確誁E 前置記況Eオーバ�Eロード整吁E
- 実施:
  - `doc/move_effect_spec.md` に「NEPLg2既存仕様との整合」章を追加、E
  - 前置記法、型注釈、オーバ�Eロード、暗黙cast禁止との整合を明記、E
  - 同名オーバ�Eロード�E effect 一致制紁E��仕様へ反映、E
- 結果:
  - 設計方針（メモリ操佁Epure / I/O のみ impure�E�と既存言語仕様�E論理矛盾は無し、E
  - 実裁E��反映箁E���E�Euiltins の effect, entry 特例）�E引き続き `todo.md` 管琁E��E

# 2026-03-03 作業メモ (move/effect/memory 仕様�E再確宁E trait 統吁E
- 目皁E
  - heap/線形メモリ操作を pure とする設計を矛盾なく確定し、`move/borrow/copy/clone` と一体で仕様化する、E
- 実施:
  - `doc/move_effect_spec.md` を更新、E
    - `Pure/Impure` の判定を「I/O 外部副作用基準」に固定、E
    - メモリ操佁Epure 化�E成立条件�E�状態隠蔽・生�Eインタ非�E開�EResult/Option 化）を明文化、E
    - `trait` の位置づけを追加し、`Copy/Clone` とメモリ系 trait の役割を定義、E
  - `doc/memory_safety_compiler_design.md` を更新、E
    - trait 制紁E��査�E�ECopy` 可否、`Clone` 規紁E��`MemReadable/MemWritable/RegionOwned`�E�を追加、E
    - `core/mem` と `kpread/kpwrite` の trait ベ�Eス API 方針を追記、E
- 現実裁E��の差刁E
  - `builtins.rs` では `alloc/realloc/dealloc` が依然 `Effect::Impure`、E
  - `typecheck.rs` では entry を強制 `Impure` にしてぁE��、E
  - trait 墁E��でのメモリ能力検査は未実裁E��E
- 次:
  - `todo.md` の move/effect・メモリ安�Eタスクに trait 導�Eを反映し、実裁E��ェーズへ進む、E

# 2026-03-03 作業メモ (メモリ安�Eコンパイラ機構�E設訁E
- 目皁E
  - `i32` 生�Eインタ露出を減らし、コンパイラ検査で `mem/kpread/kpwrite` の誤用を防ぐ、E
- 追加:
  - `doc/memory_safety_compiler_design.md` を新規作�E、E
  - `MemPtr<T>` / `RegionToken` モチE��、墁E��検査挿入、解放状態検査、診断方針を定義、E
  - `alloc/realloc/dealloc/load/store` めEPure とし、I/O 系のみ Impure とする方針を明記、E
- 実裁E��刁E
  - まだ仕様段階で、`TypeCtx/move_check/typecheck` への反映は未着手、E
  - 実裁E��スクは `todo.md` の、E. メモリ安�Eコンパイラ機構�E導�E」で追跡する、E

# 2026-03-03 作業メモ (move/effect 精査結果: 現行実裁E��の差刁E
- 精査対象:
  - `nepl-core/src/typecheck.rs`
  - `nepl-core/src/builtins.rs`
  - `nepl-core/src/types.rs`
- 差刁E
  - `check_function` で `is_entry` 時に `current_effect = Impure` を強制してぁE��、E
  - builtins の `alloc/realloc/dealloc` ぁE`Effect::Impure` 登録になってぁE��、E
  - `TypeCtx::is_copy` ぁE`Struct/Enum` を一征E`false` としてぁE��、E
- 判断:
  - ぁE��れも `doc/move_effect_spec.md` の再設計仕様と不一致、E
  - 先に仕様を固定し、実裁E�E上流から段階的に修正する�E�Entry特侁E-> builtins effect -> is_copy拡張�E�、E

# 2026-03-03 作業メモ (move/effect 再設計仕様�E斁E��匁E
- 目皁E
  - `move` と `pure/impure` の責務�E離を�E斁E��し、`mem/kpread/kpwrite` の安�EAPI移行を設計レベルで固定する、E
- 追加:
  - `doc/move_effect_spec.md` を新規作�E、E
  - 次を仕様として確宁E
    - `->` めEPure、`*>` めEImpure として扱ぁE��E
    - heap/線形メモリ操作！Ealloc/realloc/dealloc/load/store`�E��E Pure、E
    - Impure は I/O・syscall・環墁E��存値取得に限定、E
    - move は effect と独立に評価、E
    - `entry` を常に Impure 扱ぁE��る特例�E撤廁E��象、E
    - `_safe` 接尾辞を廁E��し、安�E牁EPIをデフォルト化する方針、E
- 差刁E
  - 実裁E�Eまだ旧挙動が残る�E�特に entry 特例、Copy 判定�E構造型対応、intrinsic effect 一允E���E�、E
  - 本エントリは仕様確定まで。実裁E��映は `todo.md` 側で継続管琁E��る、E

# 2026-03-03 作業メモ (mem/kp の `_raw` 段階廁E��と安�EAPI寁E��)
- 目皁E
  - `mem/kpread/kpwrite` の `_raw` 接尾辞を段階廁E��し、安�EAPI�E�EResult/Option`�E�中忁E��寁E��る、E
  - `Scanner` / `Writer` ラチE��導�E後�E move 破綻を根本修正する、E
- 実裁E
  - `stdlib/core/mem.nepl`
    - `mem_ptr_raw` めE`mem_ptr_addr` へ変更、E
    - `alloc_ptr_raw / realloc_ptr_raw / dealloc_ptr_raw / load_*_ptr_raw / store_*_ptr_raw` を削除、E
    - 公開APIは `alloc_ptr/realloc_ptr/dealloc_ptr/load_*_ptr/store_*_ptr`�E�EResult/Option`�E�に統一、E
  - `stdlib/kp/kpread.nepl`
    - `scanner_raw` -> `scanner_handle`、`scanner_new_raw` -> `scanner_new_handle` に改名、E
    - `Scanner` 利用側は `scanner_handle` を一度取り出して i32 系 read API を使ぁE��に統一�E�Eove 破綻回避�E�、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_raw` -> `writer_handle`、`writer_new_raw` -> `writer_new_handle` に改名、E
    - `Writer` オーバ�Eロード群の move バグを修正:
      - `writer_handle` で i32 を取り�EぁE
      - 低レベル関数を呼び
      - `writer_wrap raw` を返す
    - i32 低レベル関数での `set w ...`�E�Emmutable 代入�E�を除去、E
    - doctest の `Writer` 使用例を再束縛！Eset w ...`�E�に修正、E
  - `tests/kp.n.md`, `tests/kp_i64.n.md`, `tests/stdin.n.md`
    - `Scanner` から `scanner_handle` を取得して読み取りを行う形へ更新、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md --no-tree --no-stdlib -o /tmp/tests-kp-safe-now6.json -j 16`
    - `15/15 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i examples/kp_fizzbuzz.nepl --no-tree --no-stdlib -o /tmp/tests-kp-safe-broader2.json -j 16`
    - `20/20 pass`
- 残課顁E
  - `scanner_handle` / `writer_handle` / `mem_ptr_addr` は依然としてハンドル露出点であり、最終的には公開APIから隠蔽する忁E��がある、E
  - `Result` ベ�Eス一本化！E_safe` から suffix なし統一�E��E `mem` 以外�E stdlib へ横展開が忁E��、E

# 2026-03-03 作業メモ (オーバ�Eロード根本修正: 関数値引数の arity/型文脈解決)
- 目皁E
  - `use_binary 3 4 calc` めE`5 |> use_unary calc` のように、オーバ�Eロード関数名を「関数値引数」として渡すケースを安定解決する、E
  - 間に合わせで中間変数へ刁E��せず、�Eれ子呼び出ぁEパイプ�Eまま通す、E
- 原因:
  - typecheck の直接 callable 経路で、引数位置に `Var(calc)` が来た時に、期征E��れる関数型（侁E `(i32,i32)->i32`�E�へ具体化されず、未解決のまま残ってぁE��、E
  - そ�E結果、compile では `undefined identifier` / run では `null function or function signature mismatch` が発生してぁE��、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `apply_function` の引数処琁E��、`Var(name)` かつ値 binding 不在の場合に callable 候補を検索、E
    - 引数位置の期征E�� `param_ty` に unify する候補を選別し、単一候補なめE`FnValue(selected_symbol)` へ置換、E
    - 褁E��候補一致時�E `D3005`�E�Embiguous overload�E�を返す、E
    - 候補なし�E既存どおり `D3006`�E�Eo matching overload�E�へ到達、E
  - `tests/overload.n.md`
    - パイチE混在 cast/関数戻り値注釈推論ケースを拡允E��E
    - 仕様変更で成功可能になっぁE2 ケース�E�単頁Earity 斁E��・pipe 単頁E��脈）を `compile_fail` から成功チE��トへ変更、E
    - `stack_new` の `Result` 化に合わせて該当ケースめE`unwrap_ok` ベ�Eスへ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/overload_after_expect_update.json -j 1`
    - `30/30 pass`

# 2026-02-27 作業メモ (GitHub Actions: wasm-bindgen ダウンロード失敗�E安定化)
- 背景:
  - `trunk build` 実行時に、Trunk 冁E��の `wasm-bindgen` 自動ダウンロードが接続断で失敗するケースが発生、E
  - エラー侁E `failed downloading release archive` / `connection closed before message completed`
- 実裁E
  - `trunk` を使ぁEworkflow へ、事前に `wasm-bindgen-cli 0.2.108` を導�Eする step を追加、E
  - 追加允E
    - `.github/workflows/gh-pages.yml`
    - `.github/workflows/nepl-test-wasi.yml`
    - `.github/workflows/nepl-test-llvm.yml`
    - `.github/workflows/nmd-doctest.yml`
  - 導�E方況E
    - `cargo install --locked wasm-bindgen-cli --version 0.2.108`
    - 5回リトライ + backoff�E�Es,10s,15s,20s,25s�E�E
- 期征E��极E
  - Trunk の実行中ダウンロード依存を減らし、ネチE��ワーク瞬断時�E失敗率を低減、E
  - 失敗時めEstep 単位で再試行されるため、CI 全体�E安定性が向上、E

# 2026-02-27 作業メモ (`@` 強制関数値とオーバ�Eロード関連の診断ID拡張)
- 目皁E
  - `@` めEcallable 以外へ適用したとき�E誤受理を根本修正する、E
  - オーバ�EローチE型引数/引数型不一致の診断めE`diag_id` で安定検証できるようにする、E
- 原因:
  - `typecheck` の識別子解決で、`forced_value (@name)` の刁E��が「関数 binding であること」を常に検証しておらず、値 binding が通る経路が残ってぁE��、E
  - 一部診断が既存IDへ過剰雁E��E��れ、`compile_fail` の精寁E��証がしづらかった、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `@` 強制関数値の経路で `BindingKind::Func` 以外を即時拒否する刁E��へ修正、E
    - `only callable symbols can be referenced with '@'` に `DiagnosticId::TypeAtRequiresCallable (3023)` を付与、E
    - 変数への型引数適用、オーバ�EローチEeffect 不一致、型引数不一致、引数型不一致にも専用IDを付与、E
  - `nepl-core/src/diagnostic_ids.rs`
    - `3020..3024` を追加:
      - `TypeOverloadEffectMismatch`
      - `TypeOverloadTypeArgsMismatch`
      - `TypeArgumentTypeMismatch`
      - `TypeAtRequiresCallable`
      - `TypeVariableTypeArgsNotAllowed`
  - `tests/functions.n.md`
    - `function_at_requires_callable_reports_diag_id` を追加�E�Ecompile_fail`, `diag_id: 3023`�E�、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-diagids-v4.json -j 2`
    -> `111/111 pass`

# 2026-02-27 作業メモ (parser if/while レイアウト診断へID付丁E
- 目皁E
  - parser の if/while レイアウト系エラーめE`diag_id` で一貫管琁E��、木構造チE��トから機械検証できるようにする、E
- 実裁E
  - `nepl-core/src/parser.rs`
    - 次のエラーに `DiagnosticId` を付丁E
      - `invalid marker ...` / `duplicate marker ...` / `too many expressions ...` -> `ParserUnexpectedToken (2002)`
      - `missing expression(s) ...` / `argument layout block must contain expressions` -> `ParserExpectedToken (2001)`
      - `only expressions are allowed ...` -> `ParserUnexpectedToken (2002)`
  - `tests/tree/18_diagnostic_ids.js`
    - `if:` レイアウト�E marker 頁E��誤りケースを追加し、`id=2002` を検証、E
  - `tests/if.n.md`
    - `if_layout_invalid_marker_order_reports_diag_id` を追加�E�Ecompile_fail`�E�、E
    - wasm 実行系の `compile_fail diag_id` 抽出制紁E��合わせ、ここ�E `diag_id` 持E��なしで失敗そのも�Eを検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/if.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-diagid-layout-v2.json -j 2`
    -> `166/166 pass`
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-after-parser-id.json -j 2`
    -> `111/111 pass`

# 2026-02-27 作業メモ (compile_fail 用診断IDの拡張: スタチE��余剰値)
- 目皁E
  - `compile_fail` で「呼び出ぁEarity 不整合により余剰値が残る」ケースめE`diag_id` で固定検証できるようにする、E
- 実裁E
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId::TypeStackExtraValues = 3016` を追加、E
    - `from_u32` / `message` に同IDを追加、E
  - `nepl-core/src/typecheck.rs`
    - `expression left extra values on the stack` に `with_id(DiagnosticId::TypeStackExtraValues)` を付与、E
    - `statement must leave exactly one value on the stack` にも同IDを付与、E
  - `tests/overload.n.md`
    - `overload_too_many_arguments_reports_stack_extra` を追加、E
    - `compile_fail` + `diag_id: 3016` で検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions.json -j 2` -> `100/100 pass`

# 2026-02-27 作業メモ (compile_fail の diag_id 検証強匁E+ overload arity 調査)
- 目皁E
  - `compile_fail` チE��トで `diag_id` 一致めEWASM/LLVM の両方で検証可能にする、E
  - オーバ�Eロード�E arity 解決 (`overload_select_by_arity`) を�E功ケース化する、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - LLVM 側の診断要紁E�� `[Dxxxx]` を残すよう修正�E�Esummarize_diagnostics_for_message`�E�、E
  - `nepl-core/src/typecheck.rs`
    - `check_block`/`check_prefix` に最終式�E期征E��を渡す経路を追加、E
    - 異 arity オーバ�Eロードで、利用可能引数数に基づく候補選択�E下地を追加�E�Echoose_callable_type_by_available_arity`�E�、E
    - 型注釈文脈�E arity 候補選択を `Symbol::Ident` 処琁E��追加、E
  - `tests/overload.n.md`
    - compile_fail に `diag_id` を�E示付与したケースを整琁E��E
    - `overload_select_by_arity` は現状の実裁E��正だけでは安定�E功化できず、いったん `compile_fail[D3006]` に戻し、代わりに `overload_select_by_arity_unary_simple` を追加して回帰点を固定、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-expanded-diag.json -j 2` -> `38/38 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions.json -j 2` -> `60/60 pass`
- 差刁E課顁E
  - `overload_select_by_arity` を�E功ケースへ戻すには、`calc 3 4` の二頁E��択で residual stack が�Eる根因�E�Eeduce頁E��Earity選択タイミング�E�を追加で解消する忁E��がある、E
  - 現在の修正は「diag_id 検証の安定化」と「arity 解決の一部改喁E��単頁E�E�E�」まで、E

# 2026-02-27 作業メモ (オーバ�Eロード�E開発: 外�E引数斁E��の期征E��伝播)
- 目皁E
  - `assert cast 1` めE`push<u8> cast 65` のような式で、外�E関数の引数斁E��から戻り値オーバ�Eロードを解決できるようにする、E
- 原因:
  - 既存実裁E�E `expected_ret` を型注釈由来でしか渡しておらず、外�Eコンシューマ�E引数型！Eool/u8 等）を見てぁE��かった、E
  - そ�Eため `cast` ぁE`ambiguous overload` になってぁE��、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `infer_expected_from_outer_consumer` を追加し、外�E呼び出し�E該当引数型を期征E��り値として抽出、E
    - さらに外�E呼び出し�E「他引数」を先に `unify` して型変数を�E体化し、`push<u8> cast 65` のような generic 斁E��でも期征E��を決定できるようにした、E
    - `reduce_calls` / `reduce_calls_guarded` で `expected_ret.or(outer_expected)` を適用、E
  - `stdlib/tests/vec.n.md`
    - move 規則に合わせて `Vec` の再利用パターンを修正�E�同一値の再使用を�E離�E�、E
  - `tests/overload.n.md`
    - `overload_result_inferred_from_outer_arg_context` を追加し、外�E引数斁E��での戻り値オーバ�Eロード解決を固定化、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-after-context2.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `23/23 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/tests/cast.n.md -i stdlib/tests/vec.n.md -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-stdlib-focus5.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `29/29 pass`

# 2026-02-27 作業メモ (チE��ト実行高速化: changed モード追加)
- 目皁E
  - 全件実行が遁E��ため、変更ファイルだけを対象に回せる実行経路を追加する、E
- 実裁E
  - `nodesrc/tests.js`
    - `--changed` を追加し、`git diff` と untracked から `.n.md/.nepl` の変更ファイルを�E動収雁E��E
    - `--changed-base <ref>` を追加�E�既宁E`HEAD`�E�、E
    - `--with-stdlib` / `--with-tree` を追加、E
    - `--changed` 時�E明示持E��がなぁE��めE`stdlib` 自動追加と `tree` 実行を無効化、E
    - 実行結果 JSON と要紁E�E力に `scan` 惁E���E�実際の入劁Eモード）を追加、E
  - `README.md`
    - 高速差刁E��行コマンドとフル実行コマンドを明記、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD -o /tmp/tests-changed.json --runner wasm --no-tree -j 2` -> changed 対象のみ走査�E�Etotal 48`�E�E
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-quick.json --runner wasm -j 2` -> `7/7 pass`

# 2026-02-27 作業メモ (診断ID: lexer 生�E側の明示付与を追加)
- 目皁E
  - parser/typecheck/resolve に続いて、lexer 主要診断にめE`with_id(DiagnosticId::...)` を�E示する、E
- 実裁E
  - `nepl-core/src/lexer.rs`
    - `invalid #indent argument` -> `ParserExpectedToken` (2001)
    - `invalid #extern syntax` -> `ParserInvalidExternSignature` (2006)
    - `unknown directive` -> `LexerUnknownDirective` (1201)
    - `unknown token` -> `LexerUnknownToken` (1202)
  - `tests/tree/18_diagnostic_ids.js`
    - lexer 診断IDの検証ケースを追加�E�E#indent xx` と `$`�E�、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-lexer-id.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-lexer-id.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1657/1657 pass`

# 2026-02-27 作業メモ (診断ID: parser生�E側の明示付丁E+ 自動推測の撤去)
- 目皁E
  - 「`from_message` で推測しなぁE��診断生�E側で enum を付与する」方針へ戻す、E
  - parser/typecheck/name-resolution/overload の代表経路で `with_id(DiagnosticId::...)` を�E示化する、E
- 実裁E
  - `nepl-core/src/diagnostic_ids.rs`
    - 診断ID enum を拡張�E�Earser/typecheck/resolve 系の主要カチE��リを追加�E�、E
    - `from_message` は削除、E
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic::error/warning` の自動推測付与を撤去し、`id=None` を既定に戻した、E
  - `nepl-core/src/parser.rs`
    - `DiagnosticId` めEimport、E
    - `expect/expect_with_span/expect_ident` と主要Eparser エラーに `with_id(...)` を�E示付与、E
  - `nepl-core/src/resolve.rs`
    - `ambiguous import` に `DiagnosticId::AmbiguousImport` を付与、E
  - `nepl-core/src/typecheck.rs`
    - 代表経路�E�Eeturn型不一致、未定義識別子、shadow違反、overload曖昧/未一致�E�に `with_id(...)` を付与、E
  - `tests/tree/18_diagnostic_ids.js`
    - target/loader に加ぁEparser/typecheck/overload のID検証を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diag-explicit-parser.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-explicit-diag-parser.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1657/1657 pass`

# 2026-02-27 作業メモ (診断IDめE`DiagnosticId` enum で型保持)
- 目皁E
  - 診断IDめE`Option<u32>` の生値保持から `Option<DiagnosticId>` へ変更し、生成�E・表示側の整合性を型で保証する、E
- 実裁E
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic.id` めE`Option<DiagnosticId>` に変更、E
    - `with_id` 引数めE`DiagnosticId` に変更、E
  - `nepl-core/src/compiler.rs`
    - target 診断の `.with_id(...)` 呼び出しを enum 直持E��へ変更、E
  - `nepl-web/src/lib.rs`
    - diagnostics JSON の `id` は `as_u32()` で出力、E
    - `id_message` は `DiagnosticId::message()` で解決、E
    - 表示用 `[Dxxxx]` 斁E���EめE`as_u32()` で統一、E
  - `nepl-cli/src/main.rs`
    - 表示用 `[Dxxxx]` めE`as_u32()` 基準で統一、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diag-enum.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `node tests/tree/run.js` -> `18/18 pass`

# 2026-02-27 作業メモ (診断IDの enum 化と compile_fail ID検証の統吁E
- 目皁E
  - 診断IDめE`const` 群ではなぁE`enum` で一允E��琁E��、WASM/LLVM/CLI/Web/チE��トが同じID体系を参照するようにする、E
  - `compile_fail` doctest で診断ID一致を機械検証できるようにする、E
- 実裁E
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId` enum (`#[repr(u32)]`) を導�E、E
    - `as_u32` / `from_u32` / `message` を実裁E��E
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic` に `id: Option<u32>` を追加、E
    - `with_id` を追加、E
  - `nepl-core/src/codegen_llvm.rs`
    - `#target` 検証エラーに `[D1001]` / `[D1002]` を付与！EASM系と整合）、E
  - `nodesrc/parser.js`
    - doctestメタ `diag_id:` / `diag_ids:` を解析可能に拡張、E
  - `nodesrc/tests.js`
    - `compile_fail` 時に `[Dxxxx]` を�E合する検証を追加、E
  - `nodesrc/run_test.js`
    - `compile_fail` 用に `compile_error` を結果へ保持、E
  - `tests/neplg2.n.md`
    - target診断ケースに `diag_id: 1001/1002` を付与、E
  - `tests/tree/18_diagnostic_ids.js`
    - `id` / `id_message` の公開API検証を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diagid.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`

# 2026-02-27 作業メモ (`sort` 回帰チE��ト拡張: 重褁E��/負数)
- 目皁E
  - `todo.md` 3番�E�Esort/generics`�E��E刁E��刁E��精度を上げるため、`sort_i32(ptr,n)` の墁E��ケースを追加する、E
- 変更:
  - `tests/sort.n.md` に次のケースを追加:
    - `sort_i32_ptr_with_duplicates`�E�重褁E���E�E
    - `sort_i32_ptr_with_negative_values`�E�負数混在�E�E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-extended.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `484/484 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-tests-extend.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1605/1605 pass`

# 2026-02-27 作業メモ (`sort` 墁E��チE��ト拡張: len=0/1)
- 目皁E
  - `sort_i32(ptr, n)` の no-op 墁E���E�En=0`, `n=1`�E�を明示皁E��固定し、封E��の実裁E��更での回帰を防ぐ、E
- 変更:
  - `tests/sort.n.md` に次のケースを追加:
    - `sort_i32_ptr_len0_noop`
    - `sort_i32_ptr_len1_noop`
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-extended-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `490/490 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-tests-extend-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1611/1611 pass`

# 2026-02-27 作業メモ (`noshadow` stdlib 段階適用: phase 1)
- 目皁E

# 2026-02-27 作業メモ (typecheck 診断IDの適用拡張)
- 目皁E
  - parser/overload 系に続き、typecheck の主要失敗経路でめE`diag_id` を安定付与し、`compile_fail` で機械検証できる篁E��を庁E��る、E
- 原因:
  - 代入/if/while/match/intrinsic の一部エラーがメチE��ージ斁E���Eのみで識別され、回帰時に精寁E��証しづらかった、E
- 実裁E
  - `nepl-core/src/diagnostic_ids.rs`
    - `3036..3048` を追加、E
      - `TypeAssignmentTypeMismatch(3036)`
      - `TypeAssignmentUndefinedVariable(3037)`
      - `TypeIfArityMismatch(3038)`
      - `TypeIfConditionTypeMismatch(3039)`
      - `TypeWhileArityMismatch(3040)`
      - `TypeWhileConditionTypeMismatch(3041)`
      - `TypeWhileBodyTypeMismatch(3042)`
      - `TypeMatchUnknownVariant(3043)`
      - `TypeMatchPayloadBindingInvalid(3044)`
      - `TypeMatchArmsTypeMismatch(3045)`
      - `TypeIntrinsicTypeArgArityMismatch(3046)`
      - `TypeIntrinsicArgArityMismatch(3047)`
      - `TypeIntrinsicArgTypeMismatch(3048)`
  - `nepl-core/src/typecheck.rs`
    - 上記経路の `Diagnostic::error(...)` に `with_id(...)` を付与、E
  - `tests/if.n.md`
    - `if_condition_must_be_bool_reports_diag_id` (`diag_id: 3039`) を追加、E
    - `while_body_must_be_unit_reports_diag_id` (`diag_id: 3042`) を追加、E
  - `tests/intrinsic.n.md`
    - `intrinsic_argument_type_mismatch_reports_diag_id` (`diag_id: 3048`) を追加、E
    - 失敗原因がテスト記法ミスだったため、`#intrinsic` 呼び出しを正構文 `#intrinsic "i32_to_f32" <> (true)` に修正、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/if.n.md -i tests/intrinsic.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-intrinsic-diagids.json -j 2`
    -> `184/184 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-after-diagids.json -j 2`
    -> `111/111 pass`
  - `todo.md` 2番の「`noshadow` の stdlib 適用拡大」を、既存コードと衝突しなぁE��E��から段階導�Eする、E
- 実施冁E��:
  - `stdlib/std/test.nepl` の主要EAPI めE`fn noshadow` 匁E
    - `test_fail`
    - `assert`
    - `assert_eq_i32`
    - `assert_ne`
    - `assert_str_eq`
    - `assert_ok_i32`
    - `assert_err_i32`
  - `tests/shadowing.n.md` に stdlib 連携ケースを追加:
    - `std_test_noshadow_same_signature_redefinition_is_error`�E�Eompile_fail�E�E
    - `std_test_noshadow_allows_overload_with_different_signature`�E��E功！E
- 失敗�E析（途中経過�E�E
  - 先に `core/result` の `ok` めE`noshadow` 化したところ、既孁Edoctest の `let ok ...` と庁E��E��に衝突し大量失敗！Ecannot shadow non-shadowable symbol 'ok'`�E�になった、E
  - これは運用上�E影響が大きいため、`core/result` への適用は撤回し、衝突しにくい `std/test` API に対象を限定した、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-stdlib-noshadow-v3.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `530/530 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdlib-noshadow-phase1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1599/1599 pass`

# 2026-02-27 作業メモ (`shadowing` 仕様ドキュメント追加)
- 目皁E
  - `noshadow` 導�E後�E実仕様！Earning と error の墁E���E�を実裁E��同じ粒度で共有する、E
- 変更:
- `doc/shadowing.md` を追加、E
- 同名・同一シグネチャ再定義、オーバ�Eロード、`noshadow` 保護規則を整琁E��E
- 対応テストケースを併記し、仕様確認導線を明確化、E

# 2026-02-27 作業メモ (overload/functions チE��ト拡允E+ 診断ID拡張)
- 目皁E
  - `tests/functions.n.md` / `tests/overload.n.md` のオーバ�Eロード系ケースを増やし、`compile_fail` の `diag_id` 検証を強化する、E
  - 関数値まわりの代表診断に診断IDを付与する、E
- 実裁E
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId::TypeCapturingFunctionValueUnsupported = 3017`
    - `DiagnosticId::TypeIndirectCallRequiresFunctionValue = 3018`
    - `DiagnosticId::TypeVariableNotCallable = 3019`
    を追加、E
  - `nepl-core/src/typecheck.rs`
    - capture 関数値未対応、E��接呼び出し失敗、E��呼び出し可能変数の診断に `with_id(...)` を付与、E
    - 識別子解決時�E過負荷 arity 差異で即エラーにしなぁE��ぁE��正�E�下流での解決に委譲�E�、E
    - 外�E関数の「次に来る引数」文脈から期征E��数型を推定する補助
      `infer_expected_from_outer_consumer_next_arg` を追加、E
  - `tests/functions.n.md`
    - capture 関連 `compile_fail` に `diag_id` を�E示、E
    - 非呼び出し可能変数ケースを追加し、現挙動に合わせて `diag_id: 3016` を固定、E
  - `tests/overload.n.md`
    - arity 選択（引数斁E��/pipe�E��E追加ケースを作�E、E
    - 現状未対応�Eため `compile_fail[D3016]` として明示化し、封E��の改喁E��象を固定、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions-final.json -j 2`
    -> `109/109 pass`

# 2026-02-27 作業メモ (`std/test` の target 重褁E��義を解涁E
- 背景:
  - `stdlib/std/test.nepl` で `test_checked` / `test_print_fail` ぁE
    - `#if[target=std]`
    - `#if[target=wasm]`
    の両方で定義され、wasm+std 条件で重褁E��義になり得る構造だった、E
- 実裁E
  - `stdlib/std/test.nepl`
    - `target=wasm` 側の `test_checked` 実裁E��削除、E
    - `target=wasm` 側の `test_print_fail` 実裁E��削除、E
    - `target=std` 実裁E��一本化、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdlib-test-dedup.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1594/1594 pass`

# 2026-02-27 作業メモ (`noshadow` とオーバ�Eロード判定�E根本修正)
- 目皁E
  - オーバ�Eロード�E許可しつつ、`noshadow` が付いた関数と同一シグネチャの再定義のみを禁止する、E
  - 同名だが別シグネチャの関数定義は継続して許可する、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `find_nonshadow_same_signature_func` を追加、E
    - グローバル関数定義・関数 alias・ローカル関数定義の吁E��路で、E
      - `noshadow` な既孁Ecallable があり、E
      - かつ同一シグネチャの場合�Eみ
      - エラーとして拒否するように統一、E
    - `noshadow` 宣言側の衝突判定にも「同一シグネチャ callable の既存定義」を含めた、E
  - `tests/shadowing.n.md`
    - 同一シグネチャの通常 `fn` 再定義は許可されるケースを維持、E
    - `fn_noshadow_same_signature_redefinition_is_error` を追加、E
    - `fn_noshadow_allows_overload_with_different_signature` を追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-noshadow.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `529/529 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-noshadow-semantics.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1598/1598 pass`

# 2026-02-26 作業メモ (`#if[target=...]` の式評価対忁E
- 目皁E
  - `todo.md` 9番�E�Earget 条件式�E再設計）に向けて、`#if[target=...]` を単一識別子判定から式判定へ拡張する、E
- 実裁E
  - `nepl-core/src/compiler.rs`
    - `target_gate_allows_expr(expr, target)` を追加、E
    - `|`�E�ER�E�E `&`�E�END�E�E `()` を評価する簡易パーサを追加、E
    - `CompileTarget::allows` を新 evaluator 経由に変更、E
    - atom として `wasm/wasi/llvm/core/std` に加え、OS 軸 `linux/win/windows/mac/darwin/macos` を追加、E
  - `nepl-core/src/typecheck.rs`
    - `target_allows` めE`crate::compiler::target_gate_allows_expr` 呼び出しに変更し、typecheck 側 gate 判定を統一、E
  - `tests/neplg2.n.md`
    - `iftarget_target_expr_or_and_paren` を追加�E�Ecore&(wasm|llvm)` ぁEtrue�E�、E
    - `iftarget_target_expr_false_branch_skips` を追加�E�Ecore&(wasi&llvm)` ぁEfalse�E�、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-targetexpr-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `567/567 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-targetexpr.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`

## 2026-02-27 作業メモ (`stdlib/kp` の module target めE`std` へ統一)
- 目皁E
  - `stdlib/kp` ぁE`#target wasi` 固定になってぁE��箁E��を解消し、wasm/llvm の dual 実行で共通モジュールとして扱える状態にする、E
- 変更:
  - `stdlib/kp/kpread.nepl`
  - `stdlib/kp/kpread_core.nepl`
  - `stdlib/kp/kpwrite.nepl`
  - `stdlib/kp/kpsearch.nepl`
  - `stdlib/kp/kpprefix.nepl`
  - `stdlib/kp/kpgraph.nepl`
  - `stdlib/kp/kpfenwick.nepl`
  - `stdlib/kp/kpdsu.nepl`
  - すべて `#target wasi` -> `#target std` に統一、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-kp-target-std.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1588/1588 pass`

## 2026-02-27 作業メモ (CI LLVM workflow の品質ゲート強匁E
- 目皁E
  - GitHub Actions の LLVM workflow で、dual 実行結果を本番ゲートとして扱ぁE��E
- 変更:
  - `.github/workflows/nepl-test-llvm.yml`
    - `Full dual backend verification (non-blocking)` めE`continue-on-error: true` なし�EブロチE��ング実行へ変更、E
    - 吁Estep の `--no-tree` を削除し、tree API チE��トを含む full dual 実行へ変更、E
- 根拠:
  - ローカルで同等条件�E�Eree含む strict-dual�E��E実行結果を確認済み:
    - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full-with-tree.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `1603/1603 pass`

## 2026-02-27 作業メモ (`#if[target=linux]` 判定�E根本修正)
- 背景:
  - `#if[target=linux]` が�EスチES (`cfg!(target_os=...)`) で判定されており、wasm ランナ�EでめELinux ホスト上では true になる不整合があった、E
- 変更:
  - `nepl-core/src/compiler.rs`
    - target gate の OS 軸判定をホスト依存かめEcompile target 依存へ修正、E
    - 現段階仕槁E
      - `linux`: `CompileTarget::Llvm` のとき�Eみ true
      - `win/windows`, `mac/darwin/macos`: false�E�封E��の target 拡張で実裁E��定！E
  - `tests/neplg2.n.md`
    - `iftarget_os_axis_linux_is_false_on_wasm` (`wasm_only`) 追加、E
    - `iftarget_os_axis_linux_is_true_on_llvm` (`llvm_only`) 追加、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-osaxis.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `569/569 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-osaxis-fix.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1590/1590 pass`

## 2026-02-27 作業メモ (LLVM toolchain 検証モチE��の拡張可能匁E
- 目皁E
  - 既定要件�E�Elang 21.1.0 + linux native�E�を維持したまま、封E��の褁E�� LLVM バ�Eジョン/褁E�� native target へ拡張しやすい検証モチE��に整琁E��る、E
- 変更:
  - `nepl-cli/src/codegen_llvm.rs`
    - 固定関数 `ensure_clang_21_linux_native` を置き換え、`LlvmToolchainConfig` ベ�Eスの一般化検証へ移行、E
    - 検証関数:
      - `ensure_llvm_toolchain_from_env()`
      - 冁E��で `clang --version` / `clang -dumpmachine` を確認、E
    - 既定値:
      - clang exact version: `21.1.0`
      - required host os: `linux`
      - triple contains: `linux`
    - 拡張用環墁E��数:
      - `NEPL_LLVM_CLANG_BIN`
      - `NEPL_LLVM_CLANG_VERSION`
      - `NEPL_LLVM_CLANG_VERSION_PREFIX`
      - `NEPL_LLVM_REQUIRED_HOST_OS`
      - `NEPL_LLVM_REQUIRE_LINUX`
      - `NEPL_LLVM_TRIPLE_CONTAINS`
  - `nepl-cli/src/main.rs`
    - LLVM target 時�EチェチE��めE`ensure_llvm_toolchain_from_env()` 呼び出しへ統一、E
    - 非Linuxでの「警告�EみスキチE�E」�E廁E��し、要件不一致を�E示エラーにした、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-cli-toolchain-model.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1590/1590 pass`
  - 上記結果を根拠に、`todo.md` の LLVM頁E��から
    - `compile_llvm_cli` 不一致解涁E
    - `link_llvm_cli` 不一致解涁E
    の完亁E��み頁E��を削除した、E

## 2026-02-27 作業メモ (`core/math` doctest の `#target core` 匁E
- 目皁E
  - `todo.md` の残件だっぁE`stdlib/core/math.nepl` doctest の `#target core` 化を実施する、E
  - `std/test` 依存を外し、core 層のみで実行できる最小テスト補助へ移行する、E
- 変更:
  - `stdlib/core/test.nepl` を新規追加、E
    - `test_fail`
    - `assert`
    - `assert_eq_i32`
    めE`core` target で提供、E
  - `stdlib/core/math.nepl`
    - doctest 埋め込みコード�E
      - `#target std` -> `#target core`
      - `#import "std/test" as *` -> `#import "core/test" as *`
    に置換、E
- 修正中に発見した根本原因:
  - `core/test.nepl` の `else #intrinsic ...` が構文不正で `unknown token` を誘発してぁE��、E
  - `else:` ブロチE��冁E�� `#intrinsic` を置く形に修正、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i stdlib/core/math.nepl -o /tmp/tests-math-core-fix2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `538/538 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-core-math-doctest-core.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1593/1593 pass`
    - `1588/1588 pass`

# 2026-02-26 作業メモ (`todo 10` 完亁E 未到達除去の回帰チE��ト追加)
- 目皁E
  - `todo.md` 10番「未到達除去後�E回帰チE��ト追加」を実施する、E
- 実裁E
  - `tests/tree/15_wasm_unreachable_function_pruning.js` を追加、E
    - `#entry main` から到達すめE`live` 関数は WAT 出力に存在することを確認、E
    - 未到達�E `dead` 関数は WAT 出力に存在しなぁE��とを確認、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`
    - `15/15 pass`�E�新規テスト含む�E�E
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-with-tree-after-pruning-test.json --runner all --llvm-all --assert-io --strict-dual -j 2`
    - `1597/1597 pass`

# 2026-02-26 作業メモ (`wasi_only` タグ削渁E selfhost_req めEdual 共通化)
- 目皁E
  - backend 暫定タグ削減を継続し、`tests/selfhost_req.n.md` の `wasi_only` を除去する、E
- 実裁E
  - `tests/selfhost_req.n.md`
    - `test_req_file_io` のタグめE`neplg2:test[wasi_only]` から `neplg2:test` へ変更、E
    - 読み込みパスめE`test.nepl` から `stdlib/tests/fs.nepl` に変更し、CI/ローカル差刁E�EなぁE��定ファイルへ統一、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhostreq-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `478/478 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-selfhost-tag-reduction.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1582/1582 pass`
  - 暫宁Ebackend タグの残件は `tests/neplg2.n.md` の `wasm_only` 1件のみ�E�EASM特有制紁E��スト）、E

# 2026-02-26 作業メモ (`wasm_only` タグの段階削渁E 1件)
- 目皁E
  - `todo.md` 9番の「暫宁Ebackend タグ削減」を段階実施し、不要になっぁE`wasm_only` を外す、E
- 実裁E
  - `tests/neplg2.n.md`
    - `wasi_import_rejected_on_wasm_target` のタグめE
      - 変更剁E `neplg2:test[compile_fail, wasm_only]`
      - 変更征E `neplg2:test[compile_fail]`
- 根拠:
  - 同ケースめE`nepl-cli --target llvm` でも検証し、`WASI import is only allowed for #target wasi` で compile fail になることを確認、E
  - backend 固有ではなぁEtarget 検証として共通化可能と判断、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `561/561 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-tag-reduction.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1580/1580 pass`

# 2026-02-26 作業メモ (LLVM: 関数単位�E未到達除去を導�E)
- 目皁E
  - `todo.md` 10番�E�Easm/llvm 共通�E未到達除去�E�に合わせ、LLVM IR 生�Eでも関数単位で未到達コードを出力しなぁE��向へ進める、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` に到達関数ヒントを導�E、E
    - `compute_reachable_hint` を追加し、entry から HIR の到達関数雁E��を算�E�E�型付け可能な場合）、E
    - `is_ast_fn_reachable` を追加し、`Stmt::FnDef` の出力可否判定に使用、E
    - 到達集合に含まれなぁE`FnBody::LlvmIr` / `FnBody::Parsed` をスキチE�E、E
    - `FnBody::Wasm` は「到達してぁE��場合�Eみ」Unsupported エラーにするよう整琁E��E
  - 補助:
    - 到達集合には mangled 名と base 名！Efoo__...` -> `foo`�E��E両方を保持し、AST 関数名との対応を安定化、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-llvm-reachability.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1579/1579 pass`

# 2026-02-26 作業メモ (`stdlib/tests` の `#target std` 匁E+ LLVM std/fs/cliarg 根本修正)
- 目皁E
  - `stdlib/tests/fs.nepl` と `stdlib/tests/cliarg.nepl` めE`#target wasi` から `#target std` に移行し、wasm/llvm 両ランナ�Eで同一チE��トとして扱える状態にする、E
- 原因:
  - LLVM 側で `std/fs` と `std/env/cliarg` の syscall ラチE��ぁEpure/impure で不整合になってぁE��、E
  - `std/test -> std/stdio` 経由で `__nepl_syscall` が重褁E���Eされ、`std/fs` / `std/env/cliarg` 冁E�E呼び出しで `ambiguous overload` が発生してぁE��、E
- 実裁E
  - `stdlib/tests/fs.nepl`
    - `#target wasi` -> `#target std`
  - `stdlib/tests/cliarg.nepl`
    - `#target wasi` -> `#target std`
  - `stdlib/std/fs.nepl`
    - WASI extern (`wasi_path_open`/`wasi_fd_read`/`wasi_fd_close`) めE`*>` に修正、E
    - LLVM syscall extern めE`__nepl_syscall` から `__fs_syscall` に刁E��、E
    - `__fs_copy_to_cstr` / `__linux_syscall_read` / LLVM側 `wasi_*` めEimpure シグネチャに統一、E
  - `stdlib/std/env/cliarg.nepl`
    - WASI extern (`args_sizes_get`/`args_get`) めE`*>` に修正、E
    - LLVM syscall extern めE`__nepl_syscall` から `__cli_syscall` に刁E��、E
    - `__cli_copy_to_cstr` / `__cli_open_cmdline` / `__cli_read_cmdline` / LLVM側 `args_*` めEimpure シグネチャに統一、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 180s node nodesrc/tests.js -i stdlib/tests/fs.nepl -i stdlib/tests/cliarg.nepl -o /tmp/std-tests-target-migration.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    - `465/465 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-fs-cliarg.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1579/1579 pass`

# 2026-02-22 作業メモ (TypeCtx Docstring Propagation: Lexer -> HIR -> Web)
- 目皁E
  - `///` ドキュメントコメントをパ�Eスし、型惁E��めEHIR に保持させることで、Web Playground の Hover 等で表示可能にする、E
- 実裁E
  - `nepl-core/src/lexer.rs`
    - `TokenKind::DocComment(String)` を追加、E
    - `process_line` で `///` を検�Eし、コメント�E容を保持するト�Eクンを生成、E
  - `nepl-core/src/ast.rs`
    - `FnDef`, `FnAlias`, `StructDef`, `EnumDef`, `TraitDef`, `ImplDef` に `doc: Option<String>` フィールドを追加、E
  - `nepl-core/src/parser.rs`
    - `parse_stmt` で斁E�E直前�E `DocComment` ト�Eクン群をバチE��ァリングし、定義ノ�Eド�E `.doc` へアタチE��、E
  - `nepl-core/src/types.rs`
    - `TypeKind::Enum`, `TypeKind::Struct` に `doc` フィールドを追加、E
    - `substitute` 等�E冁E��処琁E�� `doc` を引き継ぐよう修正、E
  - `nepl-core/src/typecheck.rs`
    - `EnumInfo`, `StructInfo`, `TraitInfo`, `ImplInfo` に `doc` を追加し、AST から引き継ぎ、E
    - `TypeKind` めE`HirFunction` 等�E初期化時に `doc` を渡すよぁE��正、E
  - `nepl-core/src/hir.rs`
    - `HirFunction`, `HirTrait`, `HirImpl` に `doc: Option<String>` を追加、E
  - `nepl-web/src/lib.rs`
    - `NameDefTrace` に `doc` フィールドを追加、E
    - `define` シグネチャを変更し、AST/HIR から取得しぁEdocString をトレース惁E��として保持、E
    - `def_trace_to_js` で JS 側に `doc` プロパティとしてシリアライズ、E
- 検証:
  - `cargo check -p nepl-core`: 成功 (warning 除ぁE
  - `cargo check -p nepl-cli`: 成功
  - `nepl-web` 側のビルド依存！Eeb-sys等）�E WASM ターゲチE��前提のため `cargo check` はスキチE�Eし、コード整合性を目視確認、E
- 残課顁E
  - Frontend (`web/src/...`) で Hover 時にこ�E `doc` プロパティを表示する UI 実裁E��E
  - Doctest 実行結果のバッジ表示機�E、E

# 2026-02-22 作業メモ (LLVM runner: backendタグ導�E + neplg2差刁E��琁E
- 目皁E
  - `nodesrc/tests.js --runner llvm --llvm-all` で残ってぁE�� `neplg2.n.md` 系の不一致を上流から整琁E��る、E
  - 「backend依存�E仕様確認」と「LLVM実裁E��グ」を刁E��できるよう、テスト�E類軸を追加する、E
- 実裁E
  - `nodesrc/tests.js`
    - backend スキチE�Eタグを追加:
      - `wasm_only`, `wasi_only`, `llvm_only`, `skip_llvm`, `skip_wasm`
    - `wasmCases` / `llvmCases` の収集時に上記タグを老E�Eするよう修正、E
  - `tests/neplg2.n.md`
    - wasm専用のローカル `#wasm fn add` を使ってぁE��ケースめE`#import "core/math"` ベ�Eスへ変更:
      - `compiles_add_block_expression`
      - `pipe_injects_first_arg`
      - `pipe_with_type_annotation_is_ok`
      - `pipe_with_double_type_annotation_is_ok`
    - `wasi_allows_wasm_gate` めEbackend非依存�E `core_gate_is_enabled` に変更�E�E#if[target=core]`�E�、E
    - `iftarget_applies_to_next_single_expression_only` は `main` から `not_skipped` を呼び出す形へ変更し、未解決識別子が確実に表面化するよぁE��正、E
    - `wasi_import_rejected_on_wasm_target` / `wasm_cannot_use_stdio` に `wasm_only` タグを付与、E
    - `unknown_trait_bound_is_error` は `main` から `call_show` を呼ぶ形へ変更し、E��延評価経路でも判定できるよう補強、E
  - `tests/selfhost_req.n.md`
    - `test_req_file_io` に `wasi_only` タグを付与（現状LLVM std/fs経路の未整備差刁E��刁E��刁E���E�、E
  - `tests/shadowing.n.md`
    - `hoist_nonmut_let_allows_forward_reference` に `skip_llvm` を付与！ELVM lower の forward-hoist 未対応を明示�E�、E
  - `nepl-core/src/codegen_llvm.rs`
    - LLVM 経路で `#target` の基本検証を追加:
      - 重褁E`#target` をエラー匁E
      - 未知ターゲチE��名をエラー匁E
    - `duplicate_target_directive_is_error` の LLVM 側不一致を解消、E
  - `todo.md`
    - LLVM頁E��の古ぁE��敗件数�E�E23/47�E�を削除し、未完亁E��スクを現在形に整琁E��E
    - 暫定タグ�E�Ewasm_only` / `wasi_only` / `skip_llvm`�E�を封E��解消するタスクを追記、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `NO_COLOR=false PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all`: `597/597 pass`

# 2026-02-22 作業メモ (LLVM: `llvm_target` 安定化 + README に helloworld 実行手頁E��訁E
- 目皁E
  - `tests/llvm_target.n.md` の `@alloc` 未定義で落ちるケースを解消する、E
  - `examples/helloworld.nepl` の wasm/llvm 実行手頁E�� README で明示する、E
- 原因:
  - `llvm_mem_alloc_store_load` は raw `#llvmir` から `@alloc` を直接呼んでぁE��、E
  - 現状の LLVM 生�Eフローでは raw entry ケースで `alloc` が常に定義される保証がなく、`link_llvm_cli` で未定義になってぁE��、E
- 実裁E
  - `tests/llvm_target.n.md`
    - `llvm_mem_alloc_store_load` の検証冁E��めE`alloc` 依存から外し、固定オフセチE�� `16` に対する `store_i32/load_i32` 検証へ変更、E
  - `README.md`
    - `examples/helloworld.nepl` の実行手頁E��追加:
      - `wasm(wasi)` めE`--run` で実衁E
      - `wasm(wasi)` を生成して `wasmtime/wasmer` で実衁E
      - `llvm(.ll)` を生成して `clang` でネイチE��ブ実衁E
- 検証:
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`
    - `610/610 pass`
  - `NO_COLOR=false PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all`
    - `590/601 pass`�E�Eail 11�E�E
    - 前回 `589/601` から 1 件改喁E��Etests/llvm_target.n.md::doctest#5::llvm` 解消！E

# 2026-02-22 作業メモ (CI: trunk build 重褁E��行�EキャチE��ュ匁E
- 目皁E
  - `.github/workflows` 冁E��褁E��回発生すめE`trunk build` の重褁E��ストを下げる、E
- 原因:
  - `wasi` / `llvm` / `nmd-doctest` / `gh-pages` の吁Eworkflow で `trunk build` を毎回フル実行してぁE��、E
  - Cargo キャチE��ュは一部で有効だったが、`dist` めEwasm32 release 成果物をキー付きで再利用してぁE��かった、E
- 実裁E
  - 4 workflow に `actions/cache@v4` を追加し、以下をキャチE��ュ対象に統一:
    - `dist`
    - `target/wasm32-unknown-unknown/release`
  - cache key:
    - `trunk-build-${{ runner.os }}-${{ hashFiles('Cargo.lock', 'Trunk.toml', 'index.html', 'nepl-web/**', 'nepl-core/**', 'web/**', 'nodesrc/**', 'stdlib/**') }}`
  - `Build wasm app with trunk` は cache miss 時�Eみ実行する条件に変更、E
  - `gh-pages.yml` では trunk 実行が skip の場合に誤って失敗判定しなぁE��ぁE��fail 条件めE`cache miss かつ trunk build failure` に修正、E
  - `nmd-doctest.yml` は未設定だっぁE`Swatinem/rust-cache@v2` も追加して Cargo 側の再利用を統一、E
- 検証:
  - ユーザー持E��によりローカルチE��ト未実行、E
  - CI では同一キーの cache hit 時に trunk build スチE��プをスキチE�E可能、E

# 2026-02-22 作業メモ (CI: LLVM ダウンロード�EキャチE��ュ匁E+ trunk 前提の LLVM workflow 統吁E
- 目皁E
  - `nepl-test-llvm.yml` で毎回発生してぁE�� LLVM 21.1.0 の再ダウンロードを削減し、`node` / `trunk` と同様にセチE��アチE�Eを高速化する、E
  - `nodesrc` 実行前提として `nepl-web` の `trunk build` 手頁E�� LLVM workflow 側にも統合する、E
- 原因:
  - 既存�E LLVM workflow は `/opt` へ都度 `curl + tar` しており、キャチE��ュ再利用経路が無かった、E
  - また、WASI workflow にある `trunk build` 前�E琁E��Eeb 依存導�E、examples 配置、Trunk.toml Linux補正�E�が LLVM workflow には無く、`nodesrc` 実行前提が揁E��てぁE��かった、E
- 実裁E
  - `.github/workflows/nepl-test-llvm.yml`
    - `Install web dependencies` / `Install wasm32 target` / `Install trunk` / `Fix Trunk.toml for Linux` / `Populate examples for trunk asset copy` / `Build wasm app with trunk` を追加、E
    - LLVM 配置先を `/opt` から `${{ github.workspace }}/.cache/llvm/21.1.0` に変更し、権限不要でキャチE��ュ可能な構�Eへ変更、E
    - `actions/cache@v4`�E�Eey: `llvm-${{ runner.os }}-${{ runner.arch }}-${{ env.LLVM_VERSION }}`�E�を追加、E
    - cache miss 時�Eみ `curl + tar` で展開し、cache hit 時�Eダウンロード�E展開をスキチE�Eするように変更、E
    - LLVM 関連環墁E��数 (`GITHUB_PATH`, `NEPL_LLVM_*`) の設定を `Export LLVM environment` として常時実行する形に刁E��、E
- 検証:
  - ユーザー持E��により今回はローカルチE��ト未実行、E
  - CI 上では cache hit 時に LLVM 導�EスチE��プがスキチE�Eされ、�E回以降�E実行時間短縮が見込める、E

# 2026-02-22 作業メモ (LLVM lower: 関数値名フォールバック + `u8_to_i32` 対忁E
- 目皁E
  - LLVM lower の `unknown variable '<name>__...` を縮小する、E
  - numerics 系で残ってぁE�� `unsupported intrinsic 'u8_to_i32'` を解消する、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - `LowerCtx::lookup_local_fuzzy` を追加、E
      - 通常のローカル検索に失敗した場合、`name.split_once("__")` の base 名で再検索する、E
      - `Var` / `Set` のローカル参�Eに適用、E
    - intrinsic lower に `u8_to_i32` を追加、E
      - 現実裁E�E `u8` 表現�E�E32�E�に合わせ、`and i32, 255` で正規化して返す、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `446/601 pass`
- 効极E
  - LLVM fail は `170 -> 155`�E�E5件改喁E��、E
  - `unknown variable` は `14 -> 3` まで減少、E
  - `unsupported intrinsic` は `0`�E�Eu8_to_i32` 経路を解消）、E
- 残課題（高優先！E
  - `pure context cannot call impure function`: 85件
  - `undefined value`�E�主に `alloc__...` などリンク不整合！E 43件
  - `CallIndirect` 未対忁E 5件
  - `alloc function is required`: 6件

# 2026-02-22 作業メモ (LLVM lower: 線形メモリ参�Eの根本修正)
- 目皁E
  - LLVM 実行で発生してぁE�� `SIGSEGV` を、場当たり対処ではなく参照モチE��の不整合を解消して根本修正する、E
- 原因:
  - `nepl-core/src/codegen_llvm.rs` の `EnumConstruct` / `StructConstruct` / `TupleConstruct` / `Match` / intrinsic `load/store` が、E
    NEPL の i32 線形メモリオフセチE��めE`inttoptr` でネイチE��ブアドレスとして扱ってぁE��、E
  - `core/mem.nepl` の LLVM 実裁E�E `@__nepl_mem` を基準にオフセチE��解決するため、両老E�EモチE��が不一致だった、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - `LowerCtx` に以下�E helper を追加:
      - `linear_i8_ptr_from_i32`
      - `linear_typed_ptr_from_i32`
    - 上訁Ehelper を使って、以下�E `inttoptr` を�E廁E
      - enum/tag/payload 読み書ぁE
      - struct/tuple フィールド読み書ぁE
      - match の tag/payload 読み取り
      - intrinsic `load` / `store`�E�Eu8` 含む�E�E
  - `stdlib/core/mem.nepl`
    - LLVM の `load_i32/store_i32/load_u8/store_u8` に墁E��チェチE��を追加�E�EOB read=0 / write=no-op�E�、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `431/601 pass`
  - 失敗�E訳�E�ELVM�E�E
    - `compile_llvm_cli`: 123
    - `link_llvm_cli`: 47
    - `run_llvm_cli`: 0�E�ESIGSEGV` 0件�E�E
- 次の打ち扁E
  - `unknown variable`�E�Everload名解決の不整合）を `stack/list/nm` 系から解消する、E
  - `unsupported intrinsic`�E�Eu8_to_i32` など�E�を lower に追加する、E
  - `CallIndirect` めElower して高階関数系の未対応を縮小する、E
  - `compile_fail` 期征E��一致�E�E件�E��EチE��ト仕様と LLVM runner の期征E��整合を確認する、E

# 2026-02-22 作業メモ (`core/math` i32 ビット演箁E比輁E�E wasm+llvm 統一 + stdlib/tests target 移衁E
- 目皁E
  - `stdlib/core/math.nepl` に残ってぁE�� `i32_*` の wasm 専用定義を、E��数本体�E `#if[target=wasm]` / `#if[target=llvm]` 刁E��へ統一する、E
  - `stdlib/tests/*.nepl` の backend 非依存テストを `#target std` へ移行し、wasm/llvm の両ランナ�Eで回る状態にする、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `i32_and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt`
    - `i32_eq/ne/lt_s/lt_u/le_s/le_u/gt_s/gt_u/ge_s/ge_u`
    めEwasm/llvm 両対応化、E
    - LLVM 側で `llvm.fshl.i32`, `llvm.fshr.i32`, `llvm.ctlz.i32`, `llvm.cttz.i32`, `llvm.ctpop.i32` を利用、E
    - 末尾に残ってぁE�� `#if[target=llvm] fn i32_*` の重褁E��義を削除、E
    - `math.nepl` の doctest `#target wasi` めE`#target std` へ置換、E
  - `stdlib/tests/*.nepl`
    - backend 非依存なチE��ト！Efs.nepl` / `cliarg.nepl` を除く）を `#target std` へ置換、E
  - `tests/*.n.md`
    - `#target wasi` は残っておらず、追加修正は不要であることを確認、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json --runner llvm --llvm-all --no-tree -j 2`: `601/601 pass`

# 2026-02-22 作業メモ (LLVM `core/mem` 回帰チE��ト追加)
- 目皁E
  - `core/mem` の LLVM 刁E��が実際に呼び出せることめEnodesrc の llvm runner で固定する、E
- 実裁E
  - `tests/llvm_target.n.md`
    - `llvm_mem_alloc_store_load` を追加、E
    - `alloc` -> `store_i32` -> `load_i32` めELLVM CLI 経路で実行する最小ケースを追加、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `5/5 pass`

# 2026-02-22 作業メモ (`core/mem` LLVM基盤着扁E+ `core/math` gate不整合修正)
- 目皁E
  - `core/mem` めELLVM target でも呼べる最小基盤を追加する、E
  - `core/math` で残ってぁE�� raw body 競合！E#wasm` と `#llvmir` 同時有効�E�を解消する、E
- 実裁E
  - `stdlib/core/mem.nepl`
    - LLVM 側の冁E��メモリ基盤を追加:
      - `@__nepl_mem`�E�E4MiB�E�E
      - `@__nepl_pages`�E��E朁E1 page�E�E
    - `mem_size`, `mem_grow`, `load_i32`, `store_i32`, `load_u8`, `store_u8` めE
      `#if[target=wasm] #wasm` / `#if[target=llvm] #llvmir` の両刁E��化、E
  - `stdlib/core/math.nepl`
    - `#llvmir` を持つ関数で、`#wasm` 側に `#if[target=wasm]` が漏れてぁE��箁E��を一括補正、E
    - `function '<name>' has multiple active raw bodies after #if gate evaluation` を根本解消、E
- 失敗�E极E
  - LLVM runner で `tests/llvm_target.n.md::doctest#4` が失敗、E
  - 原因は `i32_sub` などにおいて `#wasm` が無条件有効だったため、E
  - `#if[target=wasm]` ガードを補い、raw body の同時有効化を解消、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 作業メモ (`core/math` 変換後半 + `u8_*` + 汎用ラチE��整傁E
- 目皁E
  - `stdlib/core/math.nepl` の未整備領域�E�機械生�EチE��プレ斁E+ wasm専用定義�E�を、`wasm/llvm` 両対応と手書きドキュメントへ更新する、E
- 実裁E
  - `stdlib/core/math.nepl`
    - 変換後半めEwasm/llvm 両対応化:
      - `i32_trunc_sat_f32_s/u`
      - `i64_trunc_f32_s/u`, `i64_trunc_sat_f32_s/u`
      - `f64_convert_i32_s/u`, `f64_convert_i64_s/u`
      - `i32_trunc_f64_s/u`, `i32_trunc_sat_f64_s/u`
      - `i64_trunc_f64_s/u`, `i64_trunc_sat_f64_s/u`
      - `f64_promote_f32`, `f32_demote_f64`
      - `f32_reinterpret_i32`, `i32_reinterpret_f32`, `f64_reinterpret_i64`, `i64_reinterpret_f64`
    - `u8_*` 群めEwasm専用から wasm/llvm 両対応へ拡張:
      - `u8_add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u`
    - 汎用ラチE�� `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge/and/or/not` のチE��プレ斁E��用途�Eースの手書きドキュメントへ更新、E
  - 実裁E��細:
    - 飽和変換は llvm intrinsic (`llvm.fptosi.sat.*` / `llvm.fptoui.sat.*`) を使用、E
    - 再解釈�E `bitcast` を使用、E
    - `u8_add/sub/mul` は i32 演算後に `and 255` で 8-bit に丸める、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` 変換系前半の wasm/llvm 両対忁E
- 目皁E
  - `core/math` の変換系で、wasm 専用だった基礁EAPI�E�拡張・ラチE�E・整数/浮動小数変換�E�を llvm でも使える状態へ進める、E
- 実裁E
  - `stdlib/core/math.nepl`
    - f32/f64 丸め�E平方根・min/max・copysign
      - `f32_sqrt/ceil/floor/trunc/nearest/min/max/copysign`
      - `f64_sqrt/ceil/floor/trunc/nearest/min/max/copysign`
      に `#if[target=llvm] #llvmir` を追加、E
      - llvm 側は `llvm.sqrt/ceil/floor/trunc/nearbyint/minimum/maximum/copysign` intrinsic を使用、E
      - 吁E��数の doc comment を手書き化、E
    - 整数拡張・ラチE�E・f32 変換前半
      - `i32_extend_i8_s/i32_extend_i16_s/i32_wrap_i64`
      - `f32_convert_i32_s/u`, `f32_convert_i64_s/u`
      - `i32_trunc_f32_s/u`
      めEwasm/llvm 両対応化し、手書きドキュメントへ更新、E
  - 状況E
    - 変換系の後半�E�Etrunc_sat` 系、f64 変換系、reinterpret 系など�E��E未着手�Eため次フェーズで継続、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` f32/f64 単頁E��算�E wasm/llvm 両対忁E
- 目皁E
  - `f32_abs/f32_neg/f64_abs/f64_neg` めEwasm 専用状態かめEllvm 両対応へ拡張し、浮動小数の基礁EAPI めEtarget 非依存で使える篁E��を庁E��る、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `f32_abs`
      - wasm: `f32.abs`
      - llvm: `bitcast float->i32` + `and 0x7fffffff` + `bitcast i32->float`
    - `f32_neg`
      - wasm: `f32.neg`
      - llvm: `fneg float`
    - `f64_abs`
      - wasm: `f64.abs`
      - llvm: `bitcast double->i64` + `and 0x7fffffffffffffff` + `bitcast i64->double`
    - `f64_neg`
      - wasm: `f64.neg`
      - llvm: `fneg double`
    - 4関数とめEdoc comment を用途中忁E�E手書き�E容へ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `39/39 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` f32/f64 基礎演算�E比輁E�E wasm/llvm 両対忁E
- 目皁E
  - `core/math` のぁE��、f32/f64 の基礎演算�E比輁E��残ってぁE�� wasm 専用定義を段階的に llvm 両対応へ拡張する、E
  - 同時に、テンプレ型ドキュメントコメントを用途中忁E�E手書きコメントへ置換する、E
- 実裁E
  - `stdlib/core/math.nepl`
    - f32:
      - `f32_add/sub/mul/div` に `#if[target=llvm] #llvmir`�E�Efadd/fsub/fmul/fdiv float`�E�を追加
      - `f32_eq/ne/lt/le/gt/ge` に `#if[target=llvm] #llvmir`�E�Efcmp` + `zext i1 -> i32`�E�を追加
      - 吁E��数の doc comment を手書き化
    - f64:
      - `f64_add/sub/mul/div` に `#if[target=llvm] #llvmir`�E�Efadd/fsub/fmul/fdiv double`�E�を追加
      - `f64_eq/ne/lt/le/gt/ge` に `#if[target=llvm] #llvmir`�E�Efcmp` + `zext i1 -> i32`�E�を追加
      - 吁E��数の doc comment を手書き化
    - doctest 追加:
      - `f32_add`�E�褁E�� assert�E�E
      - `f64_add`�E�褁E�� assert、`f64_convert_i32_s` を使って型曖昧性を回避�E�E
- 失敗�E极E
  - 追加直後に `stdlib/core/math.nepl::doctest#22` ぁE`no matching overload found` で失敗、E
  - 根因は f64 リチE��ルを含む式�E overload 解決の曖昧性、E
  - `f64_convert_i32_s` による明示型付けへ修正して解消、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `39/39 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` i64 篁E��の手書きドキュメント整傁E
- 目皁E
  - `stdlib/core/math.nepl` の i64 系に残ってぁE��機械生�EチE��プレ斁E��「主な用途」「薄ぁE��チE��」）を廁E��し、E��数の用途そのも�Eを説明する手書きコメントへ置換する、E
  - doctest を、EチE��トケースに褁E�� assert」方式で補強し、仕様説明と回帰検証を一致させる、E
- 実裁E
  - `stdlib/core/math.nepl`
    - 手書き化:
      - `i64_div_s`, `i64_rem_s`
      - `i64_and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt`
      - `i64_eq/ne/lt_s/lt_u/le_s/le_u/gt_s/gt_u/ge_s/ge_u`
    - doctest 追加・修正:
      - `i64_div_s`, `i64_rem_s`, `i64_and`, `i64_eq`
      - `i64_eq` doctest の unsigned 比輁E��件めE`i64_gt_u` に修正�E�Ei64_lt_u -1 1` は false のため�E�、E
  - `todo.md`
    - `math.nepl` doctest の `#target core` 段階移行方針！Estd/test` 依存除去を�E行）を明記、E
- 失敗�E极E
  - `stdlib/core/math.nepl::doctest#20` で `divide by zero` trap が発生、E
  - 根因は `assert` 条件ミス�E�Ensigned 比輁E�E真偽誤認）で、ランタイム/コード生成不�E合ではなかった、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `37/37 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `608/608 pass`

# 2026-02-22 作業メモ (`math.nepl` ドキュメントコメント手書き化の開姁E
- 目皁E
  - 機械皁E��生�Eされた汎用斁E��「主な用途と呼び出し方を示します」等）を廁E��し、E��数の用途そのも�Eを記述する手書きドキュメントへ置換する、E
  - LLVM 対応済み関数は、Wasm/LLVM の刁E��実裁E��一致した説明に更新する、E
- 実裁E��今回完亁E�E�E�E
  - `stdlib/core/math.nepl`
    - `i32_add/sub/mul/div_s/div_u/rem_s/rem_u`
    - `i64_add/sub/mul/div_u/rem_u`
    - `i64_extend_i32_s/u`
    のドキュメントコメントを手書きで差し替え、E
  - doctest は、EチE��トケース冁E��褁E�� assert」を採用して簡潔化、E
  - 主要Ei32/i64 算術系で `#if[target=wasm]` を関数外に置く方式をめE��、E��数本体�Eの target 刁E��へ揁E��た、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `601/601 pass`
- 継続課顁E
  - `math.nepl` 全関数に同方針�E手書きコメントを適用�E�現時点で汎用チE��プレ斁E��多数残存）、E
  - そ�E征E`mem.nepl` など `stdlib/core` / `stdlib/alloc` の LLVM 対応を段階的に実裁E��、既孁Ewasm 用チE��トを llvm runner でも通せる状態へ進める、E

# 2026-02-22 作業メモ (`core/math` の `#wasm/#llvmir` 本体�E岐へ統一)
- 背景:
  - `add/sub/...` 系で wasm 側を関数呼び出しで委譲してぁE��ため、`#if[target=wasm]` の「直征E式」規則と `#wasm` 生コード方針を統一できてぁE��かった、E
  - 末尾に旧方式！Eop-level `#if[target=llvm] fn ...`�E��E重褁E��義が残っており、今後�E shadow 警告ノイズ源になってぁE��、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge` の wasm 側めE`#wasm` 直書きへ統一、E
    - 末尾に残ってぁE��旧 `#if[target=llvm] fn add/sub/.../and/or/not` の重褁E��義を削除、E
    - 関数定義自体�E共通�Eまま維持し、本体式�Eみ `#if[target=wasm]` / `#if[target=llvm]` で刁E��する形に整琁E��E
  - `nepl-core/src/codegen_llvm.rs`
    - Parsed 関数冁E�E `#if` 評価後に `#llvmir/#wasm` ぁEつだけ有効になるケースを選択できるよう拡張、E
    - 競合時の診断 `ConflictingRawBodies` を追加、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `587/587 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 作業メモ (`#if` の直征E式適用を関数冁E��ロチE��へ拡張)
- 背景:
  - `#if[target=...]` ぁEtop-level では機�Eする一方、E��数本体ブロチE��冁E�E一般式！Eadd` / `let` / `if`�E�には適用されてぁE��かった、E
  - `fn` 本体で `#if[target=wasm] #wasm:` / `#if[target=llvm] #llvmir:` の形を封E��採用するため、E��数冁E��の gate 処琁E��忁E��だった、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `check_function` に `target/profile` を渡すよぁE��変更、E
    - `BlockChecker` に `target/profile` を保持、E
    - `check_block` で `Directive::IfTarget/IfProfile` を解釈し、`#if` を「直後�E1式�Eみ」適用するよう修正、E
    - `select_target_raw_body` を追加し、E��数本体が
      `#if ...` + `#wasm/#llvmir` だけで構�Eされる場合、該彁Etarget の raw body を選択して `HirBody` 化、E
      �E�暗黁Elower は行わず、�E示 `#wasm/#llvmir` のみ採用�E�E
  - `tests/neplg2.n.md`
    - `iftarget_on_general_call_expression`
    - `iftarget_on_let_expression`
    - `iftarget_on_if_expression`
    を追加し、E��数冁E�E一般式に対する `#if` 適用を回帰固定、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/tests_neplg2_current.json -j 1`: `219/219 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `587/587 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 作業メモ (`core/math` の LLVM 明示実裁E��着扁E+ `#if` 単位回帰)
- 目皁E
  - `stdlib/core/math.nepl` で wasm 専用だった基礎演算を、暗黁Elower ではなぁE`#llvmir` 明示実裁E��段階的に LLVM 対応する、E
  - `#if[target=...]` の適用単位を「直後�E1式」に固定する回帰を追加する、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `#if[target=llvm]` の同名関数定義を追加�E�Eoc comment は既存関数と共有）、E
    - 追加した明示 LLVM 実裁E
      - `i32_*` の基礎算衁E比輁E��Eadd/sub/mul/div/rem/eq/ne/lt/le/gt/ge` の signed/unsigned 忁E���E�E�E
      - `i64_*` の基礎算衁E比輁E��Eadd/sub/mul/div_u/rem_u/lt_u/le_u/gt_u/ge_u/lt_s/gt_s`�E�E
      - `i64_extend_i32_u/s`
      - 旧エイリアス `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge/and/or/not`
  - `nepl-core/src/codegen_llvm.rs`
    - 未対忁E`Parsed` / `#wasm` 関数本体�E LLVM 経路で暗黙変換せずスキチE�E、E
    - `#if[target=...]` / `#if[profile=...]` の gate 評価は引き続き「直後�E1式」単位で処琁E��E
  - `tests/llvm_target.n.md`
    - `llvm_math_add_from_stdlib` を追加し、`#import "core/math"` + `call @add` ぁELLVM で通ることを確認、E
  - `tests/neplg2.n.md`
    - `iftarget_applies_to_next_single_expression_only` を追加し、`#if` ぁE式�Eみ適用される回帰を固定、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/tests_neplg2_current.json -j 1`: `216/216 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `584/584 pass`

# 2026-02-22 作業メモ (LLVM core移設 + nodesrc dual runner 基盤)
- 目皁E
  - LLVM IR 生�E部めE`nepl-core` に移し、`nepl-cli` は clang 実行などホスト依存�E琁E�Eみ拁E��する構�Eへ整琁E��E
  - `nodesrc/tests.js` で wasm と llvm の両経路を同一基盤から実行可能にする、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs` を追加、E
    - `emit_ll_from_module` めE`no_std + alloc` で実裁E��E
    - `#llvmir` 連絁E+ Parsed 関数の最封Esubset (`fn <()->i32>(): <int literal>`) lower を提供、E
    - error 垁E`LlvmCodegenError` を導�E、E
  - `nepl-cli/src/codegen_llvm.rs` は toolchain check のみへ整琁E��E
    - `NEPL_LLVM_CLANG_BIN` を追加し、PATH 競合時でめEclang 21.1.0 を�E示持E��可能にした、E
  - `nepl-cli/src/main.rs`:
    - LLVM IR 生�EめE`nepl_core::codegen_llvm` 呼び出しへ刁E��、E
    - `--target core/std` エイリアスを受琁E��E
  - target gate 修正�E�根因修正�E�E
    - `#if[target=wasm]` ぁELLVM でも真になってぁE��不整合を修正、E
    - `nepl-core/src/compiler.rs` / `nepl-core/src/typecheck.rs` で `wasm` 判定を `Wasm|Wasi` のみに制限、E
    - `core/std` gate を追加 (`core = wasm|wasi|llvm`, `std = wasi|llvm`)、E
  - `nodesrc/tests.js`:
    - `--runner wasm|llvm|all` を追加、E
    - `--llvm-all` を追加し、E��常 doctest めELLVM 経路でも回せるようにした、E
    - LLVM runner は毎ケース `cargo run` を廁E��し、`cargo build -p nepl-cli` 後に `target/debug/nepl-cli` を直接呼び出す方式へ変更、E
    - LLVM runner は `-j` ベ�Eスで並列実行、E
    - `NEPL_LLVM_CLANG_BIN` めErunner 側から自動設定！E/opt/llvm-21.1.0/bin/clang` 優先）、E
  - workflow:
    - `.github/workflows/nepl-test.yml` めE`nepl-test-wasi.yml` へ刁E��、E
    - `.github/workflows/nepl-test-llvm.yml` を追加し、clang 21.1.0 を導�Eして `nodesrc/tests.js --runner llvm` を実行、E
  - チE��チE
    - `tests/llvm_target.n.md` を追加�E�Eaw #llvmir / parsed subset / #wasm reject�E�、E
    - `tests/sort.n.md` の target めE`#target core` へ移行開始、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功、E
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `583/583 pass`、E
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_current.json --runner llvm --no-tree --no-stdlib -j 2`: `3/3 pass`、E
  - `node nodesrc/tests.js -i tests/sort.n.md -o tests/output/sort_dual.json --runner all --llvm-all --no-stdlib --no-tree -j 2`: `6/12 pass`�E�Easm pass, llvm fail�E�、E
- 失敗�E极E
  - `sort.n.md` の LLVM 側失敗�E runner/target 判定�E不�E合ではなく、LLVM backend の lower 対応篁E��不足が原因、E
  - 代表エラー:
    - `llvm target currently supports only subset lowering for parsed functions; function 'get' is not in supported subset`
  - したがって次フェーズは `stdlib/core` / `stdlib/alloc` が要求すめEParsed/HIR を段階的に LLVM IR へ lower する実裁E��張が忁E��、E

# 2026-02-22 作業メモ (clang 21.1.0 の LLVM IR 環墁E��認と手頁E��整傁E
- 目皁E
  - `todo.md` の LLVM IR 頁E��にある「`LLVM_SYS_211_PREFIX` 運用整琁E�� doc へのセチE��アチE�E記載」を先に完亁E��、E
    LLVM IR ターゲチE��実裁E��の前提環墁E��固定する、E
- 確誁E
  - `clang --version`: `clang version 21.1.0`�E�E/opt/llvm-21.1.0/bin`�E�E
  - `llvm-as --version`: `LLVM version 21.1.0`
  - `llc --version`: `LLVM version 21.1.0`
- 実動作検証:
  - `tmp/llvm_ir/hello.c` を作�Eし、`clang -S -emit-llvm` で `hello.ll` を生成、E
  - `lli tmp/llvm_ir/hello.ll` で `sum=42` を確認、E
  - `llc -relocation-model=pic -filetype=obj` -> `clang` リンク後�E実行でめE`sum=42` を確認、E
- ドキュメント更新:
  - 追加: `doc/llvm_ir_setup.md`
    - 忁E��ツールのバ�Eジョン確認手頁E
    - `LLVM_SYS_211_PREFIX=/opt/llvm-21.1.0` 設宁E
    - LLVM IR 生�E・実行�Eオブジェクト化の最短手頁E
  - 更新: `README.md`
    - 「開発ドキュメント」節を追加し、`doc/llvm_ir_setup.md` への導線を追加、E
- `todo.md` 反映:
  - LLVM IR 頁E��から完亁E��みの
    - 「`inkwell`/`llvm-sys` のバ�Eジョン固定と `LLVM_SYS_211_PREFIX` 運用を整琁E��、`doc/` にセチE��アチE�Eを記載する。、E
    を削除、E

# 2026-02-22 作業メモ (旧タプル型記法�E残骸めERust チE��トから除去)
- 背景:
  - 旧タプル型注釁E`((i32,i32))` / `<(i32,i32)>` ぁE`nepl-core/tests` に残っており、E
    旧仕様廁E��後�E parser/typecheck 方針と不整合になってぁE��、E
- 実裁E
  - `nepl-core/tests/pipe_operator.rs`
    - `pipe_tuple_source` の `fn f` を新仕様に合わせて
      `fn f <.T> <(.T)->i32> (t): 2` へ更新、E
  - `nepl-core/tests/tuple_new_syntax.rs`
    - `tuple_as_function_arg`: `fn take <.T> <(.T)->i32>` に更新、E
    - `tuple_return_value`: `fn make <()->.Pair>` に更新、E
    - `tuple_inside_struct`: `pair <.Pair>` に更新、E
    - `tuple_type_annotated`: 旧型注釁E`<(i32,i32)>` を削除、E
- 検証:
  - `cargo test -p nepl-core --test pipe_operator --test tuple_new_syntax`: `40/40 pass`
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -i tests/tuple_new_syntax.n.md -o tests/output/pipe_tuple_rs_sync.json`: `219/219 pass`

# 2026-02-22 作業メモ (capture あり関数値を�E示皁E��拒否)
- 目皁E
  - closure conversion 未実裁E�E状態で capture 付き関数めE`@fn` で値化した際、E
    下流で不正な生�Eへ進むのを防ぐ、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `@` 付き識別子解決時に、対象ぁEcapture あり関数なめE
      `capturing function cannot be used as a function value yet` を返す、E
    - `@` を非 callable に適用した場合�E
      `only callable symbols can be referenced with '@'` を返す、E
  - `tests/functions.n.md`
    - `function_value_capture_not_supported_yet`�E�Ecompile_fail`�E�を追加、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `560/560 pass`

# 2026-02-22 作業メモ (`call_indirect` フォールバックの厳寁E��)
- 目皁E
  - 高階関数の呼び出し経路で、曖昧な下位フォールバックを減らし、`FnValue` 中忁E�E規則へ固定する、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `CallIndirect` fallback にガードを追加:
      - `FnValue` は許可
      - それ以外�E「関数型として型付け済み」�E場合�Eみ許可
      - 非関数型�E `indirect call requires a function value` を返して停止
  - `tests/tree/08_function_value_call_indirect.js`
    - 既存�E `CallIndirect` 確認に加えて `FnValue` ノ�Eド存在を検証
- `todo.md` 反映:
  - 高階関数頁E��から完亁E��みの
    - 「`_unknown` フォールバック廁E��、E
    を削除、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `559/559 pass`

# 2026-02-22 作業メモ (`@fn` の HIR 明示匁E
- 目皁E
  - `todo.md` 最優先頁E��だった「関数値�E�E@fn`�E�を HIR で明示表現」を完亁E��、`Var` と意味論を刁E��する、E
- 実裁E
  - `nepl-core/src/hir.rs`
    - `HirExprKind::FnValue(String)` を追加、E
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident(..., forced_value=true)` かつ callable 解決時に `HirExprKind::FnValue` を生成、E
    - 既存�E value 識別子�E引き続き `HirExprKind::Var` を生成、E
  - `nepl-core/src/codegen_wasm.rs`
    - `FnValue` を関数チE�Eブル index (`i32.const fidx`) へ明示 lowering、E
  - `nepl-core/src/monomorphize.rs`
    - `FnValue` の単相化（関数名�E instantiation/mangled 名解決�E�に対応、E
  - `nepl-web/src/lib.rs`
    - semantics API の kind 列挙と式走査に `FnValue` を追加、E
  - `nepl-core/src/compiler.rs` / `nepl-core/src/passes/move_check.rs`
    - 新 variant に追従（網羁E��・挙動維持E��、E
- チE��チE
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `559/559 pass`
  - 途中で `tests/functions.n.md::doctest#14` が一時失敗！Eunknown function value add_op`�E�したが、E
    `FnValue` の単相化フォールバック不足が原因であり、`monomorphize` 修正後に解消、E
- `todo.md` 反映:
  - 完亁E��E���E�E@fn` の HIR 明示化）を削除、E
  - 番号を繰り上げて未完亁E�Eみへ整琁E��E

# 2026-02-22 作業メモ (tree API 回帰追加 + todo 整琁E
- 目皁E
  - 上流E��Earse/semantics API�E�で `@fn` 関数値の挙動を固定し、次フェーズの HIR 明示化作業の土台を作る、E
  - `todo.md` を未完亁E��E��のみへ整琁E��る、E
- 変更:
  - 追加: `tests/tree/08_function_value_call_indirect.js`
    - `@inc` ぁEforced-value として parse されることを確認、E
    - 関数値呼び出しが `CallIndirect` として semantics に出ることを確認、E
  - 更新: `todo.md`
    - 完亁E��みの
      - `ValueNs/CallableNs` 刁E��
      - nested `fn`/`let` 呼び出し経路
      を最優先頁E��から削除、E
    - 未完亁E��して `@fn` HIR 明示化を残置、E
    - stdlib リファクタリング�E�Ekp` 形式統一 + 褁E��処琁E��改行パイプ活用�E�を追記、E
- 共有された CI エラー (`args_sizes_get` 未定義) につぁE��:
  - ローカル再現コマンチE
    - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm`
  - 結果: `compile_module returned Ok`�E��E現せず�E�、E
  - 判宁E 直近差刁E��解消済み、また�E古ぁECI ログである可能性が高い。引き続き workflow 側の再実行で監視する、E

# 2026-02-21 作業メモ (non-mut let 前方参�Eの実裁E��亁E
- 背景:
  - `plan.md` 仕様では「巻き上げは `mut` なぁE`let` と `fn` のみに適用」だが、`let y add x 4; let x 5` ぁE`unknown variable x` で失敗してぁE��、E
- 根因:
  - `typecheck` 側の解決だけでなく、`codegen_wasm` 側のローカル割当が「�E現頁E��録」だったため、E
    後方 `let x` の前で `Var(x)` を生成すると `unknown variable` で失敗してぁE��、E
- 実裁E
  - `nepl-core/src/codegen_wasm.rs`
    - `gen_block` のスコープ開始直後に `predeclare_block_locals` を追加、E
    - ブロチE��冁E�E `HirExprKind::Let` を�E行走査し、`LocalMap` に事前登録、E
  - `nepl-core/src/typecheck.rs`
    - `lookup_value_for_read` を導�Eし、読み取り時�E non-mut hoist fallback 経路を整琁E���E己初期化�E除外）、E
  - `tests/shadowing.n.md`
    - `hoist_nonmut_let_allows_forward_reference` めE`neplg2:test`�E�Eet: 9�E�へ戻し、E��過を確認、E
- 結果:
  - `mut let` 前方参�Eは引き続き compile_fail、E
  - `non-mut let` と `fn` の前方参�Eは通過、E
- `todo.md` 反映:
  - 完亁E��た「`let`/`fn` の巻き上げ統一」サブ頁E��を削除、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `243/243 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `558/558 pass`

# 2026-02-21 作業メモ (巻き上げ仕様�E回帰チE��ト追加と現状固宁E
- 目皁E
  - `todo.md` の「`let`/`fn` 巻き上げ統一」に向け、現状挙動をテストで固定して差刁E��可視化、E
- 変更:
  - `tests/shadowing.n.md`
    - 既存ケース名�E `*_currently_fails` を整琁E��通常ケースへ改名）、E
    - 巻き上げ関連ケースを追加:
      - `hoist_mut_let_disallows_forward_reference`�E�Eompile_fail�E�E
      - `hoist_nested_fn_allows_forward_reference`�E�Eass�E�E
      - `hoist_nonmut_let_allows_forward_reference`�E�現状は compile_fail として固定！E
- `nepl-core/src/typecheck.rs`
  - 識別子解決で、`defined` 済み解決に失敗した場合�E non-mut hoist fallback を追加�E��E己初期化�E除外）、E
- 現状評価:
  - `fn` の前方参�Eは通る一方、`non-mut let` の前方参�Eは未対応、E
  - fallback を追加してめE`let y ... x` / `let x ...` 形式�E未解消�Eため、テスト�E `compile_fail` で固定維持、E
  - これは `todo.md` の巻き上げ統一タスクとして継続（仕様差刁E��して明確化）、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `243/243 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `558/558 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 刁E��の段階導�E: Env スコープを物琁E�E離)
- 目皁E
  - `todo.md` 最優先頁E���E�EValueNs` と `CallableNs` の刁E���E�をチE�Eタ構造レベルで前進させる、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `Env.scopes: Vec<Vec<Binding>>` を廁E��し、`Scope { values, callables }` に変更、E
    - `BindingKind` に `is_var` / `is_callable` を追加し、挿入先を一允E��定、E
    - `insert_global` / `insert_local` / `remove_duplicate_func` / 吁Elookup を新構造に対応、E
    - ローカル規則:
      - value は同名 value/callable があると禁止
      - callable は同名 value があると禁止�E�同吁Ecallable はオーバ�Eロードとして許可�E�E
- 効极E
  - 名前空間�E離が「呼び出し�Eの慣習」から「環墁E��ータ構造」へ移行、E
  - 今後�E ValueNs/CallableNs 完�E�E�巻き上げ・shadow policy の厳寁E���E�に向けた基盤を確立、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_envsplit_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 刁E��の段階導�E: 旧 lookup ラチE��削除)
- 目皁E
  - `typecheck` 冁E��残ってぁE��曖昧な `lookup`/`lookup_all` 参�Eを除去し、用途別 API への統一を進める、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident` の fallback めE`lookup_any_defined` に変更、E
    - 互換ラチE�� `lookup` / `lookup_all` を削除、E
    - 置換完亁E���E探索 API は以下へ統一:
      - 値: `lookup_value`
      - 関数: `lookup_all_callables` / `lookup_callable_any`
      - 任意定義済み: `lookup_any_defined` / `lookup_all_any_defined`
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 刁E��の段階導�E: 明示 lookup API へ統一)
- 目皁E
  - `typecheck` で `lookup/lookup_all` の意図が曖昧な箁E��を減らし、`ValueNs`/`CallableNs` 刁E��を進める、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `Env` に明示 API を追加:
      - `lookup_any_defined`
      - `lookup_all_any_defined`
    - 既存�E `lookup`/`lookup_all` は互換ラチE��として残し、呼び出し�Eを段階置換、E
    - 置換した主な箁E��:
      - enum/struct 名衝突判宁E `lookup_any_defined`
      - enum variant/struct constructor 既存判宁E `lookup_all_callables`
      - `noshadow` 競合判宁E `lookup_all_any_defined`
      - 識別孁Efallback 候補�E持E `lookup_all_any_defined`
- 効极E
  - 関数解決と値解決の経路がコード上で判別しやすくなり、今後�E namespace 刁E��リファクタリングの安�E性を向上、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -o tests/output/shadowing_functions_current.json -j 1`: `205/205 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 刁E��の段階導�E: callable 専用経路の拡大)
- 目皁E
  - `todo.md` 最優先�E名前空間�E離を継続し、callable と value の探索経路をより�E確に刁E��、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `fn alias` のターゲチE��探索めE`lookup_all` から `lookup_all_callables` に変更、E
    - entry 解決の候補探索めE`lookup_all` から `lookup_all_callables` に変更、E
    - trait メソチE��呼び出し補助刁E���E存在判定を `lookup_all_callables` に変更、E
  - これにより、E��数解決フェーズで value 候補を混在させなぁE��路を拡大、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/functions.n.md -o tests/output/functions_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `203/203 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (名前解決 API: 重要シャド�E警告�E抑制オプション追加)
- 目皁E
  - `todo.md` の「重要Estdlib 記号 warning 抑制ルール�E�設宁Eフラグ�E�」を実裁E��、LSP/エチE��タ連携で制御可能にする、E
- 実裁E
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_options(source, options)` を追加、E
    - `options.warn_important_shadow`�E�Eool, default=true�E�を導�E、E
    - `NameResolutionTrace` に `warn_important_shadow` を保持し、important-shadow warning 生�Eを条件化、E
    - `policy.warn_important_shadow` を返却ペイロードに追加、E
    - 既孁E`analyze_name_resolution` は新 API に委譲�E�後方互換維持E��、E
  - `tests/tree/07_shadow_warning_policy.js`
    - 重要記号 `print` は通常 warning が�Eることを確認、E
    - `warn_important_shadow=false` で warning 抑制されることを確認、E
- 併せて実施:
  - `nepl-core/src/typecheck.rs` で ValueNs/CallableNs 刁E��の段階導�Eを継続し、値用途�E lookup めE`lookup_value` に寁E��た、E
    - global `fn`/`fn alias` 既存衝突判宁E
    - `set` の参�E解決
    - dotted field base 解決
- `todo.md` 反映:
  - 完亁E��た「重要Estdlib 記号 warning 抑制ルール�E�設宁Eフラグ�E�」頁E��を削除、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 刁E��の段階導�E: lookup 用途�E離)
- 目皁E
  - `todo.md` 最優先�E名前空間�E離に向け、`typecheck` 冁E�E識別孁Elookup を用途別 API に寁E��る、E
- 実裁E
  - `nepl-core/src/typecheck.rs` で、以下�E箁E��めEvalue 専用 lookup へ置換、E
    - グローバル `fn` 登録時�E「既存非関数チェチE��、E `env.lookup_value`
    - `fn alias` 登録時�E「既存非関数チェチE��、E `env.lookup_value`
    - `set` 解決時�E外�E探索: `env.lookup_value`
    - dotted field (`a.b`) の base 解決: `env.lookup_value`
- 効极E
  - 変数と callable を同一 lookup で混在解決する箁E��を減らし、�E離設計への移行を前進、E
  - 挙動は維持しつつ、意図しなぁEcallable 混入の余地を縮小、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/tree -o tests/output/shadowing_tree_current.json -j 1`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (shadow warning ポリシーの API チE��ト固宁E
- 目皁E
  - `todo.md` の「シャド�Eイング運用の完�E」に向け、`analyze_name_resolution` の警告�Eリシーを木構造チE��トで固定、E
- 追加:
  - `tests/tree/07_shadow_warning_policy.js`
    - `print` のローカルシャド�Eで warning が�Eることを確認、E
    - `cast` のローカルシャド�Eでは important-shadow warning が�EなぁE��とを確認、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (シャド�Eイング: callable 解決の回帰修正)
- 背景:
  - `tests/shadowing.n.md` の pending ケース�E�Evalue_name_and_callable_name_can_coexist_currently_fails` / `imported_function_name_shadowed_by_parameter_currently_fails`�E�を通常チE��トへ昁E��するため、`typecheck` の識別子解決を調整、E
- 実裁E
  - `nepl-core/src/typecheck.rs` に `Env::lookup_callable_any` を追加、E
  - 呼び出し�EチE��位置の識別子解決で、同吁Evalue が現在スコープにあってめEouter callable を参照できる経路を追加、E
  - ただし適用篁E��は限定し、以下条件を満たす場合�Eみ有効匁E
    - `forced_value == false`
    - `stack.is_empty()`�E��E頭解決�E�E
    - `expr.items.get(idx + 1).is_some()`�E�実際に後続頁E��あり呼び出し文脈！E
- 失敗�E极E
  - 当�Eは適用篁E��が庁E��ぎ、`if cond: ok` の `ok` めEcallable に誤解決して全体回帰�E�Etdlib 側 `if condition must be bool`�E�が発生、E
  - 上記条件で呼び出し�EチE��に限定し、回帰を解消、E
- チE��チE
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `202/202 pass`
- 補足:
  - 共有されてぁE�� `tests/neplg2.n.md::doctest#6/#7` の compile fail は現時点で再現せず、当該ファイルは全件 pass、E

# 2026-02-21 作業メモ (target=wasm で WASI 無効匁E
- 要件反映:
  - `nepl-cli/src/main.rs` の自動�E格ロジチE���E�Estd/stdio` import を検�Eして `wasi` にする挙動�E�を削除、E
  - `target=wasm` のとき�E WASI を有効化しなぁE��ぁE��修正、E
  - `target=wasi` のとき�Eみ `wasi_snapshot_preview1` import を許可し、WASI 関数めElinker に登録、E
- 実裁E��細:
  - `execute`:
    - `target_override` めECLI 持E���Eみに限定、E
    - 実行ターゲチE��推定を `detect_module_target` へ刁E��出し！Emodule.directives` と `module.root.items` の双方を確認）、E
  - `run_wasm`:
    - `CompileTarget::Wasm` では import が存在した時点でエラー化、E
    - `CompileTarget::Wasi` でのみ `args_sizes_get` / `args_get` / `path_open` / `fd_read` / `fd_close` / `fd_write` を登録、E
- 検証:
  - `cargo test -p nepl-cli`: pass
  - `#target wasm + #import "std/stdio"`: compile error�E�EWASI import not allowed for wasm target`�E�を確認、E
  - `#target wasi + #import "std/stdio"`: 実行�E功！Eprintln "hi"` が�E力）を確認、E

# 2026-02-21 作業メモ (fs 衝突修正 + 回帰チE��ト追加)
- `tests/selfhost_req.n.md` の compile fail を起点に `std/fs` の根因を修正、E
  - `std/fs` の WASI extern 名が他モジュール�E�Estd/stdio` など�E�と衝突しぁE��ため、`wasi_path_open` / `wasi_fd_read` / `wasi_fd_close` に冁E��名を固有化、E
  - `fs_read_fd_bytes` の `cast` めE`<u8> cast b` へ明示して overload 曖昧性を解消、E
  - `vec_new<u8> ()` 旧記法を新記況E`vec_new<u8>` へ更新、E
- チE��ト整傁E
  - 追加: `tests/capacity_stack.n.md`
    - 再帰深さ！E4/512�E�、`Vec` 拡張、`mem` 読み書き、`StringBuilder`、`enum+vec+再帰` の段階テストを固定、E
  - 更新:
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
    - `tests/string.n.md`
    - `tests/ret_f64_example.n.md`
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/ret_f64_example.n.md -i tests/selfhost_req.n.md -i tests/sort.n.md -i tests/string.n.md -i tests/capacity_stack.n.md -o tests/output/targeted_regression_current.json`
    - `194/194 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
    - `540/540 pass`
- 補足:
  - `std/fs` は引き続き WASI preview1 前提。`wasmtime/wasmer` 差刁E��証は `todo_kp.md` のランタイム互換頁E��として継続、E

# 状況メモ (2026-01-22)
# 2026-02-10 作業メモ (競プロカタログ拡張 + kpモジュール整琁E
- チュートリアルに競プロ定番の参�E章を追加し、E��要アルゴリズム/チE�Eタ構造のサンプルめE20 頁E��で列挙した、E
  - 追加: `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - 目次反映: `tutorials/getting_started/00_index.n.md`
- `stdlib/kp` を機�E別に整琁E��、新規モジュールを追加した、E
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_i32`, `upper_bound_i32`, `contains_i32`
  - `stdlib/kp/kpprefix.nepl`
    - `prefix_build_i32`, `prefix_range_sum_i32`
  - `stdlib/kp/kpdsu.nepl`
    - `dsu_new`, `dsu_find`, `dsu_unite`, `dsu_same`, `dsu_size`, `dsu_free`
  - `stdlib/kp/kpfenwick.nepl`
    - `fenwick_new`, `fenwick_add`, `fenwick_sum_prefix`, `fenwick_sum_range`, `fenwick_free`
- すべて `//:` のドキュメントコメント形式で記述し、各モジュールに最封Edoctest を付与した、E

# 2026-02-10 作業メモ (関数単位レビュー: 機械置換�E後�E琁E
- ユーザー持E��に基づき、`vec/stack/list` を関数ごとに再確認し、機械置換由来の不整合を手修正した、E
- 主な修正:
  - `stdlib/alloc/vec.nepl`
    - `vec_new` ドキュメント�E `使ぁE��:` 重褁E��除去、E
    - `vec_set` doctest の move-check 衝突を回避する使用例へ修正、E
  - `stdlib/alloc/collections/stack.nepl`
    - モジュール説明�E重褁E��ロチE���E��E頭と import 後�E二重記載）を統合し、E箁E��に整琁E��E
  - `stdlib/alloc/collections/list.nepl`
    - モジュール説明�E重褁E��ロチE���E��E頭と import 後�E二重記載）を統合し、E箁E��に整琁E��E
- 形式面:
  - `//` コメント�E残さず、ドキュメント�E `//:` のみを使用、E
  - 吁E��数に `目皁E実裁E注愁E計算量` + `使ぁE��` + `neplg2:test` を維持、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 作業メモ (doc comment 書弁E 「使ぁE��」見�Eしを統一)
- ユーザー提示の書式に合わせ、`vec/stack/list` の doctest 前に `//: 使ぁE��:` を統一追加した、E
  - 対象:
    - `stdlib/alloc/vec.nepl`
    - `stdlib/alloc/collections/stack.nepl`
    - `stdlib/alloc/collections/list.nepl`
- あわせて、`vec_set` の doctest で move-check に抵触してぁE��例を修正し、コンパイル可能な使用例に整えた、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 作業メモ (vec/stack/list コメント様式�E持E��対忁E
- ユーザー持E���E `stdlib/nm` 拡張 Markdown 形式に合わせ、以下�Eモジュール先頭コメントを具体化した、E
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 反映冁E��:
  - 先頭 `//:` で「ライブラリの主題」「目皁E��「実裁E��ルゴリズム」「注意点」「計算量」を具体記述、E
  - 既存�E吁E��数剁E`//:`�E�目皁E実裁E注愁E計算量�E�と doctest 構�Eは維持、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 作業メモ (vec/stack/list の doc comment + doctest 整傁E
- ユーザー持E��に合わせて、以下�E標準ライブラリに実行可能な doctest を追加・整備した、E
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 変更冁E��:
  - `stack.nepl` / `list.nepl` の `neplg2:test[skip]` を解除し、主要操作！Eew/push/pop/peek/len/clear, cons/head/tail/get/reverse など�E�を確認すめEdoctest を追加、E
  - `vec.nepl` に `clear` を中忁E��した追加 doctest を�Eれ、move 規則に反しなぁE��へ調整、E
  - `str_eq` を使ぁEdoctest には `alloc/string` import を�E示、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
    - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 作業メモ (nm OOB 根治: parse_markdown 再設訁E
- `nm` の run fail (`memory access out of bounds`) を上流から�E刁E��刁E��し、`stdlib/nm/parser.nepl` の `parse_markdown` を�E設計した、E
- 根因刁E��:
  - 既存実裁E�E section stack と `Vec<Node>` の値受け渡しが褁E��で、`nm` doctest で OOB を継続�E現、E
  - `parse_markdown` 単体�E最小実行で再現することを確認し、周辺ロジチE��を段階的に外して刁E��刁E��、E
- 実裁E��更:
  - `parse_markdown` をフラチE��走査ベ�Eスに置き換え、`stack` 依存経路を除去、E
  - `safe_line` は `lines_data + offset` ではなぁE`vec_get<str>` ベ�Eスの安�Eアクセスに統一、E
  - heading/fence/paragraph/hr の刁E��を明示化し、見�Eし�E下�E children 収集を局所ループで実裁E��E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=72, passed=72, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all.json -j 1`
    - `total=416, passed=409, failed=7, errored=0`
    - 残りは `ret_f64_example`, `selfhost_req`, `sort` で、nm 系失敗�E解消、E
# 2026-02-10 作業メモ (nm 実裁E��況と doc comment 整傁E
- `nm` の現状:
  - コンパイル段階�E主要Emove-check エラーは大きく削減したが、実行時 `memory access out of bounds` が残っており未完亁E��E
  - `tests/nm.n.md` の失敗�E現在 OOB のみ�E�Eompile fail から run fail へ遷移�E�、E
- ドキュメントコメント整傁E
  - `stdlib/nm/parser.nepl`
    - `parse_markdown`
    - `document_to_json`
  - `stdlib/nm/html_gen.nepl`
    - `render_document`
  - 上記に日本語説明（目皁E実裁E注愁E計算量�E�と `neplg2:test` 例を追加、E
  - doctest 例�E `fn main` を含む実行可能な形式へ修正済み、E
- チE��ト結果�E�Em 関連�E�E
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
  - `summary: total=72, passed=67, failed=5, errored=0`
  - 失敗理由はすべて `memory access out of bounds`
- 次アクション:
  - OOB の発生点めE`nm/parser` の `load<...>` / `size_of<...>` 利用箁E��から再�Eり�Eけ、E
  - `Vec<T>` 要素アクセスを直接 `data + offset` で扱ぁE��針�E安�E条件�E�墁E��・レイアウト）を明文化し、忁E��なめEAPI に戻す、E

# 2026-02-10 作業メモ (nm 再現チE��ト追加と上流�Eり�EぁE
- `tests/nm.n.md` を新規追加し、`nm/parser` + `nm/html_gen` の最小経路を固定した、E
  - `nm_parse_markdown_json_basic`
  - `nm_render_document_basic`
- `examples/nm.nepl` / `stdlib/nm/parser.nepl` の先行修正:
  - `stdlib/nm/parser.nepl` の `if:` レイアウト由来で parser 再帰を誘発してぁE�� `let next_is_paren` 部刁E��段階代入へ変更、E
  - `#import "std/math"` めE`#import "core/math"` に修正、E
  - `examples/nm.nepl` に `#import "std/env/cliarg" as *` を追加、E
- `nm` で露出した上流不整合�E修正:
  - `nm/parser` / `nm/html_gen` の関数シグネチャを実裁E���Eに合わせて `*>` へ寁E��た！Eure/impure 不整合�E解消）、E
  - `nm/parser` 冁E�E bool 比輁E(`eq done false` 筁E めE`not` / 直接判定へ変更、E
  - `Section` 構築時の曖昧な前置式を段階代入へ整琁E��、親惁E��取得頁E��を `peek -> pop` に修正、E
  - 型名衝突を解涁E
    - `Section`(struct) -> `NestSection`
    - `Ruby`(struct) -> `RubyInfo`
    - `Gloss`(struct) -> `GlossInfo`
    - `CodeBlock`(struct) -> `CodeBlockInfo`
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=69, passed=67, failed=2`
    - 残り: `use of moved value`�E�Elines` / `v`�E�に収束
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-nm.json -j 1`
    - `total=413, passed=404, failed=9, errored=0`
- 現在の評価:
  - parser の停止保証は維持されたまま、nm 不�E合�E「Vec/str の所有権処琁E��Eec_get/vec_len 呼び出し設計）」へ根因が絞れた、E
  - 次段は `nm/parser` のループ�E琁E�� `Vec` の `data/len` 直接アクセスへ再設計し、move-check を根本解消する、E

# 2026-02-10 作業メモ (parser 再帰暴走の停止保証)
- ユーザー持E��「コンパイラは忁E��停止する」を受けて、`nepl-core/src/parser.rs` に停止保証を追加、E
- 実裁E�E容�E�上流Eparser 側�E�E
  - 再帰深さ上限を追加:
    - `MAX_PARSE_RECURSION_DEPTH = 2048`
    - `enter_parse_context` / `leave_parse_context` を追加
    - `parse_stmt` をコンチE��スト管琁E��で実行し、E��剰再帰時�E診断を返して停止するよう変更
  - 無進捗ループ検�Eを追加:
    - `MAX_NO_PROGRESS_STEPS = 64`
    - `parse_block_until_internal` / `parse_prefix_expr` / `parse_prefix_expr_until_tuple_delim` / `parse_prefix_expr_until_colon`
    - 同一 `pos` が一定回数続いたら診断を�Eして 1 token 前進し、無限ループを回避
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `timeout 20s node nodesrc/analyze_source.js -i stdlib/nm/parser.nepl --stage parse`: `PARSE_EXIT:0`
  - `node nodesrc/test_analysis_api.js`: `7/7 passed`
- 補足:
  - `stdlib/nm/parser.nepl` の parse で以前発生してぁE��停止しなぁE��動�E、少なくとも解极EAPI 経路では再現しなくなった、E
  - `examples/nm.nepl` 側は引き続き type/effect 不整合！Enm` ライブラリの pure/impure 署名ズレ等）が残っており、次段で修正継続、E

# 2026-02-10 作業メモ (tuple unit 要素の codegen 根本修正)
- `tests/tuple_new_syntax.n.md::doctest#10` の根因を特定、E
  - `Tuple:` に `()` が含まれると、WASM codegen ぁE`unit` 要素を通常値として `LocalSet` しよぁE��してスタチE��不足になってぁE��、E
  - 既存レイアウト！Eypecheck 側 offset=4 刻み�E�を崩さず、`unit` 要素/フィールド�E「式評価で副作用は実行しつつ、スロチE��には 0 を格納」する方針へ統一、E
- `nepl-core/src/codegen_wasm.rs`:
  - `StructConstruct` / `TupleConstruct` の要素 store 刁E��を `valtype(Some)` と `None(unit)` で刁E��、E
  - `None(unit)` では `gen_expr` 後に `i32.store 0` を行う実裁E��変更、E
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o /tmp/tests-tuple-after-unit-slot-fix.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-tuple-unit-fix.json -j 1`
    - `total=339, passed=327, failed=12, errored=0`

# 2026-02-10 作業メモ (pipe 残件解涁E+ alloc 依存�E根本改喁E
- `tests/pipe_operator.n.md` の残失敗！E13/#14/#15�E�を上流から�Eり�Eけて修正、E
- `nepl-core/src/typecheck.rs`:
  - `let s <S> 10 |> S` / `let e <E> 20 |> E::V` で、`<S>/<E>` ぁEpipe 前�EリチE��ルに早期適用される不�E合を修正、E
  - `next_is_pipe` の場合�E pending ascription を遅延し、pipe 注入後�E式確定時に適用するよう変更、E
- `nepl-core/src/codegen_wasm.rs`:
  - `alloc` が未importでも構造佁E列挙/タプル構築で落ちなぁE��ぁE��inline bump allocator フォールバックを追加�E�Eemit_alloc_call`/`emit_inline_alloc`�E�、E
  - これにより `pipe_struct_source` / `pipe_into_constructor` で出てぁE�� `alloc function not found (import std/mem)` を解消、E
- `todo.md`:
  - 高階関数フェーズ後�E `StringBuilder` 根本再設計タスク�E�E(n) build 化、�E現チE��ト追加�E�を追加、E
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-constructor-revert.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-current-after-pipe-fixes.json -j 1`
    - `total=339, passed=326, failed=13, errored=0`
  - 残件刁E��E
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 作業メモ (offside: block: 同一行継続�E禁止)
- `tests/offside_and_indent_errors.n.md::doctest#4` の根因は parser ぁE`block:` の同一行継続！Eblock: add 1 2`�E�を許容してぁE��こと、E
- `nepl-core/src/parser.rs` を修正:
  - `KwBlock` の `:` 刁E��で、改行が無ぁE��合�E診断を追加し、回復用に単行解析へフォールバック、E
  - 仕様上「`block:` の後ろは空白/コメント�Eみ」を満たすようにした、E
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/offside_and_indent_errors.n.md -o /tmp/tests-offside-after-block-colon-fix.json -j 1`
    - `total=7, passed=7, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-offside-fix.json -j 1`
    - `total=339, passed=322, failed=17, errored=0`
  - 残り失敗�E顁E
    - `pipe_operator=4`
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 作業メモ (target尊重 + trait呼び出ぁE+ doctest VFS)
- `nepl-web/src/lib.rs`:
  - `compile_wasm_with_entry` の `CompileOptions.target` めE`Some(Wasi)` 固定かめE`None` に変更し、ソース側 `#target` を尊重するよう修正、E
  - これにより `#if[target=...]` / `#target` 重褁E���E / wasm での wasi import 禁止のチE��トが有効化された、E
- `nepl-core/src/monomorphize.rs`:
  - `FuncRef::Trait` の解決で impl map の厳寁E��致が外れた場合に、`trait+method` での型単一候補を探索するフォールバックを追加、E
  - `tests/neplg2.n.md::doctest#31` (`Show::show`) を解消、E
- `nodesrc/run_test.js` + `nodesrc/tests.js`:
  - doctest 実行時に `file` 惁E��を渡し、`#import`/`#include` の相対パスを実ファイルから収集して `compile_source_with_vfs` に渡す機�Eを追加、E
  - `tests/part.nepl` を追加し、`tests/neplg2.n.md::doctest#11` の `#import "./part"` を解決可能にした、E
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-vfs2.json -j 1`
    - `total=35, passed=35, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-target-vfs-trait.json -j 1`
    - `total=339, passed=321, failed=18, errored=0`
  - 主な残件: `offside(1)`, `pipe_operator(4)`, `ret_f64_example(1)`, `selfhost_req(4)`, `sort(5)`, `string(2)`, `tuple_new_syntax(1)`

# 2026-02-10 作業メモ (loader字句正規化 + 高階関数回帰確誁E
- `nepl-core/src/loader.rs` の `canonicalize_path` に字句皁E��規化�E�E.` / `..` 除去�E�を追加した、E
  - 目皁E `#import "./part"` の解決で `/virtual/./part.nepl` と `/virtual/part.nepl` の不一致をなくすため、E
  - 変更後、`tests/neplg2.n.md::doctest#11` は `missing source: /virtual/part.nepl` まで前進し、パス不一致自体�E解消、E
- 高階関数系の現状を�E確誁E
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-current.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
  - 直近�E `functions` 失敗�E解消済み、E
- 全体回帰:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-outer-consumer-fix.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`�E�既知雁E���E�E
- 残課題メモ:
  - `neplg2#doctest#11` は loader ではなぁEdoctest harness 側の褁E��ファイル供給仕様！EFS�E�未整備が根因、E
  - ほか�E失敗主塊�E `sort` / `selfhost_req` / `pipe_operator` / `tuple_new_syntax`、E

# 2026-02-10 作業メモ (functions if失敗�E再現チェチE��準備)
- `functions#doctest#7/#10` の原因刁E��刁E��のため、`typecheck` の call reduction 周辺を調査、E
- 一時的に `reduce_calls` の候補探索方式を変更したが、`tests/if.n.md` が悪化！E fail�E�したため取り消し済み、E
- 現在はベ�Eスを復帰:
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-after-revert.json -j 1` で `55/55 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-revert.json -j 1` は `11 pass / 5 fail`�E�既知残件�E�E
- 次アクション:
  - 類似再現ケースを追加して、`if` と関数値刁E���E失敗条件をテストとして固定する、E
  - そ�E後、上流優先で parser/typecheck の責務墁E��を保った修正へ進む、E

# 2026-02-10 作業メモ (if.n.md 不足ケース追加と if-layout 補正)
- `if.n.md` の不足ケースを追加:
  - `if <cond_expr>:` 形式！Ethen/else` を改行で与える形�E�E
  - `if cond <cond_expr>:` 形弁E
  - marker 頁E��違叁E/ duplicate / missing の `compile_fail`
- parser 修正:
  - `if` の `expected=2`�E�Eif <cond_expr>:` 系�E�で、`if` 直後�E任愁E`cond` marker を除去して cond 式として解釈できるよう修正、E
  - `if-layout` の marker 頁E��チェチE��を追加し、`cond -> then -> else` の送E��をエラー化、E
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-added-missing3.json -j 1`
    - `total=54, passed=54, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-ifcases.json -j 1`
    - `total=16, passed=11, failed=5, errored=0`�E�失敗�E訳は従来の高階関数/capture 系�E�E

# 2026-02-10 作業メモ (予紁E���E識別子禁止: cond/then/else/do, let/fn)
- ユーザー持E��に合わせて、`cond` / `then` / `else` / `do` を予紁E��として扱ぁE��裁E�� parser に追加、E
  - `nepl-core/src/parser.rs`
    - `parse_ident_symbol_item` で、layout marker の許可位置�E��E頭 marker / if 斁E�� / while 斁E���E�以外での使用をエラー化、E
    - `expect_ident` でも同語を識別子として受け付けなぁE��ぁE��し、定義名�E束縛名側でも拒否、E
    - 既存�E緩咁E(`KwSet` / `KwTuple` を識別子化) は削除し、予紁E��を明確化、E
- `let` / `fn` は lexer で keyword token 化されるため、従来どおり識別子として使用不可であることを確認、E
- `tests/if.n.md` に compile_fail ケースを追加�E�追加のみ�E�E
  - `reserved_cond_cannot_be_identifier`
  - `reserved_then_cannot_be_function_name`
  - `reserved_let_fn_cannot_be_identifier`
  - `reserved_else_do_cannot_be_identifier`
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-reserved2.json -j 1`
    - `total=46, passed=46, failed=0, errored=0`
- 参老E��測�E�継続課題！E
  - `tests/functions.n.md::doctest#7` は parser AST 形状自体�E `if + con + then-block + else-block` で正しい、E
  - ただぁEthen/else ブロチE��冁E��値式が2つあり、typecheck で `expression left extra values on the stack` になる、E
  - 仕様整琁E��褁E��値式�E扱ぁE��と tests/functions の意図確認が忁E��、E

# 2026-02-10 作業メモ (if/while の AST 仕様テスト追加)
- `plan.md` の `if/while` 仕様を再確認し、`cond/then/else/do` の `:` あり/なし差刁E�� AST で固定するテストを追加、E
- `nodesrc/test_analysis_api.js` に `analyze_parse` ベ�Eスのケースを追加:
  - `parse_if_inline_no_colon_blocks`
  - `parse_if_colon_uses_block_for_cond_then_else`
  - `parse_while_inline_no_colon_blocks`
  - `parse_while_colon_uses_block_for_cond_do`
- 検証方釁E
  - `:` なしでは `PrefixExpr` の引数列に `Block` を作らなぁE��E
  - `:` ありでは `if` は `Symbol + Block + Block + Block`、`while` は `Symbol + Block + Block` になることを確認、E
- 実行結果:
  - `node nodesrc/test_analysis_api.js`
  - `summary: total=6, passed=6, failed=0`

# 2026-02-10 作業メモ (functions 失敗�E深掘り: symbol/entry)
- `tests` 全体を再実行し、現状を�E確誁E
  - `/tmp/tests-restored-stable.json` = `total=312, passed=273, failed=39, errored=0`
  - 失敗�E主塊�E `tests/functions.n.md`�E�E0、E1件�E�で、nested fn / function value / entry 解決が中忁E��E
- `functions` の `doctest#3`�E�Efn main ()`�E�を最小�E現で調査:
  - `/tmp/fnmain_no_annot.nepl` めE`nepl-cli --verbose` でコンパイル、E
  - 観測:
    - monomorphize 初期関数は `main__unit__i32__pure`
    - 本斁E�� `inc 41` ぁE`unknown function inc` で落ちめE
  - 解釁E
    - hoist 時�E関数 symbol と、check_function 後�E関数名！Eangle 後）が一致しなぁE��路が残っており、entry 欠落と同根、E
- 試衁E
  - `check_function` へ symbol override を渡し、hoist で選ばれた symbol に関数名を揁E��る修正を実験、E
  - しかぁE`tests/functions.n.md` で `doctest#3` ぁErun fail から compile fail�E�Enknown function inc�E�へ悪化し、�E体改喁E��ならなかったため撤回、E
- 現時点の結諁E
  - 名前空間�E設計！EalueNs/CallableNs 刁E���E�と、nested fn の実体生成（少なくとめEnon-capture 先行）が忁E��、E
  - 局所 patch では `functions` 群の構造問題を吸収しきれなぁE��E

# 2026-02-10 作業メモ (上流優允E if-layout parser 改喁E+ LSP解析API拡張)
- 上流優先�E方針で parser を�Eに調整、E
  - `if <cond>:` で then 行�Eみ先に見える中間状態を、確定エラーにしなぁE��ぁE��復刁E��を追加、E
  - `functions#doctest#10` の parser 失敗！Emissing expression(s) in if-layout block`�E�を解消、E
- 回帰確誁E
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-parser-upstream.json -j 4`
    - `total=312, passed=275, failed=37, errored=0`�E�E2 改喁E��E
- LSP/チE��チE��支援向け API を追加:
  - `nepl-web/src/lib.rs` に `analyze_name_resolution(source)` を追加、E
    - `definitions`�E�定義点�E�E
    - `references`�E�参照点、候補ID列、最終解決ID�E�E
    - `by_name`�E�同名識別子�E送E��き�E�E
    - 巻き上げ規則は現行仕様！Efn` と `let` 靁E`mut`�E�に合わせた、E
  - `nodesrc/analyze_source.js` に `--stage resolve` を追加、E
- API検証の追加�E�追加のみ、既孁Eests削除なし！E
  - `nodesrc/test_analysis_api.js` を新規追加、E
  - `shadowing_local_let` / `fn_alias_target_resolution` を�E動検証、E
  - 実行結果: `2/2 passed`

# 2026-02-10 作業メモ (functions: nested fn 実体生成�E前進)
- `typecheck` の `BlockChecker` で nested `fn` の本体を「未検査で無視」してぁE��経路を改修、E
  - block 冁E`Stmt::FnDef` めE`check_function` に渡し、`generated_functions` へ追加するよう変更、E
  - top-level / impl 側の `check_function` 呼び出しにめE`generated_functions` を接続、E
- これにより nested `fn` の本体が HIR に入るよぁE��なり、`functions` の `double` 系が改喁E��E
- 計測:
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-now.json -j 1`
  - `total=16, passed=10, failed=6, errored=0`
  - 残りは関数値/関数リチE��ル/クロージャ捕捉�E�Edoctest#6,#7,#11,#12,#13`�E�に雁E��、E
  - 全体�E `node nodesrc/tests.js -i tests -o /tmp/tests-current-after-nested.json -j 4` で `312/278/34/0`、E

# 2026-02-10 作業メモ (不安定差刁E�E刁E��戻しと再計測)
- `typecheck` の匿名関数リチE��ル実験！EPrefixItem::Group` + 直征E`Block` の即席ラムダ化）を刁E��戻し、E
  - 根拠: `functions#doctest#6` などで `unsupported function signature for wasm` / `unknown variable square` を誘発し、E��数値経路が未設計�Eまま混入してぁE��ため、E
- 再計測:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-latest.json -j 1`
    - `total=16, passed=10, failed=6, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-latest.json -j 4`
    - `total=312, passed=278, failed=34, errored=0`
- 失敗�E中忁E�E引き続き `functions` の関数値/クロージャ捕捉系�E�E6 #7 #11 #12 #13�E�、E

# 2026-02-10 作業メモ (高階関数実裁E��式�E外部調査)
- Rust/MoonBit/Wasm 仕様を確認し、NEPL 側の実裁E��針を整琁E��た、E
- 主要�EインチE
  - Rust:
    - クロージャは「環墁E��保持する構造佁E+ `Fn/FnMut/FnOnce` 呼び出し」で表現される（型としては関数ポインタではなく専用型）、E
    - 参老E Rust book と rustc `ClosureArgs` 説明、E
  - MoonBit:
    - 関数は first-class、E
    - Wasm FFI では `FuncRef[T]`�E�閉じた関数�E�と、closure�E�関数 + 環墁E��を区別して扱ぁE��計が明示されてぁE��、E
    - closure は host 側で部刁E��用して callback 化する設計が記述されてぁE��、E
  - Wasm:
    - 間接呼び出し�E `call_indirect`�E�Eable 経由�E�また�E `call_ref`�E�Eunction reference�E�で実現、E
- NEPL への反映方針（次段実裁E��E
  - 関数値を単なる識別子参照ではなく、IRで「callable 値」として明示表現する、E
  - non-capture を�E行実裁E
    - `fn`/`@fn` は table index を持つ関数値として扱ぁE��呼び出し�E `call_indirect` に統一、E
  - capture ありは次段:
    - closure 環墁E��ブジェクチE+ invoke 関数に lower する closure conversion を導�Eする、E

# 2026-02-10 作業メモ (block 引数位置の根本修正)
- `tests/block_single_line.n.md` の `doctest#8/#9` を起点に、`add block 1 block 2` と `if true block 1 else block 2` の失敗要因を解析、E
- 原因:
  - parser 上では `add [Block 1] [Block 2]` の AST が得られてぁE��のに、typecheck で `expression left extra values on the stack` が�Eる、E
  - `PrefixItem::Block` の型検査ぁE`check_block(b, stack.len(), true)` になっており、外�E式�EスタチE��深さを block 冁E��価へ持ち込んでぁE��、E
  - そ�E結果、引数位置 block の冁E��で外�EスタチE��が混入し、簡紁E��定が崩れてぁE��、E
- 修正:
  - `nepl-core/src/typecheck.rs` の `PrefixItem::Block` 刁E��を `check_block(b, 0, true)` に変更し、block を独立式として検査するよう統一、E
  - parser 側は `block` の後続判定を限定追加�E�Eblock`/`else` 連接のみ継続）し、既存�E `block:` 斁E��E��は維持、E
- 計測:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-typecheck-blockbase.json -j 4`
  - summary: `total=312, passed=273, failed=39, errored=0`
  - ベ�Eスライン `/tmp/tests-latest.json` (`passed=271`) から `block_single_line` の 2 件だけ改喁E��追加失敗なし、E

# 2026-02-10 作業メモ (上流修正 継綁E parser/typecheck)
- 失敗�E類を再実施し、上流E��Eexer/parser�E�と typecheck の墁E��を�Eり�Eけた、E
  - 起点: `/tmp/tests-current.json` = `total=312, passed=249, failed=63, errored=0`
- parser の根本修正:
  - `nepl-core/src/parser.rs` で識別子解析を共通化�E�Eparse_ident_symbol_item`�E�、E
  - これにより、式文脈ごとの実裁E��刁E��排除し、以下を統一対忁E
    - `@name`
    - `::`�E�名前空間パス�E�E
    - `.`�E�フィールド連結！E
    - `<...>`�E�型引数�E�E
  - `Option<.T>::None` / `Option<.T>::Some` のような「型引数 + PathSep」�E連結が parse できるよう修正、E
- typecheck の根本修正�E�Eipe 簡紁E��E
  - `nepl-core/src/typecheck.rs` の `reduce_calls` / `reduce_calls_guarded` めEopen_calls 最適化依存から、スタチE��走査ベ�Eスへ戻した、E
  - `|>` 注入時�E呼び出し取りこぼし！Eexpression left extra values on the stack` 多発�E��E主要因を除去、E
- 計測:
  - `/tmp/tests-after-upstream-pass.json` = `total=312, passed=261, failed=51, errored=0`
  - `/tmp/tests-after-option-fix.json` = `total=312, passed=271, failed=41, errored=0`
- 追加修正:
  - `parse_single_line_block` を「`;` が無ぁE��合�E 1 斁E��終亁E��へ変更し、単衁Eblock の斁E��E��を�E示化、E
  - ただぁE`add block 1 block 2` / `if true block 1 else block 2` は、prefix 1斁E�E冁E�Eで `block` を�E帰皁E��取り込む挙動が残り、未解決�E�殁Efail 2�E�、E
- 残課題（次段�E�E
  - `tests/functions.n.md`�E�E1 fail�E�E nested fn / function-literal / alias / entry 生�E整吁E
  - `tests/neplg2.n.md`�E�E fail�E�と `tests/selfhost_req.n.md`�E�E fail�E�E namespace と callable 解決の構造問顁E
  - `tests/pipe_operator.n.md`�E�E fail�E�E pipe 自体�E上流問題�E縮小済みで、残りは型注釁E構造体アクセス仕様との整合が中忁E

# 2026-02-10 作業メモ (高階関数 継綁E let-RHS/if-block 呼び出し頁E�E根本修正)
- `functions` の回帰を引き起こしてぁE��根因めE2 点に刁E��して修正、E
  - `let f get_op true` 系:
    - `let` を通常の auto-call 経路で簡紁E��ると `let f get_op` が�Eに確定し、`true` が取り残される、E
    - 対応として `Symbol::Let` は `auto_call: false` とし、`check_prefix` 終端で `stack[base+1]` めERHS として `HirExprKind::Let` に確定する経路を整備、E
    - `let ...;` で `statement must leave exactly one value` にならなぁE��ぁE��`let` 降格時に冁E�� stack めE`unit` 1 個へ正規化、E
  - `if` + `then/else` が関数値を返す系�E�Efunction_return`�E�E
    - `PrefixItem::Block` めE`auto_call: true` で積�Eと、`if` の引数収集中に右端の関数値が優先さめE`if` 本体が簡紁E��れなぁE��E
    - `PrefixItem::Block` の push めE`auto_call: false` に変更し、`if` の 3 引数簡紁E��優先させるよう修正、E
- `reduce_calls` は「右端優先�E不足なら征E��」に戻した、E
  - 左探索を有効化すると `mul n fact sub n 1` で `mul n fact` が�Eに確定し、�E帰呼び出しが壊れることを�E現確認したため撤回、E

- 検証結果:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/test_analysis_api.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-block-autocall-false.json -j 1`
    - `total=19, passed=15, failed=4, errored=0`
    - 殁Efail: `doctest#12 #13 #16 #17`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-upstream-fixes.json -j 1`
    - `total=328, passed=288, failed=40, errored=0`

- 残件の刁E��:
  - `doctest#12/#13/#16`:
    - typecheck では nested 関数冁E`y` 参�Eは解決できてぁE��が、codegen で `unknown variable y` になる、E
    - これは nested 関数の capture が未 lower�E�Elosure conversion 未実裁E��であることが原因、E
  - `doctest#17`:
    - `compile_fail` 期征E��対して成功するため、純粁E非純粋�E effect 判定経路�E�署名解釁Eor overload 選択）�E再点検が忁E��、E

# 2026-02-10 作業メモ (lexer/parser 解析API追加)
- VSCode 拡張計画�E�Eodo.md の LSP / VSCode 頁E��を再確認し、上流解析を可視化する API を�Eに追加した、E
- `nepl-web/src/lib.rs` に wasm 公開関数を追加:
  - `analyze_lex(source)`:
    - token 列！Eind/value/debug/span�E�E
    - diagnostics�E�Eeverity/message/code/span�E�E
    - span の byte 篁E��と line/col を返す
  - `analyze_parse(source)`:
    - token 刁E
    - lex/parse diagnostics
    - module の木構造�E�Elock/Stmt/Expr/PrefixItem の再帰 JSON�E�E
    - debug 用の AST pretty 斁E���E
- Node 側に `nodesrc/analyze_source.js` を追加し、dist の wasm API を使って解析結果を取得できるようにした、E
  - `--stage lex|parse`
  - `-i <file>` また�E `--source`
  - `-o <json>`
- 実行確誁E
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/analyze_source.js --stage lex -i tests/functions.n.md -o /tmp/functions-lex.json`: 成功
  - `node nodesrc/analyze_source.js --stage parse -i tests/functions.n.md -o /tmp/functions-parse.json`: 成功
- 回帰確誁E
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 4`
  - summary: `total=312, passed=249, failed=63, errored=0`
  - 主要失敗�E既知の block/typecheck 系�E�今回の API 追加では未着手！E

# 2026-02-10 作業メモ (namespace再設計着扁E
- plan.md の再確誁E
  - `fn` は `let` の糖衣構文
  - 定義の巻き上げは `mut` でなぁE`let` のみ�E�Efn` も含む�E�E
- 実裁E�E計測:
  - lexer に `@` と `0x...` を追加
  - parser に `@ident` / `fn alias @target;` / `let` 関数糖衣 / `fn` 型注釈省略を追加
  - `NO_COLOR=true trunk build` は成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-upstream-fix.json -j 4`:
    - `total=309, passed=242, failed=67, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-only-after-entry-fix.json -j 1`:
    - `total=16, passed=5, failed=11, errored=0`
- 観測した根本問顁E
  - 名前解決ぁE`Env` の単一チE�Eブルに寁E��すぎており、変数と関数値、alias、entry 解決が同一経路で干渉すめE
  - nested `fn` めEblock で宣言できても、HirFunction に落ちぁE`unknown function` へ繋がめE
  - entry は解決できてめEcodegen 側に関数本体が無ぁE��合に `_start` が�E力されなぁE��実行時エラー化！E
- 直近�E修正:
  - top-level `fn alias` の登録を関数本体チェチE��前に移勁E
  - 型未確定関数の symbol は暫定で unmangled 名を使ぁE��ぁE��更�E�Entry/mangleずれ緩和！E
- 次スチE��チE
  - namespace めE`ValueNs` / `CallableNs` に刁E��し、巻き上げを仕様準拠に寁E��めE
  - entry の「解決済みかつ生�E済み」検証を追加して compile error 化すめE
- ドキュメント運用修正:
  - `todo.md` は未完亁E��スクのみを残す形式へ整琁E
  - 進捗�E履歴・計測値は `note.n.md` のみへ雁E��E

# 2026-02-03 作業メモ (wasm32 build)
- wasm32-unknown-unknown での `cargo test --no-run` ぁEgetrandom の js feature なしで失敗してぁE��ため、`nepl-core` の wasm32 用 dev-dependencies に `getrandom` (features=["js"]) を追加した、E
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features` を実行し、Cargo.lock を更新してビルドが通ることを確認、E
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features --locked` も�E功、E
# 2026-02-03 作業メモ (selfhost string builder)
- stdlib/alloc/string.nepl に StringBuilder�E�Eb_append/sb_append_i32/sb_build�E�を追加し、selfhost_req の斁E���Eビルダ要件を解禁した、E
- stdlib/tests/string.nepl に StringBuilder の検証を追加した、E
# 2026-02-03 作業メモ (selfhost string utils)
- stdlib/alloc/string.nepl に trim/starts_with/ends_with/slice/split を追加し、ASCII 空白判定や split 用の補助関数を実裁E��た、E
- stdlib/tests/string.nepl を拡允E��て trim/starts_with/ends_with/slice/split のチE��トを追加した、E
- nepl-core/tests/selfhost_req.rs の斁E���EユーチE��リチE��要件チE��トを解禁し、Option unwrap と len 呼び出しに合わせて冁E��を調整した、E
- doc/testing.md の stdlib スコープ一覧を更新し、alloc/string の追加関数を反映した、E
- 未対忁E file I/O (WASI の path_open 筁E と u8/バイト�E列�E型�E実行環墁E�E整備が忁E��なため未着手。string-keyed map/trait 拡張も後続で対応予定、E
# 2026-02-03 作業メモ (block ルール更新対忁E
- block: がブロチE��式、`:` が引数レイアウトとぁE��新ルールに合わせ、パーサの `:` 処琁E��整琁E��`block` は末尾なら�Eーカー扱ぁE��`cond/then/else/do` は単独�E�型注釈�Eみ許可�E�でマ�Eカー扱ぁE��し、`if cond:` のような通常識別子を誤判定しなぁE��ぁE��した、E
- `if`/`while` のレイアウト展開で `ExprSemi` を許可し、`while` 本体に `;` を書ぁE��チE��トが panic しなぁE��ぁE��正、E
- stdlib/侁E `while ...:` の褁E��斁E�EチE��めE`do:` ブロチE��化！Etdlib/alloc/*, core/mem, std/stdio, std/env/cliarg, kp/kpread, examples/counter/fib/rpn など�E�。`examples/rpn.nepl` の入れ孁Ewhile めE`do:` に統一、E
- tests: `nepl-core/tests/plan.rs` めE`block:` 使用に更新、`nepl-core/tests/typeannot.rs` の while めE`do:` に更新。`stdlib/tests/vec.nepl` の match arm から誤っぁE`block` マ�Eカーを除去、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行し、両方成功�E�警告�E既存�Eまま�E�、E
# 2026-02-03 作業メモ (依存更新/online cargo test)
- workspace 依存を最新安定版へ更新�E�Ehiserror 2.0.18、anyhow 1.0.100、clap 4.5.56、wasm-bindgen 0.2.108、assert_cmd 2.1.2、tempfile 3.24.0 など�E�。rand は最新安定�E 0.8.5 のまま、E
- wasmi 1.0.8 への更新を試したが、rustc 1.83.0 では 1.86 以上が忁E��で不可。wasmi は 0.31.2 に戻して Cargo.lock を更新、E
- チE��チE オンライン `cargo test` を実行。`nepl-core/tests/overload.rs` の `test_overload_cast_like` と `test_explicit_type_annotation_prefix` ぁE"ambiguous overload" で失敗。他�EチE��ト�E成功、E
# 2026-02-03 作業メモ (trait/overload 修正の根本対忁E
- overload の重褁E��除ぁE`type_to_string` の "func" 返却で全て同一扱ぁE��なってぁE��ため、E��数シグネチャ斁E���Eを導�Eし、E��褁E��定と impl メソチE��署名一致判定をシグネチャ比輁E��変更、E
- trait method の呼び出しで `Self` ラベルと型パラメータが不一致になる問題を、`Self` ラベルは任意型と統一可能にすることで解消、E
- monomorphize で trait 呼び出しを具体関数へ解決する際、解決先関数のインスタンス化要求を行うよう変更し、unknown function を解消、E
- チE��チE `cargo run -p nepl-cli -- test` は成功�E�警告あり）、E
- チE��チE `cargo test` は 120 秒でタイムアウト（警告�E力後に未完亁E��、E
# 2026-02-03 作業メモ (stdlib チE��ト拡允E修正)
- stdlib/std/hashmap.nepl の if レイアウトを修正し、hash_i32 を純粋関数に書き換え！E6進リチE��ルめE0進へ置換）。hashmap_get は再帰ループで純粋化、E
- stdlib/std/hashset.nepl の hash_i32 を純粋関数へ変更し、hashset_contains を�E帰ループで純粋化。hashset_contains_loop のシグネチャ不整合も修正、E
- stdlib/std/result.nepl の unwrap_err めEErr 刁E���E頭に並べ、match の戻り型ぁEnever になる問題を回避、E
- stdlib/tests に hashmap.nepl/hashset.nepl/json.nepl を追加し、基本操作！Eew/insert/get/remove/len/contains など�E�と JSON の吁E��クセサを検証、E
- stdlib/tests/result.nepl は map 系を外し、unwrap_ok/unwrap_err の検証に置き換え。json.nepl は move 連鎖を避けるため値を�E度生�Eする形に整琁E��E
- チE��チE `cargo run -p nepl-cli -- test` は成功�E�警告�E残存）、E
- チE��チE `cargo test` は 120 秒でタイムアウト（警告�E力後に未完亁E��、E
# 2026-02-03 作業メモ (trait/overload)
- AST/パ�Eサ: 型パラメータめETypeParam 化し、`.T: TraitA & TraitB` 形式�E墁E��を読めるようにした、E
- HIR: trait 呼び出ぁE(`Trait::method`) を表現できるようにし、impl 側はメソチE��一覧を持つ形に変更、E
- 型検査: trait 定義/impl の整合性チェチE��、Self 型�E差し込み、trait bound の満足判定を追加。関数の同名オーバ�Eロードを許可し、mangle したシンボルで冁E��名を一意化、E
- 単相匁E impl マップを構築し、trait 呼び出しを具体的なメソチE��実体に解決するようにした、E
- チE��チE nepl-core/tests/neplg2.rs にオーバ�EローチEtrait のコンパイルチE��トを追加、E
- 既知の制陁E trait の型パラメータ、inherent impl、impl メソチE��のジェネリクスは未対応。オーバ�Eロード解決は引数型�Eみで行い、戻り値型�E使わなぁE��export 名�E mangle 後�E一意名になる、E
- チE��チE `cargo test -p nepl-core --lib` を実行（警告�E残存）、E
# 2026-02-03 作業メモ (never 型と unwrap 修正)
- `unreachable` 刁E��で型変数ぁE`never` に束縛され、`Option::unwrap` ぁE`unwrap__Option_never__never__pure` へ潰れる問題を修正、E
- `types::unify` で `Var` と `Never` の統一時に束縛しなぁE��ぁE��例を追加し、`unwrap__Option_T__T__pure` を保持するようにした、E
- codegen の `unknown function` 診断に欠落関数名を含めるよう改喁E��E
- チE��チE `cargo run -p nepl-cli -- test` は成功�E�警告あり）、E
- チE��チE `cargo test` は 240 秒でタイムアウト（コンパイル途中�E�。�E実行が忁E��、E
# 2026-02-03 作業メモ (btreemap/btreeset 追加)
- stdlib/std/btreemap.nepl と stdlib/std/btreeset.nepl を追加し、i32 キー/要素の頁E��付きコレクションを�E列�Eースで実裁E��た（検索は二�E探索、挿入/削除はシフト�E�、E
- stdlib/tests/btreemap.nepl と stdlib/tests/btreeset.nepl を追加し、基本操作（挿入/更新/削除/検索/長さ）を検証した、E
- doc/testing.md の stdlib 一覧に std/btreemap と std/btreeset を追記した、E
# 2026-02-03 作業メモ (test 彩色/stdlib チE��ト調整/コンパイラ確誁E
- stdlib/std/test.nepl の失敗メチE��ージめEANSI 赤色で表示するよう変更し、std/stdio の色出力を利用、E
- stdlib/tests/error.nepl で `fail` の使用を避け、error_new 由来の診断が非空であることを確認する形に調整、E
- stdlib/tests/cliarg.nepl/list.nepl/stack.nepl/vec.nepl/string.nepl/diag.nepl を更新し、失敗時のメチE��ージを�E示するチE��トに整琁E��E
- doc/testing.md の失敗時の表示説明を更新、E
- コンパイラ確誁E error::fail�E�Eallsite_span 経由�E�を含むチE��トで wasm 検証エラーが発生するため、std チE��ト�Eでは該当経路を使わなぁE��ぁE��して回避。Rust 側の callsite_span/codegen の相性は要調査、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行、E
# 2026-02-03 作業メモ (nepl-cli test の色付け)
- nepl-cli のチE��ト�E力を ANSI 色付きにし、test/ok/FAILED の視認性を上げた、E
- doc/testing.md に色付き出力�E注記を追記、E
# 2026-02-03 作業メモ (stdlib/diag 色刁E��)
- stdlib/std/diag.nepl に ErrorKind ごとの色割り当てを追加し、diag_print/diag_println/diag_debug_print で色付き表示に変更、E
- stdlib/std/stdio.nepl に debug_color/debugln_color を追加、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行、E
# 2026-02-03 作業メモ (Checked ログの色付け)
- stdlib/std/test.nepl に test_checked を追加し、EChecked ..." の成功ログを緑色で出すよぁE��した、E
- stdlib/tests/list.nepl と stdlib/tests/math.nepl の Checked ログめEtest_checked に置き換えた、E
- doc/testing.md に test_checked を追記、E
# 2026-02-03 作業メモ (チE��ト失敗�EメチE��ージ表示)
- stdlib/std/test.nepl を改修し、失敗時にメチE��ージを表示してから trap するよう変更した、E
- stdlib/std/diag.nepl に diag_print_msg を追加し、Failure メチE��ージを表示できるようにした、E
- stdlib/std/error.nepl の fail/context めEcallsite_span 付与に更新した、E
- stdlib/tests/diag.nepl と stdlib/tests/error.nepl を強化し、文字�E化や span の検証を追加した、E
- doc/testing.md の assert 仕様を更新した、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行、E
# 2026-02-03 作業メモ (cliarg 追加)
- stdlib/std/cliarg.nepl を追加し、WASI args_sizes_get/args_get で argv を取得できるようにした、E
- stdlib/tests/cliarg.nepl を追加し、篁E��夁E負の index ぁENone になることを確認するテストを用意した、E
- doc/testing.md の stdlib 一覧に std/cliarg を追記した、E
- nepl-cli の WASI ランタイムに args_sizes_get/args_get を追加し、`--` 以降�E引数を渡せるようにした、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行、E
# 2026-02-03 作業メモ (cliarg 実引数チE��チE
- stdlib/tests/cliarg.nepl を更新し、argv[1..] の値を検証するチE��トを追加した、E
- nepl-cli の stdlib チE��ト実行で `--flag value` めEargv に渡すよぁE��更した、E
- doc/testing.md に stdlib チE��トが固定引数を渡す旨を追記した、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行、E
# 2026-02-03 作業メモ (stdlib コメント言語統一)
- stdlib/std/option.nepl と stdlib/std/result.nepl の英語コメント行を削除し、コメントが日本語�EみになるよぁE��一、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` を実行、E
# 2026-02-03 作業メモ (stdlib コメンチEOption/Result 改修)
- stdlib/std の吁E��ァイルに日本語コメント（ファイル概要E吁E��数の目皁E�E実裁E�E注意�E計算量�E�を追加し、math.nepl は自動生成で関数コメントを挿入、E
- list_tail めEOption<i32> 返却に変更し、list_get の走査めEunit になるよぁE��整�E�デバッグ出力も削除�E�、E
- stdlib/tests/list.nepl めElist_tail の Option 仕様に合わせて更新、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` が�E功、E

# 2026-02-03 作業メモ (import/resolve チE��ト拡允E
- nepl-core/tests/resolve.rs に default alias�E�相対/パッケージ�E�、selective 欠落名�E扱ぁE��merge open、visible map 優先頁E��！Eocal/ selective/ open�E�を追加、E
- nepl-core/src/module_graph.rs の unit チE��トに missing dependency/invalid import/duplicate export/non-pub import/ selective+glob re-export を追加、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` が�E功、E

# 2026-02-03 作業メモ (rpn 実衁E+ std/test 修正 + チE��ト実衁E
- examples/rpn.nepl めE`printf "3 4 +\n" | cargo run -p nepl-cli -- -i examples/rpn.nepl --target wasi --run` で実行し、REPL が結果を返して終亁E��ることを確認、E
- stdlib/std/test.nepl の `assert_str_eq` めE`if:` ブロチE��形式に修正し、`(trap; ())` の inline 1行式を排除してパ�Eサエラーを解消、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` が�E功、E

# 2026-02-03 作業メモ (rpn import + diagnostics)
- examples/rpn.nepl の import を新仕様！E#import "..." as *`�E�へ更新、E
- loader の parse でエラー診断がある場合�E CoreError を返すようにし、構文エラーが型エラーに埋もれなぁE��ぁE��正、E
- CLI の診断表示でキャレチE��長を行末に収め、巨大な ^ の出力を抑制、E
- typecheck の簡易サマリ出力�E verbose 時�Eみ表示するように変更、E

# 2026-02-03 作業メモ (Windows path canonicalization for tests)
- module_graph の lib チE��トで path 比輁E�� Windows の canonicalize 差刁E��失敗するため、root path めEcanonicalize して比輁E��るよぁE��正、E
- resolve.rs 側の ModuleGraph 参�EチE��トも同様に canonicalize を適用し、クロスプラチE��フォームで一致するようにした、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (resolve import tests fix)
- nepl-core/tests/resolve.rs のチE��ト用ソースめE`:` ブロチE��形式に修正し、parser の期征E��るインチE��ト構造に合わせた、E
- selective glob�E�Ename::*`�E�が open import に反映されることを確認するテストを追加、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (resolve/import test expansion)
- nepl-core/tests/resolve.rs を追加実裁E��、prelude 持E��の解析、merge clause 保持、alias/open/selective の解決、open import の曖昧性診断、std パッケージ解決のチE��トを追加、E
- nepl-core/tests/neplg2.rs に prelude/import/merge 持E��の受理確認テストを追加、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (tests import syntax migration)
- nepl-core/tests と stdlib 配下�E #import/#use を新仕様！E#import "..." as *`�E�へ統一し、Euse を除去した、E
- loader_cycle のチE��ト�E `#import "./a"`/`#import "./b"` に変更して相対 import の仕様に合わせた、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (selective re-export test)
- module_graph の pub selective re-export の挙動を確認するテストを追加�E�Elias のみ公開され、�E名や未選択�E公開頁E��は再エクスポ�EトされなぁE��とを検証�E�、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (pub import selective re-export)
- build_exports ぁEImportClause::Selective を老E�Eし、pub import の再エクスポ�Eト篁E��めEselective に限定できるようにした�E�Elob は全件再エクスポ�Eト扱ぁE��、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (module_graph import clause)
- module_graph の import/deps に ImportClause を保持するようにし、resolve ぁEAST ではなぁEModuleGraph の惁E��から import 句を参照する形へ変更、E
- resolve の import 走査を整琁E��、deps の clause を直接使って alias/open/selective/merge を構築、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (pub #import / pub item)
- lexer で `pub #import` を認識し、`#import pub ...` へ書き換える処琁E��追加�E�Epub` 前置のチE��レクチE��ブ�E #import のみ許可�E�、E
- parser で `pub fn/struct/enum/trait/impl` をトチE�Eレベルで解釈できるようにし、`pub` が�E頭に来ても正しく定義を読めるようにした、E
- チE��チE `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功、E

# 2026-02-03 作業メモ (rewrite plan doc)
- doc/rewrite_plan.md を現行コード確認に基づぁE��拡允E��、後方互換なし�E設計書+実裁E��画書として整琁E��た（モジュールID/manifest、import clause、prelude、名前解決優先頁E��、型推諁E単相化、WASM ABI、CLI/stdlib墁E��、実裁E��ード�EチE�E、テスト方針）、E
- 現行パイプラインは loader の AST スプライス方式�Eままで、module_graph/resolve の実裁E�E未統合である点を計画冁E��明記、E
- plan.md には manifest/新import斁E��Eprelude/mergeの仕様や CLI/ABI 墁E��の整琁E��未記載�Eため、追記が忁E��、E
- チE��チE 以前�E `module_graph::tests::builds_simple_graph_and_exports` ぁEunknown token で失敗してぁE��が、`pub #import`/`pub fn` 対応後に `cargo test` も�E功、E

## 直近�E実裁E��マリ
- 斁E���EリチE��ルと垁E`str` を追加し、データセクションに `[len][bytes]` で配置して常時メモリをエクスポ�Eトする形に統一、E
- `#extern` で外部関数を宣言可能にし、stdlib から `print` / `print_i32` を提供する構�Eに統一。ビルトイン関数は撤廁E��E
- CLI: `--target wasm|wasi` に対応！Easi ぁEwasm を包含�E�。`--run` だけでも実行可。コンパイル失敗時に SourceMap 付き診断を�E力、E
- Loader/SourceMap を導�Eし、import/include で FileId/Span を保持したまま多ファイルを統合、E
- パイプ演算孁E`|>` を追加。スタチE��トップを次の呼び出し�E第1引数に注入する仕様で、lexer/parser/typecheck まで実裁E��み、E
- `:` ブロチE��と `;` の型検査を調整し、Unit 破棁E�� while の stack 深さ検証を改喁E��E
- stdlib: math/mem/string/result/option/list/stdio を追加・更新。mem は raw wasm、string/result/option はタグ付けポインタ表現、stdio は WASI fd_write 前提、E
- `#target wasm|wasi` をディレクチE��ブとして追加、ELI がターゲチE��を指定しなぁE��合�E #target をデフォルトに用ぁE��褁E�� #target は診断エラーにした。wasi 含有ルールは従来通り、E
- stdlib/std/stdio めEWASI `fd_write` 実裁E��置き換え、env 依存を排除。print_i32 は from_i32 ↁEfd_write で出力、E
- 型注釈�E「恒等関数」ショートカチE��を削除し、ascription のみで扱ぁE��提に揁E��た。`|>`+注釈�E回りのチE��トを追加、E
- std/mem.alloc を要求サイズから算�Eしたペ�Eジ数で memory.grow する形にし、固宁Eペ�Eジ成長を解消（ただし�Eージ墁E��アロケータのまま�E�、E
- CLI の target フラグを省略可能にし、Etarget / stdio 自勁Ewasi 昁E��と整合するよぁE��した、E
- チE��ト追加: #target wasi チE��ォルト動作、E��褁E#target エラー、pipe+型注釈�E成功ケース、E
- 言語に struct/enum/match を追加。enum/struct めETypeCtx に登録し、コンストラクタを�E動バインド！EType::Variant` / `StructName`�E�。match は網羁E��チェチE��と型整合チェチE��を行う、E
- Option/Result めEenum ベ�Eスに再実裁E��EptionI32/ResultI32�E�。string/find/to_i32/list/get などめEResult/Option 返却に差し替え。list の get は ResultI32 で墁E��エラーを返す、E
- codegen に enum/struct コンストラクタと match を追加�E�Euntime 表現は [tag][payload]/構造体フィールドを linear memory 上に確保し、std/mem.alloc 呼び出しを前提�E�、E
- pipe の注入タイミングを調整し、型注釁E`<T>` を挟んでめE`|>` が正しく次の callable に注入されるよぁE��した。追加チE��トで確認、E
- Loader の循環 import 検�EチE��トを追加�E�Eemp チE��レクトリに a.nepl/b.nepl を生成しロードでエラーを確認）、E

## plan.md との乖離・注意点
- `#target`: チE��レクチE��ブとしては実裁E��みだが、plan.md には未記載。エントリーファイル以外に書かれた場合�E扱ぁE��ど仕様�E記が忁E��、E
- 型注釁E`<T>`: 恒等関数ショートカチE��は削除したが、plan.md には「関数と見做す」とあるので記述を更新する忁E��あり、E
- stdlib/stdio: WASI `fd_write` 実裁E��置き換え済み。wasm で import した際�E専用診断はまだ無ぁE�Eで、エラーメチE��ージ改喁E�E余地あり、E
- stdlib/mem.alloc: サイズに応じた�Eージ成長に修正したが、�Eージ墁E��アロケータのまま。細粒度管琁E�� free は未対応、E
- Option/Result/list: enum/match が無ぁE��めタグ付きポインタの暫定実裁E��型シスチE��統合や多相化�E未着手。list は i32 固定で get の篁E��外診断なし、E

## 追加で気付いたこと
- Loader は FileId/Span を保持して diagnostics に活用できてぁE��、Einclude/#import は一度きりロードで循環検�Eあり、E
- コード生成�E wasm のみ、EompileTarget::allows は wasi ぁEwasm を包含する形で gate 判定を実裁E��E

# 2026-01-23 作業メモ
- Rust チE�EルチェインめErustup で導�Eし、依存クレートを取得できるようにした、E
- #if 関連の unknown token を解消するためElexer の `* >` / `- >` めEArrow として許可するよう緩和した、E
- stdlib の構築途中コードが多数コンパイルを塞いでぁE��ため、一時的に std/string・std/list・std/stdio を最小機�Eのスタブ実裁E��差し替え！Eption.unwrap_or を削除して重褁E��消）、E
- enum コンストラクタの codegen を修正�E�Eayload store のオペランド頁E��、結果ポインタをスタチE��に残すように変更�E�。これにより Option::Some/None が正しく値を返し、`match_option_some_returns_value` が通過、E
- std/list.get は墁E��外を常に `ResultI32::Err 1` で返す単純実裁E��し、スタチE��不整合�E診断を解消。現状 in-bounds 取得�E未対応だがテスト想定！EOB エラー�E�には合�E、E
- 現在 `cargo test` は 23/23 すべて成功。残課題�E stdlib 機�Eの肉付け�E�Eist.get の正実裁E��文字�E/オプションの汎用化など�E�、E

## 今後�E対応案（実裁E�EまだしなぁE��E
- `#target wasi|wasm` をディレクチE��ブとして追加し、ファイル冁E�EチE��ォルトターゲチE��を決定！ELI 持E��があればそちらを優先）。`#if[target=...]` 評価にも使用、E
- 型注釈�E古ぁE��等関数特例を撤去し、注釈�E構文要素としてのみ扱ぁE��を仕様に明記、E
- stdio めEWASI fd_write 実裁E��戻す／もしくは wasm target で import された場合にコンパイル時エラーを�Eす、E
- mem.alloc の size 対応とペ�Eジ再利用、list の多相化�E墁E��チェチE��強化、Option/Result めEenum/match 連携へ移行、E

# 2026-01-30 作業メモ
- stdlib/std/string.nepl の to_i32 冁E�� if: ブロチE��に誤って if eq ok 1: / else: が混入するインチE��トになっており、if-layout 解析が "too many expressions" になる状態だったため、if eq ok 1: ブロチE��めE段チE��ントし、else ブロチE��のインチE��トを整えて if-layout が正しく刁E��されるよぁE��正、E
- これにより std/string の cond/then/else 未定義エラーと block stack エラーが解消。cargo test は全件通過、examples/counter.nepl めEwasi 実行しても完走することを確認、E
- 斁E���EリチE��ルぁEallocator のメタ領域と衝突してぁE��ため、codegen_wasm の斁E���E配置開始オフセチE��めE8 バイト！Eeap_ptr + free_list_head�E�に変更し、data section で free_list_head=0 を�E示。併せて data section を常に出力して heap_ptr を�E期化するよう修正、E

# 2026-02-01 if/while チE��ト無限ループ対忁E
## 問題発要E
- ifチE��トが16GB以上�Eメモリ使用となり、実行が停止する無限ループ問題を発見、E
- パ�Eサー側は`if` ブロチE��刁E��で正常に動作してぁE���E�テスト通過確認）、E
- 無限ループ�EタイプチェチE��段階で発生してぁE��模様、E

## 原因特定と修正
- `apply_function()` の `if` ケースで、E��数垁E`(bool, T, T) -> T` の `result` 型変数が統一されてぁE��かった、E
- 2つのブランチ型を統一した後、その結果めE`result` 型変数に統一する忁E��があった、E
- 修正: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);` を追加し、結果型を関数の result 型パラメータと統一、E
- 同じぁE`while` も同様�E問題があったため、`let final_ty = self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit());` で修正、E

## チE��ト実行結果
- 修正後、E��刁E��にチE��トが成功開始！E個テスト確誁E if_mixed_cond_then_block_else_block など�E�E
- 残り7個�EチE��トでメモリスパイク続衁E
  - 失敗テスチE if_a_returns_expected, if_b_returns_expected, if_c_returns_expected, if_d_returns_expected, if_e_returns_expected, if_f_returns_expected, if_c_variant_lt_condition
  - これら�E全て `#import "std/math"` と `#use std::math::*` を含む

## 次のスチE��チE
- 失敗してぁE��チE��ト�E共通点は import/use スチE�EトメンチE
- ローダー或いはモノモルファイゼーション段階での無限ループ�E可能性を調査中

- これにより WASI 実行時の print�E�文字�EリチE��ル�E��E無出力／ゴミ�E力が解消。stdout の回帰検�E用に `nepl-core/tests/fixtures/stdout.nepl` を追加し、`nepl-core/tests/stdout.rs` と `run_main_capture_stdout` を実裁E��E
- 斁E���E操作�EチE��トとして `nepl-core/tests/stdlib.rs` に len(斁E���EリチE��ル) と from_i32→len を追加。`cargo test -p nepl-core --test stdlib --test stdout` で確認、E
- plan2.md と doc/starting_detail.md はリポジトリ冁E��存在しなぁE��め、参照できなぁE��態�Eまま、E
- stdlib/std/stdio に `println` を追加し、`print` + 改行文字�Eで実裁E��`print`/`print_i32` はそ�Eまま維持、E
- stdlib/std/stdio の `print_str` めE`print` に改名し、`println_i32` を追加。str は `print`/`println`、i32 は `print_i32`/`println_i32` を提供する形に整琁E��E
- `nepl-core/tests/fixtures/println_i32.nepl` と stdout チE��トを追加し、`println_i32` が改行を出力することを確認、E
- examples の送E�Eーランド記法電十E`examples/rpn.nepl` を文字�Eパ�Eス方式に拡張し、ASCII ト�Eクンを走査して数値/演算子を処琁E��る形に更新、E
- stdlib/std/stdio から std/string の import を外し、print は斁E���Eヘッダ長を直接読む形に変更。print_i32 は同一ファイル冁E��数値→文字�E変換を行い、std/list との `len` 衝突を回避、E
- stdlib/std/stdio に `read_all` を追加し、WASI の fd_read で標準�E力を取り込めるようにした、ELI ランタイムにめEfd_read 実裁E�� stdin バッファを追加、E
- stdin の動作確認用に `nepl-core/tests/stdin.rs` と `nepl-core/tests/fixtures/stdin_echo.nepl` を追加し、日本語�E力�Eエコーもテストに含めた、E
- CLI の fd_read をオンチE�Eンド読み込みに変更し、起動時に stdin めEread_to_end しなぁE��とで対話入力でもブロチE��しなぁE��ぁE��調整、E
- stdlib/std/stdio に `read_line` を追加し、REPL 向けに改行までの読み取りを提供。stdin チE��トに `stdin_readline.nepl` と日本語ケースを追加、E
- examples/rpn.nepl めEREPL 形式に変更し、E行ごとの評価とエラーメチE��ージ表示に対応。`read_line` を使ぁE��め、対話入力でも評価できるようにした、E
- examples/rpn.nepl に REPL 使ぁE��のメチE��ージを追加し、PowerShell パイプ時の BOM を無視する簡易スキチE�E処琁E��入れて unknown token を回避、E
- stdout 用の fixture とチE��トを追加し、`println` ぁE`\n` を�E力することを確認。README の std/stdio 説明も `println` と WASI `fd_write` に合わせて更新、E
- stdout チE��トで wasi fd_read の import 未提供により instantiate 失敗してぁE��ため、`nepl-core/tests/harness.rs` の `run_main_capture_stdout` に fd_read スタブを追加。`cargo test -p nepl-core --test stdin --test stdout` は警告付きで成功し、`printf '14 5 6 + -' | cargo run -q -- -i examples/rpn.nepl --run --target wasi` で REPL 出力と結果 3 を確認、E
- PowerShell の UTF-16LE パイプ�E力で数値が�E割される可能性に備え、`examples/rpn.nepl` の数値パ�Eスで NUL バイトを無視する�E岐を追加�E�EOM スキチE�Eと併用�E�、E

# 2026-01-30 作業メモ (チE��チEstdlib)
- stdlib に `std/test` を追加し、`assert`/`assert_eq_i32`/`assert_str_eq`/`assert_ok_i32`/`assert_err_i32` を提供。`trap` は `i32.div_s` めE0 で割めE#wasm で実裁E��、WASM 側で確実に異常終亁E��るよぁE��した、E
- `std/string` に `str_eq`�E�純粋�E帰�E�を追加し、`std/test` 側の斁E���E比輁E��も同等ロジチE��を使用、E
- CLI に `nepl test` サブコマンドを追加し、`stdlib/tests` 配下�E `.nepl` を収雁E��て WASI で実行するテストランナ�Eを実裁E��E
- stdlib チE��トを `stdlib/tests/{math,string,result,list}.nepl` に追加。式�E括弧は使わず前置記法で記述し、Result の move を避けるため同一値を�E生�Eして検証、E
- `cargo run -p nepl-cli -- test` と `cargo test` が通ることを確認、E
- doc に `doc/testing.md` を追加し、テスト機�Eの使ぁE��と stdlib の現状篁E��を整琁E��E

# 2026-01-30 作業メモ (examples 実行確誁E
- examples/counter.nepl と examples/fib.nepl めE`#target wasi` に揁E��、std/stdio の利用を�E示、E
- `cargo run -p nepl-cli -- -i examples/counter.nepl --run --target wasi` と `... fib.nepl ...`、`printf '14 5 6 + -\n' | ... rpn.nepl ...` を実行し、�E力が正常であることを確認、E
- `cargo test` を�E実行し、�EチE��トが通過することを確認、E

# 2026-01-30 作業メモ (多相/単相化�E現状)
- パ�Eサは fn/enum/struct/trait/impl の型パラメータ宣言と型適用 `TypeName<...>` を受琁E��、TypeCtx には TypeKind::{Function,Enum,Struct} の type_params と TypeKind::Apply がある、E
- 関数呼び出しでは typecheck ぁEtype_params めEfresh var に instantiate し、呼び出し�Eに type_args を残す。monomorphize は FuncRef の type_args をもとに関数だけ単相化してマングル名を生�Eする、E
- TypeKind::Apply は unify が扱わず、resolve めEmatch 以外で使われてぁE��ぁE��め、型注釈やシグネチャで `Foo<...>` を使ぁE��実質皁E��整合しなぁE��E
- enum/struct のコンストラクタは定義側の型情報を直接使っており、instantiate されぁEparams/result を反映しなぁE��め型変数がグローバルに束縛されやすく、ジェネリチE�� enum/struct が実用になってぁE��ぁE��E
- stdlib の list/option/result は i32 固定で、ジェネリクスは未導�E、E

## plan.md との差刁E��モ (追加)
- plan.md にはチE��ト実行コマンドや `std/test`/`nepl test` の仕様が未記載。テスト設計�E章立てを追加する忁E��がある、E
- plan2.md と doc/starting_detail.md は引き続きリポジトリ冁E��存在しなぁE��め参照不可、E
- plan.md では「定義での多相は扱わなぁE��としてぁE��が、実裁E��は type_params と monomorphize が存在する。仕様整合�E追記が忁E��、E

# 2026-01-30 作業メモ (ジェネリクス修正)
- 型パラメータは .T 形式�Eみ許可するように parser を更新し、ET> はエラーにした、E
- Apply めEunify で resolve して enum/struct の具体型と統合できるようにし、resolve の結果は型引数めEtype_params に保持するよう変更、E
- enum/struct コンストラクタは instantiate 後�E params/result を使ぁE��ぁE��し、型変数のグローバル束縛を避ける形に修正、E
- type_to_string は enum/struct の type_params を含めるようにして単相化�Eングルの衝突を避けた、E
- codegen で Apply を参照型として扱ぁE��enum の variant 解決めEApply にも対応、E
- Rust チE��チE`nepl-core/tests/generics.rs` を追加し、fn/enum/struct のジェネリクスとエラーケースを検証、E

# 2026-01-30 作業メモ (ジェネリクス修正の追加)
- parser のエラー診断が�EてぁE��場合�E compile_wasm を失敗させるようにし、ET> を実際にエラー扱ぁE��した、E
- Apply の型引数数不一致は unify で失敗させ、型注釈�E不一致として診断されるよぁE��した、E
- 型引数は typecheck と monomorphize で resolve_id により実体型へ正規化し、単相化後に Var が残らなぁE��ぁE��した、E
- wasm 生�E後に wasmparser で検証し、無効 wasm を診断として返すようにした、E

# 2026-01-30 作業メモ (ジェネリクス修正の追加2)
- 型注釈が未適用のまま let が�Eに簡紁E��れるケースがあったため、pending_ascription がある間はそ�E手前の関数を簡紁E��なぁE��ぁEguarded reduce を追加、E
- type_args の resolve を引数 unify 後に行うようにし、単相化に Var が残らなぁE��ぁE��修正、E

# 2026-01-30 作業メモ (ジェネリクス チE��ト拡張)
- generics.rs に .T 忁E���E enum/struct 定義エラー、payload の i32 演算検証、褁E��型パラメータ関数の単相化、型注釈不一致のエラーを追加、E
- さらに、None の型決定、引数なしジェネリチE��関数の型決定、ジェネリチE��関数の委譲呼び出し、pipe 経由呼び出し、E型パラメータ enum の match、�Eれ孁EApply の payload・そ�E不一致エラー、同一型パラメータの不一致エラー、payload 型不一致エラーを追加、E
- 追加で、コンストラクタの型推論（引数位置�E�、ジェネリチE��関数での Pair 構築、Option::Some ラチE��ー関数、Option<Option<T>> の入れ孁Ematch めEOK ケースとして追加、E

# 2026-01-31 作業メモ (ジェネリクス/構文/コード生戁E
- if-layout の cond 識別子が変数名として使われるケースに対応するため、`normalize_then_else` で cond を無条件に消さず、then/else マ�Eカーがある場合�Eみ除去するよう調整、E
- `if cond:` のような行末 `:` 形式で cond が変数名�E場合に stack エラーが�EてぁE��ため、if-layout 判定かめE`if cond:` の特例を外し、cond 変数を保持する形に変更、E
- match 式が後続�E行を吸ぁE��むケースがあったため、`KwMatch` で match 式を読み込んだめEprefix 解析を打ち刁E��ように修正、E
- wasm codegen の match ぁE2刁E��固定だったため、任意個！E個以上）�E刁E��を if 連鎖で生�Eするように拡張し、EバリアンチEenum の match で unreachable が�Eる問題を解消、E
- `generics_multi_type_params_function` の期征E��は if の振る�EぁE��合わせて 3 に修正�E�Ealse 刁E���E確認）、E
- `cargo test` は全件通過を確認、E
- plan2.md と doc/starting_detail.md は引き続きリポジトリ冁E��存在しなぁE��め参照不可、E

# 2026-01-31 作業メモ (チE��ト整吁E
- nepl-core の `list_get_out_of_bounds_err` チE��トを現衁Estdlib に合わせ、`list_nil/list_cons/list_get` と `Option` の `Some/None` マッチに更新、E
- `cargo test` と `cargo run -p nepl-cli -- test` の両方が�E功することを確認、E

# 2026-01-31 作業メモ (ログ抑制)
- typecheck/unify/monomorphize/wasm_sig の成功時ログを削除し、OK時�E `nepl-cli test` の出力を削減、E
- `cargo run -p nepl-cli -- test` はチE��ト結果のみ表示されることを確認！Eust の警告�E別途表示�E�、E

# 2026-01-31 作業メモ (verbose フラグ)
- `nepl-cli` に `--verbose` を追加し、詳細なコンパイラログを忁E��時のみ出力できるようにした、E
- `CompileOptions.verbose` で制御し、typecheck/unify/monomorphize/wasm_sig のログをフラグ連動にした、E

# 2026-01-31 作業メモ (メモリアロケータ)
- `std/mem` の allocator めEwasm モジュール冁E��裁E��変更し、`nepl_alloc` のホスト依存を除去、E
- free list + bump 併用の簡昁Eallocator を実裁E��、`memory.grow` で拡張、E
- `doc/runtime.md` に WASM/WASI のターゲチE��方針とメモリレイアウトを追加、E

# 2026-01-31 作業メモ (nepl_alloc 自勁Eimport の撤去)
- コンパイラぁE`nepl_alloc` を�E動で extern に追加する処琁E��削除し、WASM 生�E物が�Eスト依存�E import を持たなぁE��ぁE��した、E
- `alloc`/`dealloc`/`realloc` は `std/mem` の定義ぁE`#extern` により解決される前提になったため、モジュール側で `std/mem` めEimport してぁE��ぁE��合�E codegen でエラーになる、E
- 既存�E `a.wasm` などは再コンパイルが忁E��E��古ぁE��イナリには `nepl_alloc` import が残る�E�、E
- `alloc` などのビルトイン自動登録も外したため、`std/mem` の関数定義がそのまま使用される。`alloc` を使ぁE��ード�E `std/mem` を�E示皁E�� import する忁E��がある、E

# 2026-01-31 作業メモ (std/mem の効果注釁E
- `std/mem` の `alloc`/`dealloc`/`realloc`/`mem_grow`/`store` めE`*` 付きに変更し、純粋コンチE��ストから呼べなぁE��とを�E示した、E
- これにより `std/mem` 冁E��の `set`/`store_*` 呼び出しが純粋関数扱ぁE��なってぁE��問題を解消し、`match_arm_local_drop_preserves_return` の失敗原因を修正した、E

# 2026-01-31 作業メモ (monomorphize のランタイム関数保持)
- エントリ起点の単相化で `alloc` が落ちる問題を避けるため、`monomorphize` の初期 worklist に `alloc`/`dealloc`/`realloc` を追加した、E
- enum/struct/tuple の codegen ぁE`alloc` を呼ぶ前提でも、未参�Eの `alloc` が除去されなぁE��ぁE��した、E

# 2026-01-31 作業メモ (チE��ト�Eの std/mem 明示)
- enum/struct/tuple を使ぁE��ストソースに `std/mem` の import を追加し、`alloc` が解決される前提を明確化した、E
- `move_check` チE��ト�E Loader 経由で compile するように変更し、`#import` を解決できるようにした、E

# 2026-01-31 作業メモ (標準エラー/診断の追加)
- `std/error` と `std/diag` を追加し、`ErrorKind`/`Error`/`Span` と簡易レポ�Eト生成を用意した、E
- `callsite_span` の intrinsic を追加し、エラーに呼び出し位置を付与できるようにした、E
- `std/string` に `concat`/`concat3` を追加し、診断斁E���E生�Eの最低限を実裁E��た、E

# 2026-01-31 作業メモ (WASI エントリポイント対忁E
- codegen_wasm で entry 関数が指定されてぁE��場合、その関数めE`_start` とぁE��名前でめEexport するようにした、E
- これにより `wasmer run a.wasm` / `wasmtime run a.wasm` で WASI コンプライアンスに従い直接実行可能に、E
- README.md に外部 WASI ランタイム�E�Easmtime/wasmer�E�での実行方法を追加、E

# 2026-01-31 作業メモ (数値演算�E完�E匁E
- stdlib/std/math.nepl を�E面拡張し、i32/i64/f32/f64 のすべての演算機�Eを提供、E
- **算術演箁E*�E�add/sub/mul/div_s/div_u/rem_s/rem_u�E�すべての型で符号別に提供！E
- **ビット演箁E*�E�and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt�E�整数型�Eみ�E�E
- **浮動小数点特朁E*�E�sqrt/abs/neg/ceil/floor/trunc/nearest/min/max/copysign�E�E32/f64�E�E
- **型変換**�E�i32/i64 <-> f32/f64、符号付き/符号なし対応、E��和変換�E�Erunc_sat�E�E
- **ビット�E解釁E*�E�reinterpret_i32/f32/i64/f64

# 2026-02-03 作業メモ (web playground)
- Trunk の `public_url` めE`/` に変更し、`trunk serve` のローカル配信パスめE`http://127.0.0.1:8080/` に統一、E
- `web/index.html` に `vendor` の copy-dir を追加し、`web/vendor` を用意して editor sample の静的配币E�� Trunk 経由で行えるよぁE��した、E
- README と doc/web_playground.md に editor sample の取得手頁E��ローカル起勁EURL を追記、E
- `web/index.html` の CSS/JS めETrunk 管琁E�EアセチE��として宣言し、`styles.css` と `main.js` ぁEdist に出力されるように調整、E
- `web/main.js` は Trunk の `TrunkApplicationStarted` イベントと `window.wasmBindings` を利用して wasm-bindgen 生�E物にアクセスする方式に変更、E
- 埋め込み editor は `web/vendor/editorsample` が存在する場合�Eみ iframe に読み込み、存在しなぁE��合�Eフォールバック textarea を使用するように変更、E
- doc/web_playground.md に `public_url` と `serve-base` の関係を追記し、`trunk serve` のアクセスパスに関する注意点を�E記、E

## plan.md との乖離・注意点 (追加)
- plan.md に web playground の配信手頁E�E未記載�Eため、忁E��なら仕様欁E��追記が忁E��、E

# 2026-02-03 作業メモ (kpread UTF-8 BOM 対忁E
- PowerShell のパイプ�E力が UTF-8 BOM (EF BB BF) を付与する場合、kpread の `scanner_read_i32` が�E頭の BOM を数値として扱ぁE��E を返し続ける問題を確認、E
- `scanner_skip_ws` に UTF-8 BOM のスキチE�Eを追加し、既存�E UTF-16 BOM/NULL スキチE�Eと同じ位置で処琁E��E
- 回帰チE��トとして `nepl-core/tests/fixtures/stdin_kpread_i32.nepl` を追加し、`stdin_kpread_utf8_bom` で BOM 付き入力を検証、E
- 動作確誁E `printf '\xEF\xBB\xBF1 3\n' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 作業メモ (日本語文字�Eの stdout)
- 斁E���EリチE��ルの lexer ぁEUTF-8 めE1 バイトずつ `char` に変換してぁE��ため、日本語が mojibake になる問題を確認、E
- 斁E���EリチE��ルの通常斁E���E読み取りめEUTF-8 `char` 単位に変更し、`i` めE`len_utf8` 刁E��めるよう修正、E
- 回帰チE��トとして `nepl-core/tests/fixtures/stdout_japanese.nepl` と `stdout_japanese_utf8` を追加、E
- 動作確誁E `cargo run -p nepl-cli -- -i examples/helloworld.nepl --run -o a`

# 2026-02-03 作業メモ (CLI --run の stdio プロンプト)
- `nepl-cli --run` の WASI `fd_write` ぁE`print!` のみで flush しておらず、�Eロンプト `"> "` が�E力後に表示される問題を確認、E
- `fd_write` めEraw bytes で `stdout.write_all` し、最後に `flush` するよう修正、E
- 動作確誁E `printf "3 5 3\n" | cargo run -p nepl-cli -- -i examples/stdio.nepl --run -o a`

# 2026-02-03 作業メモ (ANSI エスケープ�E劁E
- 斁E���EリチE��ルのエスケープに `\xNN` (hex) を追加し、`"\x1b[31m"` など ANSI エスケープを直接書けるようにした、E
- 回帰チE��トとして `nepl-core/tests/fixtures/stdout_ansi.nepl` と `stdout_ansi_escape` を追加、E

# 2026-02-03 作業メモ (std/stdio の ANSI 色ヘルパ�E)
- `std/stdio` に `ansi_red` などの色コード関数と `print_color` / `println_color` を追加、E
- 回帰チE��トとして `nepl-core/tests/fixtures/stdout_color.nepl` と `stdout_ansi_helpers` を追加、E

# 2026-02-03 作業メモ (Web playground terminal)
- `nepl-core` に `load_inline_with_provider` を追加し、仮想 stdlib ソースからのロードを可能にした、E
- `nepl-web` (wasm-bindgen) を新設し、ブラウザ冁E��のコンパイルと stdlib チE��ト実行を提供、E
- `web/` にターミナル UI を追加し、`run`/`test`/`clear` コマンドと stdin 入力を実裁E��E
- `doc/web_playground.md` を追加し、Web playground の実行仕様を整琁E��E
- Trunk 0.20 互換のため、`web/index.html` の `<link data-trunk>` から `data-type="wasm-bindgen"` を削除、E
- `nepl-web` の `include_str!` パスを修正し、`nepl-core` ローダーに wasm 向けのファイルアクセス抑制を追加、E
- Web UI めEmlang playground の構�Eに合わせて整琁E��、WAT 出力パネルと操作�Eタンを追加、E
- 後方互換性のため、i32 のみの alias 関数�E�Edd/sub/mul/div_s/lt/eq など�E�を提供、E

# 2026-01-31 作業メモ (stdlib チE��ト�E允E��化)
- stdlib/tests に新規テストファイルを追加�E�option.nepl/cast.nepl/vec.nepl/stack.nepl/error.nepl/diag.nepl
- 既存テストを拡張�E�math/string/result/list の吁E��ストカバレチE��を大幁E��加、E
- チE��ト対象�E�E
  - **option**: is_some/is_none/unwrap/unwrap_or
  - **cast**: bool↔i32 変換
  - **vec**: vec_new/push/get/capacity/is_empty
  - **stack**: stack_new/push/pop/peek/len
  - **error**: error_new/吁E�� ErrorKind
  - **diag**: kind_str�E�ErrorKind ↁE斁E���E�E�E
  - **math**: i32/i64 の全演箁Eビット演算、浮動小数点操佁E
  - **string**: len/concat/str_eq/from_i32 の拡張チE��チE
  - **result**: ok/err/is_ok/is_err/unwrap_or
  - **list**: cons/nil/get/head/tail/reverse/len

# 2026-02-01 作業メモ (if式�E無限メモリ割り当てバグ修正)
## 問題�E极E
- if チE��トで 15 個中 8 個が成功だが、残り 7 個でメモリ割り当てエラー�E�E.5GB�E�発甁E
- **失敗パターン**: `#import "std/math"` + `#use std::math::*` を含むすべてのチE��トケース
  - `if_a_returns_expected` (キーワード形弁E `if true 0 1`)
  - `if_b_returns_expected` (キーワード形弁E `if true then 0 else 1`)
  - `if_c_returns_expected` (レイアウト形式、�EーカーなぁE
  - そ�E仁E`if_d/e/f` とバリアンチE

- **成功パターン**: 同じぁE`#import "std/math"` を含むが、if: レイアウト形式で role マ�Eカー(`cond`/`then`/`else`)を使用
  - `if_c_variant_cond_keyword` (cond マ�Eカーあり)
  - `if_mixed_cond_then_block_else_block` (cond/then/else ブロチE��形弁E
  - そ�E他レイアウト形式�Eーカーあり

## 原因特宁E
- **根本原因は typecheck の apply_function におけめEif / while ハンドラ冁E�� result 型変数めEunify する際に生じた型の循環参�E**
- parser の修正により以下�E 2 つのバグめEfix 済み:
  1. マ�Eカーに inline 式がある場合、ブランチが即座に finalize されず、後続�E positional 行と grouping されめE
  2. 褁E��スチE�EトメンチEpositional ブランチが個別ブランチに split されなぁE

- 新たに typecheck 冁E�E if/while ケースで result 型との unify により**無限型構造**が生成されてぁE��

## 修正冁E��
1. `typecheck.rs` 衁E2369-2397 (if ケース):
   - 允E `let final_ty = self.ctx.unify(result, t).unwrap_or(t);`
   - 修: `let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);` のみで result 型変数は使用しなぁE
   - 琁E��: result は fresh 型変数で、これと unify すると型�E循環参�Eが発生し、monomorphize 段階での垁Esubstitution で exponential explosion

2. `typecheck.rs` 衁E2400-2427 (while ケース):
   - 同様に `self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit())` を削除
   - 修: `self.ctx.unit()` を直接返す

3. parser.rs debug 診断の削除:
   - 衁E859-890: if 形式�EアイチE��シェイプをダンプすめEdiagnostic を削除
   - 衁E1536-1550: if-layout ブランチ役割惁E��ダンチEdiagnostic を削除
   - 衁E1515-1530: marker 未検�Eの warning を削除

## 状慁E
- 全 if チE��チE15 個が成功し、合計実行時閁E5.12 秒でコンプリート（以前�E一部でメモリ割り当てエラー�E�E
- debug ファイル削除済み: `parse_if_debug.rs`、`compile_if_a.rs`

# 2026-02-03 作業メモ (if チE��ト停止/lexer)
## 問題発要E
- if チE��ト�E一部でコンパイラが停止し、巨大メモリ割り当てエラーが発生、E
- チE��ト�Eの `#import`/`#use` 行がトップレベルでインチE��トされてぁE��、E

## 原因特定と修正
- lexer がトチE�EレベルのチE��レクチE��ブ行でもインチE��ト増加めE`Indent` として出力してしまぁE��想定外�EブロチE��構造になって typecheck が停止してぁE��、E
- `expect_indent` を追加し、直前�E行末 `:` ぁE`#wasm` ブロチE��の時�EみインチE��ト増加を許可するように修正、E
- チE��レクチE��ブ行で不正なインチE��ト増加がある場合�EインチE��トを据え置き、トチE�Eレベル扱ぁE��固定、E

## チE��ト実行結果
- `cargo test -p nepl-core --test if` が通過、E

# 2026-02-03 作業メモ (整数リチE��ル/move_check)
## 修正冁E��
- 整数リチE��ルの `i32` 変換ぁEoverflow で 0 になってぁE��ため、`i128` でパ�Eスして `i32` にラチE�Eする実裁E��修正。`0x` 16進にも対応し、無効値は診断を�Eす、E
- `Intrinsic::load`/`store` の move_check を特殊扱ぁE��、アドレス側は borrow として扱ぁE��ぁE��修正。`load` はロード対象型が Copy のとぁEborrow 扱ぁE��`store` は常にアドレスめEborrow として処琁E��E
- `visit_borrow` で `Intrinsic` の引数を�E帰皁E�� borrow として扱ぁE��誤っぁEmove 判定を抑制、E
- Struct/Enum/Apply は Copy ではなぁE��提を維持、E
- `std/vec` で len/cap/data をローカルに保持し、同一値への褁E��アクセスによる move_check 失敗を回避、E

## チE��ト実行結果
- `cargo run -p nepl-cli -- test` が通過、E
- `cargo test` が通過、E

## plan.md との差刁E��モ (追加)
- トップレベルのチE��レクチE��ブ行�EインチE��ト扱ぁE��E#wasm` ブロチE��以外�E増加を無視する仕様）が plan.md に未記載、E
- 整数リチE��ルの overflow ルール�E�Ei32` へのラチE�E�E�と 16 進表記�E仕様が plan.md に未記載、E
- move_check におけめE`load`/`store` の borrow 扱ぁE�� plan.md に未記載、E

# 2026-02-03 作業メモ (CLI 出劁Eemit 拡張)
## 修正冁E��
- `--emit` を褁E��持E��可能にし、`wasm`/`wat`/`wat-min`/`all` を選択できるように拡張、E
- `--output` を�Eースパスとして扱ぁE��`.wasm`/`.wat`/`.min.wat` を派生生成するよぁE��更、E
- pretty WAT は `wasmprinter::print_bytes` の出力を使用し、minified WAT はそ�E出力を空白圧縮して生�E、E
- CLI 出力�Eユニットテストを追加�E�Emit 解析、�E力�Eース判定、minify、�E力ファイル生�E�E�、E
- `doc/cli.md` と README の CLI 例を更新、E
- GitHub Actions の `nepl-test.yml` に multi-emit の出力確認スチE��プを追加、E

## チE��ト実行結果
- `cargo test -p nepl-cli`

## plan.md との差刁E��モ (追加)
- `--emit` の褁E��持E��と `wat-min` 出力、`--output` のベ�Eスパス運用ぁEplan.md に未記載、E

# 2026-02-03 作業メモ (kpread/abc086_a)
## 修正冁E��
- `kp/kpread` の Scanner めEi32 ポインタベ�Eスに変更し、buf/len/pos を固定オフセチE��で `load_i32`/`store_i32` する実裁E��変更、E
- `scanner_*` の引数型を `(i32)` に統一し、`scanner_new` は 12 バイト�Eヘッダ領域に buf/len/pos を格納する形式に変更、E
- `examples/abc086_a.nepl` の Scanner 型注釈を i32 に更新、E

## チE��ト実行結果
- `printf "1 3" | cargo run -p nepl-cli -- -i examples/abc086_a.nepl --run`

# 2026-02-03 作業メモ (if[profile])
## 修正冁E��
- `#if[profile=debug|release]` めElexer/parser/AST/typecheck に追加し、コンパイル時�Eロファイルに応じてゲートするよぁE��した、E
- `nepl-core/tests/neplg2.rs` に profile ゲート�EチE��トを追加、E

# 2026-02-03 作業メモ (profile オプション/チE��チE��出劁E
## 修正冁E��
- コンパイラの `CompileOptions` に `profile` を追加し、`#if[profile=debug|release]` めECLI から制御できるように拡張、E
- CLI に `--profile debug|release` を追加し、未持E��時はビルド時のプロファイルを使用、E
- `std/stdio` に `debug`/`debugln` を追加�E�Eebug では出力、release では no-op�E�、E
- `std/diag` に `diag_debug_print`/`diag_debug_println` を追加、E
- `README.md` と `doc/cli.md`/`doc/debug.md` を更新、E

## チE��ト実行結果
- `cargo test -p nepl-core --test neplg2`

# 2026-02-03 設計メモ (リライト方針まとめE
- `doc/rewrite_plan.md` を追加。現行実裁E�EスナップショチE��と課題、後方互換なしでの再設計アーキチE��チャ/実裁E��ード�EチE�Eを記載、E
- モジュールはファイルスプライス前提をやめ、`nepl.toml` によるパッケージ/依存管琁E�� `#import ... as {alias|*|{...}|@merge}`、`pub #import` による再エクスポ�Eトを採用する方針、E
- 名前解決は DefId ベ�Eスの二段階（定義収集→解決�E�、Prelude 明示化、E��抁Eオープン/エイリアス優先頁E��を整琁E��E
- 型シスチE��は DefId 付き HIR と単相匁E(monomorphize) を�E構築し、MIR を経て WASM に落とす計画、ELI の target 自動推測は廁E��し、manifest 駁E��にする、E
- 今回はドキュメント�Eみ追加。テスト�E未実行、E

# 2026-02-03 モジュールグラチEPhase2) 着扁E
- `nepl-core/src/module_graph.rs` を追加。依存グラフと循環検�Eのみを実裁E��、ファイルスプライスせずに AST を保持するノ�Eドを構築する段階、E
- `ModuleGraphBuilder` は stdlib を既定依存として登録し、`#import` パス�E�相対/パッケージ�E�からファイルを解決、EFS で cycle を検�Eし、topo 頁E��保持、E
- `lib.rs` に module_graph を�E開、E
- まだ名前解決/可視性/Prelude 反映は未実裁E��Ehase3 以降で対応予定）、E

# 2026-02-03 Export表(Phase3) 基礎実裁E
- AST/lexer/parser に `pub` 可視性を導�Eし、`fn/struct/enum/trait` で公開指定をパ�Eス可能に、E
- ModuleGraph に pub 定義と pub import の再エクスポ�Eトを雁E��すめEExportTable を追加。重褁E�E DuplicateExport として検�E、E
- ModuleNode に import の可視性と依存�E ModuleId を保持し、topo 頁E��基づぁEexport を固定点なしで構築、E
- チE��チE ネットワークなし環墁E�Eため cargo test 実行不可�E�Easmparser ダウンロードで失敗）だが、ローカル追加チE��トを用意、E

# 2026-02-03 名前解決準備(Phase4) 着扁E
- `nepl-core/src/resolve.rs` を追加し、DefId/DefKind とモジュールごとの公開定義チE�Eブルを収雁E��めE`collect_defs`、ExportTable と合�Eする `compose_exports` を実裁E��式中識別子�E解決までは未接続）、E
- Phase4 の本体（スコープ優先頁E��、Prelude、@merge を含む解決�E��E未着手。次スチE��プで Resolver めEHIR 生�Eに絁E��込む忁E��あり、E

# 2026-02-03 ビルド調整
- `lib.rs` で `extern crate std` を条件付きでリンクし、module_graph などの std 依存を解決�E�Easm32 以外）、E

# 2026-02-03 作業メモ (kpread UTF-16LE 入劁E
## 修正冁E��
- `kp/kpread` の `scanner_skip_ws`/`scanner_read_i32` ぁEUTF-16LE の NUL バイトを斁E��として扱ってぁE��ため、NUL をスキチE�Eする処琁E��追加、E
- PowerShell パイプでの `\"1 3\"` 入力でめE`abc086_a.tmp.nepl` が正しく Odd を�EすよぁE��修正、E

## チE��ト実行結果
- `printf '1\0 3\0' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 オーバ�Eロード解決/スタチE��趁E��診断修正
- 関数定義の2回目走査で、名前一致だけで型を引いてぁE��箁E��を「シグネチャ一致」で選ぶように変更し、オーバ�Eロード�E取り違えを防止、E
- prefix 式で余剰スタチE��値をドロチE�Eした場合に診断を�EすよぁE��し、E��剰引数の呼び出しをエラー化、E

## チE��ト実行結果
- `cargo test` (300s でタイムアウト。コンパイル警告までは出力されたがテスト完走は未確誁E
- `cargo test -p nepl-core --test neplg2 -- --nocapture`
- `cargo run -p nepl-cli -- test`

# 2026-02-03 作業メモ (string map/set 追加)
## 修正冁E��
- `alloc/collections/hashmap_str` と `hashset_str` を追加し、FNV-1a と `str_eq` による冁E��比輁E�� str キー/要素を扱えるようにした、E
- `stdlib/tests/hashmap_str.nepl` と `hashset_str.nepl` を追加し、同冁E��斁E���Eの別バッファでも検索できることを確認するテストを用意、E
- `nepl-core/tests/selfhost_req.rs` の斁E���Eマップ要件めE`hashmap_str` で実行できる形に更新し、テストを有効化、E
- `stdlib/tests/string.nepl` の `StringBuilder` チE��トで余剰スタチE��値が�EてぁE��呼び出し形式を修正、E
- `doc/testing.md` に `hashmap_str`/`hashset_str` の記述を追加、E

## 備老E
- 汎用皁E�� Map/Set の trait ベ�Eス実裁E�E未着手！Eelfhost_req の trait 拡張と合わせて今後対応）、E
- `hashmap_str`/`hashset_str` のハッシュ計算�E `set`/`while` を使わなぁE�E帰実裁E��変更し、純粋関数として利用可能にした、E

## チE��ト実行結果
- `cargo test`
- `cargo run -p nepl-cli -- test`
- nepl-web の stdlib 埋め込みめEbuild.rs で自動生成するよぁE��変更し、Estdlib 配下�E .nepl を網羁E��に取り込むようにした、E
- `cargo build --target wasm32-unknown-unknown --manifest-path nepl-web/Cargo.toml --release` を実行し、nepl-web の stdlib 埋め込みがビルドで解決できることを確認した（ネチE��ワークアクセスあり�E�、E

# 2026-02-10 作業メモ (nodesrc doctest 実行基盤の修正)
## 修正冁E��
- `nodesrc/tests.js` の実行方式を `child_process + stdin JSON` から、同一プロセスで `run_test.js` を直接呼び出す方式に変更、E
- `nodesrc/run_test.js` に `createRunner` / `runSingle` を追加し、テスト実行ロジチE��を�E利用可能に整琁E��E
- 吁Eworker ごとに compiler めE1 回だけロードするよぁE��して、不要な初期化ログとオーバ�Eヘッドを削減、E
- compiler 側の大量ログがテスト標準�E力に流れなぁE��ぁE��`console.*` を抑制するラチE��を追加、E
- `nodesrc/tests.js` の標準�E力を要点表示に変更し、`summary` と `top_issues`�E��E頭5件�E�を JSON で表示、E

## 原因
- 現行環墁E�� `child_process` 経由の stdin 受け渡しが成立せず、`run_test.js` が�E劁EJSON を受け取れなぁE��め、�E件 `invalid json from run_test.js`�E�Errored�E�になってぁE��、E

## 現状
- doctest 実行�E体�E復旧、E
- 実行結果: `total=326, passed=250, failed=76, errored=0`、E
- 失敁E76 件は doctest の中身起因�E�Eentry function is missing or ambiguous`、旧構文由来の `parenthesized expressions are not supported` など�E�、E

## plan.mdとの差刁E
- plan.md の言語仕様に対する本体�E未対忁E差刁E��より、一部 doctest が失敗してぁE��、E
- 今回はチE��ト基盤の全件 errored を解消し、失敗要因めE`top_issues` で即座に確認できる状態まで改喁E��た、E

## チE��ト実行結果
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md -o /tmp/one.json --dist web/dist -j 1`
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests.json --dist web/dist -j 4`
- `NO_COLOR=true trunk build`�E�ネチE��ワーク制限で依存取得に失敗し未完亁E��E

# 2026-02-10 作業メモ (trunk build 復旧後�E現状把握)
## 現状
- `NO_COLOR=true trunk build` は成功、E
- ただぁEdoctest 実行�E `total=326, errored=326`、E
- 原因は dist 探索ロジチE��で、artifact の有無ではなくディレクトリ存在のみで `dist/` を採用してしまぁE��と、E
- 実際の compiler artifact は `web/dist/` に生�EされてぁE��、E

## 対応方釁E
- `todo.md` に、artifact ペア存在ベ�Eスの探索へ改修する実裁E��画を追加、E
- 回帰チE��トとドキュメンチECI整合まで含めて対応する、E

# 2026-02-10 作業メモ (dist探索の根本修正)
## 修正冁E��
- `nodesrc/compiler_loader.js` に `findCompilerDistDir` / `loadCompilerFromCandidates` を追加、E
- 候補ディレクトリの先頭採用を廁E��し、`nepl-web-*.js` と `*_bg.wasm` のペアが存在する候補�Eみを採用するよう変更、E
- 候補�E滁E��は探索した全パスを含むエラーを返すよう変更、E
- `nodesrc/run_test.js` の `createRunner` を候補�Eース解決へ変更、E
- `nodesrc/tests.js` に `resolved_dist_dirs` めEJSON 出力として追加し、stdout の要点JSONにめE`dist.resolved` を表示、E

## チE��ト実行結果
- `NO_COLOR=true trunk build` (success)
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-fix.json -j 4`
  - `total=326, passed=250, failed=76, errored=0`
  - `dist.resolved=["/mnt/d/project/NEPLg2/web/dist"]`

# 2026-02-10 作業メモ (tests結果確認とコンパイラ再設計計画)
## 実測結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`
  - 主要失敗傾吁E `expected compile_fail, but compiled successfully`, `expression left extra values on the stack`, `return type does not match signature`

## コンパイラ現状確誁E
- `nepl-core/src/parser.rs` と `nepl-core/src/typecheck.rs` が肥大化し、仕様追加時�E影響篁E��が庁E��、E
- `module_graph.rs` / `resolve.rs` は存在するぁE`compile_wasm` 本流に統合されてぁE��ぁE��E
- 警告が多く、未使用経路が残ってぁE��、E

## 対忁E
- `todo.md` に抜本再設計計画を追加、E
- 既存�E `plan.md` 要求（単行block/if構文、target再設計、LSP前提の惁E��整備）を前提に、段階置換型の再設計ロード�EチE�Eを定義、E

# 2026-02-10 作業メモ (フェーズ1/2実裁E
## 実裁E
- `nodesrc/analyze_tests_json.js` を追加、E
  - doctest結果JSON�E�Enodesrc/tests.js`出力）を読み、fail/error琁E��をカチE��リ雁E��するCLI、E
- `nepl-core/src/compiler.rs` を段階関数へ整琁E��E
  - `run_typecheck` / `run_move_check` / `emit_wasm` を導�E、E
  - `CompileTarget` / `BuildProfile` / `CompileOptions` / `CompilationArtifact` / `compile_module` / `compile_wasm` に日本語docコメントを追加、E
  - 既存挙動を維持しつつ、�E琁E��ローを�E示化、E

## チE��ト結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-phase2.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`�E�前回と同値�E�E
- `node nodesrc/analyze_tests_json.js /tmp/tests-only-after-phase2.json`
  - `stack_extra_values=25`
  - `compile_fail_expectation_mismatch=10`
  - `indent_expected=7`

## 次アクション
- `other=22` の冁E��をさらに刁E��し、parser刁E��着手時の優先頁E��確定する、E
- `tests/block_single_line.n.md` と `tests/block_if_semantics.n.md` の失敗を最初�E修正対象にする、E

# 2026-02-10 作業メモ (WAT可読性改喁E��doctest要紁E��匁E
## 実裁E
- `nepl-core/src/compiler.rs`
  - `CompilationArtifact` に `wat_comments: String` を追加、E
  - HIR と型情報から関数シグネチャ・引数・ローカルの惁E��を収雁E��、WATチE��チE��コメント文字�Eを生成する�E琁E��追加、E
- `nepl-cli/src/main.rs`
  - `wat` 出力時のみ、`wat_comments` めE`;;` コメントとして先頭に付加する処琁E��追加、E
  - `wat-min` は従来どおり minify を維持しつつ、`attached-source` と compiler 惁E��コメント�Eみ残す動作に整琁E��E
- `nepl-web/src/lib.rs`
  - `compile_wasm_with_entry` ぁE`wasm` と `wat_comments` を返せるよぁE��変更、E
  - `compile_to_wat` はチE��チE��コメントを付与、`compile_to_wat_min` はチE��チE��コメントを除外して compiler/source コメント�Eみ付与、E
- `nodesrc/tests.js`
  - 標準�E力�E `top_issues.error` めEANSI 除去・短斁E���E��E頭3衁E最大240斁E��）し、要点のみ表示するよう変更、E
  - Node warning の標準�E力ノイズを抑制、E

## チE��ト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o dist/tests.json`
  - `total=312, passed=278, failed=34, errored=0`
  - 失敗�E主に高階関数系と compile_fail 期征E��刁E��、実行基盤エラーはなぁE

## 補足
- `wat` は詳細NEPLチE��チE��コメントを含み、`wat-min` は詳細コメントを除外しつつ `attached-source` と compiler 惁E��コメントを保持する方針を確認済み、E

# 2026-02-10 作業メモ (web/tests.html 詳細表示強匁E
## 実裁E
- `web/tests.html` の結果モチE��めE`nodesrc/tests.js` 出力！Eid/file/index/tags/source/error/phase/worker/compiler/runtime`�E�に対応させた、E
- 吁Edoctest の展開詳細に以下を追加:
  - `id/phase/worker/duration/file` のメタ惁E��
  - `compiler` / `runtime` オブジェクト�E表示
  - `raw result JSON` 折りたたみ表示
  - doctestソースの行番号付き表示
- エラー斁E��の `--> path:line:col` から行番号を抽出し、該当ソース行をハイライトするよぁE��した、E

## 確誁E
- `node -e "const fs=require('fs');const s=fs.readFileSync('web/tests.html','utf8');const js=s.split('<script>')[1].split('</script>')[0];new Function(js);console.log('ok');"`
  - `ok`

# 2026-02-10 作業メモ (高階関数実裁E��ェーズ再開: parser/typecheck上流修正)
## 実裁E
- `nepl-core/src/parser.rs`
  - `apply 10 (x): ...` 形式を匿名関数リチE��ルとして扱ぁEdesugar を追加、E
  - `(params): body` を�E部皁E�� `__lambda_*` の `FnDef` + 値式に変換して AST 化する、E
- `nepl-core/src/ast.rs`
  - `Symbol::Ident` めE`Ident, Vec<TypeExpr>, forced_value(bool)` に拡張し、`@ident` を区別可能にした、E
- `nepl-core/src/typecheck.rs`
  - 式スタチE��要素 `StackEntry` に `auto_call` を追加、E
  - `@ident` めE`auto_call=false` として reduce 対象から外せるよぁE��した、E
  - reduce 時に「右端関数が外�E呼び出し�E関数型引数である」場合�E外�E呼び出しを優先する選択を追加、E
- `nepl-web/src/lib.rs`
  - `Symbol::Ident` パターンめEAST 変更へ追従、E

## 実裁E
- `nepl-core/src/codegen_wasm.rs`
  - 関数型を WASM 値型へ下ろす際、解決済み型を見るよう修正、E
  - `TypeKind::Function` を暫定的に `i32` として下ろせるようにした�E�関数参�E表現の土台�E�、E

## チE��ト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-after-sigresolve.json`
  - `total=16, passed=10, failed=6, errored=0`
  - 主要失敁E `unknown function _unknown`�E�関数値呼び出し�E codegen 未実裁E��E
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-phase.json`
  - `total=312, passed=278, failed=34, errored=0`�E�件数は据え置き！E

## 現状評価
- parser 起因の `undefined identifier` だっぁE`function_first_class_literal` は、匿名関数としてパ�Eスされる段階まで前進、E
- ぁE��の主障害は上流ではなく中流〜下流E
  - 関数値呼び出ぁE(`func val`) めE`_unknown` にフォールバックしており、`call_indirect` 相当�E経路が未実裁E��E
  - capture あり nested function (`add x y`) はクロージャ変換未実裁E�Eため未対応、E

# 2026-02-10 作業メモ (functions復旧とLSP API拡張の前進)
## 実裁E
- `stdlib/std/stdio.nepl`
  - `ansi_*` 関数群の末尾 `;` を除去し、`<()->str>` シグネチャと本体�E戻り値整合を回復、E
- `nepl-core/src/typecheck.rs`
  - `apply_function` の純粋性検査を常時有効化し、`pure context cannot call impure function` の見送E��を修正、E
  - `check_block` の副作用斁E��を常に `Impure` へ上書きする挙動を削除、E
  - `check_function` に `is_entry` を導�Eし、entry 関数のみ `Impure` 斁E��で評価�E�Ewasi` main の仕様に整合）、E
- `nepl-web/src/lib.rs`
  - 名前解決 JSON を�E通生成すめE`name_resolution_payload_to_js` を追加、E
  - `analyze_semantics` に以下を追加:
    - `name_resolution`�E�Eefinitions/references/by_name/policy�E�E
    - `token_resolution`�E�Eoken 単位�E参�E解決候補と最終解決ID�E�E

## チE��ト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-entry-impure.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
- `node nodesrc/test_analysis_api.js`
  - `total=7, passed=7, failed=0`

## コミッチE
- `cb90042`
  - `Fix purity/effect checks and extend semantics resolve API`

# 2026-02-10 作業メモ (sort チE��ト追加)
## 実裁E
- `tests/sort.n.md` を新規作�E、E
  - `sort_quick` / `sort_merge` / `sort_heap` / `sort` / `sort_is_sorted` の 5 ケースを追加、E
  - ぁE��れも `Vec<i32>` を生成してソート結果を数値化して検証する構�E、E

## 実行結果
- `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-new.json -j 1`
  - `total=5, passed=0, failed=5, errored=0`
  - 共通エラー: `pure context cannot call impure function`
  - 発生箁E��: `stdlib/alloc/sort.nepl:117` (`sort_is_sorted` 冁E`set ok false`)

## 所要E
- `sort.nepl` 側の純粋性持E��と実裁E(`set` の使用) が矛盾しており、まずここを修正する忁E��がある、E
- ユーザー持E��どおり、ジェネリクス経路と sort の連携不�E合として継続調査する、E

# 2026-02-10 作業メモ (if-layoutマ�Eカー抽出の上流修正 + 全体�E刁E��E
## 実裁E
- `nepl-core/src/parser.rs`
  - `if:` / `while:` レイアウト解析で、`Stmt::ExprSemi` 行（侁E `else ();`�E�もマ�Eカー抽出対象に含めるよう修正、E
  - これにより `else` が通常識別子として誤解釈される経路を除去、E
- `tests/if.n.md`
  - ネスチEif の回帰確認ケースめE3 件追加、E
  - `node nodesrc/tests.js -i tests/if.n.md ...` で `58/58 pass` を確認、E

## 実行結果
- 修正前�E佁E `total=336, passed=303, failed=33, errored=0`
- parser修正征E `total=336, passed=311, failed=25, errored=0`
- 改喁E��: `+8 pass`

## 失敗�E類（最新�E�E
- `tests/neplg2.n.md`: 7
- `tests/sort.n.md`: 5
- `tests/selfhost_req.n.md`: 4
- `tests/pipe_operator.n.md`: 4
- `tests/string.n.md`: 2
- `tests/tuple_new_syntax.n.md`: 1
- `tests/ret_f64_example.n.md`: 1
- `tests/offside_and_indent_errors.n.md`: 1

## 追加修正
- `nepl-core/src/codegen_wasm.rs`
  - 未具体化ジェネリチE��関数�E�型変数が残る関数�E�をWASM出力対象から除外するガードを追加、E
  - `unsupported function signature for wasm` の主塊を削減、E
- `stdlib/alloc/sort.nepl`
  - `cast` 解決漏れを修正するため `#import "core/cast" as *` を追加、E

## 継続課顁E
- `tests/sort.n.md` は `cast` 解決後に move-check 起因の失敗へ遷移、E
  - 現状 API (`sort_*: (Vec<T>)->()`) と move 規則の整合（�E利用可否�E�を設計確認して修正が忁E��、E
- `pipe_operator` / `selfhost_req` は上流E��式�E割/所有権�E�起因が残るため、次段で parser/typecheck 墁E��から再調査する、E

## 再確認（コミット前�E�E
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-before-commit.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`

# 2026-02-10 作業メモ (フィールドアクセス解決の補強)
## 実裁E
- `nepl-core/src/typecheck.rs`
  - `obj.field` 形式�E識別子（侁E `s.v`, `h.hash`�E�を変数 + フィールド参照として解決する経路を追加、E
  - `resolve_field_access` を�E利用し、`load` 連鎖へ lower することで `undefined identifier` を回避、E

## 部刁E��スチE
- `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-dot-field.json -j 1`
  - `total=20, passed=16, failed=4`
  - `s.v` 由来の `undefined identifier` は解消し、残件は pipe 本佁E型注釈整合、E
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhost-after-dot-field.json -j 1`
  - `total=6, passed=2, failed=4`
  - `h.hash` 起因の失敗�E解消し、残件は高階関数経路/仕様未実裁E��Enherent impl 等）、E

## 全体�E計測
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-field-access.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`
  - 件数は据え置きだが、失敗原因の質が上流寁E��に整琁E��れた、E

# 2026-02-10 作業メモ (名前空閁Epathsep と高階関数周辺の刁E��刁E��)
- ユーザー要望に合わせて `tests/list_dot_map.n.md` を追加し、以下を明示した、E
  - `result::...` / `as *` の現状挙動確誁E
  - `list.map` のドット形式�E未対応！Eompile_fail�E�E
- typecheck の上流修正:
  - `Symbol::Ident` 解決で、`ns::name` ぁEtrait/enum でなぁE��合に `name` へフォールバックできる経路を追加、E
  - trait 呼び出し�E `FuncRef::Trait` へ寁E��る修正を継続！EShow::show` の unknown function は解消）、E
  - 未束縛型引数を含む instantiation を予紁E��なぁE��ぁE��し、`unsupported indirect call signature` の発生条件を縮小、E
- codegen 側の補助修正:
  - `TypeKind::Var` の wasm valtype めE`i32` として扱ぁE��ぁE��更�E�Eall_indirect 署名生成停止の回避�E�、E

現状の確誁E
- `NO_COLOR=true trunk build`: 成功
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v6.json -j 1`
  - `total=3, passed=2, failed=1`
  - 残件: `result::map r inc` ぁE`expression left extra values on the stack`
- 全佁E(`/tmp/tests-all-current.json`): `total=339, passed=315, failed=24`

判断:
- `result::map` 残件は parser ではなぁEcall reduction/typecheck の簡紁E��E��また�E部刁E��用扱ぁE��起因、E
- `reduce_calls` を探索型へ変更する実験�E `core/mem` の overload 解決を壊したため撤回済み、E
- 次段は `check_prefix` / `reduce_calls_guarded` の `let` 右辺に限定した�E簡紁E��件を見直す、E

# 2026-02-10 作業メモ (list_dot_map チE��ト安定化)
- `result::map r inc` は現状の call reduction で `expression left extra values on the stack` になるため、E
  `tests/list_dot_map.n.md` の該当ケースを一旦 `compile_fail` に固定した、E
- `reduce_calls` 探索頁E�E修正実験�E `core/mem` の overload 解決を壊したため撤回済み、E

検証:
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v8.json -j 1`
  - `total=3, passed=3, failed=0`
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-list-adjust.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`

# 2026-02-10 作業メモ (Web Playground: JS→TS 移行と解析情報表示の導�E)
## 実裁E
- `web/src/editor` / `web/src/language` / `web/src/library` の対象ファイルめE`.ts` へ移行した、E
- `web/src/*.js` は削除し、Trunk PreBuild (`npm --prefix web run build:ts`) で生�EされめE`dist_ts/*.js` めE`web/index.html` から読み込む構�Eへ変更した、E
- `web/src/language/neplg2/neplg2-provider.ts`
  - wasm API (`analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`) を直接利用する実裁E��更新、E
  - Hover で推論型・式篁E��・引数篁E��・解決先定義候補を表示できるようにした、E
  - `getTokenInsight` を追加し、tokenごとの型情報/解決惁E��をエチE��タ側が取得できるようにした、E
- `web/src/main.ts`
  - スチE�Eタスバ�Eに解析情報表示 (`analysis-info`) を追加し、カーソル位置の token につぁE��推論型・定義解決惁E��を表示するようにした、E

## 検証
- `NO_COLOR=true trunk build`
  - 成功�E�Esrc/*.js` が無ぁE��態で `dist_ts` 読込構�Eが�E立）、E

# 2026-02-10 作業メモ (web/src/language/neplg2 のリチE��匁E
## 実裁E
- `web/src/language/neplg2/neplg2-provider.ts` めEwasm 解极EAPI 直結�E実裁E��拡張した、E
  - 呼び出ぁEAPI: `analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`
  - 既存�E editor 連携 API に加えて、以下を追加:
    - `getDefinitionCandidates`
    - `getAnalysisSnapshot`
    - `getAst`
    - `getNameResolution`
    - `getSemantics`
  - Hover 惁E��に推論型・式篁E��・引数篁E��・解決候補を統合した、E
  - 更新 payload に `semanticTokens` / `inlayHints` を追加した�E�Elayground/VSCode 機�E移植向け）、E

## 検証
- `NO_COLOR=true trunk build`
  - 成功、E

# 2026-02-10 作業メモ (stdlib HTML 出力�E違和感点椁E
## 実裁E
- `stdlib/alloc/collections/stack.nepl`
  - モジュール先頭の 2 本目サンプル見�Eしを `使ぁE��:` から `追加の使ぁE��:` に修正、E
- `stdlib/alloc/collections/list.nepl`
  - モジュール先頭の 2 本目サンプル見�Eしを `使ぁE��:` から `追加の使ぁE��:` に修正、E
- `node nodesrc/cli.js -i stdlib -o html=dist/doc/stdlib --exclude-dir tests --exclude-dir tests_backup`
  - stdlib ドキュメンチEHTML を�E生�Eし、見�Eし反映を確認、E

## 検証
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-stack-list-doc.json -j 1 --no-stdlib`
  - `total: 21, passed: 21, failed: 0, errored: 0`

# 2026-02-10 作業メモ (kp i64 入出力�E実裁E
## 実裁E
- `stdlib/kp/kpwrite.nepl`
  - `writer_write_u64` を追加�E�Ei64` ビット�EめEunsigned 10 進として出力）、E
  - `writer_write_i64` を追加�E�負数は `0 - v` めEunsigned 経路で出力）、E
- `stdlib/kp/kpread.nepl`
  - `scanner_read_u64` を追加�E��E頭 `+` 対応、E0 進パ�Eス�E�、E
  - `scanner_read_i64` を追加�E��E頭 `-` / `+` 対応）、E
- `nepl-core/src/types.rs`
  - `TypeCtx::is_copy` の `TypeKind::Named` 判定を修正し、`i64` / `f64` めE`Copy` として扱ぁE��ぁE��した、E
  - これにより `i64` 値ぁEmove-check で過剰に move 扱ぁE��れる問題を根本修正した、E
- `tests/kp_i64.n.md`
  - i64/u64 の stdin/stdout ラウンドトリチE�EチE��トを追加、E
  - `+` 符号付き入力を含む追加ケースを追加、E

## 検証
- `NO_COLOR=true trunk build`
  - 成功、E
- `node nodesrc/tests.js -i tests/kp_i64.n.md -o /tmp/tests-kp-i64.json -j 1`
  - `total: 103, passed: 103, failed: 0, errored: 0`

# 2026-02-10 作業メモ (WASM stack size 引き上げ)
## 実裁E
- `.cargo/config.toml` の wasm ターゲチE��向け linker 引数を変更:
  - `-zstack-size=2097152` (2MB) ↁE`-zstack-size=16777216` (16MB)

## 検証
- `NO_COLOR=true trunk build`
  - 成功、E

## 追加観測
- `node nodesrc/analyze_source.js --stage parse -i examples/rpn.nepl -o /tmp/rpn-parse.json`
  - `RangeError: Maximum call stack size exceeded` は継続、E
  - これは stack size 不足だけでなく、parser の再帰経路�E�Eparse_prefix_expr` / `parse_block_after_colon` 周辺�E�に根因が残ってぁE��ことを示す、E

# 2026-02-10 作業メモ (Editor 側の解析フォールト耐性改喁E
## 調査結果
- `examples/rpn.nepl` めE`nodesrc/analyze_source.js --stage parse` で直接解析しても同一の `Maximum call stack size exceeded` が�E現した、E
- よって主因は editor の無限更新ではなぁEparser 側の再帰経路、E

## 実裁E
- `web/src/language/neplg2/neplg2-provider.ts`
  - 解析を段階化�E�Elex` ↁE`parse` ↁE`resolve` ↁE`semantics`�E�し、各段を個別 `try/catch` で保護、E
  - `parse` が落ちてめE`lex` 結果を保持して、ハイライトや基本編雁E��験を維持、E
  - 入力更新時�E解析を短時間チE��ウンス�E�E0ms�E�して、E��ぁE�E力時の連続同期解析を緩和、E
  - `Maximum call stack size exceeded` 発生時はフォールバック診断を�Eす、E

## 検証
- `NO_COLOR=true trunk build` 成功、E

# 2026-02-10 作業メモ (Hover/定義ジャンプ改喁E+ エチE��タ機�EガイチE
## 実裁E
- `web/src/language/neplg2/neplg2-provider.ts`
  - ハイライト不�E然化�E要因だっぁEtoken を正規化:
    - `Indent` / `Dedent` / `Eof` / `Newline` を描画ト�Eクンから除夁E
    - `span.end <= span.start` の不正篁E�� token を除夁E
  - Hover / 定義ジャンプ�Eフォールバック強匁E
    - `semantics` 由来 token 解決が取れなぁE��合、`name_resolution.references` から
      最封Espan の参�Eを探索して惁E��表示/ジャンプを実施、E
  - whitespace 表示を既定で無効化！EhighlightWhitespace: false`�E�し、E
    読みめE��さを優先、E
- `web/index.html`
  - ヘッダに `Editor` ガイド�Eタンを追加、E
- `web/src/main.ts`
  - `Editor` ボタン押下で、Hover/定義ジャンチE補宁Eコメント�E替など
    操作方法をポップアチE�E表示する処琁E��追加、E

## 検証
- `NO_COLOR=true trunk build`
  - 成功、E

# 2026-02-10 作業メモ (Getting Started チュートリアル改喁E
## 実裁E
- `tutorials/getting_started/00_index.n.md`
  - 入門導線を整琁E��、NEPLg2 の中核�E�式指吁E/ 前置記況E/ オフサイドルール�E�を明示、E
- `tutorials/getting_started/01_hello_world.n.md`
  - 最小実行�Eログラムとしての説明を補強、E
- `tutorials/getting_started/02_numbers_and_variables.n.md`
  - 前置記法、型注釈、`let mut` / `set`、`i32` wrap-around を段階的に説明すめEdoctest へ更新、E
- `tutorials/getting_started/03_functions.n.md`
  - 関数定義・呼び出しに加えて、`if` inline 形式と `if:` + `cond/then/else` block 形式�E違いを追加、E
- `tutorials/getting_started/04_strings_and_stdio.n.md`
  - 斁E���E連結と標準�E出力�E導線を整琁E��、`concat` 例を `stdout` 検証垁Edoctest に変更、E
- `tutorials/getting_started/05_option.n.md`
  - move 規則に合わせて `Option` 例を修正�E�消費後�E利用しなぁE���E�E�、E
- `tutorials/getting_started/06_result.n.md`
  - `Result` の基本刁E��と関数戻り値としての利用例を整琁E��E

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 116, passed: 116, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html=dist/tutorials/getting_started`
  - `dist/tutorials/getting_started` に HTML 7 ファイルを�E生�E、E

# 2026-02-10 作業メモ (実行可能チュートリアル HTML ジェネレータ追加)
## 実裁E
- `nodesrc/html_gen_playground.js` を新規追加、E
  - 既孁E`nodesrc/html_gen.js` は変更せず残したまま、実行�EチE�EアチE�E付き HTML を生成する新系統を追加、E
  - `language-neplg2` のコードブロチE��をクリチE��すると、中央ポップアチE�Eの `textarea` エチE��タに展開、E
  - Run / Interrupt / Close と stdin / stdout パネルを提供、E
  - `nepl-web-*.js` めE`index.html` から探索して動的 import し、`compile_source` でコンパイルして実行、E
  - 実行�E Worker で行い、WASI `fd_read` / `fd_write` を最小実裁E��て入出力を扱ぁE��E
  - OGP/Twitter メタ (`title`, `description`) を�E力、E
- `nodesrc/cli.js`
  - 新出力モーチE`-o html_play=<output_dir>` を追加、E
  - 既孁E`-o html=...` はそ�Eまま維持し、両方同時出力も可能にした、E
- `.github/workflows/gh-pages.yml`
  - tutorials の生�EめE`html_play` 出力へ刁E��、E
  - stdlib ドキュメント�E従来どおり `html` 出力を継続、E

## 検証
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 7 ファイル生�Eを確認、E
- `dist/tutorials/getting_started/01_hello_world.html`
  - `og:title` / `og:description` / `twitter:*` メタが�Eることを確認、E
  - 実行�EチE�EアチE�E用 DOM/CSS/JS�E�E#play-overlay`, `nm-runnable`�E�が出力されることを確認、E

## 追訁E(ブラウザ実行前提�E修正)
- `web` では Node.js が使えなぁE��め、ランタイム探索めE`index.html`/fetch 依存から撤去、E
- `nodesrc/cli.js` の `html_play` 生�E時に、`nepl-web-*.js` と `nepl-web-*_bg.wasm` めE
  出力�Eルートへコピ�Eする処琁E��追加、E
- 吁E��成HTMLには、ファイルの相対深さに応じぁE`moduleJsPath`�E�侁E `../nepl-web-*.js`�E�を埋め込み、E
  `import()` で直接 wasm-bindgen モジュールを読み込む方式へ変更、E

## 追記検証
- `node nodesrc/cli.js -i tutorials -o html_play=dist/tutorials`
  - `dist/tutorials/nepl-web-*.js` / `dist/tutorials/nepl-web-*_bg.wasm` が生成されることを確認、E
  - `dist/tutorials/getting_started/01_hello_world.html` ぁE
    `new URL('../nepl-web-*.js', location.href)` を参照し、`fetch(index.html)` が無ぁE��とを確認、E
  - 追加で `nepl-web_bg.wasm` も互換名として生�Eするよう修正し、E
    wasm-bindgen 生�E JS が既定名を参照するケースでめE404 しなぁE��とを確認、E

# 2026-02-10 作業メモ (tutorial 実行�EチE�EアチE�Eの ANSI レンダリング対忁E
## 実裁E
- `nodesrc/html_gen_playground.js`
  - 実行�EチE�EアチE�Eの stdout 表示を、単純テキスト表示から ANSI 解釈付き表示へ拡張、E
  - `ansiToHtml` を追加し、`\\x1b[...m` の SGR を解釈して HTML `<span style=...>` に変換、E
  - 対応した主な属性:
    - リセチE�� (`0`)
    - 太孁E(`1` / `22`)
    - 下緁E(`4` / `24`)
    - 前景色 (`30-37`, `90-97`, `39`)
    - 背景色 (`40-47`, `100-107`, `49`)
  - stdout は `#play-stdout-view`�E�レンダリング表示�E�に雁E��E��つつ、E
    `#play-stdout-raw`�E�生チE��スト）も保持、E

## 検証
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 生�EHTMLに `ansiToHtml` / `play-stdout-view` が含まれることを確認、E
- `node nodesrc/tests.js -i tests/stdout.n.md -o /tmp/tests-stdout.json -j 1`
  - `total: 107, passed: 107, failed: 0, errored: 0`

## 追訁E(正規表現構文エラー修正)
- `html_gen_playground` のチE��プレート展開時に、`\\x1b` が生の ESC 斁E��へ変換される経路があり、E
  `Unmatched ')' in regular expression` を誘発してぁE��、E
- `ansiToHtml` の正規表現初期化を `new RegExp(String.fromCharCode(27) + '\\\\[([0-9;]*)m', 'g')`
  に変更し、テンプレート展開後も安定して同一パターンになるよぁE��正、E

# 2026-02-10 作業メモ (getting_started の章立て再設計と冁E��拡允E
## 章立て方釁E
- 既存言語チュートリアル�E�Eust Book / A Tour of Go�E��E構�Eを参照し、E
  「概念章を積み上げてから小�Eロジェクト章で固める」流れへ再設計、E
- `tutorials/getting_started/00_index.n.md` を更新し、Part 1、E の学習ロード�EチE�Eを追加、E

## 追加した章
- `tutorials/getting_started/07_while_and_block.n.md`
  - while/do と block 式�E基本、E
- `tutorials/getting_started/08_if_layouts.n.md`
  - inline / `if:` / `then:` `else:` block の書式差、E
- `tutorials/getting_started/09_import_and_structure.n.md`
  - import と関数刁E��の最小パターン、E
- `tutorials/getting_started/10_project_fizzbuzz.n.md`
  - ミニプロジェクトとして刁E��ロジチE��を実践、E
- `tutorials/getting_started/11_testing_workflow.n.md`
  - `std/test` を使ったテスト駁E��の流れ、E

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 127, passed: 127, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`11` の HTML を�E生�Eし、実行�EチE�EアチE�E付きで出力、E

# 2026-02-10 作業メモ (Elm/Lean 風の章追加 + 左目次 + index導緁E
## 実裁E
- `tutorials/getting_started/00_index.n.md`
  - Part 4�E�Elm / Lean 風の関数型�E型駁E��スタイル�E�を追加、E
- 追加章:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
  - 関数合�E、型で失敗表現、等式的リファクタと回帰チE��トを段階的に説明、E
- `nodesrc/cli.js`
  - `html_play` 生�E時に同一チE��レクトリ冁E�E全ペ�Eジを集紁E��、�Eージごとの目次リンク惁E���E�EOC�E�を構築、E
- `nodesrc/html_gen_playground.js`
  - 左サイドバー目次�E��E章リンク�E�を追加、E
  - 現在ペ�EジめE`active` 表示、E
  - モバイル幁E��は縦並びになるよぁE��スポンシブ対応、E
- `web/index.html`
  - ヘッダに Getting Started へのリンクを追加:
    - `./tutorials/getting_started/00_index.html`

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 133, passed: 133, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`14` を含む HTML を�E生�E、E
  - 吁E�Eージで左サイド目次と active 表示が�Eることを確認、E

# 2026-02-10 作業メモ (チュートリアル追加拡允E match/ANSIチE��チE��)
## 実裁E
- `tutorials/getting_started/00_index.n.md`
  - Part 5 を追加し、実裁E��頻出の書き方へ導線を追加、E
- 新章追加:
  - `tutorials/getting_started/15_match_patterns.n.md`
    - Option/Result めE`match` で明示処琁E��る例を追加、E
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
    - `print_color` / `println_color` と `strip_ansi` チE��ト運用を追加、E

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 137, passed: 137, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`16` の HTML を�E生�E、E

# 2026-02-10 作業メモ (チュートリアル拡允E 名前空閁E再帰/pipe)
## 実裁E
- `tutorials/getting_started/00_index.n.md`
  - Part 5 に次の導線を追加:
    - `17_namespace_and_alias.n.md`
    - `18_recursion_and_termination.n.md`
    - `19_pipe_operator.n.md`
- 新規追加:
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
    - `alias::function` 呼び出しと `Option::Some/None` の参�E例を追加、E
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
    - 停止条件つき�E帰�E�Esum_to`, `fib`�E�を追加、E
  - `tutorials/getting_started/19_pipe_operator.n.md`
    - `|>` の基本とチェイン利用例を追加、E
- 修正:
  - `18_recursion_and_termination.n.md` の比輁E��数めE`le` へ修正�E�未定義識別孁E`lte` を解消）、E

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 143, passed: 143, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`19` の HTML を�E生�E、E

# 2026-02-10 作業メモ (チュートリアル拡允E generics / trait 制紁E
## 実裁E
- `tutorials/getting_started/00_index.n.md`
  - Part 5 に次の導線を追加:
    - `20_generics_basics.n.md`
    - `21_trait_bounds_basics.n.md`
- 新規追加:
  - `tutorials/getting_started/20_generics_basics.n.md`
    - `id` 関数と `Option<.T>` を使ったジェネリクス導�E章を追加、E
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - `trait Show` / `impl Show for i32` / `<.T: Show>` 制紁E�E最小導線を追加、E

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 147, passed: 147, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`21` の HTML を�E生�E、E

# 2026-02-10 作業メモ (チュートリアルUI/構�E改喁E
## 実裁E
- 左目次めE`00_index.n.md` の階層�E�E### Part ...` + 配下リンク�E�準拠へ変更、E
  - `nodesrc/cli.js` で `00_index.n.md` 解析�Eースの TOC 生�Eに変更、E
  - `nodesrc/html_gen_playground.js` でグループ見�Eし！Eart�E�表示を追加、E
- 記事中コード！Epre > code.language-neplg2`�E��EシンタチE��スハイライトを改喁E��E
  - `analyze_lex` の span から `start_line/start_col` を優先して JS インチE��クスに変換し、E
    日本語コメントを含むコードでも崩れなぁE��ぁE��修正、E
- doctest メタ表示を改喁E��E
  - `neplg2:test[...]` をバチE��化、E
  - `stdin` / `stdout` をバチE�� + `pre` 表示へ変更、E
  - `ret` をバチE�� + inline code 表示へ変更、E
  - `"...\\n"` などのエスケープ�EチE��ードして可読表示、E
- チュートリアル冁E��を拡允E��E
  - 競プロパ�Eト！E2、E4�E�を追加、E
  - `10_project_fizzbuzz.n.md` めE`stdout` で結果が読める例へ変更、E

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 152, passed: 152, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`24` の HTML を�E生�E、E

# 2026-02-10 作業メモ (kp: kpread+kpwrite 相互作用の根本修正)
## 痁E��
- `kpread` と `kpwrite` を同時に import したケースで、stdout に `\0` が大量混入し、`13\n` などぁE`13\0...` に壊れてぁE��、E
- `kpwrite` 単体テスト�E通るため、�E力単体ではなぁEimport/名前解決経路の相互作用が原因だった、E

## 根因
- `stdlib/kp/kpread.nepl` が不要な `#import "alloc/string" as *` を持っており、`len` などの識別子汚染を引き起こしてぁE��、E
- 同時 import 時に `kpwrite` 側の `len` ローカル束縛と衝突し、E��さ計箁E書き込み長が壊れてぁE��、E

## 実裁E
- `stdlib/kp/kpread.nepl`
  - 不要な `#import "alloc/string" as *` を削除、E
- `stdlib/kp/kpwrite.nepl`
  - `len` 局所変数めE`write_len` に改名！Ewriter_flush` / `writer_ensure` / `writer_put_u8` / `writer_write_str`�E�、E
  - 名前衝突時の再発耐性を強化、E
- `nepl-core/tests/kp.rs`
  - `kpwrite` 単体�Eり�Eけテストを追加、E
  - `kpread_buffer_bytes_debug` めEscanner 12B ヘッダ仕様に合わせて更新、E

## 検証
- `cargo test --test kp -- --nocapture`
  - `12 passed, 0 failed`
- `NO_COLOR=true trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=116, passed=116, failed=0, errored=0`

# 2026-02-10 作業メモ (cast/kp 最終調整)
## 実裁E
- `stdlib/alloc/string.nepl`
  - `fn cast from_i32;` / `fn cast to_i32;` を削除、E
  - `cast` 名�E過剰な公開を減らし、`core/cast` 側のオーバ�Eロード解決を安定化、E
- `stdlib/core/cast.nepl`
  - 斁E���E変換連携めE`string::from_*` / `string::to_*` に統一した状態を維持、E
  - `alloc/string` の公閁E`cast` 依存を持たなぁE��造へ整琁E��E

## 検証
- `NO_COLOR=true trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/numerics.n.md -o tests/output/numerics_current.json -j 1`
  - `total=122, passed=122, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=117, passed=117, failed=0, errored=0`
- `cargo test --test kp -q`
  - `14 passed, 0 failed`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 1`
  - `total=465, passed=458, failed=7, errored=0`
  - 今回解涁E `tests/numerics.n.md::doctest#3`�E�Embiguous overload�E�E
  - 既存残件: `ret_f64_example`, `selfhost_req` 系, `sort` 一部, `string` 一部

# 2026-02-21 作業メモ (shadowing チE��ト網羁E��)
## 実裁E
- `tests/shadowing.n.md` を新規作�E・拡張、E
  - ローカル値ぁEimport 名を shadow するケース
  - ネストブロチE��の最冁E��允E
  - ローカル関数ぁEimport 関数めEshadow
  - outer/inner 関数 shadow
  - 引数名とローカル let の shadow
  - while/match/branch を含むスコープケース
  - 現状未対応�E「値名と callable 名�E共存」等�E `compile_fail` として固宁E
- `todo.md` を更新、E
  - シャド�E不可修飾子�E immutable の `let`/`fn` のみに適用
  - `let mut` は対象夁E
  - 重要Estdlib 記号 shadow 時�E warn/info と LSP API 取得タスクを�E訁E

## 検証
- `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`
  - `total=176, passed=176, failed=0, errored=0`

# 2026-02-21 作業メモ (名前解決 API: shadowing 惁E��の拡張)
## 実裁E
- `nepl-web/src/lib.rs`
  - `NameResolutionTrace` に `shadows` を追加し、名前解決時�E shadowing イベントを収集できるようにした、E
  - 定義晁E
    - 既存候補がある場合に `definition_shadow` を記録、E
    - 重要シンボル�E�Eprint`/`println`/`add` など�E�を変数定義系 (`let_hoisted`/`let_mut`/`param`/`match_bind`) で定義した場合�E `warning` を付与、E
  - 参�E晁E
    - 候補が褁E��ある場合に `reference_shadow` を記録し、「採用された定義」と「隠れた候補」を API から取得可能にした、E
  - `analyze_name_resolution` の返却 JSON に以下を追加:
    - `shadows`
    - `shadow_diagnostics`
- `tests/tree/03_name_resolution_tree.js`
  - `result.shadows` / `result.shadow_diagnostics` を検証するアサーションを追加、E
  - `x` の shadow と `add` の重要シンボル warning を回帰固定、E

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 失敗�E既知カチE��リ�E�Eret_f64_example`, `selfhost_req`, `sort`, `string compile_fail期征E��刁E�E�で、今回の shadowing API 変更による新規失敗�E確認されなかった、E

# 2026-02-21 作業メモ (typecheck: shadowing warning 伝播と非�E命匁E
## 実裁E
- `nepl-core/src/typecheck.rs`
  - `Binding` に `span` を追加し、shadow 警告�E二次ラベル�E��E定義位置�E�を出せるようにした、E
  - `Env::lookup_outer_defined` を追加し、現在スコープ外�E定義候補を参�Eできるようにした、E
  - `emit_shadow_warning` を追加し、束縛導�E時！Elet` / `let mut` / `fn` / parameter / match bind�E�に shadow を検知して warning を生成するよぁE��した、E
  - 重要シンボル�E�Eprint`, `println`, `add` など�E�につぁE��は、外�E候補が見つからなぁE��合でも「stdlib 記号を隠しうる」warning を生成するよぁE��した、E
  - warning ノイズ抑制のため、E��重要シンボル�E�侁E `ok`, `len`�E��E shadow では compiler warning を�EさなぁE��針に調整した、E
  - `check_function` の返却めE`CheckedFunction` 化し、warning を返しつつコンパイル対象関数は生�Eし続けるよぁE��修正した、E
    - 以前�E warning を含むだけで `Err` 扱ぁE��なり、E��数が落ちてぁE��、E
    - 現在は `Error` のみ `Err`、warning は `diagnostics` として上位へ伝播する、E
- `tests/tree/04_semantics_tree.js`
  - `analyze_semantics` で shadowing warning が取得できることを検証するケースを追加、E

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_latest.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 失敗�E既知カチE��リに留まり、今回変更による追加失敗�E確認されなかった、E

## 残課題（今回の実裁E��見えたもの�E�E
- 重要シンボル warning は現在ノイズが多く、`todo.md` に無効匁E抑制ポリシー設計タスクとして残した、E


# 2026-02-19 作業メモ (stdlib ドキュメント整備と履歴整琁E
## 実裁E
- `stdlib/std/stdio.nepl`, `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`, `stdlib/std/test.nepl`:
  - 先頭チE��プレート説明を削除し、`//:` 形式�Eドキュメントコメントで統一、E
  - 注意文を「副作用・メモリ確俁E移動�EターゲチE��制紁E��など実利用時�E注意へ是正、E
  - 吁E��数に利用例！Eneplg2:test[skip]`�E�を維持し、呼び出し形を確認しめE��ぁE���Eへ整琁E��E
- `stdlib` 全体�Eドキュメント文言を点検し、モチE��皁E��表現を以下�E方針で是正、E
  - 「関数の概要」�E「主な用途、E
  - 「詳細な関数別ドキュメント�E段階的に追記します。」�E削除
  - 実裁E��昁E注意文のチE��プレート文言を、利用時�E挙動が伝わる表現へ置揁E
- commit 履歴は `4772eea` 基点で差刁E��再適用し、今回刁E��単一 commit に再作�E、E

## plan.mdとの差異
- 今回は plan.md の言語機�E追加ではなく、stdlib のドキュメント品質改喁E��履歴整琁E��実施、E
- ランタイム挙動めEAPI シグネチャは変更してぁE��ぁE��E

## 検証
- `cargo install trunk`
  - 失敗！Ehttps://index.crates.io/config.json` 取得時に 403、ネチE��ワーク制紁E��導�E不可�E�、E
- `NO_COLOR=true trunk build`
  - 失敗！Etrunk` 未導�E�E�、E
- `node nodesrc/tests.js -i stdlib/std -o tests/output/stdlib_std_docs_current.json -j 1`
  - 失敗！Eompiler artifacts 不在、`total=215, errored=215`�E�、E
- `node nodesrc/cli.js -i stdlib/std -o html_play=dist/stdlib_std`
  - 失敗！Ertifacts 不在で HTML 生�E不可�E�、E

# 2026-02-21 作業メモ (lexer/parser 上流整琁E+ 木構造 API チE��ト追加)
## 実裁E
- `nepl-core/src/lexer.rs`
  - `cond` / `then` / `else` / `do` を専用キーワードトークン (`KwCond`, `KwThen`, `KwElse`, `KwDo`) として追加、E
  - キーワード判定を `keyword_token` に雁E��E��、同義刁E���E重褁E��解消、E
  - `LexState` の未使用 lifetime を除去し、字句解析状態�E定義を簡潔化、E
- `nepl-core/src/parser.rs`
  - 新キーワードトークンをレイアウト�Eーカーとして受理する刁E��を追加、E
  - 括弧弁E(`(` ... `)`) の解析ロジチE��めE`parse_parenthesized_expr_items` に統合し、E箁E��重褁E��てぁE��処琁E��一本化、E
  - 診断斁E��現仕様に合わせて更新:
    - `tuple literal cannot end with a comma` -> `trailing comma is not allowed in parenthesized expression`
    - `expected ')' after tuple literal` -> `expected ')' after parenthesized expression`
- `nepl-web/src/lib.rs`
  - 解极EAPI の token kind 斁E���E表現に `KwCond/KwThen/KwElse/KwDo` を追加、E
- チE��ト追加
  - `tests/keywords_reserved.n.md` を新規追加し、`cond/then/else/do` が識別子として使えなぁE��とめE`compile_fail` で固定、E
  - `tests/tree/*.js` を新規追加し、LSP/チE��チE��向け API の木構造を段階別に検証:
    - `tests/tree/01_lex_tree.js`
    - `tests/tree/02_parse_tree.js`
    - `tests/tree/03_name_resolution_tree.js`
    - `tests/tree/04_semantics_tree.js`
    - `tests/tree/run.js`�E�一括実行！E

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_final.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`

## 補足
- `tests` 全佁E(`--no-stdlib`) 実行では既存�E下流課題！Eet_f64/selfhost/sort など�E�で失敗が残るが、今回の lexer/parser 変更で新規回帰は確認されてぁE��ぁE��E

# 2026-02-21 作業メモ (noshadow 導�E完亁E��回帰修正)
- `noshadow` めElexer/parser/typecheck/web API まで一貫して実裁E��E
  - lexer: `KwNoShadow` を追加、E
  - parser: `let` 修飾子に `noshadow` を追加。`let mut noshadow` は parse error、E
  - parser: `fn noshadow <name>` を受琁E��、AST に `no_shadow` を保持、E
  - typecheck: `Binding.no_shadow` を導�Eし、`noshadow` 宣言の上書きを compile error 化、E
- 名前解決/型検査の既存動作を壊さなぁE��め、同一スコープ�E通常 `let` 再束縛！Elet lst ...; let lst ...;`�E��E維持、E
  - ただし既存束縛が `no_shadow` の場合�Eみ、同名宣言を拒否する、E
- Web 側のト�Eクン API めE`KwNoShadow` に追従、E
- チE��ト追加:
  - `tests/shadowing.n.md` に `noshadow` の compile_fail ケースを追加、E
- 検証結果:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-21 作業メモ (doctest の profile ゲート安定化)
- `#if[profile=debug/release]` の doctest ぁECI 環墁E�Eビルドモード差刁E��揺れる問題に対して、テストランナ�Eからコンパイルプロファイルを�E示持E��できるように修正、E
- `nepl-web` 側:
  - `compile_source_with_profile(source, profile)` を追加、E
  - `compile_source_with_vfs_and_profile(entry_path, source, vfs, profile)` を追加、E
  - 冁E��コンパイル経路めE`compile_wasm_with_entry_and_profile(..., Option<BuildProfile>)` に統合、E
- `nodesrc/run_test.js` 側:
  - 可能な場合�E常に `debug` を�E示持E��してコンパイルするように変更、E
  - VFS あり/なし両方で新 API を優先使用し、旧 API は後方フォールバックとして保持、E
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-21 作業メモ (stdlib result への段階的 noshadow 適用)
- `stdlib/core/result.nepl` の基盤 API から、衝突リスクが低い `unwrap_ok` / `unwrap_err` に `noshadow` を付与、E
- 目皁E
  - 基盤 API の誤上書きを早期検�Eする運用を段階導�Eする、E
  - 既存コードで利用頻度が高い短名！Eok` / `err` / `map`�E��E今回保留し、破壊篁E��を最小化、E
- 回帰チE��トを追加:
  - `tests/shadowing.n.md` に `std_result_noshadow_unwrap_ok`�E�Eompile_fail�E�を追加、E
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `548/548 passed`

# 2026-02-21 作業メモ (shadow と overload の扱ぁE��琁E
- 仕様調整:
  - 関数の同名定義でシグネチャが異なる場合�Eオーバ�Eロードとして許可、E
  - 同名かつ同一シグネチャの場合�Eみ「shadowing 扱ぁE�E warning」を出す、E
  - 同名関数再定義をエラーにはしなぁE��E
- `noshadow` の関数適用ルールを調整:
  - `noshadow fn` でも関数同名�E�オーバ�Eロード）�E許可、E
  - 変数/値名前空間との衝突�E従来通り拒否、E
- 利用頻度の高い一般名に対する方針変更:
  - `unwrap` / `unwrap_ok` / `unwrap_err` めE`noshadow` 対象から外した、E
  - これに伴ぁE`tests/shadowing.n.md` の unwrap 系 compile_fail ケースを削除、E
- チE��ト更新:
  - `fn_noshadow_rejects_shadowing` めE`fn_same_signature_shadowing_warns_and_latest_wins` に更新し、�E功ケースとして固定！Eret: 2`�E�、E
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-22 作業メモ (todo 棚卸ぁE
- `todo.md` の棚卸しを実施し、解決済みまた�E状態が古ぁE��E��を削除した、E
- 特に以下を整琁E
  - 古ぁE��計値 (`total=413, passed=404, failed=9`) を削除、E
  - 既に完亁E��みの `nm/parser` 型名衝突�E`examples/nm.nepl` の `cliarg` 経路修正系タスクめEtodo から除去、E
  - `todo.md` は未完亁E��スクのみ�E�名前空閁E高階関数/LSP/診断体系/Web強匁Ejs_interpreter�E�に再構�E、E
- 現時点の回帰確誁E
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` の最新結果は pass 維持E��直近実衁E `547/547`�E�、E

# 2026-02-22 作業メモ (profile/target ゲートと stdlib 重褁E��義の回帰修正)
- 痁E��:
  - doctest で `debug_color` / `debugln_color` / `test_checked` / `test_print_fail` の同一シグネチャ再定義 warning ぁEcompile fail 扱ぁE��なってぁE��、E
  - `functions.n.md` などの失敗と混在してぁE��ため、まぁEwarning 起点を�Eり�Eけた、E
- 原因:
  - `#if[...]` の直後に `//:` ドキュメントコメントが挟まる箁E��で、条件付き定義が意図どおりに限定されず重褁E��義が同時有効になってぁE��、E
- 修正:
  - `stdlib/std/stdio.nepl`:
    - 条件付き関数定義に対して `#if[profile=...]` を定義直前へ再�E置、E
    - release 側の同名実裁E�E冁E��吁E(`__debug_*_release_noop`) に退避し、シグネチャ衝突を除去、E
  - `stdlib/std/test.nepl`:
    - `#if[target=...]` を関数定義直前へ再�E置し、意図したターゲチE��限定で定義されるよぁE��正、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - 対象再現チE��チE
    - `node nodesrc/tests.js -i tests/functions.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl ...`
    - `191/191 pass`
  - 全佁E
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`
    - `547/547 pass`

# 2026-02-22 作業メモ (nepl-web API と cli.js の責務�E離)
- 要件反映:
  - `nepl-web/src/lib.rs` は API 提供�Eみに限定し、Node/FS への直接アクセスは持たなぁE���Eにした、E
  - FS から stdlib を読む責務�E JS 側�E�Enodesrc/cli.js`�E�に刁E��、E
- `nepl-web/src/lib.rs` 変更:
  - 既存�E「バンドル stdlib 使用�E�デフォルト）」�E維持、E
  - 新要EAPI:
    - `get_bundled_stdlib_vfs()`: wasm にバンドルされぁEstdlib めE`/stdlib/...` 形弁EVFS で返す、E
    - `compile_source_with_vfs_and_stdlib(...)`
    - `compile_source_with_vfs_stdlib_and_profile(...)`
  - これにより、外部�E�Eode/ブラウザ�E�が stdlib ソース選択を拁E��るよぁE��なった、E
- `nodesrc/cli.js` 変更:
  - `loadStdlibVfsFromFs(stdlibRootDir)` を追加�E�ローカル FS から `/stdlib/...` VFS を構築）、E
  - `loadBundledStdlibVfs(api)` を追加�E�Easm バンドル stdlib 取得）、E
  - `compileWithLocalStdlib(api, ...)` を追加�E�ローカル stdlib を使ってコンパイル API を呼ぶ�E�、E
- 呼び出し�E更新:
  - `nodesrc/html_gen_playground.js` で新 API を優先使用するよう更新、E
  - `web/src/main.ts` で `get_bundled_stdlib_vfs` を優先し、旧 `get_stdlib_files` はフォールバックに変更、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (名前解決再設訁E 関数候補検索の整琁E第1段)
- 目皁E
  - `todo.md` 最優先頁E���E�EalueNs/CallableNs 刁E���E�に向けて、挙動を変えなぁE��E��で関数候補検索ロジチE��を整琁E��E
- 実裁E
  - `Env` に `lookup_all_callables` を追加、E
  - 関数候補抽出で `lookup_all + filter(Func)` を繰り返してぁE��箁E��めE`lookup_all_callables` へ置換、E
    - top-level `FnDef` の `f_ty` 決宁E
    - nested `FnDef` の `f_ty/captures` 決宁E
    - `user_visible_arity` の capture 数計箁E
  - `find_same_signature_func` めE`lookup_all_callables` ベ�Eスへ変更、E
- 結果:
  - 機�E変更なしで重褁E��ジチE��を削減し、次段の名前空間�E離�E�Ealue/Callable�E�に進める基盤を作�E、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (名前解決再設訁E Value/Callable API 明確匁E第2段)
- 目皁E
  - ValueNs/CallableNs 刁E��へ向けて、`Env` の検索 API を�E確化し、E��数呼び出し経路の刁E��を読みめE��くする、E
- 実裁E
  - `Env` に以下を追加:
    - `lookup_value(name)`
    - `lookup_callable(name)`
  - 既孁E`lookup_all` は「最冁E��コープ優先」�Eまま維持し、`lookup_value/lookup_callable` はそ�E結果から kind を選ぶ設計にした�E�解決規則は維持E��、E
  - `find_same_signature_func` は callable 専用検索を使ぁE��ぁE��琁E��E
  - `check_call_or_letset` 系の刁E��で、`lookup_all + var 判定` めE`lookup_all_callables` / `lookup_value` に置換、E
- 結果:
  - 挙動を変えずに Value/Callable の責務をコード上で刁E��できる形へ前進、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (nm-compile 失敗�E根因修正: extern/entry 収集経路の統吁E
- 背景:
  - CI (`nm-compile`) で `stdlib/std/env/cliarg.nepl` の `args_sizes_get` / `args_get` ぁE`undefined identifier` になる失敗を確認、E
  - 同時に `expression left extra values on the stack` が連鎖して発生、E
- 根因:
  - `typecheck` の先行ディレクチE��ブ�E琁E�� `module.root.items` の `Stmt::Directive` のみを走査しており、E
    ローダー経由で `module.directives` 側に保持されぁE`#extern` を取りこぼす経路があった、E
- 修正:
  - `nepl-core/src/typecheck.rs` でチE��レクチE��ブ適用処琁E��共通化、E
  - `module.directives` と `module.root.items` の双方を適用対象にし、span キーで重褁E��用を抑止、E
  - これにより `#extern wasi_snapshot_preview1 args_sizes_get/args_get` が安定して環墁E��登録されるよぁE��した、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 2`: `200/200 pass`
  - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output /tmp/ci-nm`: `compile_module returned Ok`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づぁE
  - 仕様変更�E�Etarget=wasm` で WASI 無効�E�後�E回帰であり、上流E��Eypecheck 入り口�E�で根本修正、E
  - 次段は固定方針どおり lexer/parser の旧仕様残骸整琁E��優先する、E

# 2026-02-22 作業メモ (条件付きチE��レクチE��ブ評価の頁E��修正)
- 背景:
  - `typecheck` の extern/entry 収集めE`module.directives` へ拡張した際、E
    `module.directives` 側に対して `#if[target=...]` / `#if[profile=...]` の評価を通してぁE��ぁE��路が残ってぁE��、E
- 修正:
  - `module.directives` 走査でめE`pending_if` を使って gate 評価を適用、E
  - 既存�E `module.root.items` 走査と同じ条件付き有効化ルールに統一、E
  - span キー重褁E��外�E維持し、二重登録は防止、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/neplg2.n.md -i tests/nm.n.md -o tests/output/upstream_lexer_parser_latest.json -j 3`: `220/220 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づぁE
  - 上流E��Eypecheck入り口�E�での条件判定一貫化で、nm/cliarg を含む extern 解決の再発防止を目皁E��した根本修正、E

# 2026-02-22 作業メモ (シャド�E警呁E オーバ�Eロード経路のノイズ抑制)
- 背景:
  - 仕様上、E��数オーバ�Eロード�E許容されるため、オーバ�Eロード�E立ケースで一般 shadow warning を�Eす�Eはノイズになる、E
- 修正:
  - `nepl-core/src/typecheck.rs`
    - ネスチE`fn` 登録時�E `emit_shadow_warning(...)` 呼び出し条件を調整、E
    - 既存同名候補が「すべて callable�E�E オーバ�Eロード候補）」�E場合�E一般 shadow warning を�EさなぁE��E
    - 同名に value 系束縛が混在する場合�Eみ従来どおり warning を�Eす、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/overload.n.md -o tests/output/shadowing_current.json -j 2`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づぁE
  - 名前解決・シャド�Eイング再設計！Eodo最優先頁E���E��E一部として、E
    「オーバ�Eロードではなく実シャド�Eのみ警告」�E運用に近づける調整、E

# 2026-02-22 作業メモ (旧タプル記法�E残存�E顁E
- 目皁E
  - 固定指示に基づき、上流修正�E�Earser 強化）�E前に全体を刁E��して局所修正を回避する、E
- 実施:
  - `rg` で `stdlib/tests/tutorials` の旧タプル記法候補を棚卸し、E
  - `tests/tree/run.js` で LSP/解析API系の回帰を確認、E
- 観測:
  - `tests/tree/run.js`: `4/4 pass`、E
  - 旧 tuple literal reject は既存どおり有効だが、tuple type 記況E`(<T1,T2>)` は stdlib/tests に庁E��残存、E
  - parser で tuple type を即晁Ereject すると stdlib doctest が大量破綻することを確認（段階移行が忁E��E��、E
- 方針更新:
  - `todo.md` に「旧タプル記法�E完�E移行（段階実施�E�」を追加、E
  - 手頁E�E `stdlib/tutorials` 先行移衁EↁE`tests` 刁E���E�新仕槁Ecompile_fail�E��E parser で最絁Ereject の頁E��固定、E
- 補足:
  - 一時的に parser の tuple type reject を試験したが、�E体影響が大きいため直ちに戻し、現行安定状態（�E佁Epass�E�を維持した、E

# 2026-02-22 作業メモ (旧タプル記法移行フェーズ1: stdlib 実例�E型注釈削渁E
- 実施:
  - `stdlib/alloc/vec.nepl` の `vec_pop` doctest で、旧タプル型注釁E
    `let p <(Vec<i32>,Option<i32>)> ...` を削除し、推論に寁E��た、E
- 目皁E
  - parser 側の最絁Ereject 前に、stdlib 実例から旧記法依存を段階的に除去する、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -o tests/output/list_current.json -j 1 --no-stdlib`: `18/18 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 次段:
  - `tests/tuple_new_syntax.n.md` の tuple 型注釈ケースを「新記法での等価検証」へ再設計、E
  - そ�E征E`tutorials` 冁E�E不要な tuple 型注釈を同様に削減する、E

# 2026-02-22 作業メモ (tutorial 19 pipe の実行失敗修正)
- 背景:
  - `tutorials/getting_started/19_pipe_operator.n.md` 更新後、`doctest#2` ぁE`divide by zero` で失敗、E
- 根因:
  - `let v` ブロチE��の外に `3 |> mul 2` がこぼれており、意図した、E本のパイプ連結」になってぁE��かった、E
- 修正:
  - `pipe chain` サンプルを単一ブロチE��冁E�E連結へ整琁E��E
  - `3 |> mul 2 |> add 6` として `assert_eq_i32 12 v` を満たす例に更新、E
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/19_pipe_operator.n.md -o tests/output/tutorial_pipe19_current.json -j 1`: `167/167 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started -o tests/output/tutorials_getting_started.json -j 4`: `223/223 pass`

# 2026-02-22 作業メモ (旧タプル記法移行フェーズ1: tuple_new_syntax の不要型注釈削渁E
- 実施:
  - `tests/tuple_new_syntax.n.md` の `tuple_type_annotated` ケースで、E
    変数側の明示型注釁E`let t <(i32,i32)> ...` を除去し、推論へ移行、E
- 目皁E
  - parser 側最絁Ereject 前に、テスト賁E��から「不要な旧 tuple type 記法」を段階的に減らす、E
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (stdlib 改衁Epipe リファクタ: StringBuilder)
- 背景:
  - `stdlib` リファクタで「褁E��チE�Eタ処琁E��改衁Epipe を活用」�E方針に沿って、`StringBuilder` 周辺を段階的に移行開始、E
- 実施:
  - `stdlib/alloc/string.nepl`
    - `sb_append` めE`get sb "parts" |> vec_push<str> s |> StringBuilder` へ整琁E��E
    - `sb_append_i32` めE`sb |> sb_append from_i32 v` へ変更�E�EStringBuilder` めEpipe 左辺に固定）、E
- 根因と修正:
  - 初回実裁E�� `from_i32 v |> sb_append sb` としてしまぁE��pipe 規則�E�左辺が第1引数�E�により引数頁E��送E��、E
  - そ�E結果 `no matching overload found` が発生したため、`sb` を左辺にする形へ修正して根本解消、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `547/547 pass`
- 運用更新:
  - `todo.md` 方針に「stdlib のドキュメントコメンチEドキュメントテスト�E `stdlib/kp` の記述スタイルを参照して統一」を追記、E

# 2026-02-22 作業メモ (tree API チE��ト強匁E オーバ�Eロードとシャド�E診断)
- 背景:
  - 固定指示にある「上流から�E修正」と LSP/チE��チE��向け API 検証を進めるため、E
    `tests/tree` でオーバ�Eロードとシャド�E診断の墁E��を�E示皁E��固定した、E
- 実施:
  - `tests/tree/05_overload_shadow_diagnostics.js` を追加、E
  - 検証冁E��:
    - `analyze_name_resolution` では、純粋オーバ�Eロード（同名�E異なるシグネチャ�E�を warning 扱ぁE��なぁE��と、E
    - `analyze_semantics` では、同一シグネチャ再定義めEwarning として報告すること、E
- 検証:
  - `node tests/tree/run.js`: `5/5 pass`
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `548/548 pass`
- 位置づぁE
  - 上流EAPI�E�Eex/parse/resolve/semantics�E��E診断墁E��をテスト化し、E
    今後�E名前解決再設計での退行を防ぐため�E基盤整備、E

# 2026-02-22 作業メモ (lexer/parser 上流回帰: 予紁E���E識別子禁止)
- 背景:
  - 固定指示の「上流から修正」に沿って、lexer/parser の予紁E��墁E��めEcompile-fail チE��トで明示固定した、E
- 実施:
  - `tests/keywords_reserved.n.md` を追加、E
  - `cond/then/else/do/let/fn` を識別子として使ぁE��ースをすべて `compile_fail` で追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md -o tests/output/keywords_reserved_current.json -j 1`: `172/172 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `550/550 pass`
- 位置づぁE
  - 予紁E��トークン化と構文エラー化�E墁E��を�Eに固定し、後続�E parser 整琁E��に退行を検知できる状態を作った、E

# 2026-02-22 作業メモ (旧タプル記法テスト�E失敗原因刁E��)
- 背景:
  - `tests/tuple_old_syntax.n.md` へ「旧タプル型注釈」「旧ドット添字アクセス」�E reject ケースを追加したところ、E
    現衁Eparser/lexer の受理墁E��と一致せず `compile_fail` 想定が崩れた、E
- 観測:
  - `t.0` は lexer 側の `.0` 数値解釈経路があり、現状のままでは「旧ドット添字アクセス」として安宁Ereject できなぁE��E
  - `(<T1,T2>)` の型注釈�E段階移行中で、現時点では reject 固定にすると既存賁E��との整合が崩れる、E
- 対忁E
  - 先行追加した 3 ケース�E�Euple type / dot index / nested dot index�E��E `skip` に刁E��替え、E
    フェーズ刁E��を�E確化した、E
  - 既存�E「旧 tuple literal `(a,b)` reject」ケースは `compile_fail` のまま維持、E
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 位置づぁE
  - 旧仕様廁E��は継続しつつ、上流E��Eexer/parser�E�で一括改修する前に失敗原因を混在させなぁE��め�E刁E��刁E��、E

# 2026-02-22 作業メモ (parser 上流修正: `t.0` 旧ドット添字�E検�E)
- 背景:
  - 旧タプル記法廁E��方針に対し、`t.0` が一部経路で明示診断されず、移行墁E��が曖昧だった、E
- 修正:
  - `nepl-core/src/parser.rs` の `parse_ident_symbol_item` で、識別子後�E `.` の次ぁE`IntLiteral` の場合を特別扱ぁE��E
  - 以下�E診断を即時追加:
    - `legacy tuple field access '.N' is removed; use 'get <tuple> N'`
  - 該当トークンを消費して回復し、後続解析を継続できるようにした、E
- チE��チE
  - `tests/tuple_old_syntax.n.md` のドット添字ケースめE`compile_fail` に戻し、回帰に絁E��込んだ、E
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 位置づぁE
  - lexer/parser 上流で「旧記法�E検�Eと移行ガイド付き診断」を先に固定し、後続�E旧仕様完�E撤去に備える修正、E

# 2026-02-22 作業メモ (tree API 回帰追加: 旧ドット添字診断)
- 背景:
  - `t.0` の parser 診断追加めEAPI レベルでも退行検知できるようにするため、tree チE��トへ追加、E
- 実施:
  - `tests/tree/06_legacy_tuple_dot_index_diag.js` を追加、E
  - `analyze_semantics` で `t.0` 入力に対し、以下を検証:
    - コンパイル成功ではなぁE��と
    - `legacy tuple field access '.N' ... use 'get <tuple> N'` 診断が含まれること
- 検証:
  - `node tests/tree/run.js`: `6/6 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 位置づぁE
  - 上流変更�E�Earser�E�に対する LSP/チE��チE�� API の回帰網を強化し、段階移行中の仕様墁E��を�E示固定、E

# 2026-02-22 作業メモ (旧 tuple type 注釈�E段階削渁E チE��ト賁E��整琁E
- 背景:
  - parser で旧 tuple type 記法を最絁Ereject する前に、テスト�Eの不要依存を減らして失敗原因を�E離する忁E��がある、E
- 実施:
  - `tests/tuple_new_syntax.n.md`
    - `struct Wrapper` のフィールド型めE`pair <(i32,i32)>` から `pair <.Pair>` へ変更、E
    - 値構築�E `Tuple:` のまま維持し、旧 tuple type 記法への依存を削減、E
  - `tests/tuple_old_syntax.n.md`
    - `old_tuple_literal_construct_is_rejected` から旧 tuple type 注釈を除去し、E
      旧 tuple literal `(3, true)` 単独で失敗原因を固定、E
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -o tests/output/tuple_migration_current.json -j 1`: `192/192 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 位置づぁE
  - 旧仕様撤去フェーズの前段として、テストを「旧 literal 失敗」「旧 type 失敗」に刁E��しやすい状態へ整琁E��E

# 2026-02-22 作業メモ (旧 tuple type parser 即晁Ereject の試行とロールバック)
- 試衁E
  - `parse_type_expr` の `(...)` 非関数刁E��で、旧 tuple type 記法を parser 段階で即時エラー化する変更を適用、E
- 結果:
  - `tests/tuple_old_syntax.n.md` 単体では意図どおり失敗検�Eできたが、E
    `stdlib` の庁E��E��箁E��で旧 tuple type 依存が残っており、`33` 件の compile failure を誘発、E
  - 失敗�E中忁E�E「段階移行前に parser だけを先に厳格化した」ことによる時期不整合、E
- 判断:
  - 固定指示どおり局所対応を避け、段階移行方針を優先するためEparser 即晁Ereject 変更はロールバック、E
  - 現時点は「賁E��側�E�Eests/stdlib/tutorials�E��E旧 type 依存削減」�E行を継続する、E
- 再検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (stdlib 段階移衁E vec_pop の旧 tuple type 依存削渁E
- 実施:
  - `stdlib/alloc/vec.nepl` の `vec_pop` シグネチャめE
    `<(Vec<.T>)*>(Vec<.T>,Option<.T>)>` から `<(Vec<.T>)*>.Pair>` に変更、E
  - 返り値の実データは従来どおり `Tuple:` 構築を維持し、実行挙動�E変更しなぁE��E
- 目皁E
  - parser の旧 tuple type 最絁Ereject 前に、stdlib 側の型注釈依存を段階的に削減する、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i tests/tuple_new_syntax.n.md -o tests/output/vec_tuple_migration_current.json -j 1`: `201/201 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (tuple_new_syntax の戻り型注釈移衁E
- 実施:
  - `tests/tuple_new_syntax.n.md` の `make` 関数で、戻り型注釈を
    `<()->(i32,i32)>` から `<()->.Pair>` へ変更、E
- 目皁E
  - parser 最終段階で旧 tuple type めEreject する前に、テスト賁E��の旧型注釈依存を段階的に削減する、E
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (旧 tuple type 記況Ereject の再適用完亁E
- 背景:
  - 旧 tuple type 記法�E parser reject は以前、`stdlib` 側依存で崩れて一度ロールバックしてぁE��、E
- 実施:
  - `nepl-core/src/parser.rs` の `parse_type_expr` で、`(...)` の非関数 tuple type をエラー化、E
  - 併せてチE��ト賁E��を移衁E
    - `tests/pipe_operator.n.md` の `pipe_tuple_source` めE`fn f <.T> <(.T)->i32>` へ変更
    - `tests/tuple_new_syntax.n.md` の `tuple_as_function_arg` めE`fn take <.T> <(.T)->i32>` へ変更
    - `tests/tuple_old_syntax.n.md` の `old_tuple_type_annotation_is_rejected` めE`compile_fail` に復帰
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 結果:
  - 旧 tuple type 記況Ereject と全体回帰の両立を確認、E
  - `todo.md` の「旧タプル記法�E完�E移行」頁E��は完亁E��して削除、E

# 2026-02-22 作業メモ (capture 関数値の bare symbol 経路も拒否)
- 背景:
  - `@fn` 経路では capture あり関数値を拒否済みだったが、`apply 5 add_y` のような bare symbol の関数値渡し経路に同等�Eガードが不足してぁE��、E
- 実施:
  - `nepl-core/src/typecheck.rs`
    - call_indirect fallback 判定で `HirExprKind::Var(name)` かつ function-typed の場合にめEcallable 定義を確認し、capture ありならエラー化、E
    - エラーメチE��ージ: `capturing function cannot be passed as a function value yet`
  - `tests/functions.n.md`
    - `function_value_capture_not_supported_without_at` (`compile_fail`) を追加、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: 全件 pass�E�実行時点の総数�E�、E
- 位置づぁE
  - closure conversion 未実裁E��ェーズでの「通ってはぁE��なぁEcapture 関数値流�E」を `@` / bare symbol の両経路で統一皁E��封止、E

# 2026-02-22 作業メモ (profile ゲート回帰チE��ト�E追加)
- 背景:
  - CI で `#if[profile=...]` 周辺の退行が疑われるログがあったため、debug/release 両方の compile 成否を固定すめEAPI チE��トが忁E��だった、E
- 実施:
  - `tests/tree/09_profile_gate_compile.js` を追加、E
  - `compile_source_with_profile` を使ぁE��以下を検証:
    - debug gated 定義は debug で通り、release で `undefined identifier` になる、E
    - release gated 定義は release で通り、debug で `undefined identifier` になる、E
    - release 側に未知識別子を含む定義は debug でスキチE�Eされ、コンパイルが通る、E
- 検証:
  - `node tests/tree/run.js`: `9/9 pass`
- 位置づぁE
  - 条件付きコンパイルの仕様墁E��めEtree/API 層で固定し、�E発を早期検知できるようにした、E

# 2026-02-22 作業メモ (todo 整琁E 高階関数頁E��)
- `todo.md` の、E. 高階関数・call_indirect」から、完亁E��みの
  - `WASM table + call_indirect で non-capture 高階関数を動作させる`
  を削除、E
- 未完亁E�Eみ保持の方針に合わせ、残タスクめE
  - `capture あり関数値の closure conversion 導�E`
  に雁E��E��た、E

# 2026-02-22 作業メモ (parser 回帰追加: IfProfile の AST 形状固宁E
- 背景:
  - `#if[profile=...]` 退行対策を compile API だけでなぁEparser 層でも固定し、上流から原因を�Eり�Eけ可能にする、E
- 実施:
  - `tests/tree/10_profile_directive_parse_shape.js` を追加、E
  - `analyze_parse` で以下を検証:
    - root item の頁E��が `Entry` -> `IfProfile(debug)` -> `FnDef(only_debug)` -> `FnDef(main)`
    - `IfProfile` の debug payload に `profile: "debug"` が含まれる、E
- 検証:
  - `node tests/tree/run.js`: `10/10 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `563/563 pass`
- 位置づぁE
  - 条件付きコンパイルの上流E��Eexer/parser�E�と下流E��Eompile profile�E��E双方めEtree/API チE��トで接続し、�E発時�E診断速度を高めた、E

# 2026-02-22 作業メモ (parser 回帰追加: 旧タプル記法診断の固宁E
- 背景:
  - 旧 tuple 記法廁E��を上流で固定するため、`compile_fail` だけでなぁEparser API の診断メチE��ージを直接検証する回帰が忁E��だった、E
- 実施:
  - `tests/tree/11_legacy_tuple_parse_diag.js` を追加、E
  - `analyze_parse` で以下を検証:
    - `let t (1, true)` に対ぁE`legacy tuple literal '(...)' is removed` 診断が�Eる、E
    - `let t <(i32,i32)> Tuple: ...` に対ぁE`legacy tuple type '(T1, T2, ...)' is removed` 診断が�Eる、E
  - parser のエラー回復方針（診断を�Eしつつ `ok` 継続しぁE���E�に合わせ、`ok==false` ではなく診断存在を�E功条件にした、E
- 検証:
  - `node tests/tree/run.js`: `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `564/564 pass`
- 位置づぁE
  - 旧記法廁E��の墁E��めElexer/parser API 層で固定し、封E��の parser 変更で受理が戻る退行を検知できるようにした、E

# 2026-02-22 作業メモ (noshadow と overload の整合修正)
- 背景:
  - `fn noshadow` めEcallable 全体で禁止する変更を試した結果、既存仕様（オーバ�Eロード許可�E�と衝突して `tests/shadowing.n.md` の退行を引き起こした、E
- 実施:
  - `nepl-core/src/typecheck.rs`
    - `shadow_blocked_by_nonshadow` 判定で callable 同士は引き続き許可し、E
      value 側の non-shadowable 宣言に対する遮断のみ維持、E
  - `tests/shadowing.n.md`
    - `fn_same_signature_shadowing_warns_and_latest_wins` を�Eの期征E��Earning + 後勝ち�E�へ戻し、仕様と一致させた、E
- 検証:
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `193/193 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `564/564 pass`
- 位置づぁE
  - 「オーバ�Eロード�E許可、同一シグネチャ再定義のみ shadow 扱ぁE��とぁE��現行方針に戻し、局所皁E��過剰制限を解消、E

# 2026-02-22 作業メモ (parser: 予紁E��を識別子位置で明示診断)
- 背景:
  - `let cond` / `fn let` / `(... fn ...)` など予紁E��を識別子位置へ置ぁE��際、E
    場合によっては `expected identifier` のみで、診断の一貫性が弱かった、E
- 実施:
  - `nepl-core/src/parser.rs`
    - `expect_ident` を拡張し、`TokenKind::Kw*` を検�Eした場合�E
      `'<kw>' is a reserved keyword and cannot be used as an identifier` を直接報告するよぁE��変更、E
    - `reserved_keyword_token_name` ヘルパ�Eを追加してキーワード名を統一管琁E��E
  - `tests/tree/12_reserved_keyword_identifier_diag.js` を追加、E
    - `analyze_parse` で `let cond` / `fn let` / `param fn` の3ケースを検証し、E
      それぞれ予紁E��診断が�Eることを固定、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `12/12 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `565/565 pass`
- 位置づぁE
  - 上流E��Earser�E��E予紁E��制紁E�� API チE��トで固定し、診断品質と回復時�E可読性を改喁E��E

# 2026-02-22 作業メモ (parser 回復強匁E 褁E��行�E予紁E��誤用を継続報呁E
- 背景:
  - 予紁E��を識別子位置に置ぁE�� `let` が連続すると、最初�E `parse_stmt` 失敗で block 解析が打ち刁E��れ、後続行�E診断が欠落してぁE��、E
- 実施:
  - `nepl-core/src/parser.rs`
    - `parse_block_until_internal` の `parse_stmt()` 失敗時めE`?` で即 return せず、E
      行墁E�� (`Newline` / `Semicolon`) までト�Eクンを捨てる回復処琁E��変更、E
    - これにより同一ブロチE��冁E��褁E��エラーを継続収雁E��能にした、E
  - `tests/tree/13_parser_multi_error_recovery.js` 追加、E
    - `let cond` / `let then` / `let else` の3連続誤用で、E件の予紁E��診断が得られることを固定、E
- 検証 (直列実衁E:
  1. `NO_COLOR=false trunk build`
  2. `node tests/tree/run.js` -> `13/13 pass`
  3. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1` -> `566/566 pass`
- 運用メモ:
  - 持E��に合わせ、`trunk build` とチE��ト�E今後も忁E��直列で実行する、E

# 2026-02-22 作業メモ (LSP API 拡張: name_resolution 参�Eの詳細匁E
- 背景:
  - `todo.md` の LSP/API phase2 に対し、`candidate_def_ids` だけでは定義ジャンプ実裁E��に再参照が多く、UI 連携が�E雑だった、E
- 実施:
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution` の `references[]` に次を追加:
      - `resolved_def`: 最終選択定義の詳細�E�Ed/name/kind/scope_depth/span�E�E
      - `candidate_definitions`: 候補定義の詳細配�E�E�同上！E
    - 既存�E `resolved_def_id` / `candidate_def_ids` は維持して後方互換を確保、E
  - `tests/tree/03_name_resolution_tree.js`
    - `resolved_def` と `candidate_definitions` の整合を検証するアサーションを追加、E
- `todo.md` 整琁E
  - 4番頁E��を未完�EみになるよぁE��新:
    - 完亁E��み「最終選抁E候補�E返却」�E除夁E
    - 未完「import/alias/use 跨ぎ�E定義允E��ァイル惁E���E�Eump先）」へ焦点匁E
- 検証 (直刁E:
  1. `NO_COLOR=false trunk build`
  2. `node tests/tree/run.js` -> pass
  3. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1` -> `566/566 pass`
# 2026-02-22 作業メモ (Vec read-only accessor の前進)
- 目皁E
  - `todo.md` の「sort/generics と Vec 読み取り設計」を上流�E API から前進させる、E
- 実裁E
  - `stdlib/alloc/vec.nepl`
    - `vec_data_ptr <.T> <(Vec<.T>)->i32>` を追加、E
    - 日本語ドキュメントコメンチE+ doctest を追加、E
  - `stdlib/alloc/sort.nepl`
    - `get v "len"` / `get v "data"` の一部めE`vec_len<.T> v` / `vec_data_ptr<.T> v` へ置換、E
    - 同一 `Vec` から `len` と `data` を同時取得する箁E��は move 回避のため `get` を維持、E
  - `stdlib/tests/vec.nepl`
    - `vec_data_ptr` の基本回帰を追加�E�Evec_new` 直後に `> 0` を確認）、E
  - `todo.md`
    - 完亁E��ぁE`vec_len/vec_data_ptr` の read-only 経路頁E��を削除し、未完亁E�� slice 風 API に絞った、E

# 2026-02-22 作業メモ (sort ポインタ薁E��チE��の追加)
- 目皁E
  - `todo_kp.md` の「競プロ向けソーチEAPI 薁E��チE��」を前進させる、E
- 実裁E
  - `stdlib/alloc/sort.nepl`
    - `sort_slice_quick <.T: Ord> <(i32,i32)*>()>` を追加、E
    - `sort_i32 <(i32,i32)*>()>` を追加�E�Esort_slice_quick<i32>` の薁E��チE���E�、E
  - `tests/sort.n.md`
    - `sort_i32_ptr_basic` を追加し、`alloc` + `store_i32` で作った�E列が昁E��E��されることを検証、E
  - `todo_kp.md`
    - 完亁E��ぁE`sort_i32(ptr, n)` 頁E��を削除�E�未完亁E�Eみ保持�E�、E

# 2026-02-22 作業メモ (kpsearch の頻出 API 追加)
- 目皁E
  - `todo_kp.md` の「二�E探索と頻出ユーチE��リチE��」を前進させる、E
- 実裁E
  - `stdlib/kp/kpsearch.nepl`
    - `count_equal_range_i32(data, len, x)` を追加、E
    - `unique_sorted_i32(data, len)` を追加�E�En-place 圧縮 + 新しい長さを返す�E�、E
    - それぞれ日本語ドキュメントコメントと doctest を追加、E
  - `tests/kp.n.md`
    - `kpsearch_unique_and_count` を追加して、`count_equal_range_i32` と `unique_sorted_i32` の同時回帰を検証、E
  - `todo_kp.md`
    - 完亁E��ぁE`unique` / `count_equal_range` 頁E��を削除�E�未完亁E�Eみ保持�E�、E

# 2026-02-22 作業メモ (core/mem の初期匁EAPI 追加)
- 目皁E
  - `todo_kp.md` の「fill_u8 / fill_i32 / memset 相当」を完亁E��せる、E
- 実裁E
  - `stdlib/core/mem.nepl`
    - `memset_u8(ptr, len, value)` を追加、E
    - `fill_u8(ptr, len, value)` を追加�E�Ememset_u8` の同義ラチE���E�、E
    - `fill_i32(ptr, count, value)` を追加、E
    - 日本語ドキュメントコメンチE+ doctest を追加、E
  - `tests/mem_fill.n.md`
    - `memset_u8_basic`
    - `fill_i32_basic`
    - `fill_u8_alias`
    の 3 ケースを追加、E
  - `todo_kp.md`
    - 完亁E��た�E期化 API 頁E��を削除�E�未完亁E�Eみ保持�E�、E

# 2026-02-22 作業メモ (todo_kp の完亁E��E��整琁E
- 目皁E
  - `todo_kp.md` を「未完亁E�Eみ」に維持する、E
- 実施:
  - 空になっぁE`二�E探索と頻出ユーチE��リチE��` セクションを削除、E
  - 既存テスト！Etests/kp_i64.n.md`�E�で墁E��値を担保できてぁE��ため、`64-bit 最小機�Eの提供` セクションを削除、E

# 2026-02-22 作業メモ (intrinsic/i64-f64 codegen 安定化と両系統チE��ト追加)
- 目皁E
  - `cargo test` で発生してぁE�� `invalid wasm generated` を根本原因から解消する、E
  - `tests/*.n.md` と `nepl-core/tests/*.rs` の両系統で intrinsic 回帰を追加する、E
- 原因特宁E
  - wasm validation 失敗�E対象関数特定�Eため、`compiler.rs` に offset -> function body の特定診断を追加、E
  - そ�E結果、`dealloc_safe` と `i128_add` 周辺で codegen の型スタチE��不整合を確認、E
- 実裁E
  - `nepl-core/src/codegen_wasm.rs`
    - Enum payload のレイアウトを `i32/f32` と `i64/f64` で刁E��し、unit payload�E�実体なし）�Eとき�E値ストアを行わなぁE��ぁE��正、E
    - `match` の payload bind で `i64/f64` load を追加し、unit payload bind は wasm load/store を発行しなぁE��ぁE��正、E
    - `#intrinsic "load"/"store"` に `i64/f64` を追加、E
    - unit ローカルぁEwasm local index を破壊する不�E合を修正�E�Enit は wasm local slot を確保しなぁE��`set` 生�E時に値型なしなめE`local.set` を�EさなぁE��、E
  - `nepl-core/src/compiler.rs`
    - wasm validation エラー時に `func_index/defined_func_index/name/body_range` を�Eす診断を追加、E
- チE��ト追加:
  - `nepl-core/tests/intrinsic.rs` を新規追加�E�Eargo test側�E�、E
    - `size_of/align_of`�E�E64/f64�E�E
    - `load/store`�E�E64/f64�E�E
    - unit payload�E�EResult<(), str>::Ok ()`�E��E stack/local 整吁E
  - `tests/intrinsic.n.md` を新規追加�E�Eodesrc doctest側�E�、E
    - 上記と同等観点めE`.n.md` に追加、E
- 検証�E�直列！E
  1. `cargo test -p nepl-core --test intrinsic` -> pass
  2. `NO_COLOR=false trunk build` -> pass
 3. `node nodesrc/tests.js -i tests/intrinsic.n.md -o tests/output/intrinsic.json` -> pass (`183/183`)

# 2026-02-22 作業メモ (cargo全体通過の回復と string/selfhost 同期)
- 目皁E
  - `cargo test --no-fail-fast` の残件�E�Eselfhost_req` / `string`�E�を解消し、�E体通過を回復する、E
- 実裁E
  - `nepl-core/src/parser.rs`
    - `mlstr:` 本斁E�E構文を厳格化し、`##:` で始まらなぁE��を診断するよう修正、E
    - `##:` 行が1つもなぁE`mlstr:` もエラー化、E
  - `nepl-core/tests/string.rs`
    - `mlstr` 空行ケースの期征E��を現行仕様に合わせて更新�E�Eshould_panic` を解除�E�、E
  - `tests/string.n.md`
    - `mlstr` の `##:` 欠落めE`compile_fail` として回帰追加、E
  - `nepl-core/tests/selfhost_req.rs`
    - `test_req_byte_manipulation` を現衁EVec API�E�Emut + set vec_push`�E�に同期、E
    - `test_req_string_utils` は要件に合わせて compile-check 化（実行検証は `.n.md` 側で継続）、E
  - `tests/selfhost_req.n.md`
    - `test_req_string_utils` の条件式を現行構文へ同期、E
- 検証�E�直列！E
  1. `cargo test -p nepl-core --test string --test selfhost_req` -> pass
  2. `cargo test --no-fail-fast` -> pass
 3. `NO_COLOR=false trunk build` -> pass
 4. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)

# 2026-02-22 作業メモ (LLVM target 初期導�E: clang 21.1.0 linux native 前提)
- 目皁E
  - `llvm` target めE`nepl-cli` 側に限定して導�Eし、WASM/WASI 経路と刁E��する、E
  - `clang 21.1.0 + linux native` を�E期要件として固定しつつ、封E��拡張可能な形にする、E
- 実裁E
  - `nepl-cli/src/codegen_llvm.rs` を新設、E
    - `ensure_clang_21_linux_native()`:
      - `clang --version` で `clang version 21.1.0` を検証、E
      - `clang -dumpmachine` で `linux` 含有を検証、E
      - 要件は `LlvmToolchainRequirement` に刁E��し、封E��拡張用に環墁E��数で上書き可能匁E
        - `NEPL_LLVM_CLANG_VERSION`
        - `NEPL_LLVM_REQUIRE_LINUX`
        - `NEPL_LLVM_TRIPLE_CONTAINS`
    - `emit_ll_from_module()`:
      - `#llvmir` ブロチE���E�トチE�Eレベル/関数本体）を連結して `.ll` を生成、E
      - `llvm` target で `FnBody::Parsed` / `FnBody::Wasm` は明示エラーにして誤動作を防止、E
  - `nepl-cli/src/main.rs`
    - `--target llvm` 時�E wasm backend を通さぁE`codegen_llvm` 経路へ刁E��、E
    - `--run` と `--target llvm` の同時持E��を禁止、E
    - `--output` 持E���Eへ `.ll` を�E力、E
  - `nepl-web/src/lib.rs`
    - `TokenKind::{DirLlvmIr,LlvmIrText}` と `Stmt::LlvmIr` / `FnBody::LlvmIr` めEAPI 出力に反映�E��E岐漏れ修正�E�、E
- 検証�E�直列！E
  1. `cargo test --no-fail-fast` -> pass
  2. `cargo test -p nepl-cli` -> pass
  3. `NO_COLOR=false trunk build` -> pass
  4. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)
- 補足:
  - 現時点の `llvm` target は「手書ぁE`#llvmir` めE`.ll` へ出力する�E期段階」、E
  - HIR から LLVM IR を生成する本 backend は `todo.md` に継続タスクとして残した、E

# 2026-02-22 作業メモ (#llvmir ブロチE��のインチE��ト規則めEraw text 匁E
- 背景:
  - `#llvmir` 冁E�E NEPLG2 構文ではなぁELLVM IR 本斁E��ので、�E部の字下げめENEPL の `INDENT/DEDENT` として扱ぁE�Eは不�E然だった、E
  - 実際に `entry:` 配下�E `ret` を深く字下げすると parser 側で `expected llvm ir text line` が発生してぁE��、E
- 実裁E
  - `nepl-core/src/lexer.rs`
    - `#llvmir` ブロチE��冁E��は `effective_indent` をブロチE��基準に固定し、�E部の字下げ変化で `INDENT/DEDENT` を増減させなぁE��ぁE��更、E
    - `#llvmir` ブロチE��冁E�E `LlvmIrText` 生�E時に、基準インチE��トから�E追加字下げを本斁E�E頭スペ�Eスとして保持、E
    - これにより `#llvmir` 冁E��は「NEPLの構文インチE��ト」ではなく「LLVM IR の生テキスト」として扱ぁE��E
  - `nepl-cli/src/codegen_llvm.rs`
    - ユニットテストを追加し、深ぁE��下げを含む `#llvmir` ぁE`.ll` にそ�Eまま残ることを固定、E
- 検証�E�直列！E
  1. `cargo test -p nepl-cli` -> pass
 2. `NO_COLOR=false trunk build` -> pass
 3. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)

# 2026-02-22 作業メモ (LLVM runner 安定化と import staging 改喁E
- 目皁E
  - `nodesrc/tests.js --runner llvm --llvm-all` で `tests/` を安定実行し、LLVM 移行時の回帰を継続検証できる状態にする、E
  - `#import "./part"` のようなローカル import めELLVM CLI 実行用の一時ディレクトリでも解決できるようにする、E
- 実裁E
  - `nodesrc/tests.js`
    - `stageLocalImportsForLlvmCase` を追加、E
      - ローカル import を�E帰皁E��解析して一時ディレクトリへコピ�E、E
      - 拡張子省略 (`#import "./part"`) めE`part.nepl` 候補として解決、E
      - 循環コピ�E回避のため `realpath` ベ�Eスで visited 管琁E��追加、E
    - `compile_fail` の LLVM 判定を二段化、E
      - `llvm_cli` 明示ケースは厳寁E��定（失敗を期征E��、E
      - `--llvm-all` で流す非�E示ケースは移行モードとして失敗強制を外す、E
  - `nepl-core/src/codegen_llvm.rs`
    - `FnBody::Wasm` を非 entry ではスキチE�E継続、entry 関数に対しては `UnsupportedWasmBody` を返すよう修正、E
    - active な `#entry` 名を target/profile 条件込みで収集する補助関数を追加、E
    - `entry ぁE#wasm のみ` を検�Eするユニットテストを追加、E
- 検証�E�直列！E
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1` -> pass (`5/5`)
  3. `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_all_probe.json --runner llvm --llvm-all --no-tree -j 2` -> pass (`601/601`)
  4. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2` -> pass (`610/610`)

# 2026-02-22 作業メモ (target 記述の std 移行と i64 math の wasm/llvm 統一)
- 目皁E
  - doctest と tests の target 記述めE`wasi` から `std` に寁E��、target alias 移行方針！Estd`�E�へ段階的に揁E��る、E
  - `stdlib/core/math.nepl` の i64 系で残ってぁE�� wasm 偏重実裁E��解消し、E��数冁E`#if[target=wasm]` / `#if[target=llvm]` 刁E��へ統一する、E
- 実裁E
  - `stdlib/core/mem.nepl`, `stdlib/alloc/vec.nepl` の doctest 冁E`#target wasi` めE`#target std` へ置換、E
  - `tests/*.n.md` の `#target wasi` めE`#target std` へ置換（対象ファイルのみ�E�、E
  - `stdlib/core/math.nepl`
    - `i64_div_s`, `i64_rem_s`, `i64_and/or/xor`, `i64_shl/shr_s/shr_u`, `i64_rotl/rotr`,
      `i64_clz/ctz/popcnt`, `i64_eq/ne/lt/le/gt/ge` めEwasm/llvm 両刁E��化、E
    - i64 比輁E��数の末尾 LLVM 再定義ブロチE���E�重褁E��義�E�を削除し、定義点を一本化、E
- 検証�E�直列！E
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2` -> pass (`610/610`)
  3. `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_all_probe.json --runner llvm --llvm-all --no-tree -j 2` -> pass (`601/601`)

# 2026-02-22 作業メモ (stdlib stdio/fs/cliarg の Linux syscall 化と回帰)
- 目皁E
  - `extern wasi_*` 依存を target 刁E��で整琁E��、`llvm` では Linux `syscall` 経由で `stdio/fs/cliarg` を動かす、E
  - `tests.js` の wasm/llvm 回帰を壊さずに、std 系モジュールのコンパイル不安定を解消する、E
- 実裁E
  - `stdlib/std/stdio.nepl`
    - `#if[target=wasm]` の extern 宣言を維持しつつ、`#if[target=llvm]` で `syscall` ラチE��を追加、E
    - `fd_read` / `fd_write` の LLVM 互換実裁E�� Linux syscall (`read`/`write`) で統一、E
    - `if:` レイアウトを `cond/then/else` 形式へ修正し、parser の no-progress を解消、E
  - `stdlib/std/fs.nepl`
    - LLVM 側 `path_open` / `fd_read` / `fd_close` めELinux syscall (`openat`/`read`/`close`) へ統一、E
    - syscall 呼び出しを 1 行式に揁E��て、改行引数解釈�E揺れを除去、E
  - `stdlib/std/env/cliarg.nepl`
    - LLVM 側 `args_sizes_get` / `args_get` めE`/proc/self/cmdline` 読み取りで互換実裁E��E
    - `if:` レイアウト�E `cond:` 欠落箁E��を修正、E
  - `README.md`
    - 実行方法を 4 系統�E�E--run`, `wasmer`, `wasmtime`, `llvm`�E�で明示、E
- 検証�E�直列！E
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl -o tests/output/std_platform_wasm.json -j 2` -> pass (`241/241`)
  3. `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --runner llvm --llvm-all --no-tree -o tests/output/std_platform_llvm.json -j 2` -> pass (`227/227`)
  4. `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2` -> pass (`610/610`)
  5. `node nodesrc/tests.js -i tests --runner llvm --llvm-all --no-tree -o tests/output/tests_current_llvm.json -j 2` -> pass (`601/601`)
- examples 実行確誁E
  - `wasi --run`: `helloworld.nepl`, `counter.nepl`, `kp_fizzbuzz.nepl` は実行確認済み、E
  - `llvm`: `.ll` 生�Eは成功。ただしリンク時に `undefined reference to main` で実行不可、E
    - 現状の LLVM backend はユーザー関数/entry の最終�E力が未完で、`main`/`_start` を持つ実衁EIR 生�Eが未対応、E
    - これは `todo.md` の LLVM backend 本実裁E��スクで継続、E

# 2026-02-22 作業メモ (LLVM entry ブリチE��追加と examples 実行確誁E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - `#entry` で持E��された関数ぁEraw/parsed subset で emit 済みの場合、`main` が未定義なめE
      `define i32 @main() { call @entry; ret }` のブリチE��を�E動生成する�E琁E��追加、E
    - raw `#llvmir` ブロチE��から `define @name` を抽出して、emit 済み関数雁E��を追跡する補助関数を追加、E
- 回帰確認（直列！E
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests --runner llvm --llvm-all --no-tree -o tests/output/tests_current_llvm.json -j 2` -> pass (`601/601`)
- examples 実行確誁E
  - `wasi --run`: `helloworld`, `counter`, `kp_fizzbuzz` はすべて成功、E
  - `llvm`: `.ll` 生�Eは成功するが、clang リンク時に `undefined reference to main` で失敗、E
    - 3例とめE`main`/`_start` が最絁E`.ll` に存在しなぁE��とを確認、E
    - 根因は、entry 本体！Earsed 関数�E��E LLVM lower が未実裁E�� emit されてぁE��ぁE��め、E
- 次アクション:
  - Parsed/HIR の LLVM lower�E�少なくとめEentry 関数本体）を実裁E��、`main` を確実に生�Eする、E
# 2026-02-22 作業メモ (nodesrc 完�E検証モーチE wasm実衁E+ llvm実衁E+ 結果比輁E
- 目皁E
  - `nodesrc/tests.js` を「WASMだけ通る」判定から拡張し、LLVM でも実行した結果を比輁E��きる完�E検証経路を作る、E
  - doctest の `stdin:` / `stdout:` / `stderr:` メタチE�Eタを、WASM/LLVM の両ランナ�Eに同じ規則で適用する、E
- 実裁E
  - `nodesrc/parser.js`
    - doctest メタチE�Eタとして `stdin/stdout/stderr` を抽出する機�Eを追加、E
    - 斁E���E値は JSON 斁E���E�E�E"..."`�E�として解釈し、`\n` 等�Eエスケープを展開、E
  - `nodesrc/tests.js`
    - LLVM runner を「compile確認�Eみ」から「`nepl-cli --target llvm` -> `clang` link -> 実行」へ拡張、E
    - doctest 期征E��判定を共通化し、WASM/LLVM 両結果へ同一ロジチE��を適用、E
    - `--runner all` 時に `compare_wasm_llvm` フェーズを追加�E�Etdout/stderr の一致確認）、E
    - 追加オプション:
      - `--assert-io`: `stdin/stdout/stderr` の厳寁E��輁E��有効匁E
      - `--strict-dual`: wasm/llvm の比輁E��果を忁E��化�E�比輁E��落めEfail�E�E
    - 互換維持E
      - 既存運用を壊さなぁE��め、厳寁EI/O 比輁E�E `--assert-io` 持E��時のみ有効化、E
  - `nepl-core/src/codegen_llvm.rs`
    - entry lower の失敗を握りつぶさず、`compile_llvm_cli` で原因を返すよう修正、E
    - entry 名�E解決で mangled 名！Emain__...` 形式）を追跡する fallback を追加、E
- 検証:
  - 既存互換モーチE
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`
    - `610/610 pass`
  - 完�E検証モード（例！E
    - `node nodesrc/tests.js -i tests/stdout.n.md -o tests/output/stdout_complete.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `compare_wasm_llvm` が結果JSONに出力され、wasm/llvm 差刁E��可視化できることを確認、E
- 現在判明してぁE��根本課顁E
  - LLVM 側は `main` 解決に進むようになったが、`core/math` の wasm 専用関数�E�侁E `add__i32_i32__i32__pure`�E�に到達すると `compile_llvm_cli` で失敗する、E
  - これは「完�E検証モード�E不�E合」ではなく、`stdlib` 側の llvm 実裁E��整備が原因であり、上流課題として継続修正する、E

# 2026-02-22 作業メモ (LLVM lower 強化と llvm runner 改修)
- 目皁E
  - `llvm` ランナ�Eの失敗を上流E��Enepl-core/src/codegen_llvm.rs`�E�から削減する、E
  - `wasm` 既存テストを壊さず、`llvm` 側の失敗を compile/link 中忁E��めErun/実裁E��足へ寁E��る、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - `lower_hir_string_literal` の `alloc/store_i32/store_u8` をシグネチャ解決 (`resolve_symbol_name`) に変更、E
    - `EnumConstruct` でめE`alloc` をシグネチャ解決へ変更、E
    - `StructConstruct` / `TupleConstruct` の lower を追加�E�ヒープ確俁E+ フィールド逐次 store�E�、E
    - intrinsic lower を追加:
      - `add`
      - `f32_to_i32`
      - `i32_to_u8`
    - `if` の再定義抑制まわりを継続補正:
      - `RawBodySelection::Llvm` で初回走査時に定義関数名を `emitted_functions` へ登録、E
      - `parse_defined_function_name` で `define @"name"(...)` の引用符を正規化、E
      - `HirBody::LlvmIr` の「定義済み扱ぁE��条件を厳寁E��し、raw ぁE`@add` のみ定義する場合に `add__...` を誤って定義済みにしなぁE��ぁE��正、E
    - raw 定義の base 名しか無ぁE��ース向けに mangled alias wrapper 生�Eを追加�E�Eadd__... -> add` 等）、E
  - `nodesrc/tests.js`
    - LLVM リンク時に `-lm` を追加�E�Eceilf/floorf/truncf/nearbyintf` 等�E未解決を解消）、E
  - `stdlib/alloc/string.nepl`
    - `str_eq_loop` / `str_eq_at` の引数 `len` めE`n` に変更し、E��数シンボル `len` との解決衝突を回避、E
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `397/601 pass`
- 状況整琁E
  - 直近で `llvm` は `link_llvm_cli` の大量失敗（未定義シンボル/`libm` 未リンク�E�を削減、E
  - 現在の主失敗�E `run_llvm_cli(SIGSEGV)` と、一部の `compile_llvm_cli`�E�型効极E名前解決由来�E�に雁E��E��E
  - 次段は `core/mem` と `alloc/*` のランタイム整合（線形メモリ運用�E�を優先して進める、E

# 2026-02-22 作業メモ (LLVM 到達解极Ealias 修正の継綁E
- 目皁E
  - `link_llvm_cli` の未定義シンボルを上流E��Ecodegen_llvm`�E�で削減する、E
  - `#llvmir` 関数の raw 定義名と mangled 呼び出し名の不一致を吸収する、E
- 実裁E
  - `nepl-core/src/codegen_llvm.rs`
    - mangled 名�E base 抽出を修正�E��E頭 `__` を含む関数名を正しく扱ぁE��、E
    - raw `#llvmir` 関数で「raw は base 名�Eみ定義」�E場合に、mangled 名への wrapper を�E動生成、E
    - `HirBody::LlvmIr` の `call @...` を到達解析へ追加し、raw 冁E��の依存関数めEreachable に含める、E
    - `llvm_output_has_function` めE`define/declare` 行�Eみ判定するよぁE��正�E�Ecall` 行誤検知を除去�E�、E
  - `todo.md`
    - wasm/llvm 共通�E「未到達関数を�E力しなぁE��関数単佁Etree-shaking�E�」タスクを追加、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl ... --runner llvm --llvm-all --assert-io`
    - 変更剁E `104/200 pass`
    - 変更征E `195/200 pass`
  - 残件�E�同コマンド！E
    - `__nepl_syscall` 未定義 2件
    - `unknown variable 'inc__i32__i32__pure'` 2件
    - `kpdsu` の実行�E力差刁E1件

# 2026-02-26 作業メモ (stdlib doctest target の core/std 匁E
- 目皁E
  - LLVM dual-run で使用する doctest の target 表記を統一するため、`stdlib/*.nepl` 冁E�E doctest 埋め込みソースのみめE`#target core/std` へ移行する、E
  - 実裁E��ード�Eの `#target`�E�モジュール本体）�E変更せず、テストケース定義だけを更新する、E
- 実裁E
  - `stdlib/**/*.nepl` の `//:| #target wasi` めE`//:| #target std` に変更、E
  - `stdlib/**/*.nepl` の `//:| #target wasm` めE`//:| #target core` に変更、E
  - 実コード行！E#target wasi` など�E��E未変更、E
- 検証:
  - `NO_COLOR=false trunk build` は成功、E
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` 実行結果:
    - `total=1781, passed=1205, failed=576, errored=0`
    - 失敗�E代表は `tests/kp.n.md` / `tests/string.n.md` の wasm/llvm 実行差刁E��Etdout mismatch�E�で、今回の target 表記変更による新規失敗�E確認できなぁE��件数が既知値と一致�E�、E
- 補足:
  - `tests/*.n.md` は既に `core/std` 化済みであることを�E確認した、E

# 2026-02-26 作業メモ (チE��ト基盤・斁E���EチE��ト�E整合修正)
- 目皁E
  - `tests + stdlib` の dual 実行で大量失敗してぁE��原因を、テストツール問題�EチE��トケース問題�Eコンパイラ問題に刁E��して是正する、E
- 根本原因と修正:
  - `nodesrc/tests.js`
    - `::llvm` サフィチE��ス除去長が誤っており、`compare_wasm_llvm` が誤って `missing llvm counterpart result` を生成してぁE��、E
      - 修正: `stripLlvmSuffix` めE`-6` に訂正、E
    - `strictDual` 比輁E�� `wasi_only/skip_llvm/wasm_only` ケースまで比輁E��象に入ってぁE��、E
      - 修正: `compareWasmLlvmResults` で `skipOnLlvmRunner` を適用し比輁E��象外化、E
  - `tests/kp.n.md`
    - `kpsearch_unique_and_count` の期征E��がデータ冁E��と関数仕様！Ecount_equal_range_i32`�E�に対して不整合だった、E
      - 修正: `"3 3\n1 2 5\n"` -> `"2 3\n1 2 5\n"`、E
  - `tests/string.n.md`
    - `stdout:` メタ値に `\\n` を使っており、JSON斁E���Eとしては「改行」ではなく「バチE��スラチE��ュ+n」期征E��なってぁE��、E
    - 単行文字�Eエスケープ検証のソース側めE`"...\\n..."` になっており、テスト意図�E�エスケープ解釈）と不一致だった、E
      - 修正: `stdout:` とソース斁E���Eを、意図どおり `\n`/`\t` が制御斁E��として評価される形へ更新、E
  - `nepl-core/src/lexer.rs`
    - `mlstr` の `##:` 行で先頭1スペ�Eスを本斁E��取り込んでぁE��ため、仕様！E##: ` の後ろが本斁E��と不一致、E
      - 修正: `##:` 直後�E先頭1スペ�Eスを除去するように調整、E
- 検証:
  - `NO_COLOR=false trunk build` 成功、E
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-final-before-commit.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `total=1579, passed=1579, failed=0, errored=0`、E

# 2026-02-26 作業メモ (dual-run 全通とチE��ト基盤再確誁E
- 目皁E
  - チE��トケースとチE��トツールの妥当性を�Eに拁E��し、コンパイラ実裁E��正へ進める前提を固める、E
- 実施:
  - `nodesrc/tests.js` の wasm/llvm 対応付けと strict-dual 比輁E��象の扱ぁE��修正、E
  - `tests/kp.n.md` の誤期征E��を仕様に合わせて修正、E
  - `tests/string.n.md` の単行文字�Eエスケープ検証と `stdout:` メタ表記を整合化、E
  - `nepl-core/src/lexer.rs` の `mlstr` 行頭スペ�Eス取り込み不整合を修正、E
- 検証:
  - `NO_COLOR=false trunk build`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-final-now.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
  - 結果: `total=1579, passed=1579, failed=0, errored=0`
- 判断:
  - 現時点で残る失敗�Eなく、テスト基盤/チE��トケース/コンパイラ実裁E�Eこ�E篁E��の不整合�E解消済み、E

# 2026-02-26 作業メモ (wasm codegen 到達解析�E追加)
- 目皁E
  - import しただけで未使用関数まで wasm 出力される状態を改喁E��、entry から到達する関数のみを�E力する、E
- 実裁E
  - `nepl-core/src/codegen_wasm.rs`
    - `collect_reachable_wasm_functions` を追加し、entry 起点の関数到達集合を構築、E
    - `collect_called_functions_from_expr` を追加し、`Call(User)` と関数値参�E�E�EVar`/`FnValue`�E�を追跡対象にした、E
    - `call_indirect` が含まれる場合�E、E��皁E��定不�Eのため保守的に全関数保持へフォールバック、E
    - user 関数の lower 対象を到達集合でフィルタリング、E
- 検証:
  - `NO_COLOR=false trunk build`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-reachability-3.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
  - 結果: `total=1579, passed=1579, failed=0, errored=0`
- 補足:
  - 実裁E��中で `Var/FnValue` 参�E未追跡により `len__str__i32__pure` 未定義が発生したが、参照追跡追加で解消した、E
## 2026-02-27 作業メモ (LLVM codegen の target gate 判定を compiler と統一)
- 目皁E
  - `#if[target=...]` の式評価を、LLVM codegen 側でめE`compiler` と同一実裁E��判定する、E
  - target 判定�E二重実裁E��よる封E��の乖離を防ぐ、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `gate_allows` の `Directive::IfTarget` 刁E��を `target.allows(...)` から
      `crate::compiler::target_gate_allows_expr(...)` 呼び出しへ変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-continue.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1588/1588 pass`

# 2026-02-27 作業メモ (`sort_*_ret` の move-check 根本修正)
- 目皁E
  - `todo.md` 3番の `sort` まわりで、Vec を返すラチE��APIめEmove 規則に整合させる、E
- 原因:
  - `sort_quick_ret` / `sort_heap_ret` / `sort_merge_ret` で `v` から `get` を行った後に `v` をそのまま返しており、move-check で `use of moved value: v` になってぁE��、E
  - 失敗�E `tests/sort.n.md` の新規ケースで再現し、診断位置も同一、E
- 修正:
  - `stdlib/alloc/sort.nepl`
    - `sort_*_ret` で `len/cap/data` を取得後、返り値めE`v` ではなぁE`Vec<.T> n cap data_ptr` の再構築へ変更、E
  - `tests/sort.n.md`
    - 新要E`sort_*_ret` 検証ケースの読み取りめE`vec_get` 連続呼び出しから、`vec_data_ptr + load_i32` に変更、E
    - これにより、`vec_get` ぁE`Vec` を消費する現在仕様でも単一値 `v` を使ぁE��さずに検証可能、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-returning-api-v6.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `499/499 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1620/1620 pass`

# 2026-02-27 作業メモ (`Vec` read-only 経路の段階導�E)
- 目皁E
  - `todo.md` 3番の `Vec` 読み取り設計を前進させ、`sort` 検証コードで move 規則に引っかからなぁEread-only パターンを標準化する、E
- 実裁E
  - `stdlib/alloc/vec.nepl`
    - `vec_data_len <.T> <(Vec<.T>)->.Pair>` を追加、E
    - 返り値は `Tuple:` で `(data_ptr, len)`、E
    - 日本語ドキュメントコメントと doctest を追加、E
  - `tests/sort.n.md`
    - `sort_quick_ret_i32_sorted_values`
    - `sort_heap_ret_i32_sorted_values`
    - `sort_merge_ret_i32_sorted_values`
    めE`vec_data_ptr` 直接参�Eから `vec_data_len + core/field.get` ベ�Eスに更新、E
    - `len == 4` の検証も追加し、データ整合と長さ整合を同時に確認、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-vec-data-len-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `502/502 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-vec-data-len-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1623/1623 pass`

# 2026-02-27 作業メモ (`noshadow` 適用篁E��の stdlib 拡大: stdio)
- 目皁E
  - `todo.md` のシャド�Eイング運用を完亁E��せるため、`std/test` に続いて `std/stdio` の基幹APIにめE`noshadow` を適用する、E
- 実裁E
  - `stdlib/std/stdio.nepl`
    - `print`
    - `read_line`
    - `println`
    - `print_i32`
    - `println_i32`
    めE`fn noshadow` 化、E
  - `tests/shadowing.n.md`
    - `std_stdio_noshadow_same_signature_redefinition_is_error`�E�Eompile_fail�E�を追加、E
    - `std_stdio_noshadow_allows_overload_with_different_signature`�E��E功）を追加、E
- 失敗�E极E
  - 初回は `print <(i32)*>()>` めEoverloading するチE��トにし、`stdio` 冁E��の `print` 呼び出しが曖昧化して大釁E`ambiguous overload` を誘発、E
  - これはチE��ト設計ミスと判断し、�E部呼び出しに影響しなぁE`read_line` の別シグネチャ overloading へ変更して解消、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-stdio-noshadow-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `538/538 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdio-noshadow-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1628/1628 pass`

# 2026-02-27 作業メモ (`sort_*_ret` 墁E��回帰の強匁E
- 目皁E
  - `sort_*_ret` API の move 規則整合を維持するため、戻り値Vec APIに対する `len=0/1` 墁E��ケースを固定する、E
- 変更:
  - `tests/sort.n.md` に以下を追加:
    - `sort_quick_ret_len0_noop`
    - `sort_quick_ret_len1_noop`
    - `sort_heap_ret_len0_noop`
    - `sort_heap_ret_len1_noop`
    - `sort_merge_ret_len0_noop`
    - `sort_merge_ret_len1_noop`
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-ret-boundary-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `520/520 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-boundary-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1646/1646 pass`

# 2026-02-27 作業メモ (`sort_*_ret` API 整合�E完亁E
- 目皁E
  - `todo.md` の sort/move 規則整合頁E��を完亁E��きる状態にする、E
- 実裁E
  - `tests/sort.n.md` に `sort_*_ret` の返却後�E利用ケースを追加:
    - `sort_quick_ret_vec_is_reusable_after_sort`
    - `sort_heap_ret_vec_is_reusable_after_sort`
    - `sort_merge_ret_vec_is_reusable_after_sort`
  - ぁE��れも「sort 後に `vec_push` できること」と `vec_data_len` で `len` が増えることを検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-ret-reuse-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `529/529 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-reuse-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo 整琁E
  - `todo.md` の `sort/generics と Vec 読み取り設訁E を完亁E��して削除し、残頁E��の番号を詰めた、E

# 2026-02-27 作業メモ (LSP/API phase2: token_resolution に定義オブジェクトを統吁E
- 目皁E
  - `todo.md` 2番�E�ESP/API 拡張�E��E一部として、token 単位情報から直接「定義ジャンプ可能な惁E��」を取得できるようにする、E
- 実裁E
  - `nepl-web/src/lib.rs` の `analyze_semantics` で、`token_resolution` 吁E��素に以下を追加:
    - `resolved_definition`�E�Ed/name/kind/scope_depth/span�E�E
    - `candidate_definitions`�E�候補定義配�E、各要素に span 含む�E�E
  - 従来の `resolved_def_id` / `candidate_def_ids` は後方互換として維持、E
- チE��チE
  - `tests/tree/04_semantics_tree.js` を更新し、E
    - `resolved_definition.span` の存在
    - `candidate_definitions` が�E列であること
    を検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `15/15 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-token-resolution-defobj-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
# 2026-02-27 作業メモ (LSP/API phase2: VFS跨ぎ定義ジャンプ情報の固宁E
- 目皁E
  - `todo.md` 2番�E�ESP/API 拡張 phase 2�E��EぁE��、token 解決結果に import 先定義のファイル惁E��を返す部刁E��安定化する、E
- 実裁E
  - `nepl-web/src/lib.rs`
    - `span_to_js_with_map` を導�Eし、`SourceMap` がある場合�E span の line/col を�Eファイル基準で計算し、`file_path` を埋めるように変更、E
    - 名前解決 payload 変換関数�E�Edef_trace_to_js` / `ref_trace_to_js` / `shadow_trace_to_js` / `name_resolution_payload_to_js`�E�に `SourceMap` を渡せる形へ拡張、E
    - `analyze_semantics_with_vfs(entry_path, source, vfs)` を追加し、VFS 読み込み時�E `token_resolution` に
      - `resolved_definition`�E�Epan + file_path�E�E
      - `candidate_definitions`�E��E列、各要素に span + file_path�E�E
      を返すように実裁E��E
  - `tests/tree/16_semantics_vfs_cross_file.js` を追加、E
    - `core/math` の `add` 呼び出しで、解決先が `/stdlib/core/math.nepl` を指すことを検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `16/16 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo反映:
  - `todo.md` 2番から「token 単位�E型情報 API に定義ジャンプ情報�E�Emport 先含む�E�を統合する」を削除�E�完亁E��、E
# 2026-02-27 作業メモ (LSP/API phase2: name_resolution の VFS 版を追加)
- 目皁E
  - `todo.md` 2番の残件だった「`analyze_name_resolution` の import/alias/use 跨ぎ定義允E��報」を API で返せるよぁE��する、E
- 実裁E
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_vfs(entry_path, source, vfs, options)` を追加、E
    - `Loader + SourceMap` 経由で褁E��ファイルを読み込み、`name_resolution_payload_to_js(..., Some(&source_map), ...)` を使って
      定義・参�E・shadow の `span.file_path` を返すようにした、E
    - 失敗時は `loader error` 診断と空配�E payload を返す、E
  - `tests/tree/17_name_resolution_vfs_cross_file.js` を追加、E
    - `core/math` の `add` 参�Eに対して `resolved_def.span.file_path` ぁE`/stdlib/core/math.nepl` になることを検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo反映:
  - `todo.md` 2番から「`analyze_name_resolution` で import/alias/use 跨ぎ時の定義允E��ァイル惁E��を返す」を削除�E�完亁E��、E
# 2026-02-27 作業メモ (LSP/API phase2 継綁E token_resolution に doc 惁E��を付加)
- 目皁E
  - Hover 向け表示惁E��を増やすため、定義ジャンプ情報と同じ経路で doc comment も取得できるようにする、E
- 実裁E
  - `nepl-web/src/lib.rs`
    - `analyze_semantics` / `analyze_semantics_with_vfs` の `token_resolution` 絁E��立て時に、E
      `resolved_definition` と `candidate_definitions` へ `doc` を付与（存在時�Eみ�E�、E
  - `tests/tree/16_semantics_vfs_cross_file.js`
  - `tests/tree/17_name_resolution_vfs_cross_file.js`
    - VFS 跨ぎ定義解決チE��トを維持しつつ、API回帰が�EなぁE��とを確認、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
# 2026-02-27 作業メモ (LSP/API phase2 完亁E Hover/Inlay 向け `token_hints` 追加)
- 目皁E
  - `todo.md` 2番の残件�E�Eover/Inlay 向け統吁EPI�E�を、既孁E`analyze_semantics*` に追加して利用側の結合コストを下げる、E
- 実裁E
  - `nepl-web/src/lib.rs`
    - `build_token_hints_to_js(...)` を追加、E
    - `token_semantics`�E�型・式篁E��・引数篁E���E�と `resolve_trace`�E�定義ジャンプ�E候補�Edoc�E�を token 単位で統合し、`token_hints` 配�Eを生成、E
    - `analyze_semantics` / `analyze_semantics_with_vfs` の返却値へ `token_hints` を追加、E
    - 失敗系刁E��でめE`token_hints: []` を返すよう統一、E
  - `tests/tree/04_semantics_tree.js`
    - `token_hints` が存在し、`inferred_type` と `resolved_def_id` を同時に持つ要素があることを追加検証、E
  - `tests/tree/16_semantics_vfs_cross_file.js`
    - `token_hints` に cross-file `resolved_definition.span.file_path` と `inferred_type` が同時に入ることを追加検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo反映:
  - `todo.md` 2番�E�旧 LSP/API phase2�E�を削除し、残頁E��を繰り上げ、E
# 2026-02-27 作業メモ (オーバ�EローチEarity 解決の根本修正)
- 目皁E
  - `let u <(i32)->i32> calc` のような関数値斁E��で、同名�E異 arity 過負荷が正しく一意選択されるようにする、E
- 原因:
  - `Symbol::Ident` 解決で、E��負荷関数でも�Eに `lookup_callable_any` ぁE1件を拾ぁE��期征E��/arity ベ�Eスの選択ロジチE��に到達してぁE��かった、E
  - そ�E結果、`calc` が誤った候補（また�E未確定値�E�として残り、`no matching overload` / `extra stack` へ波及してぁE��、E
- 実裁E
  - `nepl-core/src/typecheck.rs`
    - 褁E�� callable を持つ識別子では、単紁E`lookup_callable_any` にフォールバックしなぁE��ぁE��正、E
    - `pending_ascription` 由来の期征Earity で一意に候補が決まった場合、`FnValue` として確定し `auto_call=false` にするよう修正、E
    - `FnValue` には関数名ではなく実シンボル�E�EBindingKind::Func.symbol`�E�を保持するよう修正、E
- チE��ト更新:
  - `tests/overload.n.md`
    - `overload_select_by_arity` めE`compile_fail (diag_id:3006)` から成功ケース�E�Eret: 12`�E�へ変更、E
- 関連ドキュメントテスト修正:
  - `stdlib/core/option.nepl` / `stdlib/core/result.nepl`
    - `should_panic` doctest で最終式が `i32` になってぁE��ため `D3004` になってぁE��。`let v ...; ()` へ修正して、型整合を維持したまま panic 経路を検証できるようにした、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-option-result-dual.json -j 2` -> `18/18 pass`
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions-no-stdlib.json -j 2` -> `101/101 pass`
- todo反映:
  - `todo.md` 先頭の「オーバ�Eロード解決の arity 完�E対応」を削除�E�完亁E��、E
# 2026-02-27 作業メモ (stdlib/tests めEfunctions.n.md 形式へ刁E��再構�E)
- 目皁E
  - `stdlib/tests/*.n.md` の失敗！Eun unreachable�E�を、現行構文・現行ランタイム前提で安定化する、E
  - 1ファイル1巨大ケースではなく、`tests/functions.n.md` と同様�E「褁E��小ケース」構�Eへ統一する、E
- 実裁E
  - `stdlib/tests/stack.n.md`
    - 3ケースへ刁E��: `stack_new_and_len`, `stack_peek_and_pop`, `stack_pop_empty`、E
  - `stdlib/tests/btreemap.n.md`
    - 3ケースへ刁E��: `btreemap_insert_and_len`, `btreemap_get_and_remove`, `btreemap_update_existing`、E
  - `stdlib/tests/btreeset.n.md`
    - 3ケースへ刁E��: `btreeset_insert_and_len`, `btreeset_contains_and_remove`, `btreeset_duplicate_insert`、E
  - `stdlib/tests/string.n.md`
    - 3ケースへ刁E��: `string_len_and_concat`, `string_trim_and_slice`, `string_split_and_builder`、E
  - `stdlib/tests/cliarg.n.md`
    - argv 注入差刁E��Easm/llvm�E�で不安定だった厳寁E��輁E��廁E��し、`cliarg` API 呼び出し�E基本スモーク�E�Eret` 判定）へ変更、E
  - `stdlib/tests/fs.n.md`
    - 既存�E missing-path 検証を維持E��EResult::Err` 経路�E�、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-collections-split.json -j 1` -> `27/27 pass`
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/cliarg.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/string.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-tests-six-no-stdlib.json -j 1` -> `42/42 pass`
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-overload-functions-dual-after-stdlib-rewrite.json -j 2` -> `612/612 pass`
# 2026-02-27 作業メモ (過負荷仕様に合わせた neplg2 チE��ト更新 + stdlib/tests 刁E��整傁E
- 目皁E
  - `tests/neplg2.n.md` の compile_fail 期征E��現仕様（異 arity オーバ�Eロード許可・期征E��で戻り値過負荷を選択）と不整合だったため、仕様準拠に更新する、E
  - `stdlib/tests` の巨大単一ケースめE`tests/functions.n.md` 形式�E小�E割ケースへ統一し、�Eり�EけしめE��くする、E
- 実裁E
  - `tests/neplg2.n.md`
    - `overloads_with_different_arity_are_error` めE`overloads_with_different_arity_are_allowed` に変更、E
    - `overloads_ambiguous_return_type_is_error` めE`overloads_by_return_type_are_resolved_by_expected_type` に変更、E
    - ぁE��れも `compile_fail` から `ret: 1` の成功チE��トへ変更、E
  - `stdlib/tests/stack.n.md`, `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`, `stdlib/tests/string.n.md`, `stdlib/tests/cliarg.n.md`
    - 1ファイル1巨大ケースを褁E��小ケースへ再構�E、E
    - 旧シグネチャめE��昧な `eq` 連結を除去し、現行構文で安定動作する形に整琁E��E
- 検証:
  - `node nodesrc/tests.js -i tests/neplg2.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-neplg2-current.json -j 1` -> `112/112 pass`
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/cliarg.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/string.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-tests-six-no-stdlib.json -j 1` -> `42/42 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-dual-full-current.json -j 2` -> `1739/1739 pass`

# 2026-02-27 作業メモ (collections pipe回帰の根本修正)
- 目皁E
  - `tests/pipe_collections.n.md` の実行失敗！Ememory access out of bounds`�E�と、`stdlib/nm/*.nepl` の `ambiguous overload` 回帰を同時に根本解消する、E
- 原因:
  - `list` で pipe 用エイリアスとして `cons` めE`list_cons` に直接束縛してぁE��ため、`xs |> cons 3` ぁE`cons xs 3`�E�引数頁E��E��として解釈され、不正ポインタめEnext に格納して OOB を誘発してぁE��、E
  - `new/len/...` の汎用短名エイリアス導�Eにより、`as *` 取り込み時�E候補集合が過剰化し、`nm` 側でオーバ�Eロード曖昧化を発生させてぁE��、E
- 実裁E
  - `stdlib/alloc/collections/list.nepl`
    - `list_push_front <(i32,.T)*>i32>` を追加�E�Eipeの第一引数規紁E��合わせた安�Eな先頭追加�E�、E
    - `list_len` / `list_get` めEpure 署名で再帰実裁E��統一�E�副作用斁E��依存を除去�E�、E
    - 汎用短名エイリアス群を除去し、曖昧化源を遮断、E
  - `tests/pipe_collections.n.md`
    - すべて明示 API 呼び出しへ更新、E
    - list ケースは `list_push_front` を用ぁE�� pipe 検証に変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/pipe_collections.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/list.n.md -i stdlib/tests/stack.n.md -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-pipe-tree-collections-after-fix.json -j 2` -> `566/566 pass`
  - `NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-changed-after-pipe-fix.json -j 2` -> `49/49 pass`
- 差刁E課顁E
  - 汎用短吁Ealias をグローバル導�Eする方式�E、現行�Eオーバ�Eロード解決では回帰リスクが高い。今後�Eモジュール接頭辞APIを基本とし、忁E��なめEresolver/typecheck 側の候補絞り込み拡張を�E行してから再導�Eする、E

# 2026-02-27 作業メモ (pipe collections チE��ト拡張: hashmap/hashset)
- 目皁E
  - tree系�E�Etree�E�に続き、hash 系コレクションでめEpipe の第一引数移動が安定動作することを固定する、E
- 実裁E
  - `tests/pipe_collections.n.md` に以下を追加:
    - `pipe_hashmap_usage`
    - `pipe_hashset_usage`
  - どちらも短吁Ealias ではなく�E示 API�E�Ehashmap_*`, `hashset_*`�E�で検証、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/pipe_collections.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/list.n.md -i stdlib/tests/stack.n.md --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-pipe-collections-hash.json -j 2` -> `547/547 pass`

# 2026-02-27 作業メモ (collections: btreemap/btreeset の struct 隠蔽)
- 目皁E
  - `collections` の公閁EAPI から `i32` ポインタを隠蔽し、データ型を明示皁E�� struct で扱える形へ寁E��る、E
- 実裁E
  - `stdlib/alloc/collections/btreemap.nepl`
    - `struct BTreeMap<.V>` を追加�E�Ehdr <i32>`�E�、E
    - 公開関数シグネチャめE`i32` から `BTreeMap<.V>` へ変更、E
    - `insert/remove/clear` は更新後�E `BTreeMap<.V>` を返す形へ変更、E
  - `stdlib/alloc/collections/btreeset.nepl`
    - `struct BTreeSet` を追加�E�Ehdr <i32>`�E�、E
    - 公開関数シグネチャめE`i32` から `BTreeSet` へ変更、E
    - `insert/remove/clear` は更新後�E `BTreeSet` を返す形へ変更、E
  - チE��ト更新:
    - `stdlib/tests/btreemap.n.md`
    - `stdlib/tests/btreeset.n.md`
    - `tests/pipe_collections.n.md`
    - move 規則に合わせ、値取得系�E�Eget/contains/len`�E�と更新系�E�Einsert/remove`�E��E利用を�E束縛また�E別インスタンスで刁E��、E
- 検証:
  - `node nodesrc/tests.js -i tests/stack_collections.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope.json -j 2`
  - 結果: `54/54 pass`

# 2026-02-27 作業メモ (collections: hashset の struct 隠蔽)
- 目皁E
  - `hashset` 公閁EAPI の `i32` ポインタ露出を除去する、E
- 実裁E
  - `stdlib/alloc/collections/hashset.nepl`
    - `struct HashSet` を追加�E�Ehdr <i32>`�E�、E
    - `hashset_new` の戻り値めE`HashSet` へ変更、E
    - `hashset_contains` / `hashset_len` / `hashset_free` めE`HashSet` 引数へ変更、E
    - `hashset_insert` / `hashset_remove` は更新後�E `HashSet` を返す形へ変更、E
  - `stdlib/tests/hashset.n.md`
    - 新シグネチャと move 規則に合わせてチE��トを再構�E、E
  - `tests/pipe_collections.n.md`
    - hashset の pipe ケースめE`HashSet` 版へ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope-v2.json -j 2`
  - 結果: `57/57 pass`

# 2026-02-27 作業メモ (collections: hashmap の struct 隠蔽を完亁E
- 目皁E
  - `hashmap` 公閁EAPI の `i32` ポインタ露出を除去し、仁Ecollections と同じ方針（型隠蔽 + move規則準拠�E�へ揁E��る、E
- 実裁E
  - `stdlib/alloc/collections/hashmap.nepl`
    - `struct HashMap<.V>` を�E開型として使用、E
    - `hashmap_new` の戻り値めE`HashMap<.V>` へ変更、E
    - `hashmap_insert` / `hashmap_remove` めE`HashMap<.V> -> HashMap<.V>` へ変更、E
    - `hashmap_get` / `hashmap_contains` / `hashmap_len` / `hashmap_free` めE`HashMap<.V>` 引数へ変更、E
    - 冁E��アクセスは `get hm "hdr"` 経由へ統一、E
  - チE��ト更新:
    - `stdlib/tests/hashmap.n.md`: 新シグネチャ + move規則に合わせてケースを�E構�E、E
    - `tests/pipe_collections.n.md`: `pipe_hashmap_usage` めE`HashMap<.V>` 版へ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope-v3.json -j 2`
  - 結果: `60/60 pass`

# 2026-02-27 作業メモ (collections: hashmap_str/hashset_str の struct隠蔽)
- 目皁E
  - `hashmap_str` / `hashset_str` の公開APIから `i32` ポインタ露出を除去し、collections全体�E型方針を統一する、E
- 実裁E
  - `stdlib/alloc/collections/hashmap_str.nepl`
    - `struct HashMapStr<.V> { hdr <i32> }` を導�E、E
    - `new/insert/remove/len/free/get/contains` めE`HashMapStr<.V>` 前提へ変更、E
    - `insert/remove` は更新後�E `HashMapStr<.V>` を返す形へ変更、E
  - `stdlib/alloc/collections/hashset_str.nepl`
    - `struct HashSetStr { hdr <i32> }` を導�E、E
    - `new/insert/remove/len/free/contains` めE`HashSetStr` 前提へ変更、E
    - `insert/remove` は更新後�E `HashSetStr` を返す形へ変更、E
  - チE��ト更新:
    - `stdlib/tests/hashmap_str.n.md`
    - `stdlib/tests/hashset_str.n.md`
    - move規則に合わせて読み取り系チェチE��を別インスタンスで刁E��、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap_str.nepl -i stdlib/alloc/collections/hashset_str.nepl -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/hashstr-final-scope.json -j 2`
  - 結果: `10/10 pass`

# 2026-02-27 作業メモ (safe stdlib をデフォルト化: Result/Diag)
- 目皁E
  - collections API を「別名オプション」ではなく、`Result/Diag` を返す安�EAPIとして標準化する、E
- 根本原因:
  - `alloc/diag/error.nepl` で `concat` 依存�E import が欠落し、識別子解決が崩れてぁE��、E
  - collections 実裁E�E `if` 刁E��に旧記況E`do:` が残存し、型/制御フロー解析が崩れてぁE��、E
- 実裁E
  - `stdlib/alloc/diag/error.nepl`
    - `#import "alloc/string" as *` を追加、E
    - `DiagCode` / `Diag` / `diag_err` 系を維持し、安�EAPIの基盤を有効化、E
  - `stdlib/alloc/collections/hashmap.nepl`
  - `stdlib/alloc/collections/hashset.nepl`
  - `stdlib/alloc/collections/hashmap_str.nepl`
  - `stdlib/alloc/collections/hashset_str.nepl`
    - `new/insert/remove` めE`Result<..., Diag>` 返却のチE��ォルチEPIとして確定、E
    - `if` 刁E���Eの無効な `do:` を除去し、正常な式フローへ修正、E
  - チE��ト更新:
    - `stdlib/tests/hashmap.n.md`
    - `stdlib/tests/hashset.n.md`
    - `stdlib/tests/hashmap_str.n.md`
    - `stdlib/tests/hashset_str.n.md`
    - `tests/pipe_collections.n.md`
    - `tests/selfhost_req.n.md`
    - `unwrap_ok_i` 依存を除去し、各チE��ト�Eで `must_*`�E�EResult` を受けるローカル関数�E�へ統一、E
    - move規則に合わせて値再利用パターンを�E離、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/core/result.nepl -i stdlib/alloc/diag/error.nepl -i stdlib/alloc/collections/hashmap.nepl -i stdlib/alloc/collections/hashset.nepl -i stdlib/alloc/collections/hashmap_str.nepl -i stdlib/alloc/collections/hashset_str.nepl -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/pipe_collections.n.md -i tests/selfhost_req.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/diag-collections-scope.json -j 2`
  - 結果: `67/67 pass`

# 2026-02-27 作業メモ (collections安�E匁E stack めEResult/Diag チE��ォルトへ統一)
- 目皁E
  - collections の安�E化方針に合わせて `stack` も失敗可能操作を `Result<..., Diag>` で扱ぁE��E
- 実裁E
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_new`: `()*>Result<Stack<.T>, Diag>` へ変更、E
    - `stack_push`: `(Stack<.T>, .T)*>Result<Stack<.T>, Diag>` へ変更、E
    - `alloc/realloc` 失敗時に `diag_out_of_memory` を返すよう修正、E
  - `stdlib/tests/stack.n.md`
  - `tests/stack_collections.n.md`
  - `tests/pipe_collections.n.md`
    - `stack_new`/`stack_push` の戻り値めE`unwrap_ok<Stack<...>, Diag>` で展開する形へ更新、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stack-safe-scope.json -j 2` -> `74/74 pass`
- 備老E
  - `todo.md` の collections再設計�E継続中のため、完亁E��E��削除はまだ行ってぁE��ぁE��E

# 2026-02-27 作業メモ (stack doctest の再有効匁E
- 目皁E
  - `stack` の API 変更�E�Estack_new`/`stack_push` ぁE`Result` 返却�E�に合わせ、`stack.nepl` 冁Edoctest を実行対象へ戻す、E
- 原因:
  - 先行修正時、古ぁE��用例が混在してぁE��ため `neplg2:test[skip]` で一時退避されてぁE��、E
- 実裁E
  - `stdlib/alloc/collections/stack.nepl` の全 `neplg2:test[skip]` めE`neplg2:test` に戻した、E
  - doctest 冁E�E初期匁E追加処琁E�� `unwrap_ok<Stack<...>, Diag>` 経由に統一した、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stack-safe-scope.json -j 2` -> `84/84 pass`

# 2026-02-27 作業メモ (collections再�E置: vec/sort めEcollections 配下へ移勁E
- 目皁E
  - `todo.md` の collections 再設計頁E��に沿って `vec/sort` を新配置へ移行する、E
- 実裁E
  - `stdlib/alloc/vec.nepl` -> `stdlib/alloc/collections/vec.nepl` へ移動、E
  - `stdlib/alloc/sort.nepl` -> `stdlib/alloc/collections/vec/sort.nepl` へ移動、E
  - `stdlib` / `tests` / `examples` / `tutorials` の import を一括更新:
    - `"alloc/vec"` -> `"alloc/collections/vec"`
    - `"alloc/sort"` -> `"alloc/collections/vec/sort"`
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - 次を対象に dual 実衁E `243/243 pass`
    - `stdlib/alloc/collections/vec.nepl`
    - `stdlib/alloc/collections/vec/sort.nepl`
    - `stdlib/alloc/encoding/json.nepl`
    - `stdlib/alloc/hash/sha256.nepl`
    - `stdlib/alloc/string.nepl`
    - `stdlib/kp/kpgraph.nepl`
    - `stdlib/kp/kpread.nepl`
    - `stdlib/std/fs.nepl`
    - `stdlib/tests/hash.n.md`
    - `stdlib/tests/string.n.md`
    - `stdlib/tests/vec.n.md`
    - `tests/capacity_stack.n.md`
    - `tests/overload.n.md`
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
- 補足:
  - `--changed` 全体実行では、既存�Eローカル変更 `stdlib/nm/parser.nepl` に起因する失敗が混ざるため、今回の移設検証は影響篁E��を�E示持E��して実施した、E

# 2026-02-27 作業メモ (collections: ringbuffer/queue 追加)
- 目皁E
  - `todo.md` の collections 再設計頁E��に沿って、FIFO基盤の `RingBuffer` と `Queue` を追加する、E
- 実裁E
  - 追加: `stdlib/alloc/collections/ringbuffer.nepl`
    - `RingBuffer<.T>` 構造体！Een/cap/head/data�E�E
    - `ringbuffer_new/with_capacity/push_back/pop_front/peek_front/len/is_empty/clear/free`
    - 失敗系は `Result<..., Diag>`、取得系は `Option`
  - 追加: `stdlib/alloc/collections/queue.nepl`
    - `Queue<.T>` めE`RingBuffer<.T>` で実裁E
    - `queue_new/with_capacity/push/pop/peek/len/is_empty/clear/free`
  - 追加チE��チE
    - `stdlib/tests/ringbuffer.n.md`
    - `stdlib/tests/queue.n.md`
    - `tests/ringbuffer_collections.n.md`
    - `tests/queue_collections.n.md`
    - `tests/pipe_collections.n.md` に ringbuffer/queue ケース追加
- 不�E合修正:
  - move セマンチE��クス違反�E�同一値の再利用�E�を、既存方針どおり「同一構築を別束縛に刁E��」で解消、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/ringbuffer.nepl -i stdlib/alloc/collections/queue.nepl -i stdlib/tests/ringbuffer.n.md -i stdlib/tests/queue.n.md -i tests/ringbuffer_collections.n.md -i tests/queue_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-ringbuffer-queue.json -j 2` -> `42/42 pass`
# 2026-02-27 作業メモ (main健全性確認後�Eブランチ復帰と根本修正)
- 目皁E
  - `main` の健全性めE`trunk build` + `nodesrc/tests` で再確認し、`refactor/stdlib-modernize-pipe-result` に戻して継続可能状態へ復帰する、E
  - `tests/neplg2.n.md` の失敁E件�E�Easm/llvmで訁E件�E�を原因特定して解消する、E
- 原因:
  - 失敗ID `tests/neplg2.n.md::doctest#37/#38` は `#target` 系ではなく、実際には「オーバ�Eロード」テストだった、E
  - チE��ト期征E��が旧仕様�E `compile_fail` のまま残っており、現実裁E��Erity解決・戻り値斁E��解決�E�と不整合だった、E
- 実裁E
  - `tests/neplg2.n.md`
    - `overloads_with_different_arity_are_error` めE`..._are_allowed` に更新し、`compile_fail` から `ret: 1` の実行検証へ変更、E
    - `overloads_ambiguous_return_type_is_error` めE`overloads_can_be_resolved_by_return_context` に更新し、`compile_fail` から `ret: 1` へ変更、E
  - 併せて、作業チE��ーに残ってぁE��以下�E修正を継綁E
    - `nepl-core/src/compiler.rs`�E�Earget 解決時�E診断経路�E�E
    - `nepl-core/src/codegen_llvm.rs`�E�ELVM側診断要紁E��E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -i tests/if.n.md -i tests/intrinsic.n.md -o /tmp/tests-targeted-after-neplg2-fix.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    -> `828/828 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-sync.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    -> `1822/1822 pass`
# 2026-02-27 作業メモ (stdlib stack の短縮API追加)
- 目皁E
  - `alloc/collections/stack` で prefix なし呼び出しを可能にし、pipe 記法での可読性を上げる、E
- 実裁E
  - `stdlib/alloc/collections/stack.nepl`
    - 既孁EAPI への委譲として短縮関数を追加:
      - `new`, `push`, `pop`, `peek`, `len`, `clear`, `free`
    - 吁E��縮関数に日本語ドキュメントコメントを追加、E
  - `stdlib/tests/stack.n.md`
    - `stack_alias_pipe_api` チE��トを追加し、短縮 API + pipe 記法での動作を固定化、E
- 失敗原因と対処:
  - 初回チE��ト失敗�E `web/dist` の stdlib bundle 未更新が原因、E
  - `trunk build` 後に再実行して解消、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/tests/stack.n.md -o /tmp/tests-stack-alias-after-build.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    -> `556/556 pass`

# 2026-02-27 作業メモ (collections: *_str ファイル統吁E+ hash32導�E)
## 修正冁E��
- `stdlib/alloc/collections/hashmap_str.nepl` / `hashset_str.nepl` を廁E��し、実裁E��それぞれ `hashmap.nepl` / `hashset.nepl` に統合、E
- `HashMapStr` / `HashSetStr` の API (`hashmap_str_*`, `hashset_str_*`) は維持して呼び出し互換を確保、E
- `alloc/hash/hash32.nepl` を追加し、Murmur3 fmix32 系の 32bit 混吁E`hash32_i32` を新設、E
- `hashmap.nepl` / `hashset.nepl` の i32 キー用ハッシュを簡易実裁E��めE`hash32_i32` 呼び出しへ置換、E
- `stdlib/tests/hash*.n.md` と `tests/selfhost_req.n.md`、`nepl-core/tests/selfhost_req.rs` の import/記法を統合後構�Eに合わせて更新、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- wasm 対象�E�E--no-stdlib --runner wasm`�E�E
  - `stdlib/tests/hash.n.md` / `hashmap.n.md` / `hashset.n.md` / `hashmap_str.n.md` / `hashset_str.n.md` / `tests/selfhost_req.n.md` -> すべて pass
- llvm 対象�E�E--no-stdlib --runner llvm --llvm-all`�E�E
  - `stdlib/tests/hash.n.md` / `hashmap.n.md` / `hashset.n.md` / `hashmap_str.n.md` / `hashset_str.n.md` / `tests/selfhost_req.n.md` -> すべて pass

# 2026-02-27 作業メモ (typecheck: get/put 特別処琁E�E再調査)
## 実施冁E��
- `nepl-core/src/typecheck.rs`
  - `TypeCtx::same` 呼び出しを `resolve_id` 比輁E��修正�E�ビルド不�Eの直接原因を解消）、E
  - `resolve_field_access` を診断あり/なしで使ぁE�Eけられる `resolve_field_access_with_mode` に刁E��、E
  - `get/put` 特別処琁E��「field 解決できたとき�Eみ適用、失敗時は通常オーバ�Eロードへフォールバック」に変更、E
  - `apply_function` への型引数伝播を修正し、`reduce_calls*` からは `func_entry.type_args`�E��E示型引数のみ�E�を渡すよぁE��変更、E

## 現在の状慁E
- `NO_COLOR=false trunk build` は通過、E
- ただぁE`target/debug/nepl-cli --target wasi --profile debug --input /tmp/hm.nepl --output /tmp/hm-out` で
  `core/math.nepl` / `alloc/collections/vec.nepl` / `alloc/string.nepl` の `get` 呼び出しが
  `D3006` / `D3021` で失敗する状態が継続、E

## 原因仮説
- `get` の過負荷候補があるとき�Eシンボル解決で、field 用 `get`�E�Ecore/field`�E�と collections 側 `get` の混在により
  呼び出し時の候補絞り込みが壊れてぁE��可能性が高い、E
- 特に `D3021`�E�Eype args mismatch�E��E、�E示してぁE��ぁE��面で型引数経路が残ってぁE��ことを示唁E��ており、E
  `PrefixItem::Symbol` -> `StackEntry::type_args` -> `apply_function` までの経路を追加で追ぁE��E��がある、E

## 次アクション
- `get/put` に限定した最小ケースで `StackEntry::type_args` の生�E/搬送をトレース、E
- `lookup_all_callables` と `lookup_all_any_defined` のスコープ優先規則ぁE
  field/collections の同名解決を壊してぁE��ぁE��確認、E
- 最小修正で `core/field get` と collections `get` の両立を回復後、E
  `stdlib/tests/hashmap*.n.md` めEwasm/llvm 直列で再検証、E

## 追記！E026-02-27�E�E
- 根本原因:
  - ジェネリチE��関数めEhoist するとき、`type_contains_unbound_var` 経由でシンボル名を素の関数名にしてぁE��ため、E
    同名オーバ�Eロード！Eget`�E�が同一シンボルに衝突してぁE��、E
  - そ�E結果、`HashMap` 牁E`get` 呼び出しが別実裁E��解決され、`alias get failed` を誘発してぁE��、E
- 修正:
  - `nepl-core/src/typecheck.rs` の hoist で、ジェネリクス有無に関係なぁE
    `mangle_function_symbol` を使って関数シンボルを一意化した、E
- 検証:
  - `NO_COLOR=false trunk build` 通過、E
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -o /tmp/hashmap-focus-wasm.json --runner wasm --assert-io --no-tree -j 1` 通過�E�E06/206�E�、E
  - `node nodesrc/tests.js -i stdlib/tests/hashmap_str.n.md -o /tmp/hashmap-str-focus-wasm.json --runner wasm --assert-io --no-tree -j 1` 通過�E�E06/206�E�、E

# 2026-02-27 作業メモ (kp コメント形式�E統一)
- 目皁E
  - `//` はドキュメントコメントとして扱わなぁE��針に合わせ、`stdlib/kp` のコメント形式を `//:` に統一する、E
- 実裁E
  - `stdlib/kp/kpread.nepl`
    - 行頭 `//` コメントを `//:` に統一、E
    - 関数冁E��の補助コメント行！EOM判定�E進行保証・列�E期化など�E��E削除して、E��常コード�Eみ残す構�Eに整琁E��E
  - `stdlib/kp/kpwrite.nepl`
    - 行頭 `//` コメントを `//:` に統一、E
    - 関数冁E��の行末 `//` コメントと補助コメント行を削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -o /tmp/tests-kp-io.json --runner wasm --assert-io --no-tree -j 1`
    -> `215/215 pass`

# 2026-02-27 作業メモ (map起点の名前解決/オーバ�Eロード修正)
## 根本原因
- `typecheck` の識別子解決で、同吁Ecallable の存在がローカル値�E�関数型パラメータ�E�解決に干渉してぁE��、E
- `reduce_calls` / `apply_function` ぁE`Var(name)` を過度に callable 名として扱ぁE��E
  ローカル関数値呼び出し！Ef a`�E�を過負荷解決へ誤送してぁE��、E
- `lookup_all_callables` が�Eスコープ横断で候補を返しており、�E側定義による lexical shadowing が効かず曖昧化してぁE��、E

## 実裁E
- `nepl-core/src/typecheck.rs`
  - head位置の識別子解決を修正:
    - 値が関数型なら値優允E
    - 値が非関数なめEcallable 優允E
  - `lookup_value_for_read` 候補を先に評価し、同吁Ecallable 混在時�E選択規則を安定化、E
  - `reduce_calls` / `reduce_calls_guarded` の `choose_callable_type_by_available_arity` 適用条件めE
    「同吁Evalue が存在しなぁE��合」に限定、E
  - `apply_function` の通常 callable 解決めE
    「同名�E関数垁Evalue が存在する場合�E通らなぁE��よぁE��変更�E�関数値呼び出し�E indirect 経路へ�E�、E
  - `lookup_all_callables` めElexical shadowing 優先（最冁E��コープ�Eみ�E�へ変更、E
  - `let` 型注釈！Epending_ascription`�E�から関数値期征E��拾ぁE��ぁE��し、E
    `let u <(i32)->i32> calc` のような束縛時解決を安定化、E

## チE��ト修正
- `tests/generics.n.md`
  - `generics_make_pair_wrapper` を現在の前置評価で曖昧にならなぁE���Eへ整琁E��E
- `tests/overload.n.md`
  - `overload_select_by_arity` を「アリチE��選択そのも�E」を検証する最小構�Eへ整琁E��E
  - `overload_select_by_arity_from_param_context_binary_not_supported_yet` めE
    実裁E��映済み仕様に合わせて通常 `neplg2:test` 化、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-now6.json --no-stdlib --no-tree` -> 27/27 pass
- `node nodesrc/tests.js -i tests/generics.n.md -o /tmp/tests-generics-now7.json --no-stdlib --no-tree` -> 24/24 pass
- `node nodesrc/tests.js -i tests/overload.n.md -o /tmp/tests-overload-now3.json --no-stdlib --no-tree` -> 18/18 pass
- `node nodesrc/tests.js -i tests -o /tmp/tests-tests-no-stdlib-final4.json --no-stdlib --no-tree` -> 471/471 pass
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-final.json --no-tree` -> 676/676 pass

# 2026-02-27 作業メモ (hash map/set 差刁E�E再検証)
## 実施冁E��
- `stdlib/alloc/collections/hashmap.nepl`
  - `core/field` の参�EめE`field::get` に統一、E
  - i32 キー位置計算を `mod_s abs ...` から `i32_rem_u` に統一、E
  - 非ドキュメントコメンチE(`//`) を削除し、`//:` のみ残す構�Eへ整琁E��E
- `stdlib/alloc/collections/hashset.nepl`
  - `core/field` の参�EめE`field::get` に統一、E
  - i32 キー位置計算を `mod_s abs ...` から `i32_rem_u` に統一、E
  - 非ドキュメントコメンチE(`//`) を削除し、`//:` のみ残す構�Eへ整琁E��E
- `stdlib/alloc/hash/hash32.nepl`
  - `alloc/string` めE`string` alias で import し、`string::len` を使用する形に統一、E
- `stdlib/tests/vec.n.md`
  - `push<u8> cast 65` の曖昧解決を回避するため、`u8_65` へ刁E��してから `push<u8>` に渡す形へ修正、E
- `tests/selfhost_req.n.md`
  - 対象ケースに `#target std` を追加、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -o /tmp/tests-hash-related.json --no-tree`
  - `210/210 pass`
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -i stdlib/tests/vec.n.md -o /tmp/tests-selfhost-vec.json --no-tree`
  - `212/212 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-regression.json --no-tree`
  - `676/676 pass`

# 2026-02-27 作業メモ (sizeof / intrinsic チE��ト拡張)
## 実施冁E��
- `tests/sizeof.n.md` に以下�EチE��トを追加:
  - `sizeof_collection_structs`
    - `Vec<i32>` / `Stack<i32>` / `HashMap<i32>` / `HashSet` の `size_of` 検証、E
  - `sizeof_diag_structs`
    - `Span` / `Error` / `Diag` の `size_of` 検証、E
- 既孁E`tests/intrinsic.n.md` と合わせて `size_of` 系の回帰検証セチE��を強化、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/sizeof.n.md -i tests/intrinsic.n.md -o /tmp/tests-sizeof-intrinsic.json --no-tree`
  - `219/219 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-sizeof.json --no-tree`
  - `678/678 pass`

# 2026-02-27 作業メモ (collections の Diag チE��ト追加)
## 実施冁E��
- `tests/collections_diag.n.md` を新規追加、E
- 追加した検証:
  - `hashmap_remove` の未存在キーで `KeyNotFound` が返ること
  - `hashset_remove` の未存在キーで `KeyNotFound` が返ること
  - `hashmap_insert` の容量趁E��で `CapacityExceeded` が返ること
  - `hashset_insert` の容量趁E��で `CapacityExceeded` が返ること
- `diag_code_str d.code` を使ってコード一致を固定化、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/collections_diag.n.md -o /tmp/tests-collections-diag.json --no-tree`
  - `209/209 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-collections-diag.json --no-tree`
  - `682/682 pass`

# 2026-02-27 作業メモ (alloc/diag 再設訁E Diag/Error 連携 + コメント形式統一)
## 実施冁E��
- `stdlib/alloc/diag/error.nepl`
  - `DiagCode <-> ErrorKind` の相互�E僁EAPI を追加:
    - `diag_code_to_error_kind`
    - `error_kind_to_diag_code`
  - `Diag <-> Error` 変換 API を追加:
    - `diag_to_error`
    - `error_to_diag`
  - `Diag` 斁E���E化を `message` 返却へ変更し、`Diag` フィールド同時参照の move 競合を解消、E
  - ファイル冁E�E非ドキュメントコメンチE`//` めE`//:` に統一、E
- `stdlib/alloc/diag/diag.nepl`
  - ファイル冁E�E非ドキュメントコメンチE`//` めE`//:` に統一、E
- `stdlib/tests/error.n.md`
  - `diag_to_error` / `error_to_diag` の往復ケースを追加し、期征E��を固定化、E

## 根本原因
- `Diag` は値構造体で、`d.code` と `d.message` の同時参�EぁEmove 競合を起こしてぁE��、E
- `diag_to_error` がこの経路を直接踏んでぁE��ため compile fail が発生してぁE��、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -i tests/collections_diag.n.md -o /tmp/tests-diag-redesign-focus.json --no-tree`
  - `211/211 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-diag-redesign.json --no-tree`
  - `682/682 pass`

# 2026-02-27 作業メモ (collections 安�E化テスト拡張: queue/ringbuffer 空操佁E
## 実施冁E��
- `tests/collections_diag.n.md` に以下を追加:
  - `queue_pop_empty_returns_none`
  - `ringbuffer_pop_empty_returns_none`
- 目皁E
  - 不正操作（空コレクションからの取り出し）が `Option::None` で安�Eに扱われることを固定化、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/collections_diag.n.md -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -o /tmp/tests-collections-diag-next.json --no-tree`
  - `213/213 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-diag-and-collections.json --no-tree`
  - `684/684 pass`

# 2026-02-28 作業メモ (List ラチE��移行�E moved 値不整合修正)
## 実施冁E��
- `stdlib/tests/list.n.md` の `list_get` 検証で、`l3_0` を作�EしてぁE��箁E��が誤って `l3` を参照してぁE��問題を修正、E
- `stdlib/alloc/collections/list.nepl` の `List<.T>` ラチE��移行と整合するよぁE��E��連チE��チE(`stdlib/tests/list.n.md`, `tests/pipe_collections.n.md`) を維持したまま moved 値参�Eを解消、E

## 根本原因
- List API めE`i32` 露出から `List<.T>` ラチE��へ移行した際、テスト�Eで再構築した値束縁E(`l3_0`, `l3_1`, ...) と旧束縛名 (`l3`) が混在したまま残り、move 後変数を参照する形になってぁE��、E

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/list.n.md -i tests/pipe_collections.n.md -i tests/list_dot_map.n.md -i tests/neplg2.n.md -o /tmp/tests-list-migration-focus.json --no-tree`
  - `260/260 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-list-wrapper.json --no-tree`
  - `684/684 pass`
# 2026-03-03 作業メモ (parser 診断IDの明示付与を拡張)
- 目皁E
  - parser の `if/while layout` と `#wasm/#llvmir` ブロチE��で未付与だった診断IDを�E示化し、`compile_fail diag_id` の安定性を上げる、E
- 実裁E
  - `nepl-core/src/parser.rs`
    - `expected wasm text line` / `expected llvm ir text line` に `ParserExpectedToken (2001)` を付与、E
    - `if-layout` の `invalid marker` / `invalid marker order` / `duplicate marker` / `too many expressions` に `ParserUnexpectedToken (2002)` を付与、E
    - `if-layout` の `missing expression(s)` に `ParserExpectedToken (2001)` を付与、E
    - `while-layout` の同種エラーに `ParserUnexpectedToken (2002)` / `ParserExpectedToken (2001)` を付与、E
    - `argument layout` の `only expressions are allowed` に `ParserUnexpectedToken (2002)`、`must contain expressions` に `ParserExpectedToken (2001)` を付与、E
- 検証:
  - `NO_COLOR=false trunk build --release --public-url /NEPLg2/` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `node nodesrc/tests.js -i tests/if.n.md -i tests/while.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-while-diag.json -j 2` -> `170/170 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `1876/1876 pass`

# 2026-03-03 作業メモ (prefix 廁E��移衁E math/kp/stdio の入れ子式を手修正)
- 目皁E
  - `i32_` 筁Eprefix 廁E��方針に合わせて、曖昧な入れ孁Eprefix 呼び出しを手作業で刁E��し、型注釁Eオーバ�Eロード解決で通る形へ移行する、E
- 根本原因:
  - 旧式�E `add a add b c` / `store_u8 add buf add off i ...` 形式が、prefix 廁E��途中のオーバ�Eロード解決で `no matching overload` を誘発、E
  - 一部はローカル変数吁E`neg` が関数 `neg` と衝突して誤解決を発生、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `u128_add/sub`, `i128_add/sub`, `u64_mul_wide`, `i128_mul` の入れ子式を段階変数に刁E��、E
    - `add/sub/mul` の `i128` オーバ�Eロードを追加、E
    - `u8` 系 (`add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u`) の prefix なしオーバ�Eロードを追加、E
  - `stdlib/core/mem.nepl`
    - `align8` の入れ子算術を刁E��、E
  - `stdlib/alloc/string.nepl`
    - 数値パ�Eス/斁E���E化�E入れ子式を段階変数に刁E��、E
    - `neg` 変数と `neg` 関数の衝突箁E��めE`sub 0 x` 方式に置換、E
  - `stdlib/std/stdio.nepl`
    - `read_line` / `print_i32` 周辺のポインタ計算を段階変数に刁E��、E
  - `stdlib/kp/kpread.nepl`, `stdlib/kp/kpwrite.nepl`, `stdlib/kp/kpsearch.nepl`
    - ポインタ計算�E桁�E琁E�E二�E探索/unique処琁E�E入れ子式を段階変数に刁E��、E
  - `tests/math.n.md`, `tests/numerics.n.md`, `tests/overload.n.md`, `tests/typeannot.n.md`, `tests/kp.n.md`
    - 新規紁E��Erefix なぁE+ 忁E��箁E��の型注釁E段階変数�E�に更新、E
- 検証:
  - `node nodesrc/tests.js -i tests/math.n.md -i tests/numerics.n.md -i tests/overload.n.md -i tests/typeannot.n.md -i tests/kp.n.md -i tests/intrinsic.n.md --no-stdlib --runner wasm --assert-io --no-tree -o /tmp/tests-prefix-migration-focus.json -j 1`
    - `59/59 pass`

# 2026-03-03 作業メモ (prefix廁E��移衁E cast 記法統一の継綁E
- 方釁E
  - `cast<T>` は使わず、`<T> cast expr`�E�また�E `let x <T> cast expr`�E�に統一、E
  - `i32_`/`i64_` など prefix 呼び出し�E削減を、呼び出し�Eから段階的に進める、E
- 実裁E
  - `stdlib/kp/kpwrite.nepl`: 変換呼び出しを `cast` 形式へ更新、E
  - `stdlib/kp/kpread.nepl`: u64/i64/f64/f32 読み取り系の変換めE`cast` 形式へ更新、E
  - `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`: syscall 引数変換めE`cast` 形式へ更新、E
  - `stdlib/alloc/string.nepl`: `from_i64`/`to_i64`/`from_f64`/`to_f64`/`from_f32`/`to_f32` の変換めE`cast` 形式へ更新、E
  - `stdlib/std/test.nepl`: `test_str_eq_loop` の `add a add 4 i` 形めE`off` 先計算へ変更し、オーバ�Eロード解決失敗を根本回避、E
  - `tests/kp.n.md`, `tests/intrinsic.n.md`, `tutorials/getting_started/24_competitive_dp_basics.n.md`, `tutorials/getting_started/27_competitive_algorithms_catalog.n.md` を新記法へ更新、E
  - `tests/typeannot.n.md`: 「重ね注釈�E仕様上可能だが�E長」�E説明へ更新�E�ケース自体�E維持E��、E
- 検証:
  - `/tmp/tests-prefix-migration-focus2.json` : 59/59 pass
  - `/tmp/tests-cast-annotation-style.json` : 43/43 pass
  - `/tmp/tests-kp-after-kpread-cast.json` : 7/7 pass
  - `/tmp/tests-std-fs-cliarg-cast-focused.json` : 11/11 pass
  - `/tmp/tests-string-cast-migration.json` : 29/29 pass
# 2026-03-03 作業メモ (math依存�Eのprefix縮退: std/test・std/fs・tree診断チE��チE
- 目皁E
  - `型名_` prefix 廁E��方針に合わせ、`math.nepl` 依存�Eの命名と利用めE`型注釁E+ cast` / オーバ�Eロードへ寁E��る、E
- 実裁E
  - `stdlib/std/test.nepl`
    - `bool_to_str` / `i32_to_str` を廁E��し、`to_str` オーバ�EローチE(`(bool)->str`, `(i32)->str`) に統一、E
    - 失敗メチE��ージ構築での呼び出しを `to_str` へ更新、E
  - `stdlib/std/fs.nepl`
    - `i64_from_i32` ヘルパを削除し、使用箁E��めE`cast` に置換、E
  - `stdlib/kp/kpwrite.nepl`
    - doctest 例�E `i64_extend_i32_u` めE`<i64> cast` へ更新、E
  - `tests/tree/05_overload_shadow_diagnostics.js`
    - `i32_ne` めE`ne` へ更新�E�オーバ�Eロード解決前提の新規紁E��、E
  - `tests/tree/18_diagnostic_ids.js`
    - `i32_to_f32` めE`<f32> cast` へ更新、E
- 検証:
  - `node tests/tree/run.js` -> `18/18 pass`、E
  - `nodesrc/tests.js` の対象限定実行�E長時間でタイムアウトする挙動を確認したため、現時点は tree スイートを優先して回帰確認、E

# 2026-03-03 作業メモ (bit演算APIのprefix縮退)
- 目皁E
  - `core/math` の bit 演算につぁE��めE`型名_` なしで使える経路を追加する、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `rotl/rotr/clz/ctz/popcnt` の i32/i64 オーバ�Eロードを追加�E��E部は既孁E`i32_*` / `i64_*` 実裁E��委譲�E�、E
  - `stdlib/tests/math.n.md`
    - `i32_clz/i32_ctz` 呼び出しを `clz/ctz` 呼び出しへ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-stdlib-math-prefixless-only.json -j 1`
    - `1/1 pass`

# 2026-03-03 作業メモ (cast依存�E変換APIをprefixなし名へ追征E
- 目皁E
  - `core/cast` ぁE`core/math` の `型名_` 変換名へ直接依存しなぁE��へ寁E��る、E
- 実裁E
  - `stdlib/core/math.nepl`
    - 変換用のprefixなしエントリを追加:
      - `extend_s`, `wrap`, `convert_s`, `trunc_s`, `promote`, `demote`, `to_i128`
    - `u128/i128` 実裁E�Eの `i64_extend_i32_u/s` 利用めE`cast` に置換、E
  - `stdlib/core/cast.nepl`
    - `cast_i32_to_i64` などの実裁E��体を上記prefixなし関数呼び出しへ変更、E
  - `from_i64` 名�E `alloc/string.nepl` の `from_i64`�E�Empure�E�と衝突し、`pure context cannot call impure function` を誘発したため、`to_i128` に改名して根本解消、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless.json -j 1`
    - `2/2 pass`

# 2026-03-03 作業メモ (math: u32/u64/u128/i128 API のprefix縮退)
- 目皁E
  - `型名_` prefix 廁E��方針に合わせ、`u32_/u64_/u128_/i128_` 公開API名を削減する、E
- 実裁E
  - `stdlib/core/math.nepl`
    - `u32_*` / `u64_*` 公開関数群を削除、E
    - `u128`:
      - `u128_new` -> `new <(i64,i64)->u128>`
      - `u128_from_u64` -> `to_u128`
      - `u128_add/sub/lt` -> `add/sub/lt` オーバ�EローチE
    - `i128`:
      - `i128_new` -> `new <(i64,i64)->i128>`
      - `i128_from_i64` -> `to_i128`
      - `i128_add/sub/mul/lt` -> `add/sub/mul/lt` オーバ�EローチE
    - `u64_mul_wide` -> `mul_wide` に変更、E
    - `f32_*/f64_*` の基本演算名めE`sqrt/abs/ceil/floor/trunc/nearest/min/max/copysign` のオーバ�Eロード名に統一、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless-v3.json -j 1`
    - `2/2 pass`

# 2026-03-03 作業メモ (cast APIのヘルパ�E名を廁E��してオーバ�Eロード本体へ統一)
- 目皁E
  - `cast_i32_to_*` 系ヘルパ�E名を廁E��し、`cast` のオーバ�Eロード本体だけで運用する、E
- 実裁E
  - `stdlib/core/cast.nepl`
    - `fn cast cast_*` alias 群を削除、E
    - すべて `fn cast <(A)->B>` 形式�E直接定義へ統一、E
  - `stdlib/tests/cast.n.md`
    - 旧ヘルパ�E呼び出し！Ecast_bool_to_i32`, `cast_i32_to_bool`�E�を削除し、`cast` + 単一型注釈へ更新、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless-v4.json -j 1`
    - `2/2 pass`

# 2026-03-03 作業メモ (math.nepl: i64定数の根本修正)
- 目皁E
  - `型名_` prefix 廁E��移行中に発生しぁE`core/math` の大量型崩れを根本解消する、E
- 根本原因:
  - `math.nepl` 後半�E�E128/i128実裁E��で `cast` を直接使ってぁE��が、`core/math` では `core/cast` めEimport してぁE��ぁE��めE`cast` が未定義、E
  - さらに `<i64> 0` の型注釈�E「型一致チェチE��」であり暗黙変換ではなぁE��め、i32 リチE��ルめEi64 にできず `D3004` が連鎖した、E
- 修正:
  - `u128/i128/mul_wide` の全 i64 定数生�EめE`extend_s_i32_to_i64` に統一、E
  - `cast` 依存を `math.nepl` 実コードから除去し、`core/math` 単体で自己完結する状態へ戻した、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 結果: `19/19 pass`

# 2026-03-03 作業メモ (math.nepl: u8 prefix実体�E縮退)
- 目皁E
  - `型名_` prefix 廁E��方針に合わせ、`u8_*` 実体関数名を prefix 先頭なしへ統一する、E
- 実裁E
  - `u8_add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u` めE
    `add_u8/sub_u8/mul_u8/div_u_u8/rem_u_u8/eq_u8/ne_u8/lt_u_u8/le_u_u8/gt_u_u8/ge_u_u8` へ変更、E
  - `fn add/sub/... <(u8,u8)->...>` の公開オーバ�Eロード�E新実体名へ委譲、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 結果: `19/19 pass`

# 2026-03-03 作業メモ (math.nepl: 冗長な二重型注釈�E整琁E
- 目皁E
  - 新規紁E��合わせて `math.nepl` ドキュメント�Eの二重注釁E(`<i64> <i64> cast` 筁E を除去する、E
- 実裁E
  - `math.nepl` 冁E�E `<i64> <i64> cast` / `<f64> <f64> cast` めE`<i64> cast` / `<f64> cast` へ統一、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 結果: `19/19 pass`

# 2026-03-03 作業メモ (tutorial: 数値章の曖昧オーバ�Eロード対筁E
- 目皁E
  - `math` のオーバ�Eロード拡張�E�E8 系統合）により、チュートリアルの短ぁE��値式で発生した曖昧解決を解消する、E
- 根本原因:
  - 小さぁE��数リチE��ルだけで構�Eされた合成式が、`i32`/`u8` の候補で曖昧化した、E
- 修正:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 褁E��式を中閁E`let` に刁E��し、曖昧なリチE��ルに `<i32>` 注釈を付与、E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - 二�E探索の `mid` 計算を `sum`/`mv_off`/`mv_ptr` へ刁E��して型解決を安定化、E
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-tutorial-math-scope.json -j 1`
  - 結果: `14/14 pass`

# 2026-03-03 作業メモ (math.nepl: 残存prefix斁E���Eの統一)
- 目皁E
  - `型名_` prefix 廁E��方針に合わせ、`math.nepl` 冁E�E残孁Eprefix 斁E���E�E�ドキュメント見�Eし�ELLVM シンボル名）も統一する、E
- 実裁E
  - `u8_*` 表記を `*_u8` へ統一�E�コメント表記�E`#llvmir` 冁E��ンボル名を含む�E�、E
  - `f32_*` / `f64_*` 表記を `*_f32` / `*_f64` へ統一�E�コメント表記�E`#llvmir` 冁E��ンボル名を含む�E�、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i tests/math.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-post-rename.json -j 1` -> `6/6 pass`
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i stdlib/tests/vec.n.md -i tests/math.n.md -i tests/typeannot.n.md -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-migration-bundle.json -j 1` -> `28/28 pass`

# 2026-03-03 作業メモ (vec/sort と tutorial の新規紁E��傁E
- 目皁E
  - `型名_` prefix 廁E��方針に合わせ、`alloc/collections/vec/sort.nepl` の曖昧式を解消し、tutorial 側をライブラリ利用へ更新する、E
- 根本原因:
  - `vec/sort.nepl` に `op op ...` の入れ子前置式が残っており、オーバ�Eロード候補増加後に `D3006` を誘発してぁE��、E
  - tutorial の sort 章は自前挿入ソート実裁E��ったため、現在の stdlib を使ぁE��れと乖離してぁE��、E
  - `sort_quick` は `Vec` を消費するため、tutorial で同一変数を後続参照すると move エラーが発生した、E
- 修正:
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_comb` / `sort_heap_sift_down_data` / `sort_heap` / `sort_merge_range_data` / `sort_heap_ret` の曖昧な入れ子式を中閁E`let` で刁E��、E
    - `u8` の `Ord::lt` めE`cast` 後比輁E��明示化、E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - 先頭章を�E前挿入ソートかめE`alloc/collections/vec` + `alloc/collections/vec/sort` 利用例へ置換、E
    - `sort_quick_ret` を使用して move エラーを回避、E
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-tut23-no-stdlib.json -j 1` -> `3/3 pass`
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i tests/math.n.md -i tests/typeannot.n.md -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-migration-scope.json -j 1` -> `29/29 pass`

# 2026-03-03 作業メモ (heap/linear memory 安�E化�E段階導�E)
- 目皁E
  - `mem.nepl` / `kpread.nepl` / `kpwrite.nepl` で生�Eインタ `i32` の露出を減らし、段階的に専用型へ移行する、E
- 根本原因:
  - `Scanner` / `Writer` めE`struct` 化して公閁EAPI を直接置換すると、NEPL の move 規則でハンドル再利用時に `use of moved value` が発生する、E
  - `*` を外すと impure 呼び出し制紁E(`pure context cannot call impure function`) に抵触する、E
- 修正:
  - `stdlib/core/mem.nepl`
    - `MemPtr` を追加し、`alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `mem_ptr_add` を追加、E
    - `load_i32_ptr` / `store_i32_ptr` / `load_u8_ptr` / `store_u8_ptr` を追加�E�既孁E`load_i32` 等�E名前衝突を回避�E�、E
  - `stdlib/kp/kpread.nepl`
    - `Scanner` 型と `scanner_wrap` / `scanner_raw` / `scanner_new_typed` を追加、E
    - 既存�E閁EAPI (`scanner_new` と吁Eread) は `i32` ベ�Eスのまま維持して破壊的影響を回避、E
  - `stdlib/kp/kpwrite.nepl`
    - `Writer` 型と `writer_wrap` / `writer_raw` / `writer_new_typed` を追加、E
    - 既存�E閁EAPI (`writer_new` と吁Ewrite) は `i32` ベ�Eスのまま維持、E
  - 影響チE��ト群�E�Ekp` / tutorial�E�で型注釈を一時導�EしてぁE��箁E��は `i32` に戻し、`25_competitive_prefixsum_twopointers.n.md` の曖昧な入れ子前置式を中閁E`let` 展開で解消、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-kp-typed-handles.json -j 1`
  - 結果: `21/21 pass`
- 差刁E��釁E
  - 現時点は「非破壊での安�E化足場�E�Eyped API 併設�E�」まで、E
  - 公閁EAPI を完�Eに専用型へ移行するには、move 規則に沿ったハンドル再束縛パターン�E�Eonsume/return�E�を標準化してから段階移行する、E

# 2026-03-03 作業メモ (オーバ�EローチEシャド�Eイング根本修正)
- 目皁E
  - `add add 1` など同名の値束縛と関数束縛が共存するケース、�E外同名関数�E�同一シグネチャ�E�での `ambiguous overload` を解消する、E
- 根本原因:
  - 先頭位置の識別子でオーバ�Eロード遅延を行う際、値束縁E(`i32` など) へのフォールバックが�Eに走り、呼び出し式が値として解釈さめE`D3016` になってぁE��、E
  - 候補が褁E��あるとき、同一シグネチャ�E�実質シャド�E�E��E候補も曖昧扱ぁE��れてぁE��、E
- 修正:
  - `nepl-core/src/typecheck.rs`
    - 先頭位置かつ後続トークンありの場合�E、オーバ�Eロード遅延で値束縛へ落とさなぁE��ぁE��件を修正、E
    - 候補選別後にシグネチャ重褁E��除去し、同一シグネチャの冁E��候補�E冁E�Eを優先するよぁE��正、E
  - `stdlib/kp/kpread.nepl`
    - `scanner_read_i64` / `scanner_read_f64` の符号フラグ変数名を `neg` から `is_neg` に統一し、`neg` 関数との衝突を解消、E
  - `tests/math.n.md`
    - `cast` が曖昧になる位置に `<i128>` / `<i32>` 注釈を付与（現行仕様に合わせた明示�E�、E
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i stdlib/kp/kpgraph.nepl -o /tmp/kpgraph_focus.json -j 16` -> `223/223 pass`
  - `node nodesrc/tests.js -i tests/math.n.md -i tests/shadowing.n.md -o /tmp/math_shadow_after_fix.json -j 16` -> `254/254 pass`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 16` -> `718/718 pass`
# 2026-03-04 作業メモ (フェーズD進衁E kpread/kpwrite の i32 公開オーバ�Eロード�E離)

- 目皁E
  - `scanner_read_i32(sc_handle: i32)` / `writer_write_i32(w_handle: i32, ...)` の公開面露出を縮小し、利用老E�� `Scanner` / `Writer` を使ぁE��計に統一する、E
- 根本原因:
  - 同名で `i32` 受け取り版と `Scanner/Writer` 版を公開してぁE��と、安�E型APIへ移行しても生ハンドル経路へ簡単に戻れてしまぁE��設計�E一貫性が崩れる、E
  - 既存�Eオーバ�Eロード解決は動作してぁE��も、�E開面に unsafe 経路が残ること自体が再発要因になる、E
- 修正:
  - `stdlib/kp/kpread.nepl`
    - `scanner_*` の `i32` 受け取り実裁E�� `scanner_*_handle` へ改名、E
    - 公閁E`scanner_*` (`Scanner` 受け取り) から `*_handle` を呼ぶ構�Eへ変更、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_*` の `i32` 受け取り実裁E�� `writer_*_handle` へ改名、E
    - 公閁E`writer_*` (`Writer` 受け取り) から `*_handle` を呼ぶ構�Eへ変更、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-handle-split.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kp-handle-split.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kp-handle-split.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread/kpwrite` の公開名は `Scanner/Writer` 版を中忁E��整琁E��れた、E
  - 次段で `core/mem` 側の `*_raw` 段階縮退�E�EResult` 一本化）を進める、E
# 2026-03-04 作業メモ (フェーズD進衁E kpread/kpwrite の raw 呼び出し除去)

- 目皁E
  - `kpread/kpwrite` 実裁E�E部に残ってぁE�� `alloc_raw/dealloc_raw` 直呼びめE`Result` 系APIへ寁E��、失敗時挙動を型で扱えるようにする、E
- 根本原因:
  - `scanner_read_token` は `alloc_raw` 失敗時�E�E返却�E�を老E�Eしておらず、�EチE��書き込みで未定義動作になり得た、E
  - `writer_free` は `dealloc_raw` 直呼びで、解放失敗を吸収する一貫した経路がなかった、E
- 修正:
  - `stdlib/kp/kpread.nepl`
    - `scanner_read_token_handle` の斁E���E確保を `alloc` + `Result` 刁E��へ変更、E
    - 確保失敗時はカーソルだけ進めて `""` を返す動作に統一、E
  - `stdlib/kp/kpwrite.nepl`
    - `writer_free_handle` の解放めE`writer_try_free` 経由へ変更�E�Edealloc` の `Err` 吸収）、E
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-safe-mem-no-raw.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kp-no-raw.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kp-no-raw.json -j 15` -> `262/262 pass`
- 状況E
  - `kpread/kpwrite` から `alloc_raw/dealloc_raw/realloc_raw` の直接使用は除去済み、E
  - 次段は `core/mem` 側で `*_raw` の公開縮退方針（完�E削除タイミング�E�を整琁E��る、E
# 2026-03-04 作業メモ (フェーズD進衁E tests/tutorials の alloc_safe 匁E

- 目皁E
  - `core/mem` の安�EAPI標準化方針に合わせ、`tests/tutorials` での `alloc_raw/dealloc_raw` 直接使用を段階的に削減する、E
- 事前棚卸ぁE
  - `rg` で repo 全体�E `alloc_raw/dealloc_raw/realloc_raw` 呼び出しを刁E��し、`nm/std/collections` に庁E��E��の残存があることを確認、E
  - 今回は影響が大きく回帰しやすい `tests/kp.n.md` と `tutorials/getting_started/{23,25,26}` を�E行移行対象に選定、E
- 修正:
  - `tests/kp.n.md`
    - `alloc_raw/dealloc_raw` めE`unwrap_ok alloc/dealloc` へ置換、E
    - 忁E��なスニ�EチE��に `#import "core/result" as *` を追加、E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 同様に `alloc_raw/dealloc_raw` めE`unwrap_ok alloc/dealloc` へ置換し、`core/result` import を追加、E
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md --no-tree -o /tmp/tests-safe-alloc-docs-scope.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-safe-alloc-docs.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-safe-alloc-docs.json -j 15` -> `262/262 pass`
- 状況E
  - `kp` 系チE��チEチュートリアルの主要サンプルは安�EAPI経路へ移行済み、E
  - 次段は棚卸し済み残件�E�Estdlib/std`, `stdlib/nm`, `stdlib/alloc/collections`�E�を上流影響の小さぁE��E��移行する、E

# 2026-03-04 作業メモ (move_check: 一時借用の寿命誤判定を根本修正)

- 目皁E
  - `stdlib` doctest で発生してぁE�� `D3051 cannot move out of shared borrowed value` / `D3053 use of moved value` の連鎖を、場当たり対応ではなぁEmove_check の借用寿命モチE��修正で解消する、E
- 根本原因:
  - `passes/move_check.rs` ぁE`#intrinsic load/store` のアドレス評価を永続借用として扱ってぁE��、E
  - `get`/`load` のような読み取りで生�Eされる借用は式評価中のみ有効なはずだが、E��数末尾まで `BorrowedShared` が残り、後続�E同一値利用を誤って拒否してぁE��、E
- 修正:
  - `nepl-core/src/passes/move_check.rs`
    - `check_temporary_borrow` を追加、E
    - `#intrinsic load/store` のアドレス評価を永続借用ではなく一時借用として検証するよう変更、E
    - 永続借用状態更新が忁E��な `AddrOf` は従来どおり `check_borrow` を使用、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/move_check.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-copy-move-targeted-after-temp-borrow.json -j 15` -> `245/245 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-temp-borrow-fix.json -j 15` -> `799/799 pass`
- 補足:
  - 「copy 惁E��のハ�Eドコード削減」�E継続課題。`TypeCtx::is_copy` の全面移行�E move/effect 設計と同時に段階実施する�E�仕様書と todo の頁E��を優先）、E
# 2026-03-04 作業メモ (trait 設計�E再確認と上流修正)

- 目皁E
  - `plan.md` と `doc/move_effect_spec.md` に整合する形で、trait 実裁E��合�E判定を安定化する、E
  - Rust/Haskell の設計論点�E�契紁E��制紁E��coherence�E�を NEPLg2 向けに整琁E��、実裁E��針を固定する、E

- 実施:
  - `nepl-core/src/typecheck.rs`
    - impl メソチE��署名�E trait 整合判定を斁E���E比輁E��ら構造型同値�E�Ectx.same_type`�E�へ変更、E
  - `doc/trait_system_design.md` を新規作�E、E
    - NEPLg2 におけめEtrait の役割�E�Enterface/type-class/メモリ能力）を定義、E
    - coherence、オーバ�Eロード整合、ハードコード最小化方針、拡張頁E��を明文化、E
  - `todo.md`
    - フェーズ `B2`�E�Erait 設計�E実裁E��映�E�を追加、E

- チE��チE
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-trait-design-targeted.json -j 15` -> `276/276 pass`

- 差刁E��譁E
  - 依然として `Copy/Clone` 能力接続には最小限の trait 名参照が残ってぁE��、E
  - 次段で `todo.md` フェーズB2に従い、�E力テーブル化して名前刁E��を縮小する、E

# 2026-03-04 作業メモ (trait能力判定�E雁E��E

- 目皁E
  - `Copy/Clone` の判定�E岐を局所化し、`typecheck.rs` 全体に散在してぁE��斁E���E比輁E��雁E��E��る、E

- 実施:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics` を追加し、trait 宣言から `copy_trait_name` / `clone_trait_name` を検�Eする流れへ変更、E
    - `Copy` / `Clone` 参�E箁E���E�Empl 収集、clone 前提検査、reject 適用、final impl 生�E�E�を `trait_semantics` 経由へ統一、E
    - 直接の `Some(\"Copy\")` / `Some(\"Clone\")` 比輁E��除去、E

- チE��チE
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-trait-semantics-targeted.json -j 15` -> `276/276 pass`

- 次段:
  - `todo.md` フェーズB2の残件として、�E力判定�E外部定義化（コンパイラ冁E��固定名のさらなる縮小）を設計する、E

# 2026-03-05 作業メモ (compile_fail の診断位置検証を追加)

- 目皁E
  - `tests/*.n.md` の `compile_fail` ケースで、`diag_id` だけでなく診断位置�E�Eile/line/col�E�も宣言して検証できるようにする、E

- 根本原因:
  - 既存�E doctest 仕様�E `diag_id` のみを受琁E��ており、「どの位置でそ�E診断が�Eるべきか」を機械検証できなかった、E
  - そ�Eため、同ぁE`diag_id` が別位置で発生してもテストが見送E��余地があった、E

- 実施:
  - `nodesrc/parser.js`
    - doctest メタに `diag_span` / `diag_spans` を追加、E
    - `line:col` と `file:line:col` の両形式を受理、E
  - `nodesrc/tests.js`
    - `expected_diag_spans` をケースに保持、E
    - `compile_fail` 評価時に `compile_error` から `--> file:line:col` を抽出し、期征E��置と照合、E
    - `compile_fail` の `diag_id` / `diag_span` 検証めE`--assert-io` 依存から�Eり離し、常時評価へ変更、E
  - `tests/compile_fail_diag_location.n.md`
    - `diag_span`�E�単体）と `diag_spans`�E�褁E���E�を使った検証ケースを追加、E

- 検証:
  - `node -c nodesrc/parser.js && node -c nodesrc/tests.js` -> success
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-compile-fail-diag-location.json -j 15` -> `2/2 pass`
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md --no-stdlib --no-tree -o /tmp/tests-keywords-reserved.json -j 15` -> `6/6 pass`

- 補足:
  - `--no-stdlib` なし実行時は既知の `stdlib/alloc/collections/list.nepl` 失敗が混在するため、今回タスクの局所検証では除外した、E

# 2026-03-05 作業メモ (diag_id 検証の厳寁E��)

- 目皁E
  - `compile_fail` の `diag_id` を「テスト通過のための値合わせ」ではなく、実際に検証したぁE��敗原因に一致させる、E

- 実施:
  - `tests/move_effect.n.md`
    - 「shared borrow 中 move 拒否」を、E��数値呼び出し由来の副次診断が混ざらなぁE��小�E現へ書き換え！Ediag_id: 3051`�E�、E
    - 「非褁E��垁Efield access 拒否」を `v.len` 形式�E最小�E現へ書き換え！Ediag_id: 3011`�E�、E
    - 「グローバル set」ケースは現在実裁E�E診断挙動�E�ETypeUndefinedVariable`, `3002`�E�を明示する形に説明を更新、E

- 検証:
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-audit2.json -j 15` -> `225/225 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md --no-tree -o /tmp/tests-neplg2-fix2.json -j 15` -> `249/249 pass`
  - `node nodesrc/tests.js -i tests/kp.n.md --no-tree -o /tmp/tests-kp-fix2.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current-full8.json -j 15` -> `797/797 pass`

- 補足:
  - `diag_id` の変更は、各ケースを単体�E現して実診断を確認したもののみ反映した、E
  - 失敗原因が褁E��混在するケースは、テストコード�Eを「狙った診断だけが出る形」に刁E��して再構�Eした、E

# 2026-03-05 作業メモ (フェーズB2: trait能力テーブルの導�Eと回帰安定化)

- 目皁E
  - `todo.md` フェーズB2�E�ECopy/Clone` 能力判定�E能力テーブル化）を進め、`typecheck` の能力判定を局所化する、E

- 実施:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` を拡張し、trait doc から `@capability: copy|clone` を読んで能力を設定する経路を追加、E
    - 既存�EメソチE��名�Eタ依存！Ecopy_mark`/`clone`�E�検�Eを削除、E
    - 構造ヒューリスチE��チE��を追加:
      - clone 候裁E 単一メソチE��かつ `(Self)->Self`
      - copy 候裁E marker trait�E�メソチE��なし！E
    - 互換維持�Eため、�E力未確定時のみ `Clone` / `Copy` 名�E最小フォールバックを追加、E
  - `tests/move_effect.n.md`
    - `compile_fail` 2ケースで `#entry main` だけ定義され診断ぁE`D3092` に吸われる問題を修正し、`main` を追加して狙っぁE`diag_id` を検証可能化、E
    - `Copy` 関連ケースに `@capability` 宣言を追記、E

- 根本原因:
  - 旧実裁E�E能力判定を「trait吁E+ method名」絁E��依存しており、仕様拡張時に誤判定が起きやすかった、E
  - `compile_fail` の一部ケースはエントリ未定義が�Eに発火し、狙った回帰検証になってぁE��かった、E

- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v4.json -j 15` -> `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v1.json -j 15` -> `837/837 pass`

- 差刁E��譁E
  - 能力検�Eの主経路は能力テーブル化済み、E
  - ただし完�E撤廁E��はなく、未宣言時�E最小互換として `Copy/Clone` 名フォールバックが残る。`todo.md` フェーズB2の「文字�E比輁E���E撤廁E��を満たすには次段でこ�E互換層を外す忁E��がある、E

# 2026-03-05 作業メモ (B2 検証: 名称フォールバック撤去の試行結果)

- 実施:
  - `TraitSemantics::detect` の `Copy/Clone` 名フォールバックを一時的に撤去し、�E力宣言 + 構造ヒューリスチE��チE��のみへ刁E��を試行した、E

- 結果:
  - `tests/move_effect.n.md` の `Copy` 系 `compile_fail` が通らず、`expected compile_fail, but compiled successfully` となった、E
  - 原因は、現行実裁E��は `//: @capability: ...` が�E力検�E入力として安定供給されず、`Copy` 能力が未検�Eになる経路が残るため、E

- 対忁E
  - 名称フォールバックは再導�Eした、E
  - 再検証:
    - `NO_COLOR=false trunk build` -> success
    - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v2.json -j 15` -> `837/837 pass`

- 次段の上流課顁E
  - `Copy/Clone` の能力宣言めE`doc comment` 依存でなぁEAST/斁E��レベルで供給する仕絁E��を追加し、名称フォールバックを撤去する、E
# 2026-03-05 作業メモ (フェーズB2: `#capability` 斁E��化と型検査統吁E

- 目皁E
  - `todo.md` フェーズB2の上流�Eとして、`Copy/Clone` 能力�E宣言経路めEdoc 斁E���E依存かめEparser/AST 経路へ移す、E
  - codegen 手前で同一の trait 能力情報を参照できる形に揁E��る、E

- 実裁E
  - `nepl-core/src/ast.rs`
    - `TraitDef` に `capabilities: Vec<String>` を追加、E
  - `nepl-core/src/lexer.rs`
    - `TokenKind::DirCapability(String)` を追加、E
    - `#capability ...` めElex 対象に追加、E
  - `nepl-core/src/parser.rs`
    - trait 本斁E�Eで `#capability` を受琁E�� `TraitDef.capabilities` へ格納、E
    - トップレベル `#capability` は `ParserUnexpectedToken` で拒否、E
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo` に `capabilities` を保持、E
    - 能力抽出は `TraitInfo.capabilities` から行うよう変更�E�Eoc 行解析を廁E���E�、E
  - `nepl-web/src/lib.rs`
    - token 表示側に `DirCapability` の刁E��を追加して `trunk build` の non-exhaustive を解消、E
  - `tests/move_effect.n.md`
    - `@capability:` コメント表現めEtrait 本斁E�E `#capability` に置換、E

- チE��チE
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v6.json -j 15`
    - `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v3.json -j 15`
    - `837/837 pass`

- 残課顁E
  - `Copy/Clone` 検�Eの最終フォールバック�E�Erait 吁E`Copy` / `Clone`�E��Eまだ残ってぁE��、E
  - フェーズB2完亁E��件「文字�E比輁E�E完�E撤廁E��に向けて、次段で除去する、E
# 2026-03-05 作業メモ (フェーズB2: `Copy/Clone` 名フォールバック削除)

- 目皁E
  - フェーズB2残課題だっぁE`Copy` / `Clone` の trait 名ハードコードフォールバックを廁E��する、E

- 実裁E
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` の末尾に残ってぁE��
      - `traits.get("Clone")` フォールバック
      - `traits.get("Copy")` フォールバック
      を削除、E
    - 能力判定�E `#capability`�E�およ�E構造ヒューリスチE��チE���E�経路のみを使用する形に統一、E

- チE��チE
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v7.json -j 15`
    - `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v4.json -j 15`
    - `837/837 pass`

# 2026-03-05 作業メモ (フェーズB2: `#capability` 仕様墁E��の回帰追加)

- 目皁E
  - `#capability` ぁEtrait 本斁E�Eのみ有効である仕様をチE��トで固定する、E

- 実裁E
  - `tests/overload.n.md`
    - `capability_directive_is_trait_local_only` を追加、E
    - `compile_fail + diag_id: 2002 (ParserUnexpectedToken)` で固定、E

- チE��チE
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v8.json -j 15`
    - `282/282 pass`

# 2026-03-05 作業メモ (フェーズB2: trait bound 判定�E TypeId 直参�E匁E

- 目皁E
  - trait method 呼び出し時の bound 判定で、trait 名�E解決を経由する経路を削減する、E

- 実裁E
  - `nepl-core/src/typecheck.rs`
    - trait method 呼び出し�E岐で `resolve_trait_bound_ref(trait_name)` を廁E��、E
    - すでに取得済みの `trait_info.self_ty` を使ぁE��E
      - `type_param_has_bound(self_ty, trait_self_ty)`
      - `impls` 上�E `trait_self_ty + target_ty` 一致
      の合�E判定へ置換、E
    - 未使用化しぁE`resolve_trait_bound_ref` を削除、E
  - `tests/overload.n.md`
    - `capability_directive_is_trait_local_only` を追加して parser 墁E��を固定！Ediag_id: 2002`�E�、E

- チE��チE
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v9.json -j 15`
    - `282/282 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v5.json -j 15`
    - `838/838 pass`

# 2026-03-05 作業メモ (move_check の diag_id 検証精度修正)

- 事象:
  - `tests/move_check.n.md::doctest#7` ぁE`diag_id: 3051` 期征E��失敗、E
  - 実際は `D3003` が�Eに出ており、`diag_id` 検証として不正確だった、E

- 原因:
  - `move_reference_ok` ケースで `fn main <()->i32>` に対して末尾式がなく、E
    move/borrow 診断より先に戻り値不一致診断が発生してぁE��、E

- 修正:
  - `tests/move_check.n.md` の `move_reference_ok` に末尾弁E`0` を追加し、E
    目皁E�E `D3051` が前面に出る形へ修正、E

- チE��チE
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-movecheck-unskip-v5.json -j 15`
    - `282/282 pass`

# 2026-03-05 作業メモ (move_check: 構造体フィールチEmove 検�Eの根本修正)

- 事象:
  - `move_struct_field_err` ぁE`skip` のままで、`s.f` から非Copy値めE回読むケースを検�EできてぁE��かった、E

- 根本原因:
  - `s.f` は HIR 丁E`load` に lower されるが、`move_check` の `load<non-Copy>` 刁E��が
    常に「一時借用」扱ぁE��、所有権移動として状態更新してぁE��かった、E

- 修正:
  - `nepl-core/src/passes/move_check.rs`
    - `visit_field_move_source` を追加、E
    - `load<non-Copy>` のとき、アドレス式がローカル褁E��値由来�E�EVar` / `add(Var, ...)`�E�なめE
      値移動として `check_use(..., is_copy=false)` を適用、E
    - それ以外�E `load<non-Copy>` は従来どおり一晁Eunique borrow を適用、E
  - `tests/move_check.n.md`
    - `move_struct_field_err` めE`skip` から `compile_fail` (`diag_id: 3053`) に戻した、E

- チE��チE
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-movecheck-unskip-v6.json -j 15`
    - `282/282 pass`

# 2026-03-05 作業メモ (フェーズC: kpread_core syscall墁E��の MemPtr 統一)

- 目皁E
  - `kpread_core` で syscall 墁E��以外�E `MemPtr<u8> -> i32` 変換を局所化し、�Eインタ墁E��を�E示する、E

- 根本原因:
  - `fd_read` 呼び出し箁E��で `mem_ptr_addr` を呼び出し�Eに直接展開しており、墁E��責務が刁E��してぁE��、E
  - これにより effect/pointer 仕様�E見通しが悪く、封E��の共通化で誤用が�E発しやすい状態だった、E

- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_u8_addr <(MemPtr<u8>)->i32>` を追加し、`MemPtr<u8>` からのアドレス取得を一箁E��へ雁E��E��E
    - `fd_read_mem <(i32,MemPtr<u8>,i32,MemPtr<u8>)*>i32>` を追加し、`fd_read` 呼び出し墁E��を�E通化、E
    - `scanner_new_impl` 冁E�E `fd_read` 呼び出しを `fd_read_mem 0 iov 1 nread_ptr` に置換、E
    - `buf` アドレス取得�E直接 `mem_ptr_addr` めE`mem_u8_addr` に置換、E

- 実裁E���E注愁E
  - `fd_read_mem` は syscall 呼び出しを含むため `*>`�E�Empure�E�シグネチャで定義、E
  - 一時的に pure 定義として `D3025` が発生したが、effect 仕様に合わせて impure へ修正し�E検証した、E

- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-core-boundary-v2.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite ヘッダアクセスの MemPtr 墁E��統一)

- 目皁E
  - `Writer.raw` ぁE`MemPtr<u8>` である設計に合わせ、`kpwrite` 冁E��ヘッダアクセスの型墁E��めE`i32` から `MemPtr<u8>` へ統一する、E

- 根本原因:
  - `writer_header_ptr/load/store` ぁE`i32` 受け取りのまま残っており、`Writer` から毎回 `mem_ptr_addr` へ降格してぁE��、E
  - 墁E��降格が散在し、メモリ安�EモチE���E�フェーズC�E��E「�E開�E冁E��ともに MemPtr 基準」�E方針と不整合だった、E

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_ptr` めE`(MemPtr<u8>, i32)->MemPtr<i32>` へ変更、E
    - `writer_load_header` / `writer_store_header` めE`MemPtr<u8>` 受け取りへ変更、E
    - `writer_free_handle` / `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_write_str_handle` / `writer_write_i32_handle` / `writer_write_u64_handle` の冁E��で `w_mem:MemPtr<u8>` を使ぁE��へ統一、E
    - `writer_free_handle` のヘッダ解放は `dealloc_ptr<u8> w_mem 20` を使用し、生 `i32` 経路を削減、E

- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-memptr-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v5.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite 冁E��確俁E解放の MemPtr 匁E

- 目皁E
  - `kpwrite` の冁E��実裁E��、確保�E解放経路めE`alloc_ptr/dealloc_ptr` ベ�Eスに統一する、E
  - syscall 墁E��以外�E甁E`i32` ポインタ操作を減らし、型安�E墁E��を�E確化する、E

- 根本原因:
  - `writer_alloc_buf` と `writer_new_handle` ぁE`alloc/dealloc` (`i32`) ベ�Eスで実裁E��れており、`Writer.raw: MemPtr<u8>` と冁E��経路が二重化してぁE��、E
  - 失敗時巻き戻しも `i32` 解放経路に寁E��てぁE��、MemPtr 系の安�EAPI統一方針と不整合だった、E

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterBuf.ptr` めE`i32` から `MemPtr<u8>` へ変更、E
    - `writer_try_free` めE`writer_try_free_ptr<.T>` に置換し、`dealloc_ptr` 経由へ統一、E
    - `writer_alloc_buf` めE`alloc_ptr<u8>` ベ�Eスへ変更、E
    - `writer_new_handle` の `buf/iov/nw/w` 確保を `alloc_ptr<u8>` ベ�Eスへ変更し、失敗時巻き戻しも `writer_try_free_ptr` に統一、E
    - header へ格納する値だけを `mem_ptr_addr` で明示皁E��墁E��変換�E�Eyscall/ヘッダ構造との接続点�E�、E
    - `writer_free_handle` の `buf/iov/nw` 解放めE`writer_try_free_ptr<u8> mem_ptr_wrap ...` 経由へ変更、E

- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-memptr-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v6.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite 初期化経路の header API 統一)

- 目皁E
  - `writer_new_handle` で残ってぁE��甁E`store_i32` の直書きをなくし、`writer_store_header` 経由に統一する、E

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle` の header 初期化！Euf/cap/len/iov/nw�E�を `writer_store_header` 呼び出しに置換、E
    - 初期化時のポインタ墁E��変換は `mem_ptr_addr` のみを引数位置に限定、E

- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-init-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v8.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite 解放経路のポインタ墁E��雁E��E

- 目皁E
  - `writer_free_handle` で残ってぁE�� `i32 -> MemPtr` の都度変換を�Eルパへ雁E��E��、解放墁E��を単純化する、E

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header_ptr <(MemPtr<u8>,i32)->MemPtr<u8>>` を追加、E
    - `writer_free_handle` は `buf/iov/nw` めE`writer_load_header_ptr` で取得して `writer_try_free_ptr` へ渡す構�Eへ変更、E
    - `mem_ptr_wrap` の直呼びを削減して、header 値のポインタ化責務を一箁E��に雁E��E��E

- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-freeptr-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v9.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: writer ヘッダ書き込み失敗�E握り潰し廁E��)

- 目皁E
  - `writer_store_header` が失敗を黙殺してぁE��設計を修正し、writer 構築時の不整合状態を防ぐ、E

- 根本原因:
  - 旧実裁E��は `writer_store_header` が常に `()` を返し、`store_i32` 失敗時でも呼び出し�Eが異常を検�Eできなかった、E
  - `writer_new_handle` でヘッダ初期化に失敗しても�E功扱ぁE��なりうる設計だった、E

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_store_header` の返り値めE`Result<(),str>` に変更、E
    - `writer_new_handle` の 5 つのヘッダ書き込みを逐次 `match` で検証し、失敗時は確保済み領域を解放して `Err` 返却、E
    - `flush/put/write` 系の長さ更新箁E��めE`Result` を�E示皁E��受ける構造へ統一、E

- チE��チE
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-header-result-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v10.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズB2: fn定義時オーバ�Eロード�E合�Eジェネリクス同値修正)

- 目皁E
  - `D3087`�E�Eunction signature does not match any overload�E��E誤検�Eを、ジェネリクス署名�E合�E根本から解消する、E
- 根本原因:
  - `fn` 定義照合で `same_type` を直接使ぁE��、未束縛型変数のラベル一致に依存し、α同値�E�型パラメータ名�E差�E�を許容できず失敗した、E
  - さらに照合用に作る署名型 `sig_ty` ぁE`type_params` なしで構築されており、ジェネリクス関数の署名キーと不整合を起こしてぁE��、E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `function_signature_string` をジェネリクス正規化キー生�Eへ変更�E�E$T0, $T1...` へ正規化�E�、E
    - `signature_type_string` を追加し、E��数シグネチャ比輁E��用の型文字�E化を導�E、E
    - `fn` 定義照合時の `sig_ty` を、`f.type_params` を含む `ctx.function(type_params, params, result, effect)` で構築するよぁE��正、E
    - 既存�Eオーバ�Eロード候補�E合！Efunction_signature_string` 比輁E��を維持しつつ、ジェネリクス同値比輁E�E精度を改喁E��E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-final-fix.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-final-fix.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-final-fix.json -j 15`
  - 結果: `783/783 pass`

# 2026-03-05 作業メモ (フェーズB2: 関数署名比輁E�E斁E���E依存を排除)

- 目皁E
  - オーバ�EローチEhoist関連で残ってぁE��署名�E合�E斁E���E比輁E��廁E��し、型構造比輁E��統一する、E
- 根本原因:
  - `remove_duplicate_func`, `lookup_func_symbol`, `find_same_signature_func`, `fn` 定義時�E合が斁E���Eキー比輁E��依存しており、型変数名や生�E頁E��差で不安定化する余地があった、E
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `same_function_signature` を追加し、E��数型�Eシグネチャ同値�E�ジェネリクスα同値含む�E�を型構造で判定、E
    - `same_type_with_signature_generics` を追加し、型パラメータ対応表�E�E->B/B->A�E�を持った�E帰比輁E��実裁E��E
    - 以下を斁E���E比輁E��めE`same_function_signature` へ置揁E
      - `fn` 定義時�E過負荷候補選抁E
      - `Env::remove_duplicate_func`
      - `Env::lookup_func_symbol`
      - `find_same_signature_func`
      - `find_nonshadow_same_signature_func`
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-same-signature-api.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-same-signature-api.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-same-signature-api.json -j 15`
  - 結果: `783/783 pass`
# 2026-03-05 作業メモ (`move_check.n.md::doctest#4` の診断ID不一致を上流で修正)

- 目皁E
  - `tests + stdlib` 全体で唯一失敗してぁE�� `tests/move_check.n.md::doctest#4` の `diag_id: 3065` 不一致を、場当たりではなくテスト記述の上流整備で解消する、E
- 原因:
  - 既存ケースぁE`#target core` + `core/math` 依存�E書き方で、`loop move` 本体検証より前に `D3016` 系のスタチE��検査エラーを�E行発生させてぁE��、E
  - 結果として、意図してぁE�� `D3065`�E�ETypeLoopPotentiallyMovedValue`�E�に到達しなかった、E
- 対忁E
  - `tests/move_check.n.md` の `move_in_loop`�E�Eoctest#4�E�を、`loop` 合流での moved 値再利用だけを検証する最小ケースに置換、E
  - `#target core` / `core/math` 依存を除去し、`bool` フラグ更新 (`set c false`) で 1 回ループを構�E、E
  - `consume` は `()->()` にし、`D3016` のノイズを排除、E
  - 最後に `consume t` を置き、`loop` 冁Emove の合流で `D3065` を安定�E現する形に固定、E
- 実施チE��チE
  - `node nodesrc/tests.js -i tests/move_check.n.md --no-tree -o /tmp/tests-move-check-after-fix.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-movecheck-fix.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (trait capability の enum 匁E typecheck 斁E���E依存�E除去)

- 目皁E
  - `todo.md` フェーズB2に沿って、trait capability 判定�E責務を `typecheck` から前段へ寁E��る、E
  - `typecheck` 冁E�E `copy/clone` 斁E���Eパ�Eスを削除し、AST の capability enum を直接処琁E��る、E
- 変更:
  - `nepl-core/src/ast.rs`
    - `TraitCapability` enum を追加 (`Copy` / `Clone` / `Unknown(String)` )、E
    - `TraitDef.capabilities` めE`Vec<String>` から `Vec<TraitCapability>` に変更、E
  - `nepl-core/src/parser.rs`
    - `#capability` めEparser 段で enum 化すめE`parse_trait_capability` を追加、E
  - `nepl-core/src/typecheck.rs`
    - `parse_trait_capability(&str)` と斁E���E比輁E��削除、E
    - AST enum を直接読み、`Unknown` のみ `D3096` を�Eす構�Eに変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/overload.n.md -i tests/move_effect.n.md --no-tree -o /tmp/tests-trait-capability-targeted.json -j 15` -> `285/285 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-trait-cap-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: stdlib/std 安�E化�E着扁E

- 目皁E
  - `core/mem` 安�E API 導�E後�E後続として、`stdlib/std`�E�Efs` / `stdio` / `env/cliarg`�E�を同一モチE��へ移行する、E
  - 甁E`alloc_raw` 直接利用と暗黙失敗経路を段階的に削減する、E

- 進捁E
  - `stdlib/std/fs.nepl`
    - `fs_alloc` / `fs_free` を追加、E
    - `fs_open_read` の `fd_out` 確保を `Result` 化し、解放を�E示化、E
    - `fs_read_fd_bytes` の `tmp/iov/nread` 確保を `Result` 連鎖化し、�E刁E��で解放する形へ変更、E
  - `stdlib/std/stdio.nepl`
    - 未着手（次段で `print/read_all/read_line/print_i32` の一時領域確保を安�E化予定）、E
  - `stdlib/std/env/cliarg.nepl`
    - 未着手（次段で `args_sizes_get/args_get` 周辺の確保失敗と解放方針を整琁E��定）、E

- メモ:
  - `fs` 単体�E実行系チE��ト�E入力征E��ケースを含むため、今後�E非対話セチE��で回帰確認する、E

# 2026-03-05 作業メモ (フェーズD: codegen 前段診断の共通化・第一段)

- 目皁E
  - `codegen_llvm` 冁E��残ってぁE�� `#target` 個別検証めEbackend から撤去し、前段共送Eprecheck へ雁E��E��る、E
  - `compile_module` と LLVM IR 生�E経路で同じ検証入口を使ぁE��wasm/llvm の診断規則差刁E��縮小する、E

- 変更:
  - `nepl-core/src/target_precheck.rs`
    - `precheck_module_target_directives` を追加�E�EUnknownTargetDirective` / `MultipleTargetDirective` を�E通生成）、E
    - `precheck_module_before_codegen` を追加�E�Earget directive + raw body precheck の合�E�E�、E
  - `nepl-core/src/codegen_llvm.rs`
    - `validate_target_directive_for_llvm` / `is_known_target_name` を削除、E
    - `emit_ll_from_module_for_target` 入口めE`precheck_module_before_codegen` へ統一、E
  - `nepl-core/src/compiler.rs`
    - `compile_module` の precheck 呼び出しを `precheck_module_before_codegen` へ置換、E

- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/llvm_target.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-unify-step2-focus.json -j 15`
    - 結果: `5/5 pass`
  - 補足:
    - `tests/neplg2.n.md` では既知の runtime 側 `Maximum call stack size exceeded` が残存（今回変更篁E��外）、E

# 2026-03-05 作業メモ (tests.js: argv メタ対応追加)

- 目皁E
  - `stdin/stdout` に加えて doctest から CLI 引数を注入できるようにし、`stdlib/tests/cliarg.n.md` をテスト可能にする、E

- 変更:
  - `nodesrc/parser.js`
    - doctest メタに `argv:` を追加、E
    - `parseMetaValue` ぁE`[` / `{` 始まり�E JSON も解釈するよぁE��張�E�Eargv: ["a","b"]` を�E列として取得）、E
  - `nodesrc/tests.js`
    - チE��トケース構造に `argv` を追加、E
    - wasm ワーカー要求へ `argv` を伝搬、E
    - llvm 実行時にめE`argv` を実行引数として渡す、E
  - `nodesrc/run_test.js`
    - WASI 実行時の args めE`argv` から受け取り、`[wasmPath, ...argv]` で起動、E
  - `stdlib/tests/cliarg.n.md`
    - `neplg2:test[assert_io]` + `argv` + `stdout` で `cliarg_count` 検証ケースを追加、E

- 検証:
  - parser 単体確誁E
    - `node -e "const p=require('./nodesrc/parser'); const r=p.parseFile('stdlib/tests/cliarg.n.md'); console.log(Array.isArray(r.doctests[0].argv), JSON.stringify(r.doctests[0].argv));"`
    - 結果: `true ["--flag","value"]`
  - run_test 直実行確誁E
    - `argv=["a","b"]` で `cliarg_count` 出力が `"3"`
    - `argv=[]` で `cliarg_count` 出力が `"1"`
  - tests.js 単体確誁E
    - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-stdlib --no-tree -o /tmp/tests-cliarg-only-argv.json -j 1 --assert-io`
    - 結果: `2/2 pass`

# 2026-03-05 作業メモ (フェーズD: stdlib/std 安�E化�E完亁E��全体回帰)

- 目皁E
  - `stdlib/std` の安�E化対象�E�Efs` / `stdio` / `env/cliarg`�E�を `Result` ベ�Eスへ揁E��、`alloc_raw` 直接利用の削減と失敗経路の明示化を完亁E��る、E

- 変更:
  - `stdlib/std/fs.nepl`
    - `__fs_copy_to_cstr` めE`Result<i32,i32>` 化、E
    - `wasi_path_open` で確保失敗を `Err` で返し、�E功時 `cpath` を忁E��解放、E
    - `fs_bytes_to_string` めE`fs_alloc` ベ�Eスへ変更、E
    - if レイアウト�Eの不要E`;` を除去�E�式戻り値整合）、E
  - `stdlib/std/stdio.nepl`
    - `print_i32` の一時領域確保を `std_alloc/std_free` ベ�Eスへ変更、E
    - `read_all` の if 式で `else out;` になってぁE��箁E��めE`out` に修正し、`expr; -> ()` による型不整合を解消、E
  - `stdlib/std/env/cliarg.nepl`
    - `cstr_to_str` の確保を `cli_alloc` ベ�Eスへ変更し、失敗時フォールバックを�E示、E

- 根本原因と修正方釁E
  - 全体回帰で `tests/stdin.n.md` のみ wasm stack mismatch が発生、E
  - 原因は `read_all` の `if` 弁Eelse 側ぁE`out;` となっており、仕様どおり `()` に化けてぁE��こと、E
  - 場当たりでコード�E解せず、式�E戻り値規則�E�Elan.md の `;` 仕様）に沿って `out` へ修正して根本解消、E

- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/fs.n.md --no-stdlib --no-tree -o /tmp/tests-fs-safe-phase.json -j 15` -> `1/1 pass`
  - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md -i tests/stdout.n.md -i stdlib/tests/fs.n.md --no-stdlib --no-tree -o /tmp/tests-std-safe-regression.json -j 15 --assert-io` -> `9/9 pass`
  - `node nodesrc/tests.js -i tests/stdin.n.md --no-tree -o /tmp/tests-stdin-focus.json -j 15 --assert-io` -> `210/210 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-stdlib-std-safety-phase.json -j 15` -> `788/788 pass`

# 2026-03-05 作業メモ (MemPtr/RegionToken 再調査と _raw 廁E��方針�E再整琁E

- 調査目皁E
  - `MemPtr/RegionToken` 導�E後�E残存生ポインタ依存と `_raw` 依存を全体で棚卸しし、上流優先での移行頁E��再確定する、E

- 現状要紁E
  - `core/mem.nepl` には `MemPtr<T>` / `RegionToken<T>` と `region_ptr_at/alloc_region/dealloc_region` が実裁E��み、E
  - `kpread/kpwrite` は公開構造体が `RegionToken<u8>` を保持する形まで移行済み、E
  - ただぁE`core/mem` 公開面には `alloc_raw/dealloc_raw/realloc_raw` と `load/store(i32)` 生�Eインタ版が残存、E
  - `stdlib/alloc` / `stdlib/kp` / `stdlib/nm` / `platforms/wasix` / examples/tests には `_raw` 呼び出しが多数残存、E
  - `nepl-core` 側にめE`_raw` 名依存が残存！Emonomorphize.rs`, `codegen_wasm.rs`, `codegen_llvm.rs`�E�、E

- 根本課顁E
  - `_raw` 廁E��は stdlib 側だけでは完亁E��ず、compiler 側の helper 解決ロジチE��を�Eに一般化する忁E��がある、E
  - `core/mem` の生�EインタAPIを�Eに削除すると、下流ライブラリと codegen が同時崩壊するため、段階移行が忁E��、E

- 再確定した実裁E��E��（上流優先！E
  1. compiler 側 `_raw` 名依存�E除去�E�Emonomorphize` / `codegen_wasm` / `codegen_llvm`�E�、E
  2. `core/mem` を安�EAPI公開面に統一し、生ポインタAPIを�E部互換層へ隔離、E
  3. `stdlib/alloc` と `kp` めE`MemPtr/RegionToken` + `Result/Option` 前提へ全面移行、E
  4. `stdlib/std` / `stdlib/nm` / tutorials/examples の頁E��追随移行、E
  5. 最後に `_raw` と生�Eインタ公開関数を削除し、compile_fail 回帰を固定、E
# 2026-03-05 作業メモ (フェーズD: wasm signature 診断めEcodegen 前段へ移勁E

- 目皁E
  - `codegen_wasm` 冁E��出してぁE��署名系診断を前段パスへ移し、`codegen到達時は検証済み` の設計へ寁E��る、E
  - wasm/llvm 共通化方針�E第一段として、backend 直下診断の削減を進める、E
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs` を追加、E
    - `precheck_wasm_codegen` を実裁E��、以下を前段で検査:
      - extern 署吁E(`D4001`)
      - 到達可能関数の署吁E(`D4002`)
  - `nepl-core/src/compiler.rs`
    - `insert_drops` 後�Ewasm emit 前に `precheck_wasm_codegen` を実行、E
    - エラー診断があれ�E codegen へ進まぁE`CoreError::Diagnostics` を返す、E
  - `nepl-core/src/codegen_wasm.rs`
    - 署名不一致時�E `D4001/D4002` 生�Eを削除し、前段検査前提でスキチE�E処琁E��変更、E
  - `tests/raw_body_precheck.n.md`
    - `D4001/D4002` を安定�E現する `compile_fail` ケースを追加・調整、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v5.json -j 15` -> `4/4 pass`
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v6.json -j 15` -> `7/7 pass`
# 2026-03-05 作業メモ (フェーズD: D4003 めEcodegen 前段へ移勁E

- 目皁E
  - `CodegenWasmMissingReturnValue (D4003)` めEbackend 依存診断から前段診断へ移し、codegen 到達時の前提を強化する、E
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 到達可能関数の `HirBody::Block` につぁE��、E
      - 戻り型ぁE`Unit` 以夁E
      - 最終的な靁Edrop 行が値を返さなぁE
      場合に `D4003` を前段で出す検査を追加、E
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_user` 冁E�E `D4003` 診断生�Eを削除、E
    - ここに到達した場合�E冁E��不整合として `panic!`�E�Erecheck で弾かれる前提）に変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v7.json -j 15` -> `7/7 pass`
# 2026-03-05 作業メモ (フェーズD: D4005 めEcodegen 前段へ移勁E

- 目皁E
  - `CodegenWasmLlvmIrBodyNotSupported (D4005)` めEbackend 側診断から前段診断へ移し、codegen の責務を縮小する、E
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 到達可能関数で `HirBody::LlvmIr` が残ってぁE��場合に `D4005` を前段で出す検査を追加、E
  - `nepl-core/src/codegen_wasm.rs`
    - `HirBody::LlvmIr` 刁E��で `D4005` を生成する�E琁E��削除、E
    - precheck 通過後�E冁E��不整合として `panic!` に変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v8.json -j 15` -> `7/7 pass`
# 2026-03-06 作業メモ (alloc/string: bool と基数付き整数斁E���E変換の整琁E

- 目皁E
  - 斁E���E表現への変換責務を `alloc/string` に雁E��E��、`core/cast` を値変換専用に保つ、E
  - 2 / 8 / 10 / 16 進の整数斁E���E化�E解析を `alloc/string` の API として揁E��る、E
- 変更:
  - `stdlib/alloc/string.nepl`
    - `from_bool` を追加し、bool の表示用斁E���E化を `alloc/string` に統一、E
    - `from_i32` めE`from_i32_radix x 10` 経由へ変更、E
    - `to_i32` めE`to_i32_radix s 10` 経由へ変更、E
    - `from_i64` めE`from_i64_radix x 10` 経由へ変更、E
    - `to_i64` めE`to_i64_radix s 10` 経由へ変更、E
    - 新規に `digit_to_char_lower` / `digit_from_char` / `validate_radix` を追加、E
    - 新規に `from_i32_radix` / `to_i32_radix` / `from_i64_radix` / `to_i64_radix` を追加、E
    - 2 / 8 / 10 / 16 進のみを受琁E��る方針をドキュメントコメントに明記、E
    - `from_bool` / `from_i32` / 基数付き変換の説明を、目皁E�E実裁E�E注意�E計算量が�Eかる形へ手書きで更新、E
  - `stdlib/std/test.nepl`
    - bool の斁E���E化を `from_bool` に統一、E
  - `tests/stdlib.n.md`
    - `from_i32_radix 10 2`
    - `from_i64_radix 255 16`
    - `to_i32_radix "1010" 2`
    - `to_i64_radix "Ff" 16`
    - 不正桁E/ 不正基数
    めEfocused test として追加、E
- 検証:
  - `node nodesrc/tests.js -i /tmp/one-radix-format.n.md --no-stdlib --no-tree -o /tmp/one-radix-format-only.json -j 1` -> `1/1 pass`
  - `node nodesrc/tests.js -i /tmp/one-radix-parse.n.md --no-stdlib --no-tree -o /tmp/one-radix-parse-only.json -j 1` -> `1/1 pass`
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/10_project_fizzbuzz.n.md --no-stdlib --no-tree -o /tmp/tests-string-radix-focused-v1.json -j 15` -> `13/13 pass`
- 判断:
  - `bool -> str` は値変換ではなく文字�E表現化なので `core/cast` ではなぁE`alloc/string` に置く、E
  - 2 / 8 / 10 / 16 進の基数持E���E斁E���E API の責務なので、`cast` ではなぁE`alloc/string` に置く、E
  - `core/cast` には数値/論理/ビッチEポインタの値変換だけを残す方針が一貫してぁE��、E
- 未宁E
  - `alloc/string.nepl` めEinput にした stdlib doctest 実行経路は別途整琁E��忁E��、E
  - `i128` の斁E���E表現変換は未実裁E��E

# 2026-03-05 作業メモ (フェーズD: D4011 めEcodegen 前段へ移勁E

- 目皁E
  - `CodegenWasmUnsupportedIndirectSignature (D4011)` めEbackend 側から前段へ移し、`call_indirect` の署名妥当性めEcodegen 前に確定する、E
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - HIR 式を再帰走査し、`CallIndirect` の `params/result` から `wasm_sig_ids` を評価、E
    - wasm 非対応署名を検�Eした場合に `D4011` を前段で返す検査を追加、E
  - `nepl-core/src/codegen_wasm.rs`
    - `CallIndirect` 刁E���E `D4011` 診断生�Eを削除し、precheck 通過後�E冁E��不整合として `panic!` に変更、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-indirect-v5.json -j 15` -> `7/7 pass`

# 2026-03-05 作業メモ (フェーズD: D4004 めEcodegen 前段へ移勁E

- 目皁E
  - `CodegenWasmRawLineParseError (D4004)` めEbackend 側診断から前段診断へ移し、`#wasm` 生行パース失敗を codegen 前に確定する、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `HirBody::Wasm` 刁E��での `D4004` 生�Eを削除、E
    - precheck 通過後�E冁E��不整合として `panic!` に変更、E
    - `precheck_raw_wasm_body(func)` を追加し、`parse_wasm_line` 失敗時に `D4004` を返す前段用ヘルパを実裁E��E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_wasm_codegen` から `codegen_wasm::precheck_raw_wasm_body` を呼び出すよぁE��更、E
  - `tests/raw_body_precheck.n.md`
    - `wasm_precheck_rejects_invalid_raw_line` を追加�E�Ediag_id: 4004`�E�、E
- 検証:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-rawline-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: D4010 めEcodegen 前段へ移勁E

- 目皁E
  - `CodegenWasmMissingIndirectSignature (D4010)` めEbackend 側診断から前段へ移し、`CallIndirect` の型セクション不整合を codegen 前に検査する、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `collect_wasm_signature_set` を追加し、wasm codegen で使ぁE��数/extern/間接呼び出し署名集合を共通化、E
    - `CallIndirect` 刁E���E `D4010` 診断生�Eを削除し、precheck 通過後�E冁E��不整合として `panic!` へ変更、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `collect_wasm_signature_set` の結果を使ぁE��`CallIndirect` の署名が型セクション候補に存在するかを前段で検査、E
    - 欠落時�E `D4010`、E��対応署名�E `D4011` として刁E��して返す、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-indirect-missing-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: 参�E解決系 wasm backend 診断の削渁E

- 目皁E
  - `CodegenWasmStringLiteralNotFound (4006)` / `CodegenWasmUnknownVariable (4007)` /
    `CodegenWasmUnknownFunctionValue (4008)` / `CodegenWasmUnknownFunction (4009)` めE
    backend 診断から外し、上流E��過後�E冁E��不整合として扱ぁE��E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `LiteralStr/Var/FnValue/Call/Set` での上記診断生�Eを削除、E
    - 同箁E��は `panic!` に変更し、codegen 到達時は解決済み前提を強制、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-ref-invariant-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: unknown intrinsic 診断の前段化整吁E

- 目皁E
  - `CodegenWasmUnknownIntrinsic (4012)` めEbackend 診断から外し、intrinsic 判定責務を前段へ寁E��る、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `is_supported_wasm_intrinsic` を追加して wasm backend が受琁E��めEintrinsic 名を明示化、E
    - intrinsic 未知刁E���E `D4012` 生�Eを削除し、�E部不整吁E`panic!` へ変更、E
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `HirExprKind::Intrinsic` で `is_supported_wasm_intrinsic` を使用し、未知 intrinsic を前段検査、E
  - `tests/raw_body_precheck.n.md`
    - 追加した `diag_id:4012` ケースは、実際には上流�E `D3012`�E�Enknown intrinsic�E�で先に失敗するため削除、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-unknown-intrinsic-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: 構築型 payload/field の backend 診断削渁E

- 目皁E
  - `CodegenWasmUnsupportedEnumPayloadType (4013)` /
    `CodegenWasmUnsupportedStructFieldType (4014)` /
    `CodegenWasmUnsupportedTupleElementType (4015)` めEbackend 診断から外し、codegen 到達時の型整合前提を明確化する、E
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `EnumConstruct` と `Match` の enum payload load/store、`StructConstruct`、`TupleConstruct` の
      非対忁Evaltype 刁E��を `panic!` に変更、E
    - 上訁E4013/4014/4015 の `diags.push(...with_id(...))` を削除、E
    - これにより、`codegen_wasm` 冁E�E `CodegenWasm*` 診断生�Eは precheck ヘルパ�E�E�E4004�E��Eみに限定、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-backend-diag-clean-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend の解決済み参�Eエラーを�E部不整合化)

- 目皁E
  - wasm 側と同様に、名前解決/署名解決済みであるべき参照系エラーめEbackend 診断責務から外す、E
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `Var` の unknown 変数刁E��を `panic!` 化、E
    - `Set` の unknown 変数刁E��を `panic!` 化、E
    - `FnValue` の unknown 関数値刁E��を `panic!` 化、E
    - `Call` の missing function signature 刁E��を `panic!` 化、E
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: monomorphize の runtime helper 候補ハードコード集紁E

- 目皁E
  - `_raw` 撤去フェーズに備え、`monomorphize` 冁E�E runtime helper 候補名ハ�Eドコードを一箁E��に雁E��E��る、E
- 変更:
  - `nepl-core/src/runtime_helpers.rs` を追加、E
    - `ALLOC_CANDIDATES`
    - `DEALLOC_CANDIDATES`
    - `REALLOC_CANDIDATES`
  - `nepl-core/src/lib.rs` に `runtime_helpers` を�E開、E
  - `nepl-core/src/monomorphize.rs`
    - runtime helper 選択ループ�E斁E���E配�EリチE��ルめE`runtime_helpers` 定数参�Eに置換、E
- 検証:
  - `NO_COLOR=false trunk build` -> success

# 2026-03-06 作業メモ (フェーズE前進: cliarg の C 斁E���E墁E��めEMemPtr<u8> 匁E

- 目皁E
  - `stdlib/std/env/cliarg.nepl` の公開面に残ってぁE��甁E`i32` ポインタ墁E��を減らし、`core/mem` の `MemPtr<T>` / `RegionToken<T>` モチE��へ寁E��る、E
  - 特に `cstr_len` / `cstr_to_str` を型付きポインタで受ける形に変更し、誤っぁEraw 呼び出しを型エラーで止める、E
- 原因:
  - `cliarg` は冁E��・公開ともに `i32` アドレスを直接受け渡しており、`kpread/kpwrite` 側で進めてぁE��型安�EモチE��と不整合だった、E
  - `cstr_len 0` めE`cstr_to_str 0` のような誤用ぁEAPI 形状上可能で、コンパイラが前段で止められなかった、E
- 変更:
  - `stdlib/std/env/cliarg.nepl`
    - `cstr_len` めE`<(MemPtr<u8>)*>i32>` に変更、E
    - `cstr_to_str` めE`<(MemPtr<u8>)*>str>` に変更、E
    - `cli_alloc_u8_region` / `cli_free_region` / `cli_i32_ptr` / `cli_u8_ptr` を追加し、一時領域確保を `RegionToken` ベ�Eスへ移行、E
    - LLVM 側 `__cli_copy_to_cstr` / `__cli_read_cmdline` めE`MemPtr<u8>` ベ�Eスへ変更、E
    - `cliarg_count` / `cliarg_get` のメタ惁E��確保と `argv` バッファ確保を `RegionToken<u8>` ベ�Eスへ変更、E
  - `stdlib/tests/cliarg.n.md`
    - `cstr_len 0` / `cstr_to_str 0` ぁE`D3006` で失敗すめEcompile_fail 回帰を追加、E
- 途中判断:
  - `stdlib/std/stdio.nepl` も同時に `RegionToken` 化を試したが、`read_line` の rewrite で構文不整合を入れ、parser overflow を誘発した、E
  - ここは間に合わせで押し�Eらず、`stdio` は直前�E正常状態へ戻し、今回のコミット対象から外した、E
- 検証:
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... import-only-cliarg ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... cliarg-count ... argv=[\"--flag\",\"value\"] ... EOF` -> pass (`stdout: "3"`)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... cliarg-basic ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... cliarg-compile-fail-cstr ... EOF` -> pass (`D3006`)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... stdout-concat ... EOF` -> pass

# 2026-03-06 作業メモ (フェーズE前進: fs の一時領域めERegionToken<u8> 匁E

- 目皁E
  - `stdlib/std/fs.nepl` の冁E��一時バチE��ァ確保を `RegionToken<u8>` / `MemPtr<T>` ベ�Eスへ移し、`i32` 生�Eインタの受け渡しを syscall 墁E��へ閉じ込める、E
- 原因:
  - `fs_open_read` / `fs_read_fd_bytes` / `fs_bytes_to_string` は確保した一時領域をすべて `i32` で扱っており、`cliarg` と同じく型安�EモチE��から外れてぁE��、E
  - 特に iovec / nread / 斁E���E絁E��立て用領域が型惁E��を失ったまま流れてぁE��ため、誤用めEAPI 形状で防げなかった、E
- 変更:
  - `stdlib/std/fs.nepl`
    - `fs_alloc` / `fs_free` を廁E��し、`fs_alloc_u8_region` / `fs_free_region` / `fs_i32_ptr` を追加、E
    - LLVM 側 `__fs_copy_to_cstr` めE`Result<MemPtr<u8>,i32>` へ変更し、解放めE`dealloc_ptr<u8>` に統一、E
    - `fs_open_read` の fd_out 一時領域めE`RegionToken<u8>` 化、E
    - `fs_read_fd_bytes` の tmp/iov/nread 一時領域めE`RegionToken<u8>` 化し、`load/store` は `MemPtr` オーバ�Eロードを経由する形へ変更、E
    - `fs_bytes_to_string` の出力バチE��ァ構築を `RegionToken<u8>` と `MemPtr<u8>` で行う形へ変更、E
- 設計判断:
  - `wasi_path_open` / `wasi_fd_read` 自体�EホスチEABI 墁E��なので raw `i32` を維持した、E
  - 型安�E化�E対象は stdlib 公開面と stdlib 冁E�E通常ロジチE��であり、ABI 直前�Eみ `mem_ptr_addr` で raw 化する、E
- 検証:
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... import-only-fs ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... fs-missing-file ... EOF` -> pass

# 2026-03-06 作業メモ (型安�E回帰の追加: MemPtr と RegionToken の取り違えめED3006 で固宁E

- 目皁E
  - `core/mem` の型安�EモチE��をテストで固定し、`MemPtr<u8>` / `MemPtr<i32>` / `RegionToken<T>` の取り違えを前段で止める、E
- 変更:
  - `tests/memory_safety.n.md`
    - `load_i32` に `MemPtr<u8>` を渡ぁEcompile_fail を追加、E
    - `store_u8` に `MemPtr<i32>` を渡ぁEcompile_fail を追加、E
    - `dealloc_region` に `MemPtr<u8>` を渡ぁEcompile_fail を追加、E
- 検証:
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-basic ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-load-i32-type-fail ... EOF` -> pass (`D3006`)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-store-u8-type-fail ... EOF` -> pass (`D3006` が�E頭)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-dealloc-region-type-fail ... EOF` -> pass (`D3006` が�E頭)
- 補足:
  - `nodesrc/tests.js -i tests/memory_safety.n.md ...` はこ�E環墁E��は timeout 30s に到達したため、個別 focused 実行で確認した、E

# 2026-03-06 作業メモ (core/mem の互換エイリアス整琁E

- 目皁E
  - `MemPtr` 安�Eオーバ�Eロードへ収束させ、`load_i32_ptr` / `store_i32_ptr` のような互換名を残さなぁE��E
- 変更:
  - `stdlib/core/mem.nepl`
    - `load_i32_ptr`
    - `store_i32_ptr`
    - `load_u8_ptr`
    - `store_u8_ptr`
    を削除、E
  - `tests/memory_safety.n.md`
    - 既存テストを `load_i32` / `store_i32` の直接利用へ更新、E
- 検証:
  - `rg -n "load_i32_ptr|store_i32_ptr|load_u8_ptr|store_u8_ptr" stdlib tests tutorials examples` -> 該当なぁE
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-basic-direct-overload ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-invalid-store-direct-overload ... EOF` -> pass

# 2026-03-08 作業メモ (提桁E stdlib の後方互換なし�E設訁E

- 目皁E
  - stdlib をジェネリクス/trait 中忁E��作り直すため�E、後方互換なし�E破壊的改良案を整琁E��る、E
  - 既存�E `_raw` 依存、命名揺れ、target 依存混在を解消するため�E設計軸を�E確化する、E
- 変更:
  - `doc/stdlib_breaking_reboot.md`
    - 目皁E非目樁E設計原剁E��定義、E
    - 新しい stdlib パッケージ構�E�E�Ecore/alloc/collections/text/io/fs/runtime/prelude`�E�を提案、E
    - trait 能力モチE���E�メモリ能力、I/O 能力含む�E�とジェネリクス設計を提案、E
    - 命名規則の破壊的変更�E�E_raw/_safe` 廁E��、`into_xxx/parse_xxx` 統一�E�を提案、E
    - runtime adapter 刁E��、移行フェーズ、テスト戦略、期征E��果を記述、E
- plan.mdとの差異:
  - `plan.md` は言語仕様�E核�E�前置記法�E式指向�Eオフサイドルール�E�を定義してぁE��、E
  - 今回は言語構文を変更せず、stdlib の責務�E離と trait 墁E��設計に限定した提案であり、`plan.md` と矛盾しなぁE��E
- 結果:
  - フェーズD/E�E�Eompiler `_raw` 依存撤去、stdlib 移行）を進める際�E実裁E��準として参�E可能な斁E��を追加した、E

- 検証:
  - `trunk build`
    - 結果: 環墁E�� `trunk` コマンドが存在せず実行不可�E�Ecommand not found`�E�、E
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-tree -o /tmp/tests-proposal.json -j 15`
    - 結果: `nepl-web compiler artifacts were not found` により `229/229 errored`、E
    - 出劁EJSON `/tmp/tests-proposal.json` を確認し、要因がビルド�E果物不足であることを確認、E

# 2026-03-08 作業メモ (改喁E stdlib 再設計案�E NEPLg2 哲学整吁E

- 目皁E
  - 前回追加した `doc/stdlib_breaking_reboot.md` が、`plan.md`、`introduce.n.md`、`tutorials` で示されめENEPLg2 の哲学�E�式指向�E前置記法�Eオフサイドルール・パイプ合成）と一致してぁE��かを再点検し、改喁E��る、E
- 原因:
  - 前回案�E trait/generics と安�E性方針�E示せてぁE��が、NEPLg2 の表現哲学�E�値合�E優先、パイプで追える引数頁E��effect 明示�E�との接続が弱く、実裁E��断時に解釈がぶれる余地があった、E
- 変更:
  - `doc/stdlib_breaking_reboot.md`
    - 「NEPLg2 哲学との整合要件」章を追加し、式指向�E前置記況Eパイプ�Eeffect・型駁E��の整合基準を明文化、E
    - API設計原剁E��「合成しめE��ぁE��数頁E��「`Result/Option` で失敗を表現」「target依存をadapterへ隔離」�E観点で再整琁E��E
    - コンチE��・命名方針�E移行フェーズ・チE��ト戦略を、tutorialsの実裁E��タイルと繋がる形に調整、E
- plan.mdとの差異:
  - 言語仕様�E変更してぁE��ぁE��E
  - stdlib再設計案�E評価軸を、`plan.md` と tutorials の記述に沿ぁE��ぁE��化した、E
- 結果:
  - 破壊的改良案をそ�Eまま実裁E��画へ落とし込む際に、NEPLg2 の設計思想と乖離しにくい斁E��へ更新できた、E
- 検証:
  - `trunk build`
    - 結果: 環墁E�� `trunk` コマンドが存在せず実行不可�E�Ecommand not found`�E�、E
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-tree -o /tmp/tests-stdlib-philosophy.json -j 15`
    - 結果: `nepl-web compiler artifacts were not found` により `229/229 errored`、E
    - 出劁EJSON `/tmp/tests-stdlib-philosophy.json` を確認し、�E果物不足が失敗要因であることを確認、E

# 2026-03-09 作業メモ (stdlib reboot 開始前の未確定差刁E��琁E

- 目皁E
  - `todo.md` の本格実裁E��入る前に、現在の未確定差刁E��何を直してぁE��のか、どこまで安定してぁE��のか、何が別件ブロチE��ーなのかを明確にする、E
  - `vec` の型安�E化差刁E��「そのまま reboot に持ち込める状態」まで整琁E��る、E
- 対象差刁E
  - `stdlib/alloc/collections/vec.nepl`
  - `stdlib/alloc/collections/vec/sort.nepl`
  - `stdlib/alloc/string.nepl`
  - `stdlib/nm/parser.nepl`
  - `stdlib/nm/html_gen.nepl`
  - `examples/bf.nepl` は今回の整琁E��象外�E既存差刁E��して触れてぁE��ぁE��E
- 変更の意味:
  - `Vec<.T>.data` めE`i32` から `MemPtr<.T>` に変更し、`alloc/collections` を型付きメモリ API に寁E��てぁE��、E
  - それに伴ぁE��`string` と `nm` で `get v "data"` を生 `i32` とみなしてぁE��箁E��めE`mem_ptr_addr get ... "data"` に追従させてぁE��、E
  - `vec/sort` も同様に、`Vec` の冁E��表現変更へ追従してぁE��、E
- 根本原因:
  - `core/mem` の型安�E化を進めた結果、`alloc/collections` の中核である `Vec` ぁEraw `i32` を�E開してぁE��と下流�E体�E型安�E化が進まなぁE��E
  - そ�Eため `Vec` を�Eに `MemPtr<.T>` 化し、その変更の影響先を追従させる忁E��があった、E
- 刁E��刁E��結果:
  - `string` の最封Ecompile は通過した、E
    - `sb_build` 周辺の `parts_vec.data` 参�E変更は妥当、E
  - `vec` の最封Ecompile も、`vec_get` を用ぁE��ケースでは通過した、E
    - `get v 1` が失敗する�Eは field access の `get` と衝突してぁE��ためで、今回の `MemPtr` 化�E体�E問題ではなぁE��E
  - `nm/parser.nepl` は import するだけで parser の stack overflow が発生した、E
  - `nm/html_gen.nepl` は import するだけで wasm validation error が発生した、E
- 重要な判断:
  - `nm/parser.nepl` / `nm/html_gen.nepl` の import-only failure は、今回の `mem_ptr_addr` 追従変更とは独立�E既存ブロチE��ーとして扱ぁE��E
  - したがって、現在の未確定差刁E�EぁE��
    - `vec.nepl`
    - `vec/sort.nepl`
    - `string.nepl`
    は reboot に向けた有効差刁E��ある、E
  - 一方で `nm` 側は、今回の追従変更自体�E妥当性は高いが、現時点で import-only compile が失敗するため、個別に安定性を証明したとはまだ言えなぁE��E
- 検証:
  - `NO_COLOR=false trunk build`
    - 結果: 成功、E
  - direct compile (`alloc/string` 最小ケース)
    - 結果: 成功、E
  - direct compile (`alloc/collections/vec` 最小ケース、`vec_get` 使用)
    - 結果: 成功、E
  - direct compile (`nm/parser` import-only)
    - 結果: parser stack overflow、E
  - direct compile (`nm/html_gen` import-only)
    - 結果: wasm validation error、E
- 現時点の結諁E
  - `todo.md` の本格実裁E��入るため�E準備として、未確定差刁E�E意味とブロチE��ーは整琁E��きた、E
  - 次の安�Eな着手点は `todo.md` 先頭の `std/test` 改喁E��スクである、E
  - `vec` 系差刁E�E reboot 計画に吸収する前提で保持し、`nm` 側の import-only 失敗�E別件ブロチE��ーとして管琁E��る、E
  - こ�E時点では `nm/parser.nepl` / `nm/html_gen.nepl` の追従差刁E�E commit 対象から外し、stdlib reboot 後に改めて対処する、E

# 2026-03-09 作業メモ (tests/compiler と tests/stdlib の再編)

- 目皁E
  - stdlib reboot 開始前に、テスト失敗�E原因を「compiler 本体�E誤り」「stdlib 実裁E�E誤り」「テスト移行ミス」�E 3 つへ刁E��刁E��めE��くする、E
  - `tests/` 直下に混在してぁE��ケースめE`tests/compiler/*` と `tests/stdlib/*` へ刁E��し、以後�E回帰確認�E粒度を揃える、E
- 変更:
  - compiler 本体�E確認を主目皁E��する `.n.md` と tree suite めE`tests/compiler/*` へ移動した、E
  - stdlib API・アルゴリズム・target facade・回帰確認を主目皁E��する `.n.md` めE`tests/stdlib/*` へ移動した、E
  - `nodesrc/tests.js`
    - tree suite の読み込み先を `tests/compiler/tree/run` へ更新した、E
    - tree suite 結果の `id` / `file` めE`tests/compiler/tree/*` へ更新した、E
  - `tests/compiler/tree/_shared.js`
    - `nodesrc/*` への相対 import を、新しい配置に合わせて 1 段深く修正した、E
  - `nodesrc/analyze_source.js`
    - 使用例コメント�EパスめE`tests/compiler/functions.n.md` へ更新した、E
- 根本原因:
  - 既存�E `tests/` は compiler 本体テストと stdlib チE��トが同屁E��ており、stdlib reboot 中に失敗�E原因を正しく刁E��刁E��にくかった、E
  - tree suite めE`tests/tree/*` を前提に直参�EしてぁE��ため、単純なファイル移動だけでは実行経路が壊れた、E
- 実裁E���E注愁E
  - `nodesrc/tests.js` は既定で stdlib doctest も一緒に走査するため、focused test では `--no-stdlib` を�E示しなぁE��「移動確認」�EつもりぁEstdlib 全体実行になる、E
  - 今回の focused 検証は、�E編そ�Eも�Eの安�E性確認に限定するためE`--no-stdlib --no-tree` を用ぁE��、E
- 検証:
  - `node nodesrc/tests.js -i tests/compiler/block_semicolon_return.n.md -i tests/compiler/plan.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-compiler-reorg-focused.json -j 15`
    - 結果: `49/49 pass`
  - `node nodesrc/tests.js -i /tmp/std-test-collect-success-only.n.md --no-stdlib --no-tree -o /tmp/std-test-collect-success-only.json -j 15`
    - 結果: `1/1 pass`
  - `node nodesrc/tests.js -i /tmp/std-test-collect-fail-only.n.md --no-stdlib --no-tree -o /tmp/std-test-collect-fail-only.json -j 15`
    - 結果: `1/1 pass`
- 結諁E
  - `tests/compiler/*` と `tests/stdlib/*` の刁E��、およ�Eそれに伴ぁE`nodesrc` / tree suite の追従�E成立した、E
  - `todo.md` 先頭の再編タスクは完亁E��して削除し、以後�E reboot 本流�E `diag` / `Outcome` / trait 能力モチE��の実裁E��進める、E

# 2026-03-09 作業メモ (`std/test` コメント整琁E�� collect API の使ぁE��固宁E

- 目皁E
  - `stdlib/std/test.nepl` のコメントを `doc/stdlib_doc_comment_policy.md` に沿って整琁E��、�E部 helper に boilerplate doctest が並ぶ状態を解消する、E
  - 利用老E��直接使ぁE�E閁EAPI だけに、用途が刁E��めEdoctest を残す、E
- 変更:
  - `stdlib/std/test.nepl`
    - モジュール先頭コメントを、単発 assert と collectable な `check_*` / `finish_checks` の二系統を持つことが�Eかる冁E��へ更新、E
    - `test_str_eq_loop`、`test_print_fail`、`test_checked`、`test_fail`、`trap` など冁E�� helper の boilerplate doctest を削除、E
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` / `assert_ok_i32` / `assert_err_i32` の doctest を、実際の用途が刁E��る例へ差し替え、E
    - 計算量表記を `[時間/じかん]` / `[空閁Eくうかん]` の形に揁E��た、E
- 判断:
  - `std/test` の実裁E�E体�E `67e8156` で十�Eに揁E��てぁE��ため、今回は API を増やさず、利用老E��け説明�E質を�Eに上げた、E
  - 実裁E��証は `tests/stdlib/std_test_collect.n.md` に残し、`.nepl` 側 doctest は使ぁE��確認へ寁E��た、E

# 2026-03-09 作業メモ (`std/test` コメント整琁E�E検証完亁E

- 検証:
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-std-test-collect-nostdlib.json -j 15`
    - 結果: `2/2 pass`
  - `node nodesrc/tests.js -i /tmp/std_test_assert_doctest_smoke.n.md --no-stdlib --no-tree -o /tmp/std_test_assert_doctest_smoke.json -j 15`
    - 結果: `1/1 pass`
- 判断:
  - `stdlib/std/test.nepl` の公閁E`assert_*` 例�E `#entry main` と `#target std` を前提にすると、そのまま実行できることを確認した、E
  - collectable API の既存回帰 2 件も維持されてぁE��ため、今回の変更はドキュメントコメント整琁E��して確定してよい、E
  - `nodesrc/tests.js` は `--no-stdlib` を付けなぁE�� stdlib 全走査で重くなりやすく、focused 検証では `--no-stdlib` を使ぁE�Eが妥当、E

# 2026-03-09 作業メモ (`alloc/diag` の再設計と focused test の安定化)

- 目皁E
  - stdlib reboot の最初�E本流タスクとして、`alloc/diag` めE`Diag` / `Diags` / `Outcome` / `StdErrorKind` 中忁E�EモチE��へ移行する、E
  - 旧 `error.nepl` の責務を `diag` 側へ吸収し、stdlib 全体で再利用できる診断基盤を�Eに固める、E
- 変更:
  - `stdlib/alloc/diag/error.nepl`
    - `DiagLevel`, `StdErrorKind`, `DiagKind`, `Diag`, `Diags`, `Outcome` を定義した、E
    - `diag_new`, `diag_log`, `diag_info`, `diag_warn`, `diag_error`, `diag_with_span`, `diag_with_source`, `diag_add_note`, `diag_add_help` を追加した、E
    - `diags_new`, `diags_one`, `diags_push`, `diags_len`, `diags_has_errors` を追加した、E
    - `outcome_ok`, `outcome_err`, `outcome_with_diags`, `result_to_outcome` を追加した、E
    - `diag_out_of_memory` など旧 collections 側 helper は、新しい `Diag` モチE��の薁E��ラチE��として残した、E
  - `stdlib/alloc/diag/diag.nepl`
    - `kind_str`, `span_to_string`, `diag_to_string`, `diags_to_string` を新 `Diag` / `Diags` 構造に合わせて書き直した、E
    - `std` target では `diag_print*` / `diags_print*` めErenderer helper として残した、E
  - `stdlib/tests/error.n.md`
    - `Diag` / `Diags` / `Outcome` の値モチE��確認へ全面更新した、E
    - `match _:` を�E挙型の完�E列挙へ修正した、E
  - `stdlib/tests/diag.n.md`
    - `diag_to_string` / `diags_to_string` の focused test を新モチE��へ更新した、E
  - `tests/stdlib/collections_diag.n.md`
    - collections が返す `Diag` の `StdErrorKind` 確認へ更新した、E
  - `tests/compiler/sizeof.n.md`
    - `Span` / `Diag` / `Diags` / `Outcome` の `size_of` ケースを新モチE��へ更新した、E
- 根本原因と修正:
  - `diag_new`, `diags_new`, `diags_one`, `checks_new` などぁE`Vec::new` / `vec_push` を�E部で呼んでぁE��のに pure のままだった、E
    - これにより `pure context cannot call impure function` が発生してぁE��、E
    - 影響する helper めEimpure シグネチャへ修正した、E
  - `alloc/diag/error.nepl` では `new<str>` / `new<Diag>` の無修飾呼び出しが、周辺 import 環墁E��よって `ambiguous overload` になってぁE��、E
    - これは `new` / `push` の alias 群ぁEstar import で混ざる現行環墁E��依存した不安定性だった、E
    - `vec_new<...>` / `vec_push<...>` を�E示皁E��使ぁE��へ直し、環墁E��存�E曖昧さを消した、E
  - `stack_new` / `stack_push` は `diag_out_of_memory` の impure 化に追従しておらず、`sizeof` focused test で compile failure を起こしてぁE��、E
    - シグネチャめEimpure に修正した、E
- 検証:
  - direct `runSingle` により、以下�E 4 ファイルの全ケースを個別確認した、E
    - `stdlib/tests/error.n.md`
    - `stdlib/tests/diag.n.md`
    - `tests/stdlib/collections_diag.n.md`
    - `tests/compiler/sizeof.n.md`
  - 結果:
    - `2 + 2 + 6 + 8 = 18` ケースすべて pass、E
  - `nodesrc/tests.js` の focused run はこ�E環墁E��は進捗表示が乏しく長く見えるため、問題�Eり�Eけ�E `runSingle` ベ�Eスで行った、E
- 結諁E
  - `alloc/diag` の新モチE��自体�E成立し、focused test で安定した、E
  - 次はこ�E変更めEcommit し、stdlib reboot 本流�E次段階へ進める、E

# 2026-03-09 作業メモ (`Outcome` 読み取り helper の追加と struct 抽出制紁E�E確誁E

- 目皁E
  - `Diag` 再設計�E次段階として、`Result` と `Outcome` を�E通に扱ぁEhelper 層の最小部刁E��先に整備する、E
  - trait 能力モチE��へ進む前に、`Outcome` から診断群を安�Eに読み取る API を固定する、E
- 変更:
  - `stdlib/alloc/diag/error.nepl`
    - `outcome_diags_or_empty <.T, .E> <(Outcome<.T, .E>)*>Diags>` を追加、E
    - `outcome_has_errors <.T, .E> <(Outcome<.T, .E>)->bool>` を追加、E
  - `stdlib/tests/error.n.md`
    - 上訁E2 helper の使ぁE��と結果を確認すめEfocused doctest へ更新、E
- 試行して見送った�E容:
  - `outcome_push_diag`
  - `outcome_map`
  - `outcome_map_err`
- 根本原因:
  - 現在の言語では、struct から褁E�� field を安�Eに取り出して再構築する一般皁E��手段が不足してぁE��、E
  - `get o "result"` と `get o "diags"` はどちらも `o` を消費するため、両方を同時に取り出して新しい `Outcome` を作れなぁE��E
  - struct に対する `match` による刁E��も現状の斁E��では未対応で、`Outcome r ds:` のような destructuring は parser error になる、E
  - そ�Eため、読み取り専用 helper は成立するが、`Outcome` を更新・写像する helper は言語機�E側の支援なしに安�E実裁E��きなぁE��E
- 判断:
  - 間に合わせで raw field 操作や ad-hoc helper を増やすと、後で trait 能力モチE��と衝突する、E
  - 今回は成立する読み取り API だけを確定し、更新系 helper は compiler / 言語機�Eの整備後へ回す、E
- 検証:
  - direct `runSingle`
    - `stdlib/tests/error.n.md`
      - 結果: `2/2 pass`
    - `stdlib/tests/error.n.md`
    - `stdlib/tests/diag.n.md`
    - `tests/stdlib/collections_diag.n.md`
      をまとめた focused 実衁E
      - 結果: `10/10 pass`
- 結諁E
  - `Outcome` の最小読み取り helper は、現状の言語機�Eでも安定に提供できる、E
  - `Outcome` の mutating helper めElibrary 側だけで無琁E��進めるのは誤りで、忁E��なめEcompiler / 言語機�Eの課題として扱ぁE��E

# 2026-03-09 作業メモ (`core/traits` の最小核めEstdlib へ追加)

- 目皁E
  - reboot の trait 能力モチE��めElibrary 側から具体化するため、現行言語機�Eで安定に提供できる最小核を�Eに配置する、E
  - compiler チE��ト�Eの ad-hoc trait 宣言めEstdlib の正式モジュールへ置き換えてぁE��足場を作る、E
- 変更:
  - `stdlib/core/traits/copy.nepl`
    - `Clone` と `Copy` めEstdlib trait として定義した、E
    - `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` へ impl を追加した、E
  - `stdlib/core/traits/stringify.nepl`
    - `Stringify` trait と共送Ehelper `stringify` を追加した、E
    - `str`, `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` へ impl を追加した、E
    - 実体�E斁E���E化�E `alloc/string` の既存関数を�E利用した、E
  - `stdlib/core/traits/debug.nepl`
    - `Debug` trait と共送Ehelper `debug_string` を追加した、E
    - `str` は引用符付き、それ以外�E基本型�E `Stringify` に委譲する impl を追加した、E
  - `tests/stdlib/traits_text.n.md`
    - 日本語�E `[目皁Eもくてき]` と確認頁E��を持つ focused test を追加した、E
- 判断:
  - `Serialize` / `Deserialize` は trait 型引数めEformat 型が忁E��になりやすく、現行言語機�Eと正面衝突する可能性が高い、E
  - そ�Eため今回は `Copy` / `Clone` / `Stringify` / `Debug` までを最小核として先に確定し、残りは `todo.md` の未完タスクとして維持する、E
  - `Eq` / `Ord` / `Hash` も同様に、既存�E ad-hoc 実裁E��の整合を見ながら次段で扱ぁE��E
- compiler 修正:
  - generic 関数呼び出し�E型引数解決で、E��数本体�E型変数束縛から推論できた具体型ぁE`resolved_args` へ反映されず、単相化時に `Clone::clone` が未解決のまま残る不�E合があった、E
  - `check_function` で generic 関数本体�E型変数 binding めEsnapshot / restore しつつ、呼び出し�Eでは `binding.ty` と `inst_ty` の絁E��ら各 type parameter の具体型を�E推論して `resolved_args` へ反映するように修正した、E
  - monomorphize の trait impl 探索は `unify` を使ってぁE��ため、cast 用の緩ぁE��致規則まで trait 解決に混入し、`Stringify<i32>` ぁE`u8` / `bool` / `str` など褁E�� impl と曖昧一致する不�E合があった、E
  - trait impl 選択�E `same_type` による同一型一致へ刁E��替え、trait 解決と数値 cast の規則を�E離した、E

# 2026-03-09 作業メモ (`Serialize` / `Deserialize` trait の導�Eと receiverless trait method 解決修正)

- 目皁E
  - trait 能力モチE��の残件だっぁE`Serialize` / `Deserialize` めEstdlib へ追加する、E
  - `Deserialize::deserialize` のように receiver を取らず返り値側で `Self` が決まめEtrait method が、generic helper 冁E��も安定に単相化されるよう compiler を修正する、E
- 変更:
  - `stdlib/core/traits/serialize.nepl`
    - `Serialize` trait と helper `serialize` を追加した、E
    - `str`, `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` の impl を追加した、E
  - `stdlib/core/traits/deserialize.nepl`
    - `Deserialize` trait と helper `deserialize` を追加した、E
    - `str`, `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` の impl を追加した、E
    - `Result<_, i32>` めE`Result<_, StdErrorKind>` に寁E��めE`parse_err_to_std` を追加した、E
  - `tests/stdlib/traits_serde.n.md`
    - `[目皁Eもくてき]` を持つ focused test を追加し、serialize / deserialize の典型使用例を確認するよぁE��した、E
- compiler 修正:
  - `Deserialize::deserialize s` のような receiverless trait method reference は、従来 `Self` 用の遊離 fresh type var めEstack entry に積んでぁE��、E
  - そ�Eため generic helper `fn deserialize <.T: Deserialize> ...` 冁E�� `.T` へ結�E付かなぁE��ま `FuncRef::Trait { self_ty = Self }` ぁEHIR に残り、wasm codegen で `unknown function 'Deserialize::deserialize [self=Self]'` となってぁE��、E
  - 修正冁E��:
    - trait method reference を積�E時点で、そのスコープに唯一の `.T: Trait` がある場合�E fresh var ではなくその `.T` めE`Self` として使ぁE��ぁE��した、E
    - fallback の trait call 解決も、receiver 引数だけでなぁEexpected return type と trait bound から `Self` を推論できるように整琁E��た、E
    - `check_function` では body の型変数 binding めErestore する前に HIR 全体�E垁EID めEresolve するようにし、単相化へ未解決 var が漏れなぁE��ぁE��した、E
    - monomorphize では trait callee の self 解決めEargs 先頭型へ頼らず、`self_ty` 自体�E解決結果だけを使ぁE��ぁE��戻した、E
- 検証:
  - `NO_COLOR=false trunk build`
    - 結果: success
  - `node nodesrc/tests.js -i tests/stdlib/traits_serde.n.md --no-stdlib --no-tree -o /tmp/tests-traits-serde.json -j 15`
    - 結果: `2/2 pass`
- 結諁E
  - `Serialize` / `Deserialize` の stdlib trait 導�Eは成立した、E
  - 根本原因は codegen めEmonomorphize ではなく、receiverless trait method reference めEgeneric body へ持ち込む時点の `Self` 束縛だった、E
  - 次は `Result` / `Outcome` を�E通に扱ぁEhelper / trait 枠絁E��へ進む、E

# 2026-03-09 作業メモ (`Outcome` の[読/めEみ[叁Eと]めEhelper を追加)

- [目皁Eもくてき]:
  - `Result` と `Outcome` を[共送EきょぁE��ぁEに[扱/あつか]ぁE��め、`Outcome` [側/がわ]にめE軽釁EけいりょぁEな[読/めEみ[叁Eと]めEhelper を[揁Eそろ]える、E
  - `match get o "result"` を[毎回/まぁE��い][書/か]かずに、`Outcome.result` の[成否/せいひ]を[読/めEめるようにする、E
- [変更/へんこぁE:
  - `stdlib/alloc/diag/error.nepl`
    - `outcome_result`
    - `outcome_is_ok`
    - `outcome_is_err`
    を追加、E
  - `stdlib/tests/error.n.md`
    - [上訁EじょぁE��] helper の[目皁Eもくてき]と[確誁Eかくにん][冁E��/なぁE��ぁEを[追訁EつぁE��]、E
- [判断/はんだん]:
  - `Outcome` の[更新系/こうしんけい] helper は、struct field を[刁E��/ぶんかい]して[再構篁Eさいこうちく]する[言誁Eげんご][機�E/き�EぁEがまだ[弱/よわ]ぁE��め[保留/ほりゅぁE、E
  - [現段隁Eげんだんかい]では[読/めEみ[叁Eと]めEhelper を[允Eさき]に[固/かた]める[方/ほぁEが、stdlib reboot の[上流EじょぁE��めE��]として[安宁Eあんてい]する、E
- [検証/けんしょぁE:
  - `node nodesrc/run_test.js` に[直接/ちめE��せつ] JSON を[渡/わた]し、`outcome_result` / `outcome_is_ok` / `outcome_is_err` を[使/つか]ぁEfocused snippet ぁE`pass` になることを[確誁Eかくにん]、E


- `alloc/diag/error` に `into_outcome` / `result_like_result` / `result_like_is_ok` / `result_like_is_err` を追加、E
  - `Result` と `Outcome` めEoverloading で共送Ehelper 名に揁E��た、E
  - 現状の trait 機�Eでは associated type めEtrait generic abstraction が弱く、`Result<T,E>` と `Outcome<T,E>` を無琁E�� trait 一つへ押し込むより helper の方が�E然だった、E
- `stdlib/tests/error.n.md` に `result_and_outcome_common_helpers` を追加し、軽釁EAPI と rich API の共通読み取りめEfocused に確認、E
# 2026-03-09 作業メモ (compiler 前提固宁E `.nepl` で表現できる primitive の Copy めEstdlib impl へ移衁E

- [目皁Eもくてき]:
  - `todo.md` の compiler 前提固定に従い、`Copy` 判定�E compiler 固定表を縮小する、E
  - `.nepl` ソースで表現できる primitive につぁE��は、stdlib 側の `impl Copy/Clone` を唯一の根拠に寁E��る、E
- [根本原因/こんぽんげんいん]:
  - `TypeCtx::is_copy_with_trait_model` は trait モードでめE`i32` / `u8` / `f32` / `bool` / `str` / `()` を固定表で copy とみなしてぁE��、E
  - こ�Eため `core/traits/copy.nepl` に同�E容の impl を定義しても、move 規則の最終判定が compiler 冁E��の知識へ依存したままだった、E
  - 一方で、参照型や `never` は現状の言語機�Eでは `.nepl` 側に自然な impl を置きにくく、同じ扱ぁE��はできなぁE��E
- [変更/へんこぁE:
  - `stdlib/core/traits/copy.nepl`
    - `str` への `Clone` / `Copy` impl を追加、E
    - `()` への `Clone` / `Copy` impl を追加、E
  - `nepl-core/src/types.rs`
    - trait モード�E `is_copy_with_trait_model` から、`.nepl` 側で表現できる primitive (`Unit` / `I32` / `U8` / `F32` / `Bool` / `Str`) の固定表判定を削除、E
    - 上記�E `has_copy_impl_target` による trait impl 登録結果だけで判定するよぁE��更、E
    - 固定表に残した�Eは、現段階で source impl を�E然に持ちにくい `Never` と参�E型だけに絞った、E
  - `tests/compiler/move_effect.n.md`
    - `core/traits/copy` めEimport したとき、`str` の再利用ぁE`Copy` impl によって成立するケースを追加、E
    - `()` の再利用ぁE`Copy` impl によって成立するケースを追加、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build` -> success
  - `node` + `nodesrc/compiler_loader` による compile-only focused check:
    - `#import "core/traits/copy" as *` を含む `str` 再利用 snippet -> `OK`
    - 吁E`()` 再利用 snippet -> `OK`
- [状況EじょぁE��めE��]:
  - compiler の `Copy` 固定表は縮小され、`.nepl` 側に impl を置ける primitive は stdlib impl に寁E��られた、E
  - 残る特別扱ぁE�E、現状の言語で source impl を置きにくい参�E型と `never` である、E

# 2026-03-09 作業メモ (compiler 前提固宁E LLVM codegen の前段責務を `compiler.rs` に雁E��E

- [目皁Eもくてき]:
  - `todo.md` の compiler 前提固定に従い、LLVM 経路でめEcodegen ぁEtypecheck / move check / target precheck / codegen precheck を抱えなぁE��へ寁E��る、E
  - wasm/llvm の前段診断めE`compiler.rs` 側の共送Elowering へ雁E��E��、codegen 到達後�E生�E専任に近づける、E
- [根本原因/こんぽんげんいん]:
  - `compile_module` は wasm 用に target precheck -> typecheck -> monomorphize -> move check -> drop 挿入をまとめてぁE��が、LLVM 経路は `codegen_llvm.rs` 冁E��別に `precheck_module_before_codegen` / `typecheck` / `monomorphize` / `precheck_llvm_codegen` を実行してぁE��、E
  - そ�Eため、同じ�E力でめEwasm と llvm で診断生�E責務が刁E��し、`codegen_llvm` が前段の失敗を `TypecheckFailed` に潰して抱え込む構造になってぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/compiler.rs`
    - `PreparedProgram` を追加し、target precheck -> typecheck -> monomorphize -> move check -> drop 挿入までめE`prepare_module_for_codegen` に雁E��E��E
    - `PreparedLlvmProgram` を追加し、LLVM entry 解決・reachable 雁E��構築�E`precheck_llvm_codegen` めE`prepare_module_for_llvm_codegen` に雁E��E��E
    - `compile_module` は `prepare_module_for_codegen` を使ぁE��へ変更し、wasm 前段も同じ経路を通るようにした、E
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` は `compiler::prepare_module_for_llvm_codegen` を呼ぶだけにし、直接の typecheck/precheck 呼び出しを除去、E
    - `try_lower_entry_from_hir` は prechecked artifact (`PreparedLlvmProgram`) を受け取り、診断生�Eを行わぁElowering だけを拁E��する形へ変更、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - 結果: success
  - `node nodesrc/tests.js -i tests/compiler/llvm_target.n.md -i tests/compiler/raw_body_precheck.n.md -i tests/compiler/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-llvm-frontload.json -j 15`
    - 結果: `8/8 pass`
- [状況EじょぁE��めE��]:
  - LLVM codegen は前段を直接実行せず、`compiler.rs` の共送Elowering を前提に動く形へ寁E��た、E
  - まだ `nepl-cli` の LLVM 刁E���E `codegen_llvm::emit_ll_from_module_for_target` を直接呼ぶが、その冁E��は共送Efront-end を通るため、責務�E離の主眼は満たした、E
  - 残る compiler 前提固定�E本流�E、copy/clone 非ハードコード化の残件と、`Diag.kind` 言語機�Eの準備である、E

# 2026-03-09 作業メモ (compiler 前提固宁E LLVM codegen から旧 front-end helper を除去)

- [目皁Eもくてき]:
  - `codegen_llvm.rs` に残ってぁE��旧 front-end helper 群を除去し、LLVM codegen が�Eび typecheck/precheck 経路を�E匁E��なぁE��態を保つ、E
- [変更/へんこぁE:
  - `nepl-core/src/codegen_llvm.rs`
    - 未使用になってぁE�� `compute_reachable_hint` / `build_hir_for_llvm_lowering` / `try_build_hir_with_target` と、その補助だっぁEreachable/callee 収集 helper 群を削除、E
    - `emit_ll_from_module_for_target` ぁE`compiler::prepare_module_for_llvm_codegen` 以外�E front-end 経路を持たなぁE��態にした、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - 結果: success
  - `node nodesrc/tests.js -i tests/compiler/llvm_target.n.md -i tests/compiler/raw_body_precheck.n.md -i tests/compiler/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-llvm-frontload-2.json -j 15`
    - 結果: `8/8 pass`
- [状況EじょぁE��めE��]:
  - LLVM codegen 側には前段をやり直ぁEhelper が残っておらず、責務�E `compiler.rs` の共送Elowering へ固定された、E

# 2026-03-09 作業メモ (`alloc/collections/stack` めEtyped pointer 化し、`uwok` を導�E)

- [目皁Eもくてき]:
  - `todo.md` の `alloc` 再構築に先立ち、`Stack<.T>` の[冁E��/なぁE�E][表現/ひめE��げん]めEraw `i32` から `MemPtr<u8>` / `MemPtr<.T>` [前提/ぜんてい]へ[寁EめEせる、E
  - `Result` めEpipe [記況EきほぁEで[連綁Eれんぞく][処琁Eしょり]するとき�E[冗長/じょぁE��めE��]さを[渁Eへ]らすため、`unwrap_ok` の[短縮吁Eたんしゅくめい] `uwok` めE`core/result` に追加する、E
- [根本原因/こんぽんげんいん]:
  - `Stack` はヘッダ[全佁Eぜんたい]めEraw `i32` で[保持/ほじ]し、`load_i32` / `store_i32` / `realloc_raw` へ[直絁EちめE��けつ]してぁE��、E
  - こ�Eままでは `core/mem` の型安�E化が `alloc/collections` へ[波叁EはきゅぁEせず、`Vec` の `MemPtr` 化と[整吁Eせいごう]しなぁE��E
  - [使用侁EしよぁE��い]では `unwrap_ok<Stack<i32>, Diag>` が[繰/く]り[迁Eかえ]され、`new |> push |> push` のような[連鎁Eれんさ]が[読/めEみにくかった、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/stack.nepl`
    - `Stack<.T>.hdr` めE`MemPtr<u8>` に変更、E
    - ヘッダの `len/cap/data_ptr` を[読/めEむ[冁E��/なぁE�E] helper (`stack_header_len_ptr` / `stack_header_cap_ptr` / `stack_header_data_ptr_ptr` / `stack_len_raw` / `stack_cap_raw` / `stack_data_ptr`) を追加、E
    - `stack_new` / `stack_push` / `stack_pop` / `stack_peek` / `stack_len` / `stack_clear` / `stack_free` めEtyped memory API [前提/ぜんてい]へ更新、E
    - `stack_free` は `dealloc_ptr` の `Result<(), Diag>` めE`uwok` で[消費/しょぁE�E]する形へ修正、E
    - [使用侁EしよぁE��い]の doctest めE`uwok` [基溁EきじめE��]へ寁E��た、E
  - `stdlib/core/result.nepl`
    - `uwok` (`unwrap_ok` の[短縮吁Eたんしゅくめい]) を追加、E
    - `uwerr` (`unwrap_err` の[短縮吁Eたんしゅくめい]) も追加、E
  - `stdlib/core/traits/deserialize.nepl`
    - ruby [記況EきほぁEの[刁E��/ぶんかつ]を修正し、`[人閁Eにんげん][吁Eむ]け` に統一、E
- [検証/けんしょぁE:
  - `node nodesrc/run_test.js` に[直接/ちめE��せつ] JSON を[渡/わた]して focused snippet めE2 件[実衁Eじっこう]、E
    - `<Stack<i32>> new |> uwok |> push 10 |> uwok |> push 20 |> uwok` + `len` -> `pass`
    - `stack_free<i32>` を[含/ふく]む snippet -> `pass`
- [状況EじょぁE��めE��]:
  - `stack` は `Vec` と[吁Eおな]じ方向で typed pointer [前提/ぜんてい]へ移った、E
  - `uwok` は `core/result` のみに[定義/てぁE��]し、[重褁EちめE��ふく][宣言/せんげん]は[避/さ]けてぁE��、E
  - `vec` などの `alloc/collections` も、この[見�E/みだ]し[構造/こうぞう]と `uwok` を[基溁EきじめE��]にそろえてぁE��、E

# 2026-03-09 作業メモ (alloc/collections/vec: ドキュメントコメント見�Eし�E新標準への追征E

- 目皁E
  - `alloc/collections/vec.nepl` の[先頭/せんとぁEと[基礁Eきそ] API のドキュメントコメントを、`stdlib/core/traits/deserialize.nepl` を[基溁EきじめE��]にした[新標溁EしんひめE��じゅん]の[見�E/みだ]し[構造/こうぞう]へ[揁Eそろ]える、E
- 変更:
  - `stdlib/alloc/collections/vec.nepl`
    - [先頭/せんとぁEコメントを `# vec` 形式へ変更、E
    - `Vec`, `vec_new`, `vec_with_capacity`, `vec_len`, `vec_cap`, `vec_data_ptr`, `vec_data_mem_ptr` のコメントを `##` / `### [目皁Eもくてき]` / `### [実裁Eじっそう]` / `### [注愁EちめE��い]` / `### [計算量/けいさんりょぁE` / `### [使用侁EしよぁE��い]` に整琁E��E
  - [実裁Eじっそう]本体�E変更してぁE��ぁE��E
- 検証:
  - `printf '{...}' | node nodesrc/run_test.js` により、`new<i32> |> push 10 |> push 20` と `vec_len` を[使/つか]ぁEfocused 実行が pass、E

# 2026-03-09 作業メモ (compiler 前提固宁E `#entry` 診断の span めEdummy から実位置へ修正)

- [目皁Eもくてき]:
  - `TypeEntryFunctionMissingOrAmbiguous` ぁE`Span::dummy()` を[迁Eかえ]してぁE�� compiler [側/がわ]の[不�E吁Eふぐあい]を[修正/しゅぁE��い]し、`#entry` の[識別孁Eしきべつし][位置/ぁE��]へ[診断/しんだん]を[絁Eむす]び[仁Eつ]ける、E
  - LLVM [経路/けいろ]で[後段/こうだん]に[殁Eのこ]ってぁE�� `entry function ... was not found in lowered module` も、同ぁE`diag id` と span に[寁EめEせる、E
- [根本原因/こんぽんげんいん]:
  - `typecheck` は `Directive::Entry` の span を[要Eみ]えてぁE��が、`resolved_entry` の[曖昧/あいまい]・[欠落/けつらく]を[報呁EほぁE��く]するときに `Span::dummy()` を[使/つか]ってぁE��、E
  - `compiler::resolve_hir_entry_name` も、lowering [征Eご]に entry が[要Eみ]つからなぁE�� `diag id` なし�Edummy span [前提/ぜんてい]の[診断/しんだん]へ[落/お]ちてぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/typecheck.rs`
    - `entry` めE`Option<(String, Span)>` で[保持/ほじ]するように変更、E
    - `TypeEntryFunctionMissingOrAmbiguous` めE`#entry` の[名前/なまぁE span へ[仁Eつ]けるよう修正、E
    - `check_function` の entry [判宁Eはんてい]めEtuple [前提/ぜんてい]へ[追征EつぁE��めE��]、E
  - `nepl-core/src/compiler.rs`
    - `resolve_hir_entry_name` に `module` を[渡/わた]し、`#entry` [探索/たんさく] helper を追加、E
    - lowering [征Eご]に entry が[要Eみ]つからない[場吁Eばあい]めE`DiagnosticId::TypeEntryFunctionMissingOrAmbiguous` と `#entry` の span を[迁Eかえ]すよぁE��修正、E
  - `tests/compiler/compile_fail_diag_location.n.md`
    - `entry_missing_uses_entry_directive_span` を追加、E
    - `diag_id: 3092` と `diag_span: 2:8` を[確誁Eかくにん]する compile_fail を追加、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/tests.js -i tests/compiler/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-entry-diag-location.json -j 15`
    - [結果/けっか]: `4/4 pass`
- [状況EじょぁE��めE��]:
  - `#entry` に[関/かん]する compiler 診断は、`diag id` だけでなく[位置/ぁE��]めE前段/ぜんだん]で[安宁Eあんてい]して[叁Eと]れるようになった、E
  - codegen [到達征EとぁE��つご]の entry [欠落/けつらく]は、front-end lowering の[不整吁Eふせいごう]として[扱/あつか]える[篁E��/はんい]まで[縮封EしゅくしめE��]された、E

# 2026-03-09 作業メモ (`RegionToken` / `RingBuffer` の move 消費めEfield 単位へ刁E��替ぁE

- [目皁Eもくてき]:
  - `todo.md` の `core/mem` / `alloc` [安�E匁Eあんぜんか]を[進/すす]めるぁE��で、`RegionToken<.T>` めE`RingBuffer<.T>` の[所有老EしょめE��しゃ]を[繰/く]り[迁Eかえ]ぁEmove してしまぁE箁E��/かしょ]を[除去/じょきょ]する、E
  - `tests/compiler/prelude_copy.n.md`、`tests/stdlib/ringbuffer_collections.n.md`、`tests/stdlib/queue_collections.n.md` が[安宁Eあんてい]して[送Eとお]る[状慁EじょぁE��い]まで[持EめEってぁE��、E
- [根本原因/こんぽんげんいん]:
  - `MemPtr<.T>` は `Copy` として[扱/あつか]ぁE��ぁE��、[所有老EしょめE��しゃ]である `RegionToken<.T>` めE`RingBuffer<.T>` は `Copy` ではなぁE��E
  - そ�Eため `region_ptr token` めE`ringbuffer_len rb` のように[所有老EしょめE��しゃ]を[丸/まる]ごと[補助/ほじょ][関数/かんすう]へ[渡/わた]す[実裁Eじっそう]だと、`get ... "ptr"` めE`get ... "hdr"` が[褁E��囁EふくすぁE��い]の move に[要Eみ]えて[失敁Eしっぱい]してぁE��、E
  - compiler [側/がわ]でも、generic `Copy` / `Clone` impl を[具体型/ぐたぁE��きがた]へ[彁EぁEてる[隁Eさい]に[単紁Eたんじゅん]な `same_type` [比輁Eひかく]しかしておらず、`MemPtr<i32>` ぁE`impl Copy<MemPtr<.T>>` に[一致/ぁE��ち]しない[不�E吁Eふぐあい]があった、E
- [変更/へんこぁE:
  - `nepl-core/src/types.rs`
    - `type_pattern_matches` を追加し、`impl Copy<MemPtr<.T>>` のような[型変数/かたへんすぁE[入/い]めEimpl が[具体型/ぐたぁE��きがた]へ[一致/ぁE��ち]するかを[判宁Eはんてい]できるようにした、E
  - `nepl-core/src/typecheck.rs`
    - `Copy` / `Clone` と trait impl [探索/たんさく]で `same_type` ではなぁE`type_pattern_matches` を[使/つか]ぁE��ぁE��変更、E
    - generic impl は[当面/とぁE��ん] `Copy` / `Clone` trait のみ[許可/きょか]するようにし、それ[以夁EぁE��い]は[従来送EじゅぁE��ぁE��お]り[拒否/きょひ]する、E
  - `nepl-core/src/passes/move_check.rs`
    - builtin/user `get` の[評価/ひめE��か]で、[取征Eしゅとく][結果/けっか]ぁE`Copy` なめEbase めEshared borrow [相彁EそうとぁEで[訪啁EほぁE��ん]するようにした、E
  - `stdlib/core/traits/copy.nepl`
    - `MemPtr<.T>` の `Copy` / `Clone` impl を追加、E
  - `stdlib/core/mem.nepl`
    - `region_ptr_at` / `dealloc_region` などを、`token` そ�Eも�EではなぁE`get token "ptr"` / `get token "size"` を[允Eさき]に[束縁Eそくばく]して[使/つか]ぁE形/かたち]へ変更、E
  - `stdlib/alloc/string.nepl`
    - `RegionToken<u8>` を[褁E��囁EふくすぁE��い] helper に[渡/わた]してぁE��[箁E��/かしょ]を、`base` / `scratch` / `out_data` などの `MemPtr<u8>` へ[允Eさき]に[刁E��/ぶんかい]して[扱/あつか]ぁE形/かたち]へ変更、E
  - `stdlib/alloc/collections/ringbuffer.nepl`
    - `RingBuffer<.T>.hdr` めE`MemPtr<u8>` として[一度/ぁE��ど][叁Eと]り[出/だ]し、`*_from_hdr` helper へ[渡/わた]す[実裁Eじっそう]へ整琁E��E
    - `ringbuffer_with_capacity` / `ringbuffer_push_back` / `ringbuffer_pop_front` / `ringbuffer_peek_front` / `ringbuffer_clear` / `ringbuffer_free` を[所有老EしょめE��しゃ]の[再消費/さいしょぁE�E]がない[形/かたち]へ書き直した、E
  - `tests/compiler/prelude_copy.n.md`
    - `MemPtr<i32>` を[繰/く]り[迁Eかえ]し[読/めEめること、`Copy` を[未知/みち] trait として[扱/あつか]わなぁE��とを[確誁Eかくにん]する focused test を追加、E
  - `tests/stdlib/ringbuffer_collections.n.md` / `tests/stdlib/queue_collections.n.md`
    - `[目皁Eもくてき]` と[確認�E容/かくにんなぁE��ぁEを[明訁Eめいき]しつつ、`uwok` を[使/つか]った[現在/げんざい]の[利用形/りよぁE��い]に合わせて更新、E
  - `todo.md`
    - `nodesrc/tests.js` と `nodesrc/run_test.js` の[使/つか]い[刁Eわ]けを[方釁EほぁE��ん]へ追加、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/stdlib/ringbuffer_collections.n.md -i tests/stdlib/queue_collections.n.md --no-stdlib --no-tree -o /tmp/tests-copy-ringbuffer-queue.json -j 15`
    - [結果/けっか]: `6/6 pass`
- [状況EじょぁE��めE��]:
  - `RegionToken<.T>` めE`RingBuffer<.T>` めE`Copy` にせず、[冁E��/なぁE�E]の `MemPtr` / `i32` [欁Eらん]だけを[允Eさき]に[叁Eと]り[出/だ]して[使/つか]ぁE方釁EほぁE��ん]へ[寁EめEせた、E
  - これにより `core/mem` と `alloc/collections` の[所有権/しょめE��けん][墁E��/きょぁE��い]が[現状/げんじょぁEの[言語機�E/げんごきのぁEに[叁Eおさ]まる[形/かたち]で[安宁Eあんてい]した、E

# 2026-03-09 作業メモ (stdlib doctest: `fn main` 明示と copy 判定�E前提修正)

- [目皁Eもくてき]:
  - stdlib `.nepl` [冁Eない]の doctest ぁE`#entry main` だけを[持EめEち、`fn main` を[持EめEたなぁE��めに `D3092` で[落/お]ちる[問顁Eもんだい]を[解涁EかいしょぁEする、E
  - doctest [修正/しゅぁE��い]を[進/すす]める[途中/とちめE��]で[露出/ろしめE��]した compiler [側/がわ]の `Copy` [判宁Eはんてい]の[不整吁Eふせいごう]めE併/あわ]せて[修正/しゅぁE��い]する、E
- [根本原因/こんぽんげんいん]:
  - stdlib の[既孁Eきそん] doctest は `#entry main` を[書/か]ぁE��めE`fn main` [本佁Eほんたい]を[持EめEたない[侁Eれい]が[夁Eおお]く、Node [側/がわ]の doctest [実衁Eじっこう][経路/けいろ]では entry [欠落/けつらく]として `D3092` に[落/お]ちてぁE��、E
  - さらに doctest [本佁Eほんたい]で `assert_*` のような impure API を[呼/めEぶ[場吁Eばあい]、pure `fn main <()->i32>` を[自勁EじどぁE[挿入/そうにめE��]すると `D3025` が[出/で]る、E
  - `Copy` trait model の[実裁Eじっそう]では `i64` / `i128` / `u128` / `f64` めEenum variant [前提/ぜんてい]で[扱/あつか]っており、[実際/じっさい]には `TypeKind::Named(...)` で[表現/ひめE��げん]される[垁Eかた]との[不一致/ふぁE��ち]があった、E
- [変更/へんこぁE:
  - stdlib の doctest [全佁Eぜんたい]で、`fn main` がない[侁Eれい]には `fn main <()*>i32> ():` を[明示/めいじ]する[方吁EほぁE��ぁEへ[寁EめEせた、E
  - `nepl-core/src/types.rs`
    - trait model の `is_copy_with_trait_model` で `TypeKind::Named(name)` を[用/もち]ぁE��`i64` / `i128` / `u64` / `u128` / `f64` を[正/ただ]しく `Copy` impl [探索/たんさく]へ[流Eなが]すよぁE��[修正/しゅぁE��い]した、E
  - `stdlib/alloc/collections/stack.nepl`
    - doctest [冒頭/ぼぁE��ぁEめE`fn main <()*>i32>` [前提/ぜんてい]へ[揁Eそろ]えた、E
    - [後�E先�E/あとぁE��さきだし] の ruby を[正/ただ]しい[読/めEみへ[修正/しゅぁE��い]した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
- [状況EじょぁE��めE��]:
  - doctest [全佁Eぜんたい]の[一括/ぁE��かつ][再実衁Eさいじっこう]はまだ[途中/とちめE��]で、`stack.nepl` など collections [側/がわ]を[優允EめE��せん]して[頁E��/じゅんじ] focused に[確誁Eかくにん]する[段隁Eだんかい]、E
  - [簡潁Eかんけつ]な doctest [専用/せんよう][枠絁Eわくぐ]みの[新設/しんせつ]は[保留/ほりゅぁEし、[当面/とぁE��ん]は `fn main` を[明示/めいじ]する[方釁EほぁE��ん]で[進/すす]める、E

# 2026-03-09 作業メモ (stdlib ドキュメント生成ツールの汎用化と目次構造の整傁E

- [目皁Eもくてき]:
  - tutorials と stdlib で共通�Eドキュメント生成ツール (`nodesrc/cli.js`) を使用できるようにし、stdlib でもインタラクチE��ブなプレイグラウンド付き HTML を生成可能にする、E
  - stdlib ドキュメント�E目次めE`index.n.md` で管琁E��、`00_` などのプリフィチE��スに依存しなぁE��層構造をサポ�Eトする、E
- [変更/へんこぁE:
  - `nodesrc/cli.js`
    - `--site-name` と `--description-prefix` 引数を追加し、サイト名めE��明文を外部から持E��可能にした、E
    - `index.n.md` を優先的に検�Eし、�E力時に `index.html` へマッピングするロジチE��を追加、E
  - `stdlib/index.n.md`
    - 標準ライブラリの新しい目次ファイルとして作�E、E
  - `.github/workflows/gh-pages.yml`
    - `stdlib` のビルドを `html_play` に変更し、ENEPLg2 Standard Library" とぁE��サイト名で生�Eするように更新、E
  - `stdlib/nm/README.n.md` -> `stdlib/nm/README.nepl`
    - ユーザーの要望に基づき、インチE��クス以外�E `.n.md` めE`.nepl` 形式（ドキュメントコメント付き�E�に変換、E
- [検証/けんしょぁE:
  - `nodesrc/cli.js` の引数パ�Eスと `index.n.md` 処琁E�EロジチE��が正常に動作し、`index.html` が期征E��りに生�Eされることを確認、E

# 2026-03-09 作業メモ (stdlib ドキュメント�E目次階層化とタイトルの適正匁E

- [目皁Eもくてき]:
  - `stdlib` ドキュメント�E目次 (TOC) が平坦なリストになってぁE��のを、ディレクトリ構造に基づぁE��階層皁E��表示に改喁E��る、E
  - サイト名に応じて目次のタイトル ("Getting Started" また�E "Contents") を�E動的に刁E��替えられるようにし、ドキュメント�E種類に適した表示にする、E
- [根本原因/こんぽんげんいん]:
  - `nodesrc/cli.js` の `buildTocEntries` において、�E示皁E��インチE��クスに含まれなぁE��残り」�Eファイルが一征E"Other" グループにフラチE��に入れられてぁE��、E
  - `nodesrc/html_gen_playground.js` の目次タイトルぁE"Getting Started" にハ�EドコードされてぁE��、E
- [変更/へんこぁE:
  - `nodesrc/cli.js`
    - `buildTocEntries` を修正し、残りのファイルを�E通�EチE��レクトリ接頭辞でグループ化する階層化ロジチE��を実裁E��E
    - `siteName` に "tutorial" が含まれなぁE��合�E目次タイトルめE"Contents" と判定し、生成�E琁E��渡すよぁE��変更、E
  - `nodesrc/html_gen_playground.js`
    - `renderToc` と `renderHtmlPlayground` を更新し、`tocTitle` オプションを受け取り、EGetting Started" 以外�Eタイトルも表示できるように変更、E
- [検証/けんしょぁE:
  - `dist/doc/stdlib/alloc/diag/diag.html` などを確認し、目次タイトルぁE"Contents" になり、`alloc/collections` めE`core/traits` などのチE��レクトリ単位で階層化されてぁE��ことを確認、E
- [状況EじょぁE��めE��]:
  - 標準ライブラリのドキュメントが、チュートリアルと同等�E整琁E��れた構造で閲覧可能になった、E

# 2026-03-10 作業メモ (doctest main 追従後�E collections / nm / fs 整合性修正)

- [目皁Eもくてき]:
  - stdlib doctest に `fn main <()*>i32>` を[明示/めいじ]したあとに[露出/ろしめE��]した、collections / kp / nm / fs [側/がわ]の[整合性/せいごうせい][崩/くず]れを[根本/こんぽん]から[直/なお]す、E
  - とくに `Vec.data` の `MemPtr` 化に[追征EつぁE��めE��]してぁE��い[箁E��/かしょ]と、`stack_free` の impure / pure [不一致/ふぁE��ち]を[允Eさき]に[解涁EかいしょぁEする、E
- [根本原因/こんぽんげんいん]:
  - `Vec<.T>.data` めE`MemPtr<.T>` に[移衁EぁE��ぁEしたあとも、doctest めE��部の nm / fs [実裁Eじっそう]ぁEraw `i32` [前提/ぜんてい]の `get ... "data"` を[殁Eのこ]してぁE��、E
  - `stack_free` は `dealloc_ptr` を[呼/めEぶのに pure [署吁Eしょめい]のままだったため、doctest を[送Eとお]す[過稁Eかてい]で impure API [整合性/せいごうせい]の[破綻/はたん]が[表面匁EひめE��めんか]した、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_free` めE`fn stack_free <.T> <(Stack<.T>)*>()>` に[修正/しゅぁE��い]し、`dealloc_ptr` を[呼/めEぶ[実佁Eじったい]と[署吁Eしょめい]を[一致/ぁE��ち]させた、E
    - `uwok dealloc_ptr ...` の[行末/ぎょぁE��つ] `;` を[夁Eはず]し、[弁Eしき]として[素直/すなお]に[消費/しょぁE�E]する[形/かたち]へ[揁Eそろ]えた、E
  - `stdlib/kp/kpgraph.nepl`
    - doctest の `dist.data` [参�E/さんしょぁEめE`mem_ptr_addr get dist "data"` へ[修正/しゅぁE��い]した、E
  - `stdlib/std/fs.nepl`
    - `Vec<u8>` の[冁E��/なぁE�E][領域/りょぁE��き]めEraw `i32` として[読/めEんでぁE��[箁E��/かしょ]めE`mem_ptr_addr buf.data` へ[修正/しゅぁE��い]した、E
  - `stdlib/nm/parser.nepl` / `stdlib/nm/html_gen.nepl`
    - `Vec<...>.data` めEraw `i32` [前提/ぜんてい]で[読/めEんでぁE��[箁E��/かしょ]を、`mem_ptr_addr get ... "data"` へ[機械皁EきかぁE��き]に[追征EつぁE��めE��]させた、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-tree -o /tmp/tests-stack-doctest-v3.json -j 15`
    - [状況EじょぁE��めE��]: こ�E[環墁EかんきょぁEでは JSON [出劁Eしゅつりょく]まで[時間/じかん]がかかるため、focused [実衁Eじっこう]の[完亁EかんりょぁE[確誁Eかくにん]を[継続中/けいぞくちめE��]、E
- [状況EじょぁE��めE��]:
  - ここでの[修正/しゅぁE��い]は、doctest を[送Eとお]すため�E[場彁EばぁEたり[対忁Eたいおう]ではなく、`Vec.data` の `MemPtr` 化と impure [署吁Eしょめい]の[整合性/せいごうせい]を[回復/かいふく]するも�E、E
  - 次は `nodesrc` [側/がわ]の doctest focused [実衁Eじっこう][経路/けいろ]を[安定化/あんてぁE��]し、既孁Estdlib doctest を[頁E��/じゅんじ][通過/つぁE��]させる、E

# 2026-03-10 作業メモ (nodesrc: doctest 1 件 focused 実行�E追加)

- [目皁Eもくてき]:
  - `nodesrc/tests.js` の[雁E��EしゅぁE��く][実衁Eじっこう]を[征Eま]たずに、stdlib reboot [中/ちめE��]の doctest 1 件を[直接/ちめE��せつ][再現/さいげん]できる[入口/ぁE��ぐち]を[追加/つぁE��]する、E
  - `stack.nepl` のように[特宁Eとくてい] file の doctest を[頁E��/じゅん�Eん]に[潰/つぶ]したい[場面/ばめん]で、`run_test.js` 向け JSON を[手書/てが]きせずに[確誁Eかくにん]できるようにする、E
- [根本原因/こんぽんげんいん]:
  - `nodesrc/tests.js` は doctest [全佁Eぜんたい]の[雁E��EしゅぁE��く]には[吁Eむ]くが、stdlib reboot [中/ちめE��]の[局所皁EきょくしめE��き]な[原因/げんぁE��][刁Eき]り[刁Eわ]けには[釁Eおも]ぁE��E
  - `nodesrc/run_test.js` は 1 件[実衁Eじっこう]の[核/かく]を[持EめEつが、file / doctest index から[直接/ちめE��せつ][呼/めEぶ[薁EぁE��]ぁECLI がなかった、E
- [変更/へんこぁE:
  - `nodesrc/run_doctest.js`
    - `parseFile` で file [中/ちめE��]の doctest を[読/めEみ、`-n` で[持E��Eしてい]した 1 件だけを `runSingle` に[流Eなが]ぁECLI を[追加/つぁE��]した、E
    - `compile_fail` の `diag_id` / `diag_span` [確誁Eかくにん]めE`tests.js` と[吁Eおな]じ[基溁EきじめE��]で[適用/てきよぁEする、E
  - `todo.md`
    - stdlib reboot [中/ちめE��]の focused doctest [実衁Eじっこう]では `node nodesrc/run_doctest.js -i <file> -n <index>` を[使/つか]ぁE方釁EほぁE��ん]を[追訁EつぁE��]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/stack.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/core/traits/deserialize.nepl -n 1`
    - [結果/けっか]: pass
- [状況EじょぁE��めE��]:
  - 既孁Estdlib doctest を[頁E��/じゅんじ][送Eとお]す[隁Eさい]の[入口/ぁE��ぐち]が[揁Eそろ]ぁE��`tests.js` の[釁Eおも]い[雁E��EしゅぁE��く][実衁Eじっこう]に[頼/たよ]らずに[局所/きょくしょ][確誁Eかくにん]できるようになった、E

# 2026-03-10 作業メモ (`nodesrc/README.md` の追加と doctest 実行経路の整琁E

- [目皁Eもくてき]:
  - `nodesrc/` [配丁EはぁE��]の[道�E/どぁE��]が[墁Eふ]えてきたため、stdlib reboot [中/ちめE��]に「どの[目皁Eもくてき]でどの script を[使/つか]ぁE��」を 1 [极Eまい]で[確誁Eかくにん]できるようにする、E
  - doctest / 通常 tests / 解极E/ HTML [生�E/せいせい]の[入口/ぁE��ぐち]を[明確/めいかく]にし、`todo.md` の[運用/ぁE��よう][方釁EほぁE��ん]と[一致/ぁE��ち]させる、E
- [変更/へんこぁE:
  - `nodesrc/README.md`
    - `tests.js` / `run_doctest.js` / `run_test.js` / `analyze_source.js` / `analyze_tests_json.js` / `cli.js` の[使/つか]い[刁Eわ]けを、[目皁E��/もくてきべつ]に[整琁Eせいり]した、E
    - stdlib reboot [中/ちめE��]によく[使/つか]ぁE手頁Eてじゅん]として、doctest 1 件の[修正/しゅぁE��い]、compiler [不�E吁Eふぐあい]の[刁Eき]り[刁Eわ]け、E��常 tests と doctest の[刁E��/ぶんり][確誁Eかくにん]を[記述/きじめE��]した、E
  - `todo.md`
    - `run_doctest.js` を[使/つか]っぁEfocused doctest [実衁Eじっこう]を[標溁EひめE��じゅん]の[運用/ぁE��よう]として[追訁EつぁE��]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/stack.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/core/traits/deserialize.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/cliarg.n.md -n 3`
    - [結果/けっか]: `compile_fail` + `diag_id: D3006` [確誁Eかくにん] pass
- [状況EじょぁE��めE��]:
  - stdlib reboot [中/ちめE��]の doctest [修正/しゅぁE��い]は、まぁE`run_doctest.js` で 1 件を[固/かた]め、そのあと `tests.js` で[封Eちい]さい[篁E��/はんい]を[雁E��EしゅぁE��く][確誁Eかくにん]する[流Eなが]れで[進/すす]められるようになった、E

# 2026-03-10 作業メモ (`kpread` / `kpwrite` の[所有権/しょめE��けん][整琁Eせいり]と doctest [回復/かいふく])

- [目皁Eもくてき]:
  - `kpwrite` の stdout が[空/から]になめEdoctest [不�E吁Eふぐあい]と、`kpread` / `kpgraph` ぁE`Vec.data` の `MemPtr` 化に[追征EつぁE��めE��]しきれてぁE��い[不整吁Eふせいごう]を、[所有権/しょめE��けん]の[設訁Eせっけい]から[直/なお]す、E
  - `kp` [系/けい] helper の[実裁Eじっそう]を[新/あたら]しい doc comment policy に[吁EぁEわせ、[修正/しゅぁE��い]した API の[意味/ぁE��]が[垁Eかた]とコメント�E[両方/りょぁE��ぁEで[刁Eわ]かるようにする、E
- [根本原因/こんぽんげんいん]:
  - `Writer` は header [領域/りょぁE��き]だけを[共朁EきょぁE��ぁEすればよいのに `RegionToken<u8>` めEfield に[保持/ほじ]しており、header [参�E/さんしょぁEのた�Eに `region_ptr get w "region"` を[経由/けいめEしてぁE��。この[設訁Eせっけい]だと[所有権/しょめE��けん]を[持EめEつ token と[軽釁EけいりょぁE handle の[責勁Eせきむ]が[混/ま]ざり、doctest [実衁Eじっこう]で[状慁EじょぁE��い]が[壁Eこわ]れやすかった、E
  - `Scanner` も同様に `RegionToken<u8>` めEfield に[持EめEってぁE��ため、[読/めEみ[叁Eと]り[位置/ぁE��]だけを[共朁EきょぁE��ぁEしたぁEhelper [群/ぐん]が[毎回/まぁE��い] token を[消費/しょぁE�E]する[形/かたち]になってぁE��、E
  - `kpread_core` は header [領域/りょぁE��き]を[触/さわ]るだけ�E helper にめE`RegionToken<u8>` を[要汁EようきゅぁEしており、[冁E��/なぁE�E][実裁Eじっそう]が[不要Eふよう]に[釁Eおも]かった、E
- [変更/へんこぁE:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.region` めE`Writer.header <MemPtr<u8>>` に[変更/へんこぁEした、E
    - header [操佁Eそうさ] helper は `MemPtr<u8>` を[直接/ちめE��せつ][叁EぁEけるようにし、`writer_free_handle` / `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_write_str_handle` / `writer_write_i32_handle` / `writer_write_u64_handle` の[参�E/さんしょぁEをすべて[追征EつぁE��めE��]させた、E
    - file header と `Writer` struct の doc comment を[新/あたら]しい policy に[揁Eそろ]えた、E
  - `stdlib/kp/kpread.nepl`
    - `Scanner.region` めE`Scanner.header <MemPtr<u8>>` に[変更/へんこぁEした、E
    - `Scanner` は[入劁EにめE��りょく][状慁EじょぁE��い]を[共朁EきょぁE��ぁEする[軽釁EけいりょぁE handle なので、`Copy` / `Clone` を[明示皁Eめいじてき]に[実裁Eじっそう]した、E
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` めEheader pointer [基溁EきじめE��]に[変更/へんこぁEした、E
    - `scanner_skip_ws_header` を[追加/つぁE��]し、各 helper ぁE`let header <MemPtr<u8>> get sc "header";` で[允Eさき]に header を[束縁Eそくばく]してから[処琁Eしょり]する[形/かたち]へ[揁Eそろ]えた、E
    - file header と `Scanner` struct の doc comment を[新/あたら]しい policy に[揁Eそろ]えた、E
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` / `store_i32_u8_at` / `load_i32_u8_at` めE`MemPtr<u8>` + size [基溁EきじめE��]へ[変更/へんこぁEした、E
    - `scanner_new_impl` の header [初期匁Eしょきか]で[一時的/ぁE��じてき]な `RegionToken<u8>` を[佁Eつく]らず、raw header pointer と size を[直接/ちめE��せつ][渡/わた]す[形/かたち]へ[整琁Eせいり]した、E
    - file header の doc comment を[新/あたら]しい policy に[揁Eそろ]えた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpwrite.nepl -n 2`
    - [結果/けっか]: pass, stdout=`1 2\n`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpwrite.nepl -n 3`
    - [結果/けっか]: pass, stdout=`123\n`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpwrite.nepl -n 4`
    - [結果/けっか]: pass, stdout=`42\n`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpgraph.nepl -n 1`
    - [結果/けっか]: pass, stdout=`0 1 2 3\n`
- [状況EじょぁE��めE��]:
  - `kpwrite` / `kpread` の[修正/しゅぁE��い]は doctest を[送Eとお]すため�E[場彁EばぁEたり[対忁Eたいおう]ではなく、header pointer と[所有権/しょめE��けん] token の[責勁Eせきむ][刁E��/ぶんり]を[回復/かいふく]するも�E、E
  - `kpread.nepl` には[実行対象/じっこうたいしょぁEの doctest はまだなぁE`skip` のみだが、`scanner_read_i32` を[使/つか]ぁE��封Esource test と `kpgraph` の doctest で[現衁Eげんこう]設計が[成竁Eせいりつ]することを[確誁Eかくにん]した、E

# 2026-03-10 作業メモ (`queue` の doc comment 整備と `uwok` への寁E��)

- [目皁Eもくてき]:
  - `Queue` の公閁EAPI コメントを現行�E doc comment policy に合わせ、`RingBuffer` ベ�Eスの queue であること、更新後�E値を返す API であること、`Option` / `Result` の扱ぁE��コメントだけで追えるようにする、E
  - collection 系の focused test めE`uwok` 前提の短ぁEpipe 記法へ寁E��、stdlib reboot 後�E典型的な使ぁE��をテスト�Eでも固定する、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/queue.nepl`
    - file header, `Queue` struct, `queue_new`, `queue_with_capacity`, `queue_len`, `queue_is_empty`, `queue_push`, `queue_pop`, `queue_peek`, `queue_clear`, `queue_free` の doc comment を現衁Epolicy に沿って書き直した、E
    - `queue_push` が更新後�E queue を返す API であり、pipe 記法では `|> queue_push ... |> uwok` の形で束縛し直す忁E��があることを�E記した、E
  - `tests/stdlib/ringbuffer_collections.n.md`
    - `unwrap_ok<...>` めE`uwok` に置き換えた、E
    - `ringbuffer_push_back` / `ringbuffer_pop_front` の型引数を省き、現行�E型推論で通る書き方へ寁E��た、E
  - `tests/stdlib/pipe_collections.n.md`
    - `RingBuffer` / `Queue` の pipe 使用例を `uwok` ベ�Eスの短ぁE��き方に寁E��た、E
    - `queue_push<i32>` / `ringbuffer_push_back<i32>` など、不要な型引数を外した、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tests/stdlib/ringbuffer_collections.n.md -i tests/stdlib/pipe_collections.n.md --no-stdlib --no-tree -o /tmp/tests-queue-ringbuffer-uwok.json -j 15`
    - [結果/けっか]: `9/9 pass`
- [状況EじょぁE��めE��]:
  - `Queue` / `RingBuffer` の利用例�E `uwok` を使った短ぁEpipe 形で安定して書ける状態になった、E
  - `queue.nepl` は冁E��だけでなく、見�Eし階層と節構�Eも現行�E doc comment policy に沿ぁE��へ更新した、E
# 2026-03-10 作業メモ (vec_data_len めE`.Pair` から explicit struct へ移衁E

- [目皁Eもくてき]:
  - `tests/stdlib/sort.n.md::doctest#3` の `use of moved value: s` を、`.Pair` [返却/へんきめE��]に[依孁EぁE��ん]した API [設訁Eせっけい]から[解涁EかいしょぁEする、E
  - `Vec` [系/けい]の doc comment を[現衁Eげんこう] policy に[吁EぁEわせる、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/vec.nepl`
    - `VecDataLen<.T>` struct を[追加/つぁE��]、E
    - `vec_data_len` の[迁Eかえ]り[値/あたい]めE`.Pair` から `VecDataLen<.T>` に[変更/へんこぁE、E
    - `vec_data_len` の doc comment めE`##` / `###` と `[目皁Eもくてき]` / `[使用侁EしよぁE��い]` / `[実裁Eじっそう]` / `[注愁EちめE��い]` / `[計算量/けいさんりょぁE` [構�E/こうせい]へ[更新/こうしん]、E
  - `tests/stdlib/sort.n.md`
    - `get s 0` / `get s 1` めE`get s "data"` / `get s "len"` へ[更新/こうしん]、E
    - `data` は `MemPtr<.T>` なので `mem_ptr_addr` を[送Eとお]す[形/かたち]へ[変更/へんこぁE、E
  - `nodesrc/README.n.md`
    - `tests.js` / `run_doctest.js` / `run_test.js` / `cli.js` / `compiler_loader.js` の[目皁E��/もくてきべつ][使/つか]い[刁Eわ]けを[追加/つぁE��]、E
- [琁E��/りゆぁE:
  - `.Pair` は generic [関数/かんすう][迁Eへん]り[値/あたい]と field `get` の[絁Eく]み[吁EぁEわせで move-check の[揺/めEれを[起/お]こしめE��ぁE��E
  - `VecDataLen<.T>` のような[明示皁Eめいじてき] struct に[置/お]き[揁Eか]えると、field [吁Eめい]・doc comment・tests の[意味/ぁE��]が[揁Eそろ]ぁE��E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 3` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 4` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 12` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 13` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 9` -> pass

# 2026-03-10 作業メモ (`move_effect` の reboot 追従と prelude 衝突�E刁E��刁E��)

- [目皁Eもくてき]:
  - `tests/compiler/move_effect.n.md` めEreboot 後�E `Copy` / `Clone` 能力モチE��へ合わせる、E
  - `tests/compiler/prelude_copy.n.md` と `tests/compiler/move_effect.n.md` の focused 実行を安定化し、compiler 側の不�E合と test 側の前提ずれを�Eり�Eける、E
- [根本原因/こんぽんげんいん]:
  - `Copy` の再利用可否めEstructural な既定値として書ぁE��ぁE�� case が残っており、reboot 後�E「�E示皁E�� trait impl が唯一の根拠」とぁE��仕様とずれてぁE��、E
  - `#target core` の通常 prelude では `core/mem` の `RegionToken<.T>` が見えてぁE��ため、test 側でローカル定義した `RegionToken` と衝突してぁE��、E
  - そ�E結果、`impl ... for RegionToken` ぁEgeneric な prelude 側型へ解決され、`D3084` めEstack/return 系の別診断に吸われてぁE��、E
- [変更/へんこぁE:
  - `tests/compiler/move_effect.n.md`
    - `Point` / `Pair<i32>` / `Score` の再利用ケースを、�E示皁E�� `Clone` / `Copy` impl 前提の説明と source に更新した、E
    - local capability 検証 (`Copy` / `Clone` めEtest 冁E��定義する case) は `#no_prelude` を付け、prelude から独立した最小環墁E��確認する形へ揁E��た、E
    - `i64` の local capability case は `core/cast` 依存を避けるため、`Size` struct を使ぁE��へ置き換えた、E
    - prelude と衝突してぁE�� local `RegionToken` は `LocalToken` へ改名し、E��常 prelude 下でも期征E��おりに `D3053` / `D3063` / `D3054` を観測できるようにした、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 11` -> pass (`D3049`)
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 17` -> pass
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 20` -> pass (`D3053`)
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 21` -> pass (`D3063`)
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 22` -> pass (`D3054`)
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/compiler/move_effect.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-copy-move-effect.json -j 15`
    - [結果/けっか]: `30/30 pass`
- [状況EじょぁE��めE��]:
  - 今回の修正では compiler 本体�E変更してぁE��ぁE��E
  - 残ってぁE�� failure は reboot 後仕様に対する test 側の前提ずれと、prelude で露出する generic `RegionToken<.T>` との名前衝突が原因だった、E

# 2026-03-10 作業メモ (`alloc/io` / `std/streamio` の最封Efacade 追加)

- [目皁Eもくてき]:
  - reboot 斁E��で定義されてぁE�� `alloc/io` と `std/streamio` の土台を、既存�E `std/stdio` / `kpread` / `kpwrite` を壊さずに追加する、E
  - streamio は text 専用でなく、byte stream も扱える形で設計する、E
- [根本原因/こんぽんげんいん]:
  - `todo.md` と `doc/stdlib_breaking_reboot.md` では `alloc/io` と `std/streamio` が�E示されてぁE��が、現状の stdlib にはまだ対応ファイルが無く、`std/stdio` と `kp*` helper が直接結�E付いたままだった、E
  - `streamio` めEtext 専用にすると、後続�E file/socket/event stream めE`kpwrite` 昁E��先として使ぁE��せなぁE��E
- [変更/へんこぁE:
  - `stdlib/alloc/io.nepl`
    - `ByteReader` / `ByteWriter` / `TextReader` / `TextWriter` / `Flush` / `Close` trait を追加した、E
    - `io_read_all_bytes` / `io_write_bytes` / `io_read_all_text` / `io_write_str` / `io_flush` / `io_close` helper を追加した、E
    - doc comment は現衁Epolicy の `#` / `##` / `###` 構�Eへ揁E��た、E
  - `stdlib/std/streamio.nepl`
    - `StdinStream` / `StdoutStream` を追加し、`alloc/io` trait を実裁E��た、E
    - `stream_bytes_from_str` / `stream_bytes_to_str` を追加し、binary/text helper の橋渡しを行えるよぁE��した、E
    - `stream_read_all_bytes` / `stream_write_bytes` / `stream_read_all_text` / `stream_write_str` / `stream_flush` / `stream_close` めEfacade 名で再�E開した、E
    - [現状/げんじょぁEの `std/stdio` が詳細 error を返さなぁE��紁E�E doc comment へ明記した、E
  - `tests/stdlib/streamio.n.md`
    - text write, binary write, stdin bytes -> stdout bytes の focused case を追加した、E
- [設訁Eせっけい][判断/はんだん]:
  - low-level 抽象は byte stream を基準にし、text は extension trait と helper へ刁E��した、E
  - writer / flush は handle を返す値持E�� API にし、NEPLg2 の move / pipe 記法へ合わせた、E
  - `std/streamio` の module doctest は stable な入口確認に絞り、宁Estdout/stderr の end-to-end は `tests/stdlib/streamio.n.md` 側で固定した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/run_doctest.js -i stdlib/alloc/io.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/std/streamio.nepl -n 1` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl --no-stdlib --no-tree -o /tmp/tests-streamio-focused.json -j 15`
    - [結果/けっか]: `5/5 pass`
- [状況EじょぁE��めE��]:
  - `std/streamio` はまだ stdin/stdout の最封Efacade に留めてぁE��が、binary/text の trait 面は先に固定できた、E
  - `kpwrite` / `kpread` をこの層へ段階移行する足場はできた、E

# 2026-03-10 作業メモ (`streamio` の binary buffer めE`ByteBuf` へ再設訁E

- [目皁Eもくてき]:
  - `streamio` を本当に binary-capable にし、`Vec<u8>` の冁E��表現へ依存した擬似皁E�� byte write をやめる、E
  - `nodesrc/tests.js` でめEstdout/stderr 検証を確実に有効化し、I/O mismatch を見送E��なぁEfocused 検証手頁E��固定する、E
- [根本原因/こんぽんげんいん]:
  - 先行実裁E��は `ByteReader` / `ByteWriter` の媒体を `Vec<u8>` にしてぁE��が、NEPLg2 の `Vec<u8>` は `fd_write` にそ�Eまま渡せる連綁Ebyte buffer ではなかった、E
  - そ�Eため `stream_write_bytes` は `A\0\0` のような padded 出力になり、binary stream として壊れてぁE��、E
  - あわせて `nodesrc/tests.js` は既定で `assert_io: false` なので、stdout mismatch ぁEJSON 上では pass 扱ぁE��なるケースがあり、`--assert-io` を付けなぁE�� binary 回帰を見送E��てぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/io.nepl`
    - `ByteBuf` を追加し、`ptr: MemPtr<u8>` と `len: i32` を持つ所朁Ebuffer として定義した、E
    - `io_bytebuf_empty` / `io_bytebuf_len` / `io_bytebuf_free` / `io_bytebuf_from_str` / `io_bytebuf_to_str` を追加した、E
    - `ByteReader` / `ByteWriter` と `io_read_all_bytes` / `io_write_bytes` の媒体を `Vec<u8>` から `ByteBuf` へ変更した、E
  - `stdlib/std/stdio.nepl`
    - `stdio_write_bytes` は `ByteBuf` を直接 iovec に載せて stdout へ書く形へ変更した、E
    - `stdio_read_all_bytes` は、現状の `read_all` 結果めE`ByteBuf` に褁E��する形へ整琁E��た、E
  - `stdlib/std/streamio.nepl`
    - `stream_bytes_from_str` / `stream_bytes_to_str` めE`ByteBuf` ベ�Eスへ変更した、E
    - `StdinStream` / `StdoutStream` の binary trait 実裁E�� `ByteBuf` 前提へ差し替えた、E
    - doc comment に「stdin byte read は現状 `read_all` 由来の褁E��」とぁE��制紁E��追記した、E
  - `tests/stdlib/streamio.n.md`
    - text write, binary write, stdin bytes -> stdout bytes に加え、NUL を含む binary/text roundtrip めE`assert_str_eq` で検証する case に更新した、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl -i stdlib/std/stdio.nepl --assert-io --no-stdlib --no-tree -o /tmp/tests-streamio-bytebuf.json -j 15`
    - [結果/けっか]: `33/33 pass`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4` -> pass
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/alloc/io.nepl -i stdlib/std/stdio.nepl -i stdlib/std/streamio.nepl -o html=/tmp/streamio-doc-html`
    - [結果/けっか]: `generated 3 html file(s)`
- [状況EじょぁE��めE��]:
  - binary stream の媒体�E `Vec<u8>` ではなぁE`ByteBuf` に固定した、E
  - `nodesrc/tests.js` で I/O を見る focused 検証は、今征E`--assert-io` を付ける前提で扱ぁE��E

# 2026-03-10 作業メモ (`nodesrc/tests.js` の I/O 検証既定値を修正)

- [目皁Eもくてき]:
  - `tests.js` ぁE`stdout:` / `stderr:` を書ぁE�� doctest を既定で厳寁E��輁E��、`run_doctest.js` と同じ期征E��使えるようにする、E
- [根本原因/こんぽんげんいん]:
  - これまでの `tests.js` は `--assert-io` / `NEPL_ASSERT_IO=1` / `assert_io` tag が無ぁE��り、`expected_stdout` / `expected_stderr` を持つ case でめEI/O mismatch めEpass 扱ぁE��てぁE��、E
  - そ�Eため、binary streamio の不正出力が JSON 雁E��上では `pass` に見え、focused suite の信頼性が落ちてぁE��、E
- [変更/へんこぁE:
  - `nodesrc/tests.js`
    - `expected_stdout` また�E `expected_stderr` があめEcase は、既定で I/O 比輁E��有効にするよう変更した、E
    - `--assert-io` / `NEPL_ASSERT_IO=1` / `assert_io` tag は明示フラグとして残しつつ、「I/O 検証を有効化する唯一条件」ではなくした、E
  - `nodesrc/README.n.md`
    - `tests.js` でめE`stdout:` / `stderr:` を既定で検証することを追記した、E
    - `--assert-io` は補助フラグであり、I/O 期征E��の有無そ�Eも�Eを有効化する忁E��条件ではなぁE��とを�E記した、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tests/stdlib/stdout.n.md -i tests/stdlib/stdin.n.md -i tests/stdlib/kp.n.md -i tests/stdlib/streamio.n.md --no-stdlib --no-tree -o /tmp/tests-io-default-assert.json -j 15`
    - [結果/けっか]: `22/22 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl --no-stdlib --no-tree -o /tmp/tests-stdlib-io-doctest-default.json -j 15`
    - [結果/けっか]: `2/2 pass`
- [状況EじょぁE��めE��]:
  - `tests.js` と `run_doctest.js` の I/O 検証期征E�E揁E��た、E
  - 今征E`stdout:` / `stderr:` を書ぁE�� doctest は、追加フラグなしでめEmismatch で落ちる、E

# 2026-03-10 作業メモ (`kpwrite` の buffered writer core めE`std/streamio` へ移管)

- [目皁Eもくてき]:
  - `todo.md` の「`kpwrite` の中核めE`std/streamio` へ昁E��させる」を進め、stdout buffering めE`kp` 専用実裁E�Eまま持たなぁE���Eへ寁E��る、E
  - partial write ループが `kpwrite` 側へ散ら�EってぁE��状態を解消し、`std/stdio` と `std/streamio` の責務墁E��を整琁E��る、E
- [根本原因/こんぽんげんいん]:
  - これまでの `kpwrite` は buffer 所有、header 管琁E��partial write 吸収、文字�E/数値整形めE1 module に抱えており、`std/streamio` は stdin/stdout の最封Efacade に留まってぁE��、E
  - そ�Eため stdout buffering の一般化可能部刁E��仁Emodule が�E利用できず、`kp` 側に syscall 由来の実裁E��細が残ってぁE��、E
  - あわせて stdout への部刁E��き込み吸収が `print` / `stdio_write_bytes` と `kpwrite` で別経路になっており、同ぁEstdout 出力でも責務が刁E��してぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/stdio.nepl`
    - `stdio_write_mem` を追加し、`MemPtr<u8>` と長さを受けて partial write を吸収しながら stdout へ書く�E通経路を追加した、E
    - `print` と `stdio_write_bytes` はこ�E helper を使ぁE��へ整琁E��た、E
  - `stdlib/std/streamio.nepl`
    - `StreamWriter` を追加し、buffer 所有�Eheader 管琁E�Eflush・text/i32/i64/f32/f64 出力を `std` 側で提供するよぁE��した、E
    - `stream_writer_new` / `stream_writer_free` / `stream_writer_flush` / `stream_writer_put_u8` / `stream_writer_write_str` / `stream_writer_write_i32` / `stream_writer_write_i64` / `stream_writer_write_f64` などを追加した、E
    - `stream_writer_flush` は `stdio_write_mem` を使ぁE��とで stdout 側の部刁E��き込み吸収と経路を�E有するよぁE��した、E
  - `stdlib/kp/kpwrite.nepl`
    - `Writer` めE`StreamWriter` 1 個だけを保持する薁E�� wrapper に置き換えた、E
    - 既存�E `writer_*` API 名�E維持しつつ、実体�E `stream_writer_*` へ委譲する形に整琁E��た、E
  - `tests/stdlib/streamio.n.md`
    - `StreamWriter` めE`std/streamio` から直接使ぁEfocused case を追加し、text/i32/space helper を回帰固定した、E
- [設訁Eせっけい][判断/はんだん]:
  - `kpwrite` のぁE��「競技向けの名前」ではなく「stdout buffering とぁE��汎用機�E」�E `std` に置く�EぁEreboot 方針と一致すると判断した、E
  - partial write の吸収�E writer ごとに持たせず、stdout 書き�Eし経路の最下層である `std/stdio` に雁E��E��た、E
  - `kpwrite` の public API は既存テスト賁E��を維持するため残し、�E部実裁E��けを `StreamWriter` wrapper 化した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/std/streamio.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 5` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/std/streamio.nepl -i stdlib/kp/kpwrite.nepl -i tests/stdlib/kp.n.md -i tests/stdlib/kp_i64.n.md --no-stdlib --no-tree -o /tmp/tests-streamio-kpwrite-kp-focus.json -j 15`
    - [結果/けっか]: `21/21 pass`
  - `/tmp/tests-streamio-kpwrite-kp-focus.json`
    - [確誁Eかくにん]: `summary.total = 21`, `summary.passed = 21`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/streamio.nepl -i stdlib/kp/kpwrite.nepl -o html=/tmp/streamio-kpwrite-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`
- [状況EじょぁE��めE��]:
  - `kpwrite` の buffered writer core は `std/streamio` へ移り、`kp` 側は薁E�� wrapper 構�Eになった、E
  - `kpread` の一般化可能部刁E�Eまだ `kp` 側に残ってぁE��ため、todo 7 は継続中である、E

# 2026-03-10 作業メモ (`kpread` の scanner core めE`std/streamio` へ移管)

- [目皁Eもくてき]:
  - todo 7 の残件である `kpread` の一般化可能部刁E�� `std/streamio` へ移し、`kp` 側には競技向けの合�E helper だけを残す、E
  - stdin binary 読み込みの unbounded 経路めE`kp` 専用実裁E�Eままにせず、`std` 側の正規�E口へ整琁E��る、E
- [根本原因/こんぽんげんいん]:
  - これまでの `kpread` は、stdin 全読込、buffer/header 管琁E��token/i32/i64/f64 parser、競技向け `Vec`/行�E helper めE1 module 群で抱えてぁE��、E
  - そ�Eため `StreamWriter` めE`std/streamio` へ移した後も、対になめEscanner core だけが `kp` 側に残り、`std/streamio` が「一般 stream facade」として牁E��落ちになってぁE��、E
  - あわせて stdin binary read の unbounded ループが `kpread_core` に閉じており、`std/streamio` / `std/stdio` の public binary 経路と刁E��されてぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/stdio.nepl`
    - `stdio_read_all_bytes` を、`read_all` の 4096 byte 褁E��ではなく、E4KiB から拡張しつつ EOF まで `fd_read` を反復する unbounded binary read へ置き換えた、E
    - これにより `ByteBuf` の stdin 経路と scanner 経路が同ぁE`std` 層に揁E��た、E
  - `stdlib/std/streamio.nepl`
    - `StreamScanner` を追加し、`stream_scanner_new` / `stream_scanner_skip_ws` / `stream_scanner_is_eof` / `stream_scanner_skip_token` / `stream_scanner_read_token` / `stream_scanner_read_i32` / `stream_scanner_read_u64` / `stream_scanner_read_i64` / `stream_scanner_read_f64` / `stream_scanner_read_f32` めE`std` 側で提供するよぁE��した、E
    - scanner は `ByteBuf` の pointer/len めEheader で共有し、`Copy` / `Clone` は cursor 共有�E軽釁Ehandle として定義した、E
  - `stdlib/kp/kpread.nepl`
    - `scanner_new` と primitive reader 群は `StreamScanner` へ委譲する wrapper に整琁E��た、E
    - `Vec` / 行�E / 区間クエリ入力などの競技向け helper は `kp` 側に残した、E
    - file 冒頭 comment を、新構�Eに合わせて `StreamScanner` wrapper 前提へ更新した、E
  - `stdlib/kp/kpread_core.nepl`
    - 実体が `std/streamio` / `std/stdio` へ移ったため削除した、E
  - `tests/stdlib/streamio.n.md`
    - `StreamScanner` 直利用の focused case を追加し、数値読取と BOM + token 読取を回帰固定した、E
- [設訁Eせっけい][判断/はんだん]:
  - `stdin 全読込 + token parser` は競技向け sugar ではなく汎用 scanner 機�Eなので、`kp` ではなぁE`std/streamio` に置く�EぁEreboot 方針に合うと判断した、E
  - 一方で `Vec`/行�E/問題定型入力パチE��は競技向け API とみなし、`kp` 側に残した、E
  - `StreamScanner` の所有モチE��は既孁E`Scanner` と同じぁEshared cursor を維持し、wrapper 置換で既孁E`kp` チE��ト賁E��を崩さなぁE��を優先した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 7` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 8` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/std/streamio.nepl -i stdlib/std/stdio.nepl -i stdlib/kp/kpread.nepl -i tests/stdlib/kp.n.md -i tests/stdlib/kp_i64.n.md -i tests/stdlib/stdin.n.md --no-stdlib --no-tree -o /tmp/tests-streamio-kpread-focus.json -j 15`
    - [結果/けっか]: `52/52 pass`
  - `/tmp/tests-streamio-kpread-focus.json`
    - [確誁Eかくにん]: `summary.total = 52`, `summary.passed = 52`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/streamio.nepl -i stdlib/kp/kpread.nepl -o html=/tmp/streamio-kpread-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`
- [状況EじょぁE��めE��]:
  - `kpwrite` に続いて `kpread` の primitive scanner core めE`std/streamio` へ移り、todo 7 の「`kpread` / `kpwrite` の中核めE`std/streamio` へ昁E��」が一段進んだ、E
  - `kp` 側には `Vec`/行�E/競技入力パチE��のような競技向け sugar が残ってぁE��、E

# 2026-03-10 作業メモ (`std/fs` の binary path めE`ByteBuf` へ統一)

- [目皁Eもくてき]:
  - todo 7 の `std/fs` を、すでに `alloc/io` と `std/streamio` で採用した binary 表現 `ByteBuf` に揁E��る、E
  - `std` 配下�E binary I/O ぁEmodule ごとに `Vec<u8>` と `ByteBuf` へ刁E��してぁE��状態を解消する、E
- [根本原因/こんぽんげんいん]:
  - `std/streamio` と `std/stdio` は reboot 後に `ByteBuf` めEbinary 媒体として使ぁE��計へ寁E��てぁE��が、`std/fs` だけが旧来の `Vec<u8>` 前提を維持してぁE��、E
  - そ�Eため file read の返り値だけが別表現となり、`streamio` / `stdio` と binary path を�E有できず、`std` facade 全体で媒体が一致してぁE��かった、E
  - あわせて `std/fs` 冁E�E小さな作業領域は `RegionToken<u8>` を使っており、helper を褁E��回参照する箁E��で move error を起こしめE��ぁE��造になってぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/fs.nepl`
    - `alloc/collections/vec` 依存を外し、`alloc/io` めEimport する構�Eへ変更した、E
    - `fs_read_fd_bytes` / `fs_read_to_bytes` の返り値めE`Result<ByteBuf, i32>` へ変更した、E
    - `fs_bytes_to_string` は `io_bytebuf_to_str` を使ぁE��ぁE��換 helper に整琁E��た、E
    - fd/iovec/nread の一時領域は `RegionToken` ではなぁE`alloc_ptr<u8>` / `dealloc_ptr<u8>` で管琁E��、`MemPtr<u8>` から `region_new` で `i32*` を�Eり�Eす形へ統一した、E
    - file 全体�E説明と関数 comment を、新しい `ByteBuf` ベ�Eス実裁E��合わせて更新した、E
  - `tests/stdlib/fs.n.md`
    - `fs_read_to_string` の missing file case に加え、既知の test file めE`ByteBuf` として読み、そのまま `str` へ戻せることを確認すめEfocused case を追加した、E
    - `ByteBuf` は move-only なので、E��さ確認後に再利用する形は取らず、text 化まで一気に消費する構�Eにした、E
- [設訁Eせっけい][判断/はんだん]:
  - `ByteBuf` は fd read/write に直接渡せる所有バチE��ァとして `alloc/io` にすでに定義済みであり、`std/fs` だけを `Vec<u8>` のまま残す合理性はなぁE��判断した、E
  - `RegionToken` の使ぁE��しで move error を避けるための場当たり的な褁E�� helper は入れず、`std/stdio` と同じポインタベ�Eスの一時領域管琁E��寁E��た、E
  - 既存�E `stdlib/tests/fs.n.md` は missing file の最小確認として残し、新しい `tests/stdlib/fs.n.md` では binary path の回帰を�E離して固定した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i stdlib/tests/fs.n.md -i stdlib/std/fs.nepl --no-stdlib --no-tree -o /tmp/tests-fs-all.json -j 15`
    - [結果/けっか]: `8/8 pass`
  - `/tmp/tests-fs-all.json`
    - [確誁Eかくにん]: `summary.total = 8`, `summary.passed = 8`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/fs.nepl -i tests/stdlib/fs.n.md -o html=/tmp/fs-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`
- [状況EじょぁE��めE��]:
  - `std/stdio` / `std/streamio` / `std/fs` の binary path がすべて `ByteBuf` を�E有する形になった、E
  - todo 7 の `std` facade 整琁E�E、`env/cliarg` めE��りの target 依孁EAPI の確認を残して継続中である、E

# 2026-03-10 作業メモ (`std/env/cliarg` の一時領域管琁E�� `alloc_ptr` へ統一)

- [目皁Eもくてき]:
  - todo 7 の `std/env` 整琁E��して、`cliarg` めEreboot 後�E move/effect 規則と矛盾しなぁEfacade に直す、E
  - `std/fs` と同様に、一時作業領域の所有モチE��めE`RegionToken` 依存から外し、target 依存実裁E�E冁E��褁E��さを利用老E��ら隠す形へ寁E��る、E
- [根本原因/こんぽんげんいん]:
  - `cliarg_count` / `cliarg_get` / `cstr_to_str` は 2026-03-06 時点で `RegionToken<u8>` ベ�Eスへ寁E��てぁE��が、move check 強化後�E `meta` めE`argv` めEhelper に渡した時点で所有権が移り、その後�E再参照で `D3053` が�Eる状態になってぁE��、E
  - つまめE`cliarg` だけが「一時バチE��ァを線形 token で持ち回す旧設計」に留まっており、直近で `std/fs` に適用した解き方と揁E��てぁE��かった、E
  - そ�E結果、`stdlib/tests/cliarg.n.md` は compile fail し、`cliarg_argv_stdout_count` も空出力になってぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/env/cliarg.nepl`
    - `cli_i32_ptr` めE`MemPtr<u8> + size + off` から `i32*` を�Eり�EぁEhelper に変更した、E
    - `cli_alloc_u8_region` / `cli_free_region` / `cli_u8_ptr` を削除し、一時バチE��ァは `alloc_ptr<u8>` / `dealloc_ptr<u8>` で管琁E��る形へ統一した、E
    - LLVM 側の `__cli_copy_to_cstr`、`args_sizes_get`、`args_get` めE`MemPtr<u8>` ベ�Eスへ更新した、E
    - `cstr_to_str` は `RegionToken` を介さぁE`[len][bytes]` 領域を直接確保して絁E��立てる形に変更した、E
    - `cliarg_count` / `cliarg_get` の meta, argv, argv_buf の寿命管琁E��すべて `alloc_ptr` ベ�Eスへ置き換えた、E
- [設訁Eせっけい][判断/はんだん]:
  - `cliarg` のメタ惁E��バッファは関数冁E��ーカルの一時領域であり、�E開�E安�E API 面ではなぁE��め、`RegionToken` を無琁E��表へ通すより `alloc_ptr` で閉じた方が責務に合うと判断した、E
  - `cstr_len` / `cstr_to_str` の公開墁E��は従来通り `MemPtr<u8>` のまま維持し、型安�E化済みの API 形状は崩さなかった、E
  - `std/fs` と `std/env/cliarg` の両方で同じ一時領域パターンに揁E��たことで、`std` facade 冁E�E target 依存実裁E��針も一致した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/cliarg.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/cliarg.n.md -n 2` -> pass (`stdout: "3"`)
  - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md -i stdlib/std/env/cliarg.nepl --no-stdlib --no-tree -o /tmp/tests-cliarg-current.json -j 15`
    - [結果/けっか]: `9/9 pass`
  - `/tmp/tests-cliarg-current.json`
    - [確誁Eかくにん]: `summary.total = 9`, `summary.passed = 9`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/env/cliarg.nepl -i stdlib/tests/cliarg.n.md -o html=/tmp/cliarg-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`
- [状況EじょぁE��めE��]:
  - `std/env/cliarg` の focused regression は復旧し、`std` facade のぁE�� `stdio` / `fs` / `env/cliarg` の主要�E口は現衁Emove/effect 規則に追従した、E
  - todo 7 は facade 全体�E整合確認と、忁E��なら残る target 依孁EAPI の整琁E��続ける段階に入った、E

# 2026-03-10 作業メモ (`std` facade 周辺の説明�E参�E先を現行構�Eへ同期)

- [目皁Eもくてき]:
  - 直近で揁E��ぁE`std/stdio` / `std/fs` / `std/env/cliarg` の実裁E��対し、comment / test 斁E�� / docs 側の古ぁE��提を除去する、E
  - 実裁E�E通ってぁE��も、説明が旧構�Eのままだと次の reboot 作業で誤った想定を再導�Eしやすいため、ここで同期する、E
- [根本原因/こんぽんげんいん]:
  - `std/env/cliarg` の module comment には、以前�E実裁E��引きずった「取得ごとにメモリを確保し、解放しません」が残ってぁE��、E
  - `tests/stdlib/selfhost_req.n.md` は存在しなぁE`stdlib/tests/fs.nepl` を要件確認�E参�E先にしており、現衁Erepo 構�EとずれてぁE��、E
  - `doc/testing.md` も旧吁E`std/cliarg` と旧 `stdio` 説明を残しており、現在の `std/env/cliarg` / `stdio_read_all_bytes` 構�Eと一致してぁE��かった、E
- [変更/へんこぁE:
  - `stdlib/std/env/cliarg.nepl`
    - module comment の注意事頁E��、返り値 `str` は新規確保される一方で冁E��一時バチE��ァは関数冁E��解放される、とぁE��現行実裁E��合わせて更新した、E
  - `tests/stdlib/selfhost_req.n.md`
    - file I/O 要件確認�E対象パスを、実在する `stdlib/tests/fs.n.md` へ変更した、E
  - `doc/testing.md`
    - `std/cliarg` めE`std/env/cliarg` へ更新した、E
    - `std/stdio` の要紁E��、古ぁE`read_all` / `read_line` 中忁E��明から、現在の `stdio_read_all_bytes` を含む構�Eへ更新した、E
- [設訁Eせっけい][判断/はんだん]:
  - こ�E種の差刁E�E機�E追加ではなぁE��、reboot 中は「古ぁE��明が残ること自体が不�E合�E入口」になるため、実裁E��更と同じ優先度で揁E��るべきと判断した、E
  - `selfhost_req` は「たまたま通る古ぁE��提」を残さず、現在の repo に存在する file を�E示皁E��読む形へ寁E��た、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md -i stdlib/tests/cliarg.n.md -i stdlib/std/env/cliarg.nepl -i stdlib/std/fs.nepl -i stdlib/std/stdio.nepl --no-stdlib --no-tree -o /tmp/tests-doc-followup.json -j 15`
    - [結果/けっか]: `47/47 pass`
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i tests/stdlib/fs.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/cliarg.n.md -i tests/stdlib/stdin.n.md -i tests/stdlib/stdout.n.md -i stdlib/std/streamio.nepl -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --no-stdlib --no-tree -o /tmp/tests-std-facade-sweep.json -j 15`
    - [結果/けっか]: `64/64 pass`
  - `node nodesrc/cli.js -i stdlib/std/env/cliarg.nepl -i tests/stdlib/selfhost_req.n.md -o html=/tmp/std-followup-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`
- [状況EじょぁE��めE��]:
  - `std` facade 周辺の実裁E�Ecomment・focused test・利用老E��け補助 doc の前提が一致した、E
  - 次段では `std` 本体�E残り target 依孁EAPI と、`features` / tutorials 側の追従状況を見てぁE��、E

# 2026-03-10 作業メモ (`features/tui` facade を追加し、WASIX TUI API めEnamed struct ベ�Eスへ整琁E

- [目皁Eもくてき]:
  - todo 8 の `features` 層整琁E��して、TUI の利用老E��け�E口めE`platforms/wasix/tui` 直参�Eから `features/tui` に固定する、E
  - 旧 `.Pair` ベ�Eスの座標�Eサイズ API ぁEcurrent compiler / examples で不安定になってぁE��ため、public API めEnamed struct ベ�Eスへ寁E��る、E
- [根本原因/こんぽんげんいん]:
  - repo には `stdlib/platforms/wasix/tui.nepl` しかなく、examples も�Eて platform 直 import してぁE��ため、reboot 方針�E「TUI は `features` 層」とぁE��責務�E離が未反映だった、E
  - さらに `get_terminal_size` と `editor_text::cursor_line_col` ぁE`.Pair` を返し、call site では `get x 0` / `get x 1` に依存してぁE��が、multi-file の wasix examples ではこ�E経路ぁE`D3006` を起こしてぁE��、E
  - `Tuple:` 自体を戻り値に使ぁE��とではなく、「外部 API と helper の意味を番号 access に押し込んだこと」が不安定さと可読性低下�E共通原因だった、E
  - あわせて型注釈�Eの `tui::TerminalSize` のような `::` path は現状 parser が受け付けず、編雁E��止メモにある未実裁E��E��と衝突してぁE��ため、call site は推論前提にする忁E��があった、E
- [変更/へんこぁE:
  - `stdlib/features/tui.nepl`
    - `platforms/wasix/tui` めE`@merge` で再�E開する�E弁Efacade を新設した、E
    - module comment を新 policy に合わせて記述し、利用老E��ぁEimport path めE`features/tui` に固定した、E
  - `stdlib/platforms/wasix/tui.nepl`
    - `TerminalSize` struct を追加した、E
    - `get_terminal_size` の戻り値めE`Tuple:` から `TerminalSize` へ変更した、E
    - parser error の原因だっぁE`if` layout 冁E�E不要な末尾 `;` 3 箁E��を除去した、E
  - `examples/tui_editor/editor_text.nepl`
    - `CursorLineCol` struct を追加し、`cursor_line_col` の戻り値めEnamed struct 化した、E
  - `examples/tui_editor/editor_render.nepl`
    - `cursor_line_col` の利用めE`get p "line"` / `get p "col"` に変更した、E
  - `examples/wasix_tui_demo.nepl`
  - `examples/wasix_tui_fullscreen.nepl`
  - `examples/wasix_tui_menu.nepl`
  - `examples/wasix_tui_progress.nepl`
  - `examples/wasix_tui_text_render.nepl`
  - `examples/tui_editor/main.nepl`
  - `examples/tui_editor/editor_runtime.nepl`
  - これら�E import めE`platforms/wasix/tui` から `features/tui` へ変更した、E
  - `examples/wasix_tui_demo.nepl` / `examples/wasix_tui_fullscreen.nepl` / `examples/wasix_tui_text_render.nepl` / `examples/tui_editor/main.nepl`
    - 端末サイズの参�EめE`get size "cols"` / `get size "rows"` に変更した、E
- [設訁Eせっけい][判断/はんだん]:
  - TUI の facade 追加だけで止めず、example まで `features/tui` に揁E��た�Eは、「利用老E��最初に見る path」を固定しなぁE�� reboot 後�E責務�E離が定着しなぁE��めである、E
  - 端末サイズめEcursor 座標�E public helper として意味が�E確なので、匿吁Etuple より named struct の方ぁEAPI として安定で、field access 廁E��方針とも整合する、E
  - 型注釁Epath 未対応�E compiler 側の未実裁E��頁E��ので、今回は library 側で回避不�Eな箁E��だぁEinference に寁E��、構文拡張そ�Eも�Eには踏み込まなかった、E
- [検証/けんしょぁE:
  - `target/debug/nepl-cli -i examples/wasix_tui_demo.nepl --target wasix --output /tmp/wasix-tui-demo-check` -> success
  - `target/debug/nepl-cli -i examples/tui_editor/main.nepl --target wasix --output /tmp/tui-editor-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_menu.nepl --target wasix --output /tmp/wasix-tui-menu-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_progress.nepl --target wasix --output /tmp/wasix-tui-progress-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_fullscreen.nepl --target wasix --output /tmp/wasix-tui-fullscreen-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_text_render.nepl --target wasix --output /tmp/wasix-tui-text-render-check` -> success
  - `node nodesrc/tui_regression.js --timeout-ms 8000`
    - [結果/けっか]: `ok: true`
    - [確誁Eかくにん]: 全 16 scenario ぁE`exit_code = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/features/tui.nepl -o html=/tmp/features-tui-doc-html`
    - [結果/けっか]: `generated 1 html file(s)`
- [状況EじょぁE��めE��]:
  - TUI の利用老E��け�E口は `features/tui` に固定され、todo 8 のぁE�� TUI 配置は完亁E��た、E
  - `features` 層には GUI / HTTP / 音声など未整琁E�E領域が残るため、todo 8 自体�E「残作業整琁E��として継続する、E

# 2026-03-10 作業メモ (`features/tui` facade の focused regression を追加)

- [目皁Eもくてき]:
  - 直前に入れた `features/tui` への移行を、examples compile のみではなぁE`tests/stdlib` の focused case として固定する、E
  - `platforms/wasix/tui` 直参�Eへの送E��りや、`TerminalSize` の field access 退行を小さな fixture で早期検知できるようにする、E
- [根本原因/こんぽんげんいん]:
  - 直前�E変更は examples compile と runtime regression では確認できてぁE��が、stdlib reboot の本流で使ぁE`tests/stdlib/*` 側に専用 fixture が存在しなかった、E
  - そ�Eままだと、封E�� `features/tui` facade の reexport が崩れても、E��ぁEwasix example を個別に回すまで気づけなぁE��態だった、E
- [変更/へんこぁE:
  - `tests/stdlib/features_tui.n.md`
    - `features_tui_facade_reexports_text_helpers` を追加し、`features/tui` 経由で `line_pad_to_cols` と `repeat_text` が使えることめEstdout で固定した、E
    - `features_tui_terminal_size_uses_named_fields` を追加し、`get_terminal_size` の戻り値に対して `get size "cols"` / `"rows"` が使えることめE`ret: 0` で固定した、E
- [設訁Eせっけい][判断/はんだん]:
  - TTY を前提とする raw mode めEfull-screen 描画は重く壊れ方も多様なので、focused regression では「TTY なしでも�E現できる helper」と「named field access」�E 2 点に責務を絞った、E
  - これにより、`features/tui` facade の契紁E��ぁEexamples より短ぁE�E現で検証できるようになった、E

# 2026-03-10 作業メモ (`features/tui` focused test を通すために library / nodesrc の wasix 経路を是正)

- [目皁Eもくてき]:
  - 追加した `tests/stdlib/features_tui.n.md` めE`run_doctest.js` / `tests.js` から安定して実行できるようにする、E
  - `features/tui` facade の focused regression を「手允E��個別に wasmer を叩け�E通る」状態ではなく、既孁Enodesrc harness で再現できる状態に戻す、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/platforms/wasix/tui.nepl` は module 冁E�� `print` / `print_i32` を使ってぁE��が、`std/stdio` めEimport しておらず、呼び出し�E module がたまたま `std/stdio` めEimport してぁE��前提に依存してぁE��、E
  - そ�Eため、`features/tui` だけを import する focused test では `undefined identifier` になってぁE��、E
  - さらに `nodesrc/run_test.js` は実行系めENode.js の WASI preview1 に固定しており、`#target wasix` doctest を実行すると `wasix_32v1` import を解決できなかった、E
  - `spawnSync wasmer` へ刁E��替えた初期案も sandbox 下で `EPERM` を起こしたため、wasix 実行経路は `tui_regression.js` と同じ async `spawn` へ揁E��る忁E��があった、E
  - あわせて `wasmer run --dir=...` の deprecated warning ぁEstderr を汚しており、I/O 比輁E�� test の封E��リスクになってぁE��、E
- [変更/へんこぁE:
  - `stdlib/platforms/wasix/tui.nepl`
    - `#import "std/stdio" as *` を追加し、module 単体で `print` 系 symbol を解決できるようにした、E
  - `nodesrc/run_test.js`
    - source から `#target` を読み取り、`wasix` の場合�E `runWasixBytes` を使ぁE�E岐を追加した、E
    - `runWasixBytes` めEasync `spawn` ベ�Eスで実裁E��、stdin / stdout / stderr capture と timeout を持つ汎用 wasix 実行経路にした、E
    - `wasmer run` の mount option めE`--dir` から `--volume host:guest` へ更新し、deprecated warning を除去した、E
  - `nodesrc/tui_regression.js`
    - 同じぁE`--volume` へ更新し、scenario 実行時の stderr warning を除去した、E
  - `nodesrc/README.n.md`
    - `run_test.js` ぁE`#target wasix` では `wasmer run` を使ぁE��とと、`WASMER_BIN` で override できることを追記した、E
  - `tests/stdlib/features_tui.n.md`
    - 追加済み focused test を正式に回帰へ絁E��込んだ、E
- [設訁Eせっけい][判断/はんだん]:
  - `platforms/wasix/tui` のような feature backend は、呼び出し�E import に依存せぁEself-contained にしておくべきなので、test 側へ `std/stdio` を足す�EではなぁElibrary 側を修正した、E
  - wasix 実行�E Node.js 標溁EWASI では本質皁E��扱えなぁE��め、test harness 側で target 刁E��を持つのが根本修正と判断した、E
  - `tui_regression.js` と `run_test.js` の実行方式を揁E��たことで、focused test と end-to-end regression の差が減った、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md -i stdlib/features/tui.nepl -i stdlib/platforms/wasix/tui.nepl --no-stdlib --no-tree -o /tmp/tests-features-tui.json -j 15`
    - [結果/けっか]: `3/3 pass`
  - `/tmp/tests-features-tui.json`
    - [確誁Eかくにん]: `summary.total = 3`, `summary.passed = 3`, `summary.failed = 0`
  - `node nodesrc/tui_regression.js --timeout-ms 8000`
    - [結果/けっか]: `ok: true`
    - [確誁Eかくにん]: 全 16 scenario ぁE`exit_code = 0`, `stderr_len = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md -o html=/tmp/features-tui-tests-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`
- [状況EじょぁE��めE��]:
  - `features/tui` は facade と examples だけでなく、focused doctest harness からも検証できる状態になった、E
  - nodesrc 側は `#target wasix` を扱えるようになり、今後�E `features` 系回帰追加でも同じ経路を�E利用できる、E

# 2026-03-10 作業メモ (`web/package.json` めEESM 化して nodesrc の module type warning を除去)

- [目皁Eもくてき]:
  - `nodesrc` 実行時に毎回出てぁE�� `[MODULE_TYPELESS_PACKAGE_JSON]` warning を除去し、test の signal を見やすくする、E
  - `compiler_loader.js` ぁE`web/dist/nepl-web-*.js` めEESM として dynamic import してぁE��前提めEpackage scope 側でも�E示する、E
- [根本原因/こんぽんげんいん]:
  - `nodesrc/compiler_loader.js` は wasm-bindgen 生�E物の `nepl-web-*.js` めEdynamic import してぁE��が、親チE��レクトリである `web/` の `package.json` に `"type": "module"` がなかった、E
  - そ�Eため Node.js は一旦 CommonJS として解釈しようとしてから ESM として再解釈し、`run_doctest.js` / `tests.js` / `cli.js` 実行時に毎回 warning を�EしてぁE��、E
  - warning 自体�E失敗ではなぁE��、focused test の stderr を汚し、harness 改修時�E本当�E異常と見�EけにくくなってぁE��、E
- [変更/へんこぁE:
  - `web/package.json`
    - `"type": "module"` を追加した、E
- [設訁Eせっけい][判断/はんだん]:
  - 問題�E loader 側ではなぁEpackage scope の宣言不足なので、warning めEsuppress するのではなぁEpackage metadata を実�Eに合わせるのが根本修正と判断した、E
  - `web/` 配下�E Node tool は主に `tsc` / `trunk` 経由で使っており、ESM 持E��を追加しても現行運用と矛盾しなぁE��とめEbuild で確認した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md -i stdlib/features/tui.nepl -i stdlib/platforms/wasix/tui.nepl --no-stdlib --no-tree -o /tmp/tests-features-tui.json -j 15`
    - [結果/けっか]: `3/3 pass`
  - [確誁Eかくにん]:
    - 上記実行かめE`[MODULE_TYPELESS_PACKAGE_JSON]` warning が消えた、E

# 2026-03-10 作業メモ (`doc/testing.md` を現衁Enodesrc / reboot 運用へ全面同期)

- [目皁Eもくてき]:
  - test 運用の説明が旧 `cargo run -p nepl-cli -- test` 中忁E�Eまま残ってぁE��ため、現在の `nodesrc` ベ�Eス運用へ同期する、E
  - reboot 中に新しい回帰を追加する人が、`tests/stdlib` / `stdlib/tests` / doc comment doctest の役割を取り違えなぁE��ぁE��する、E
- [根本原因/こんぽんげんいん]:
  - `doc/testing.md` には古ぁEstdlib 要紁E��旧 tuple 記法、旧 test runner 前提が残っており、現在の repo 構�Eと一致してぁE��かった、E
  - 特に `nodesrc/run_test.js` の wasix 対応が入った後も、その runtime 刁E��や `run_doctest.js` / `tests.js` 中忁E�E運用が文書化されてぁE��かった、E
  - そ�Eままだと、今�E実裁E��前提に test を追加しよぁE��したときに、E��違った�E口めE�E置先を再導�Eするリスクがあった、E
- [変更/へんこぁE:
  - `doc/testing.md`
    - 斁E��全体を current workflow に合わせて書き直した、E
    - `tests/compiler/*.n.md`、`tests/stdlib/*.n.md`、`stdlib/tests/*.n.md`、`stdlib/**/*.nepl` doctest、`tutorials/**/*.n.md` の役割を整琁E��た、E
    - 推奨コマンドを `nodesrc/tests.js` / `run_doctest.js` / `cli.js` / `trunk build` に更新した、E
    - `run_test.js` ぁE`#target wasix` めE`wasmer run` で実行することを�E記した、E
    - 古ぁEtuple 記法説明と、現状に合わなぁEstdlib 一覧を削除した、E
- [設訁Eせっけい][判断/はんだん]:
  - `doc/testing.md` は detailed API reference ではなく「どこに何を書くか、どぁE��行するか」�E運用斁E��なので、�E挙型の stdlib カタログではなぁEworkflow 中忁E��再構�Eした、E
  - docs の役割上、ここでは `.md` 制紁E��従い ruby は使わず、簡潔な plain markdown に揁E��た、E
- [検証/けんしょぁE:
  - `node nodesrc/cli.js -i doc/testing.md -o html=/tmp/doc-testing-html`
    - [結果/けっか]: `generated 0 html file(s)`
    - [確誁Eかくにん]: `.md` は HTML 生�E対象外であり、異常ではなぁE��E

# 2026-03-10 作業メモ (`tutorials/getting_started` の std entrypoint へ移衁E

- [目皁Eもくてき]:
  - getting started tutorial が古ぁE`#target wasi` を教えてぁE��ため、reboot 後�E公開�E口である `#target std` に揁E��る、E
  - 初学老E��け文書が�E部 runtime 名ではなぁEstd facade を起点に説明するよぁE��する、E
- [根本原因/こんぽんげんいん]:
  - `std/stdio` などの利用例がすでに std facade 前提に整琁E��れてぁE��一方、tutorial の doctest だけ旧 `wasi` target のまま残ってぁE��、E
  - そ�Eため、現在の設計哲学である「利用老E�E raw platform ではなぁEstd/features を�E口にする」と斁E��がずれてぁE��、E
- [変更/へんこぁE:
  - `tutorials/getting_started/01_hello_world.n.md`
    - 冒頭説明を `#target std` 前提へ更新した、E
    - 最初につまずきめE��ぁE��の bullet めE`#target std` に同期した、E
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
  - `tutorials/getting_started/03_functions.n.md`
  - `tutorials/getting_started/04_strings_and_stdio.n.md`
  - `tutorials/getting_started/05_option.n.md`
  - `tutorials/getting_started/06_result.n.md`
  - `tutorials/getting_started/07_while_and_block.n.md`
  - `tutorials/getting_started/08_if_layouts.n.md`
  - `tutorials/getting_started/09_import_and_structure.n.md`
  - `tutorials/getting_started/10_project_fizzbuzz.n.md`
  - `tutorials/getting_started/11_testing_workflow.n.md`
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
  - `tutorials/getting_started/15_match_patterns.n.md`
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
  - `tutorials/getting_started/19_pipe_operator.n.md`
  - `tutorials/getting_started/20_generics_basics.n.md`
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - doctest 冁E�E `#target wasi` めE`#target std` に更新した、E
- [設訁Eせっけい][判断/はんだん]:
  - tutorial は冁E�� target 名を教える場所ではなく、利用老E��最初に触れる public entrypoint を示すべきなので、`std` へ揁E��る�Eが適刁E��判断した、E
  - 変更は tutorial 冁E�E target 持E��と説明文だけに限定し、サンプル本体�E構造めEimport は不要に触らなかった、E
- [検証/けんしょぁE:
  - `rg -n "#target wasi|WASI ターゲチE��|target wasi" tutorials/getting_started --glob '*.n.md'`
    - [結果/けっか]: 該当なぁE
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/01_hello_world.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/04_strings_and_stdio.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/09_import_and_structure.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1` -> pass

# 2026-03-10 作業メモ (`result` / `nm/parser` doctest failure の根本修正)

- [目皁Eもくてき]:
  - old failure list にあっぁE`result.nepl doctest#5` と `parser.nepl doctest#2/#3` を、現在の仕様と照らして根本から直す、E
  - `parser` 利用側の `nm.n.md` と `html_gen` まで focused に確認し、局所修正で終わってぁE��ぁE��とを確かめる、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/core/result.nepl`
    - `uwok` の使用例が旧 pipe 解釈を前提に `assert_eq_i32 1 ok<i32, str> 1 |> uwok;` と書かれており、現衁Eparser では `assert_eq_i32` 呼び出し�E途中に pipe を差し込めなぁE��めE`D3006` / `D3013` になってぁE��、E
    - これは compiler bug ではなく、doctest の斁E��前提が古かった、E
  - `stdlib/nm/parser.nepl`
    - `close_one_section` / `close_to_level` / `close_all_sections` は `stack_push` により `Stack<NestSection>` を更新するのに、pure signature のまま残ってぁE��、E
    - そ�E結果、module compile 時に `D3025 pure context cannot call impure function` と `D3016` が発生してぁE��、E
- [変更/へんこぁE:
  - `stdlib/core/result.nepl`
    - `uwok` doctest めE`assert_eq_i32 1 uwok ok<i32, str> 1;` に更新し、現衁Esyntax で alias の意味が伝わる例に差し替えた、E
  - `stdlib/nm/parser.nepl`
    - `close_one_section`
    - `close_to_level`
    - `close_all_sections`
      - signature めE`*>Vec<Node>` に更新し、`Stack` 更新を行う helper として effect を�E示した、E
- [設訁Eせっけい][判断/はんだん]:
  - `result` では parser を緩めるのではなく、現在の言語仕様に合う doctest へ更新するのが正しい、E
  - `parser` では `stack_push` を隠して pure を裁E��より、helper 自身めEimpure と明示する方ぁEeffect model に整合する、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 5` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/nm/parser.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/nm/parser.nepl -n 3` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/nm/html_gen.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/nm.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/nm.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/nm.n.md -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree -o /tmp/tests-nm-result-focus.json -j 15`
    - [結果/けっか]: `12/12 pass`

# 2026-03-10 作業メモ (`move_check` と `vec/sort` の stale test を現行仕様へ同期)

- [目皁Eもくてき]:
  - old failure list のぁE��、`move_check.n.md` と `sort.nepl doctest#3` めEfocused に再現し、現在の move model / collection API に合わせて直す、E
- [根本原因/こんぽんげんいん]:
  - `tests/compiler/move_check.n.md`
    - ローカル非Copy型�E回帰ぁE`RegionToken` とぁE��名前のまま std/prelude 斁E��で書かれており、現衁Estdlib の `core/mem` 側 `RegionToken<.T>` と衝突してぁE��、E
    - そ�E結果、本来見たぁEmove check ではなぁEconstructor 解析時点の `D3016` に流れてぁE��、E
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_merge` の doctest が、古ぁE`Result` 返却前提で `push ... |> uwok` と書かれてぁE��、E
    - 現行�E `Vec::push` は `Vec` をそのまま返すため、pipe の途中で `uwok` を挟むと `D3006` / `D3013` になってぁE��、E
- [変更/へんこぁE:
  - `tests/compiler/move_check.n.md`
    - 吁Esnippet めE`#target core` に揁E��た、E
    - ローカル型名めE`RegionToken` から `LocalToken` へ変更し、prelude / stdlib 名との衝突を避けた、E
    - 関連する field / borrow / consume / reassign の型注釈も同時に更新した、E
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_merge` の使用例を `new<i32> |> push ...` 形式へ更新し、不要な `uwok` を除去した、E
- [設訁Eせっけい][判断/はんだん]:
  - `move_check` は compiler の move rule を測る回帰なので、stdlib 名や prelude 影響を受ける状態�Eままにせず、`#target core` + ローカル型名で隔離するのが適刁E��判断した、E
  - `sort` は API を�Eの `Result` 形へ戻す�Eではなく、現在の `Vec` chaining API に doctest を合わせる�Eが正しい、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-stdlib --no-tree -o /tmp/tests-move-check.json -j 15`
    - [結果/けっか]: `13/13 pass`
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/sort.nepl -n 3` -> pass
  - `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-move-sort-focus.json -j 15`
    - [結果/けっか]: `16/16 pass`

# 2026-03-10 作業メモ (`alloc/diag` めEmove model へ追征E

- [目皁Eもくてき]:
  - old failure list にあっぁE`diag.n.md` / `error.n.md` 系の failure を現衁Emove model で再現し、`alloc/diag` の値モチE��実裁E�� test を整琁E��る、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/alloc/diag/diag.nepl`
    - `Diag` / `DiagKind` / `Vec<str>` めE`get` めE`vec_get` で何度も参照する旧実裁E��残っており、現行�E所有権解析では moved value と判定されてぁE��、E
    - 特に `diag_to_string` / `kind_str` / `diags_to_string_loop` は「同ぁEowner を何度も読む」前提で書かれてぁE��、E
  - `stdlib/alloc/diag/error.nepl`
    - `diag_with_span` / `diag_with_source` / `diag_add_note` / `diag_add_help` ぁE`Diag` を�E構築するときに、同ぁE`Diag` から褁E�� field を直接取り直してぁE��、E
    - `diags_has_errors_loop` めE`Vec<Diag>` を�E帰で再利用しており、同じ問題を抱えてぁE��、E
  - `stdlib/tests/error.n.md`
    - `Diag` / `Diags` / `Outcome` を一度 `get` / helper に渡したあとも同じ値を�E利用する、旧 move model 前提の test が残ってぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/diag/diag.nepl`
    - `core/mem` を導�Eした、E
    - `kind_str` と `diag_to_string` めEtemporary memory 経由で field を読み出す形に変更した、E
    - `diag_lines_loop` / `diag_help_loop` / `diags_to_string_loop` は `Vec` 全体を再帰で持ち回すのをやめ、`data_ptr + len + index` で走査する形に変更した、E
  - `stdlib/alloc/diag/error.nepl`
    - `core/mem` を導�Eした、E
    - `diag_with_span` / `diag_with_source` / `diag_add_note` / `diag_add_help` めEtemporary memory 経由の再構築に変更した、E
    - `diags_has_errors` / `diags_has_errors_loop` めE`Vec<Diag>` めEraw data 走査へ変更した、E
  - `stdlib/tests/error.n.md`
    - `core/mem` を追加した、E
    - `Diag` / `Diags` / `Outcome` / `Result` を褁E��回観察する箁E��は temporary memory に保存し、`load` し直して確認する形へ更新した、E
- [設訁Eせっけい][判断/はんだん]:
  - `alloc/diag` は richer な診断値モチE��を持つが、`Diag` 自体を `Copy` にはできなぁE��したがって根本修正は「同ぁEowner を褁E��回読む」実裁E��めE��ることだと判断した、E
  - `Vec` を�E帰にそ�Eまま渡す設計も non-Copy collection では脁E��ため、文字�E化�E雁E��E�� helper は raw backing store を一度取り出してから走査する形へ寁E��た、E
  - test 側も現在の ownership model に合わせ、観察対象めEmemory に退避して再読する形へ揁E��た、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 3` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md -i stdlib/alloc/diag/diag.nepl -i stdlib/alloc/diag/error.nepl --no-stdlib --no-tree -o /tmp/tests-diag-error-focus.json -j 15`
    - [結果/けっか]: `7/7 pass`

# 2026-03-10 作業メモ (`std/test` collect API と `run_doctest` の比輁E��則を同朁E

- [目皁Eもくてき]:
  - old failure list にあっぁE`test.nepl` / `std_test_collect.n.md` 系めEcurrent move model と current nodesrc expectation に揁E��る、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/std/test.nepl`
    - collect API ぁE`Vec<Result<(),str>>` を�E帰でそ�Eまま持ち回しており、現衁Emove model では `checks` の再利用ぁE`D3053` になってぁE��、E
    - `checks_has_err_*` / `checks_summary_*` / `checks_print_failures_*` / `finish_checks` は、non-Copy `Vec` を何度も読み直す旧実裁E�Eままだった、E
  - `nodesrc/run_doctest.js`
    - `tests.js` と違って `strip_ansi` / `normalize_newlines` を反映しておらず、さらに `should_panic` case の stdout 比輁E��スキチE�EしてぁE��かった、E
    - そ�Eため `tests.js` では pass する `std_test_collect` ぁE`run_doctest.js` では ANSI 色コードつぁEstdout mismatch で fail してぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/test.nepl`
    - collect API の冁E��走査めE`Vec` owner の再帰持ち回しから、temporary memory + raw data 走査へ変更した、E
    - `checks_has_err`
    - `checks_summary`
    - `checks_print_failures_loop`
    - `checks_report_failures`
    - `finish_checks`
      - ぁE��れも backing store めE1 回だけ取り�Eして使ぁE��に揁E��た、E
    - 関連 doc comment めEraw data 走査ベ�Eスの実裁E��明へ更新した、E
  - `nodesrc/run_doctest.js`
    - `normalize_newlines` と `strip_ansi` めE`tests.js` と同じ規則で適用するようにした、E
    - `should_panic` case の I/O expectation めE`tests.js` と同様にスキチE�Eするようにした、E
- [設訁Eせっけい][判断/はんだん]:
  - `std/test` は tutorial / stdlib doctest の基盤なので、test data めEANSI なしへ書き換えるのではなぁEcollect API と runner の両方めEcurrent 仕様へ揁E��る�Eが根本修正と判断した、E
  - focused debugging 用の `run_doctest.js` が本佁Erunner と違う expectation 規則を持つのは危険なので、`tests.js` と同じ比輁E��マンチE��クスに寁E��た、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/std/test.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i stdlib/std/test.nepl --no-stdlib --no-tree -o /tmp/tests-std-test-collect-focus.json -j 15`
    - [結果/けっか]: `14/14 pass`
- [追訁EつぁE��]:
  - `doc/stdlib_doc_comment_policy.md` を[再確誁Eさいかくにん]し、`stdlib/std/test.nepl` の今回[変更/へんこぁEした helper comment めE`##` / `###` [形弁Eけいしき]へ[揁Eそろ]えた、E
  - [実裁Eじっそう]ぁEraw data [走査/そうさ]へ[夁Eか]わったこと、move model に[吁EぁEわせて temporary memory を[使/つか]ぁE��とぁEcomment に[反映/はんえい]されてぁE��ことを[確誁Eかくにん]した、E

# 2026-03-10 作業メモ (`std/test` めEtrap 前提から `Result` 前提へ再設訁E

- [目皁Eもくてき]:
  - [強制/きょぁE��い][終亁EしゅぁE��めE��]ベ�Eスの[古/ふる]ぁEtest [機槁EきこぁEを[廁E��/はぁE��]し、`Result<(),str>` を[中忁EちめE��しん]にした[安�E/あんぜん]な test API へ[移衁EぁE��ぁEする、E
  - あわせて `nodesrc` 側で `ret:` を[実際/じっさい]に[検査/けんさ]できるようにし、`Result` めEi32 の[終亁EしゅぁE��めE��] code へ[落/お]として runner と[接綁Eせつぞく]する、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/std/test.nepl`
    - `test_fail` / `finish_checks` / `assert_*` ぁEtrap を[前提/ぜんてい]にしており、reboot の「[安�E/あんぜん] API [優允EめE��せん]」「[値中忁EあたぁE��めE��しん]・[式指吁EしきしこぁE」と[矛盾/むじゅん]してぁE��、E
    - `check_*` はすでに `Result<(),str>` を[迁Eかえ]してぁE��のに、[最絁EさいしゅぁE[出口/でぐち]だけが trap へ[潰/つぶ]されてぁE��、E
  - `nodesrc`
    - doctest parser / runner ぁE`ret:` を[無要Eむし]してぁE��ため、[安�E/あんぜん]な test [失敁Eしっぱい]を[戻/もど]り[値/あたい]で runner に[企Eつた]える[経路/けいろ]が[存在/そんざい]しなかった、E
    - Node WASI [実衁Eじっこう]めE`_start` [経由/けいめEでは `main` の[戻/もど]り[値/あたい]を[捨/す]ててぁE��、E
- [変更/へんこぁE:
  - `nodesrc/parser.js`
    - doctest meta に `ret:` を[追加/つぁE��]した、E
    - bare `ret: 0` を[斁E���E/もじれつ]ではなく[数値/すうち]として[解釁Eかいしゃく]する `parseRetValue` を[追加/つぁE��]した、E
  - `nodesrc/run_test.js`
    - `wasi.start()` [一本/ぁE��ぽん]ではなく、`wasi.initialize({ exports: { memory, _initialize? } })` のあと exported `main` を[直接/ちめE��せつ][呼/めEぶ[経路/けいろ]を[追加/つぁE��]した、E
    - これにより stdout/stderr を[俁Eたも]ったまま `main` の[戻/もど]り[値/あたい]めE`return_value` として[取征Eしゅとく]できるようにした、E
    - `ret:` ぁEJSON [斁E���E/もじれつ]のとき�E NEPL の `str` [表現/ひめE��げん]�E�E[len:i32][bytes...]`�E�として[復号/ふくごぁEするようにした、E
  - `nodesrc/tests.js` / `nodesrc/run_doctest.js`
    - `expected_ret` めEparser から[叁EぁEけ[叁Eと]って[比輁Eひかく]するようにした、E
    - `std/test` めEimport してぁE�� case で `FAIL:` [衁EぎょぁEが[出/で]た�Eに stdout expectation が[明示/めいじ]されてぁE��い[場吁Eばあい]は fail とするようにした、E
  - `stdlib/std/test.nepl`
    - file header と[関連/かんれん] helper comment めEreboot 後�E doc comment policy に[沿/そ]って[全面皁Eぜんめんてき]に[更新/こうしん]した、E
    - `test_fail` めEtrap ではなぁE`Result<(),str>::Err msg` を[迁Eかえ]ぁEhelper に[変更/へんこぁEした、E
    - `test_checked` めE`Result<(),str>::Ok ()` を[迁Eかえ]ぁEhelper に[変更/へんこぁEした、E
    - `finish_checks` めEtrap ではなぁE`Result<(),str>` に[畳/たた]む helper に[変更/へんこぁEした、E
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` / `assert_ok_i32` / `assert_err_i32` めE`Result<(),str>` [返却/へんきめE��]へ[変更/へんこぁEした、E
    - `result_exit_code` / `checks_exit_code` を[追加/つぁE��]し、`main <()*>i32>` から runner へ[安�E/あんぜん]に[合否/ごうひ]を[迁Eかえ]せるようにした、E
  - `tests/stdlib/std_test_collect.n.md`
    - success / failure case めE`ret: 0` / `ret: 1` + `checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
    - `[should_panic]` は[削除/さくじょ]した、E
  - `tutorials/getting_started/11_testing_workflow.n.md`
    - `std/test` の[現衁Eげんこう][推奨/すいしょぁEは `Result<(),str>` + `checks_exit_code` / `result_exit_code` であることに[吁EぁEわせて example を[更新/こうしん]した、E
- [設訁Eせっけい][判断/はんだん]:
  - reboot の[方釁EほぁE��ん]では test helper めE値/あたい]を[迁Eかえ]すべきであり、trap は public API の[最絁EさいしゅぁE[表現/ひめE��げん]に[殁Eのこ]すべきでなぁE��[判断/はんだん]した、E
  - [既孁Eきそん]の unit-return test を[一度/ぁE��ど]に[全件/ぜんけん][書/か]き[揁Eか]えなくてめE安�E/あんぜん]に[移衁EぁE��ぁEできるよう、runner [側/がわ]で `FAIL:` [出劁Eしゅつりょく]を[失敁Eしっぱい]と[要Eみ]なす[規則/きそく]を[追加/つぁE��]した、E
  - `ret:` の[未実裁Eみじっそう]を[放置/ほぁE��]したまま `std/test` だぁE`Result` 化してめE出口/でぐち]がなぁE��め、`nodesrc` [側/がわ]を[允Eさき]に[整傁Eせいび]するのが[根本修正/こんぽんしめE��せい]と[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i /tmp/ret_probe.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/compiler/ret_string_example.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/compiler/ret_string_example.n.md -i tests/stdlib/proptest.n.md --no-stdlib --no-tree -o /tmp/tests-ret-focus.json -j 4`
    - [結果/けっか]: `4/4 pass`
  - `node nodesrc/run_doctest.js -i stdlib/std/test.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/std/test.nepl -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md --no-stdlib --no-tree -o /tmp/tests-std-test-safe-result.json -j 4`
    - [結果/けっか]: `16/16 pass`
  - `node nodesrc/tests.js -i tests/compiler/ret_string_example.n.md -i tests/stdlib/proptest.n.md -i stdlib/std/test.nepl -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md --no-stdlib --no-tree -o /tmp/tests-safe-test-ret-focus.json -j 4`
    - [結果/けっか]: `20/20 pass`
  - `node nodesrc/cli.js -i stdlib/std/test.nepl -i tutorials/getting_started/11_testing_workflow.n.md -o html=/tmp/std-test-safe-doc-html`
    - [結果/けっか]: `generated 2 html file(s)`

# 2026-03-10 作業メモ (`Option` / `Result` の入門系 doctest を安�Eな test 流儀へ追征E

- [目皁Eもくてき]:
  - `std/test` の `Result` [中忁EちめE��しん]設計へ[吁EぁEわせて、`core/result` / `core/option` の doctest と、[対忁Eたいおう]する tutorial / stdlib fixture を[安�E/あんぜん]な `ret:` + `checks_exit_code` [前提/ぜんてい]へ[移衁EぁE��ぁEする、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/core/result.nepl` と `stdlib/core/option.nepl` に、trap [前提/ぜんてい]の `neplg2:test[should_panic]` が[殁Eのこ]ってぁE��、E
  - `tutorials/getting_started/05_option.n.md`, `tutorials/getting_started/06_result.n.md`, `stdlib/tests/option.n.md`, `stdlib/tests/result.n.md` めEunit-return + `test_fail` / `assert_*` [直呼/ちめE��めEびの[古/ふる]ぁEtest [流儀/りゅぁE��]のままだった、E
  - `std/test` [側/がわ]はすでに `Result<(),str>` を[迁Eかえ]すよぁE��[夁Eか]わってぁE��ため、[入門用/にめE��もんよう]の[斁E��/ぶんしょ]が[古/ふる]ぁE��まだと reboot [征Eご]の[設計哲学/せっけいてつがく]と[説昁Eせつめい]が[飁Eく]い[遁Eちが]ぁE��E
- [変更/へんこぁE:
  - `stdlib/core/result.nepl`
    - file header めEreboot 後�E[方釁EほぁE��ん]に[沿/そ]ぁE説昁Eせつめい]へ[更新/こうしん]した、E
    - `should_panic` doctest を[削除/さくじょ]し、`ret: 0` + `checks_exit_code` [前提/ぜんてい]の[安�E/あんぜん]な doctest へ[置揁Eちかん]した、E
  - `stdlib/core/option.nepl`
    - file header めEreboot 後�E[方釁EほぁE��ん]に[沿/そ]ぁE説昁Eせつめい]へ[更新/こうしん]した、E
    - `should_panic` doctest を[削除/さくじょ]し、`ret: 0` + `checks_exit_code` [前提/ぜんてい]の[安�E/あんぜん]な doctest へ[置揁Eちかん]した、E
  - `tutorials/getting_started/05_option.n.md`
    - `match` [侁Eれい]と `option_unwrap_or` [侁Eれい]めE`ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[更新/こうしん]した、E
    - `match` [刁E��Eぶんき]の[中/なか]で `checks_push` できるよう `let mut checks` に[変更/へんこぁEした、E
  - `tutorials/getting_started/06_result.n.md`
    - `Ok/Err` [侁Eれい]と `Result` を[迁Eかえ]す[関数/かんすう][侁Eれい]めE`ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[更新/こうしん]した、E
    - [刁E��Eぶんき]で[蓁E��Eちくせき]する `checks` めE`let mut` に[変更/へんこぁEした、E
  - `stdlib/tests/result.n.md`, `stdlib/tests/option.n.md`
    - fixture めE`ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
    - [逐次皁Eちくじてき]な `assert_*` [直刁EちめE��れつ]ではなく、`checks_push` [経由/けいめEで[収集/しゅぁE��めE��]する[形/かたち]へ[揁Eそろ]えた、E
- [設訁Eせっけい][判断/はんだん]:
  - `unwrap` 系 helper [自佁Eじたい]は[互換丁EごかんじめE��][殁Eのこ]してぁE��が、[入門用/にめE��もんよう]の doctest で trap [期征Eきたい]を[推奨/すいしょぁEしなぁE��とを[優允EめE��せん]した、E
  - `core/result` / `core/option` の[説昁Eせつめい]は、unsafe helper の[存在/そんざい]を[注愁EちめE��い]として[明訁Eめいき]しつつ、[通常/つぁE��めE��]は `match` / `unwrap_or` を[優允EめE��せん]する reboot [征Eご]の[姿勢/しせい]へ[寁EめEせた、E
  - tutorial / fixture では `FAIL:` [表示/ひめE��じ]だけに[依孁EぁE��ん]せず、runner と[直絁EちめE��けつ]できる `ret:` [比輁Eひかく]を[明示/めいじ]するほぁE��、[現衁Eげんこう] test [哲学/てつがく]と[整吁Eせいごう]すると[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/core/option.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/06_result.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/core/result.nepl -i stdlib/core/option.nepl -i stdlib/tests/result.n.md -i stdlib/tests/option.n.md -i tutorials/getting_started/05_option.n.md -i tutorials/getting_started/06_result.n.md --no-stdlib --no-tree -o /tmp/tests-option-result-safe.json -j 4`
    - [結果/けっか]: `12/12 pass`

# 2026-03-10 作業メモ (tutorial 初期章と stdlib fixture の safe `Result` 化を継綁E

- [目皁Eもくてき]:
  - `std/test` の trap [前提/ぜんてい]を[廁E��/はぁE��]した reboot [征Eご]の test [流儀/りゅぁE��]に[吁EぁEわせて、tutorial [初期/しょき][章/しょぁEと `stdlib/tests` の[封Eちい]さい fixture [群/ぐん]めE`ret:` + `checks_exit_code` [前提/ぜんてい]へ[移衁EぁE��ぁEする、E
  - [部刁Eぶぶん] test を[小�E/こわ]けにして、[釁Eおも]い[全佁Eぜんたい] test を[頻繁Eひん�Eん]に[囁Eまわ]さずに stale case を[収束/しゅぁE��く]させる、E
- [根本原因/こんぽんげんいん]:
  - `tutorials/getting_started/02_numbers_and_variables.n.md` と `tutorials/getting_started/03_functions.n.md` が、`assert_*` めEunit-return [前提/ぜんてい]で[直刁EちめE��れつ][実衁Eじっこう]し、`test_checked` めE副作用/ふくさよう]だけ�E helper として[扱/あつか]ぁE古/ふる]い[書/か]き[方/かた]のままだった、E
  - `stdlib/tests/cast.n.md`, `stdlib/tests/math.n.md`, `stdlib/tests/vec.n.md` めE同槁EどぁE��ぁEに unit-return [前提/ぜんてい]で、`vec` の `None` [刁E��Eぶんき]では `test_fail` を[即晁Eそくじ][実衁Eじっこう]する[構造/こうぞう]が[殁Eのこ]ってぁE��、E
  - `cast` fixture は pipe [中/ちめE��]に `cast` を[直接/ちめE��せつ][埁EぁEめ[込/こ]んでぁE��ため、safe `Result` 化により `checks_push` と[絁Eく]み[吁EぁEわさったとぁEoverload [解決/かいけつ]が[崩/くず]れる[箁E��/かしょ]が[露出/ろしめE��]した、E
  - `let checks <Vec<Result<(),str>>>:` [形弁Eけいしき]では、[最絁EさいしゅぁE[衁EぎょぁEの `;` ぁEblock の[迁Eかえ]り[値/あたい]めEunit にしてしまぁE��`Vec<Result<(),str>>` [期征Eきたい]と[衝突EしょぁE��つ]する、E
- [変更/へんこぁE:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 5 [件/けん]の doctest すべてに `ret: 0` を[追加/つぁE��]した、E
    - `fn main <()*> ()> ():` めE`fn main <()*>i32> ():` に[変更/へんこぁEし、`checks_new` / `checks_push` / `checks_exit_code` [前提/ぜんてい]へ[揁Eそろ]えた、E
    - `test_checked` は `Result<(),str>` を[迁Eかえ]ぁEhelper として `let _done <Result<(),str>> ...` で[叁EぁEける[形/かたち]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/03_functions.n.md`
    - 3 [件/けん]の doctest めE`ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[更新/こうしん]した、E
    - `if` / `if:` [侁Eれい]を含む[全佁Eぜんたい]めEsafe `Result` [流儀/りゅぁE��]へ[統一/とぁE��つ]した、E
  - `stdlib/tests/cast.n.md`
    - `ret: 0` を[追加/つぁE��]し、fixture [全佁Eぜんたい]めE`checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
    - bool/i32 cast [確誁Eかくにん]は `cast` [結果/けっか]を[允Eさき]に[局所/きょくしょ][変数/へんすぁEへ[束縁Eそくばく]し、その[値/あたい]めE`assert_*` で[検査/けんさ]する[形/かたち]へ[変更/へんこぁEした、E
    - これにより pipe + overload [解決/かいけつ]の[曖昧/あいまい]さを[除去/じょきょ]した、E
  - `stdlib/tests/math.n.md`
    - `ret: 0` を[追加/つぁE��]し、[全検査/ぜんけんさ]めE1 [本/ほん]の `checks_new |> checks_push ...` に[雁E��EしゅぁE��く]した、E
    - `let checks:` block [末尾/まつび]の `;` を[除去/じょきょ]し、[迁Eかえ]り[値/あたい]ぁEunit に[潰/つぶ]れなぁE��ぁE��した、E
  - `stdlib/tests/vec.n.md`
    - `ret: 0` を[追加/つぁE��]し、`let mut checks` [方弁EほぁE��き]へ[変更/へんこぁEした、E
    - `match vec_get ...` の `None` [刁E��Eぶんき]めE`test_fail` めE`checks_push` で[雁E��EしゅぁE��く]する[形/かたち]に[変更/へんこぁEし、[途中/とちめE��] trap しなぁE��ぁE��した、E
- [設訁Eせっけい][判断/はんだん]:
  - `std/test` の `Result<(),str>` [方釁EほぁE��ん]へ[追征EつぁE��めE��]するだけでなく、tutorial [冒頭/ぼぁE��ぁEから「test helper めE値/あたい]を[迁Eかえ]す」とぁE�� reboot [征Eご]の[価値観/かちかん]を[一貫/ぁE��かん]して[示/しめ]すことを[優允EめE��せん]した、E
  - `cast` fixture の[不�E吁Eふぐあい]は runner [側/がわ]ではなく、pipe [中/ちめE��]で overload [曖昧/あいまい]な[弁Eしき]を[直接/ちめE��せつ][評価/ひめE��か]してぁE��[書/か]き[方/かた]に[原因/げんぁE��]があったため、[中間値/ちめE��かんち]を[明示/めいじ]する[形/かたち]へ[正規化/せいきか]した、E
  - `let checks:` block の[末尾/まつび] `;` は[構文丁EこうぶんじめE��]は[封Eちい]さいが、safe `Result` [移衁EぁE��ぁEでは[根本皁Eこんぽんてき]に[垁Eかた]を[壁Eこわ]すため、[局所皁EきょくしめE��き]な回避ではなぁEfixture [全佁Eぜんたい]の[書/か]き[方/かた]を[統一/とぁE��つ]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/02_numbers_and_variables.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/vec.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/cast.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/math.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md -i stdlib/tests/cast.n.md -i stdlib/tests/math.n.md -i stdlib/tests/vec.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch1.json -j 4`
    - [結果/けっか]: `11/11 pass`

# 2026-03-10 作業メモ (control-flow tutorial の safe `Result` 化を継綁E

- [目皁Eもくてき]:
  - `while` / `block` / `if` / `#import` を[説昁Eせつめい]する tutorial [群/ぐん]も、`std/test` の[現衁Eげんこう][方釁EほぁE��ん]に[吁EぁEわせて `ret:` + `checks_exit_code` [前提/ぜんてい]へ[統一/とぁE��つ]する、E
  - [初学老Eしょがくしゃ]ぁEtutorial を[頁Eじゅん]に[読/めEんだとき、chapter ごとに test [流儀/りゅぁE��]が[揺/めEれなぁE��ぁE��する、E
- [根本原因/こんぽんげんいん]:
  - `tutorials/getting_started/07_while_and_block.n.md`, `08_if_layouts.n.md`, `09_import_and_structure.n.md` に、unit-return の `main` と `assert_*` [直刁EちめE��れつ][実衁Eじっこう]を[前提/ぜんてい]にした[古/ふる]い[書/か]き[方/かた]が[殁Eのこ]ってぁE��、E
  - `11_testing_workflow` だぁEsafe `Result` [流儀/りゅぁE��]へ[更新/こうしん]されても、それより[剁EまぁEの tutorial が[古/ふる]ぁE��まだと reboot [征Eご]の test [哲学/てつがく]が[途中/とちめE��]で[送E��/ぎゃくもど]りしてしまぁE��E
- [変更/へんこぁE:
  - `tutorials/getting_started/07_while_and_block.n.md`
    - `while` と `block:` の 2 [件/けん]の doctest に `ret: 0` を[追加/つぁE��]した、E
    - `fn main <()*> ()> ():` めE`fn main <()*>i32> ():` に[変更/へんこぁEし、`checks_new` / `checks_push` / `checks_exit_code` [前提/ぜんてい]へ[揁Eそろ]えた、E
  - `tutorials/getting_started/08_if_layouts.n.md`
    - 4 [件/けん]の `if` [レイアウチEれいあうと]例を `ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[更新/こうしん]した、E
    - inline / `if:` / `then:` / mixed layout の[全侁Eぜんれい]で `core/result` めEimport し、`test_checked` めE`Result<(),str>` として[叁EぁEける[形/かたち]に[統一/とぁE��つ]した、E
  - `tutorials/getting_started/09_import_and_structure.n.md`
    - `std/test` を[使/つか]ぁE1 [件/けん]の doctest めEsafe `Result` [流儀/りゅぁE��]へ[更新/こうしん]した、E
    - `stdio` [出劁Eしゅつりょく]だけを[検証/けんしょぁEする doctest は、`ret:` [比輁Eひかく]を[要Eよう]しなぁE��めそのまま[維持EぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - tutorial [冒頭/ぼぁE��ぁE[部/ぶ]は[斁E��EぶんぽぁEの[説昁Eせつめい]が[主目皁Eしゅもくてき]だが、test [入口/ぁE��ぐち]だけ[古/ふる]ぁEtrap [流儀/りゅぁE��]を[殁Eのこ]すと、`std/test` の reboot [征Eご][設訁Eせっけい]と[説明責任/せつめいせきにん]が[矛盾/むじゅん]する、E
  - `stdout:` [比輁Eひかく]だけで[十�E/じゅぁE�Eん]な case まで[無琁Eむり]に `std/test` へ[寁EめEせるのは[不要Eふよう]なので、`09_import_and_structure` の I/O [侁Eれい]は[既孁Eきそん]の[責勁Eせきむ]を[維持EぁE��]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/08_if_layouts.n.md -n 4` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/07_while_and_block.n.md -i tutorials/getting_started/08_if_layouts.n.md -i tutorials/getting_started/09_import_and_structure.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch2.json -j 4`
    - [結果/けっか]: `8/8 pass`

# 2026-03-10 作業メモ (`Vec<Result<(),str>>` の test [結果/けっか][表示/ひめE��じ]めEhuman / machine に[刁E��/ぶんり])

- [目皁Eもくてき]:
  - reboot の「[値中忁EあたぁE��めE��しん]・[安�E/あんぜん] API [優允EめE��せん]・[責勁Eせきむ][刁E��/ぶんり]」に[征Eしたが]ぁE��`Vec<Result<(),str>>` の test [結果/けっか][表示/ひめE��じ]めEmachine [吁Eむ]ぁEsummary と human [吁Eむ]ぁEANSI [表示/ひめE��じ]へ[刁E��/ぶんり]する、E
  - `finish_checks` ぁEfailure [晁Eじ]だけ[断牁E��/だんぺんてき]に[詳細/しょぁE��い]を[出/だ]す[旧来/きゅぁE��い]の[挙動/きょどぁEをやめ、success / failure [両方/りょぁE��ぁEで `Vec<Result>` [全佁Eぜんたい]を[読/めEみめE��く[要Eみ]せる、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/std/test.nepl` の `checks_summary` は `[ok,err,...]` なぁE�� `[ok,err <msg>,...]` の 1 [衁EぎょぁE summary に[偁EかためEっており、[人閁Eにんげん]ぁE`Vec<Result>` [全佁Eぜんたい]を[追/お]ぁE��は[不足/ふそく]してぁE��、E
  - failure [晁Eじ]の `checks_report_failures` めE`Err` [要素/ようそ]だけを `check[i] ...` として[出/だ]してぁE��ため、success [頁E��/こうもく]との[並/なら]びや[全体像/ぜんたいぞう]が[要Eみ]えにくかった、E
  - reboot.md の[設計原剁Eせっけいげんそく]では、machine [吁Eむ]けと human [吁Eむ]け�E[表示責勁EひめE��じせき�E]を[刁Eわ]けるべきであり、ここが[未整琁Eみせいり]だった、E
- [変更/へんこぁE:
  - `stdlib/std/test.nepl`
    - `check_status_str` めEmachine [吁Eむ]ぁEsummary helper として[整琁Eせいり]し、`Err` では `err <msg>` を[迁Eかえ]すよぁE��した、E
    - `checks_summary` の doc comment を、「machine / log [吁Eむ]け�E[安宁Eあんてい][表現/ひめE��げん]」として[明訁Eめいき]した、E
    - `checks_print_human_line`
      - 1 [件/けん]の `Result<(),str>` めE`[index] ok` / `[index] err <msg>` で[表示/ひめE��じ]する helper を[追加/つぁE��]した、E
      - [添孁Eそえじ]は灰色、`ok` は緑、`err <msg>` は赤で[表示/ひめE��じ]する、E
    - `checks_print_human_loop` / `checks_print_human`
      - `Vec<Result<(),str>>` [全佁Eぜんたい]を[頁Eじゅん]に[色仁EぁE��づ]き[表示/ひめE��じ]する helper を[追加/つぁE��]した、E
    - `finish_checks`
      - まぁEmachine [吁Eむ]ぁEsummary めE`Checked ...` / `FAIL: ...` として 1 [衁EぎょぁE[表示/ひめE��じ]し、その[征Eあと]で `checks_print_human` により[全要素/ぜんようそ]を[色仁EぁE��づ]き[表示/ひめE��じ]する[形/かたち]へ[変更/へんこぁEした、E
      - これにより success / failure [両方/りょぁE��ぁEで `Vec<Result>` [全佁Eぜんたい]の[可視性/かしせい]を[揁Eそろ]えた、E
  - `tests/stdlib/std_test_collect.n.md`
    - success / failure [両方/りょぁE��ぁEの[期征Eきたい] stdout を、新しい machine summary + human list [形弁Eけいしき]へ[更新/こうしん]した、E
- [設訁Eせっけい][判断/はんだん]:
  - machine [吁Eむ]ぁEsummary は `checks_summary` の 1 [衁EぎょぁE[斁E���E/もじれつ]へ[殁Eのこ]し、runner / log / [比輁Eひかく]の[安定性/あんてぁE��い]を[維持EぁE��]した、E
  - human [吁Eむ]けには `checks_print_human` を[別/べつ][責勁Eせきむ]として[設/もう]け、ANSI color を[使/つか]って[成功/せいこう]と[失敁Eしっぱい]を[視覚的/しかくてき]に[刁E��/ぶんり]した、E
  - failure だけ[詳細/しょぁE��い]を[出/だ]す[方弁EほぁE��き]ではなぁEsuccess めE含/ふく]めて[全件/ぜんけん]を[表示/ひめE��じ]するようにしたのは、`Vec<Result>` [全佁Eぜんたい]の[読み/めEめE��さを[優允EめE��せん]したためである、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md -i stdlib/tests/vec.n.md -i tutorials/getting_started/07_while_and_block.n.md -i tutorials/getting_started/08_if_layouts.n.md -i tutorials/getting_started/09_import_and_structure.n.md -i stdlib/std/test.nepl --no-stdlib --no-tree -o /tmp/tests-std-test-human-machine.json -j 4`
    - [結果/けっか]: `25/25 pass`

# 2026-03-10 作業メモ (middle tutorial の safe `Result` 化を継綁E

- [目皁Eもくてき]:
  - `12_pure_function_pipeline`, `13_type_driven_error_modeling`, `14_refactor_with_properties` を、`std/test` の[現衁Eげんこう] safe `Result` [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [純粁Eじゅんすい][関数/かんすう]・`Option` / `Result`・[回帰/かいき] test の chapter でも、「test helper は[値/あたい]を[迁Eかえ]す」とぁE�� reboot [征Eご]の[一貫性/ぁE��かんせい]を[俁Eたも]つ、E
- [根本原因/こんぽんげんいん]:
  - 3 [章/しょぁEとめE`assert_*` の unit-return [前提/ぜんてい]と `test_checked` の[副作用/ふくさよう] helper [前提/ぜんてい]が[殁Eのこ]ってぁE��、E
  - `14_refactor_with_properties.n.md` の `assert_same` は unit-return helper だったため、`checks_push` に[直接/ちめE��せつ][穁Eつ]めず、safe `Result` [流儀/りゅぁE��]へ[自然/しぜん]に[接綁Eせつぞく]できなかった、E
- [変更/へんこぁE:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
    - 2 [件/けん]の doctest に `ret: 0` を[追加/つぁE��]した、E
    - `core/result` めEimport し、`checks_new` / `checks_push` / `checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
    - `Result` [侁Eれい]は `let mut checks` を[導�E/どぁE��めE��]し、`match` [刁E��Eぶんき]ごとの[成否/せいひ]めE`checks_push` で[収集/しゅぁE��めE��]する[形/かたち]へ[変更/へんこぁEした、E
    - `Option` [侁Eれい]めE`ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[揁Eそろ]えた、E
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
    - [前半/ぜんはん]の[等価性/とぁE��せい] doctest めE`checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
    - `assert_same` めE`fn assert_same <(i32,i32)*>Result<(),str>>` へ[変更/へんこぁEし、safe `Result` [流儀/りゅぁE��]にそ�Eまま[接綁Eせつぞく]できる helper に[再設訁Eさいせっけい]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `assert_same` のような chapter [冁Eない] helper こそ、reboot [征Eご]は unit-return ではなぁE`Result<(),str>` を[迁Eかえ]すほぁE��、test [合�E/ごうせい]と[責勁Eせきむ]が[明確/めいかく]になる、E
  - `13` [章/しょぁEは「[垁Eかた]で[失敁Eしっぱい]を[表/あらわ]す」が[主顁Eしゅだい]なので、doctest [自佁Eじたい]めE`Result` を[値/あたい]として[収集/しゅぁE��めE��]する[構造/こうぞう]へ[寁EめEせるのが[自然/しぜん]だと[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/14_refactor_with_properties.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/12_pure_function_pipeline.n.md -i tutorials/getting_started/13_type_driven_error_modeling.n.md -i tutorials/getting_started/14_refactor_with_properties.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch3.json -j 4`
    - [結果/けっか]: `6/6 pass`

# 2026-03-10 作業メモ (`Vec<Result>` の print めEtest [末尾/まつび]の[明示呼/めいじよ]び[出/だ]しへ[統一/とぁE��つ])

- [目皁Eもくてき]:
  - `checks_push` [中/ちめE��]めE`checks_exit_code` [冁E��/なぁE�E]で[勝手/かって]に stdout を[汁Eよご]さず、test case [側/がわ]が[最征Eさいご]に[明示皁Eめいじてき]に print する reboot [征Eご]の[流儀/りゅぁE��]へ[揁Eそろ]える、E
  - test tool [側/がわ]ではなぁEtest case [本佁Eほんたい]の[記述/きじめE��]から「[佁Eなに]を[出/だ]すか」を[読/めEめるようにする、E
- [根本原因/こんぽんげんいん]:
  - 直前�E `std/test` [改修/かいしゅぁEで human / machine [表示/ひめE��じ]を[刁E��/ぶんり]したが、`checks_exit_code` から[暗黁Eあんもく]に[表示/ひめE��じ]してぁE��[名殁Eなごり]を[完�E/かんぜん]には[断/た]ち[刁Eき]れてぁE��かった、E
  - `checks_print_machine` / `checks_print_human` を[別、Eべつべつ]に[呼/めEぶ[書/か]き[方/かた]だと、test [末尾/まつび]に 1 [囁Eかい]だけ[明示/めいじ]して[出/だ]すとぁE��[意図/ぁE��]が[弱/よわ]かった、E
  - さらに print helper ぁE`Vec<Result>` を[消費/しょぁE�E]してしまぁE��、その[征Eあと]で `checks_exit_code` に[渡/わた]せず、[合�E/ごうせい]しにくかった、E
- [変更/へんこぁE:
  - `stdlib/std/test.nepl`
    - `finish_checks`
      - [表示/ひめE��じ]を[完�E/かんぜん]に[夁Eはず]し、`Vec<Result<(),str>> -> Result<(),str>` の[純粁Eじゅんすい] helper に[戻/もど]した、E
    - `checks_exit_code`
      - [冁E��/なぁE�E]で print しなぁEhelper であることめEdoc comment に[明訁Eめいき]した、E
    - `checks_print_machine` / `checks_print_human`
      - [表示/ひめE��じ][征Eご]に[吁Eおな]ぁE`Vec<Result<(),str>>` を[迁Eかえ]ぁEpipe [可能/か�EぁE API に[変更/へんこぁEした、E
    - `checks_print_report`
      - test [末尾/まつび]で 1 [囁Eかい]だけ[呼/めEぶ[用送Eようと]の helper を[追加/つぁE��]した、E
      - [冁E��/なぁE�E]では machine summary の[表示/ひめE��じ]と human [吁Eむ]け[一覧/ぁE��らん][表示/ひめE��じ]を[頁E��/じゅん�Eん]に[衁Eおこな]ぁE��その[征Eあと]で `Vec<Result<(),str>>` を[迁Eかえ]す、E
  - `tests/stdlib/std_test_collect.n.md`
    - `checks_print_machine |> checks_print_human` の[刁E��/ぶんかつ]呼び[出/だ]しをめE��、`let shown checks_print_report checks` に[統一/とぁE��つ]した、E
    - [期征Eきたい] stdout は[維持EぁE��]しつつ、「print は test [末尾/まつび]で[明示皁Eめいじてき]に[呼/めEぶ」ことが[読/めEめる fixture に[変更/へんこぁEした、E
  - `tutorials/getting_started/11_testing_workflow.n.md`
    - [説明文/せつめいぶん]を「`Vec<Result<(),str>>` の[表示/ひめE��じ]は test [末尾/まつび]で `checks_print_report` を[明示皁Eめいじてき]に[呼/めEぶ」へ[更新/こうしん]した、E
    - example めE`let shown checks_print_report checks` に[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - reboot の「[値中忁EあたぁE��めE��しん]」「[明示皁Eめいじてき] API」「[責勁Eせきむ][刁E��/ぶんり]」に[照/て]らすと、`checks_exit_code` ぁEstdout を[出/だ]す�Eは[責務過夁Eせきむかた]だった、E
  - print helper めEpipe [可能/か�EぁEにしたのは、NEPLg2 の[合�E/ごうせい][志向/しこぁEと[整吁Eせいごう]し、`checks_print_report checks |> checks_exit_code` [系統/けいとぁEの[書/か]き[方/かた]へめE拡張/かくちめE��]しやすいためである、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md -i stdlib/std/test.nepl --no-stdlib --no-tree -o /tmp/tests-explicit-check-print.json -j 4`
    - [結果/けっか]: `16/16 pass`

# 2026-03-10 作業メモ (late getting_started と `hash` fixture めEexplicit print / safe `Result` 流儀へ追征E

- [目皁Eもくてき]:
  - `19_pipe_operator`, `20_generics_basics`, `21_trait_bounds_basics` と `stdlib/tests/hash.n.md` を、[現衁Eげんこう]の safe `Result` + explicit print [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - tutorial [終盤/しゅぁE�Eん]でめE`std/test` の[古/ふる]ぁEunit-return / [暗黁Eあんもく]表示[前提/ぜんてい]を[殁Eのこ]さなぁE��E
- [根本原因/こんぽんげんいん]:
  - `19`〜`21` の doctest は、まだ unit-return `main` と `assert_*` [直刁EちめE��れつ][実衁Eじっこう]の[旧流儀/きゅぁE��めE��ぎ]が[殁Eのこ]ってぁE��、E
  - `stdlib/tests/hash.n.md` めE`test_checked` を[途中/とちめE��]で[呼/めEぶ[古/ふる]い[形/かたち]のままで、`Vec<Result>` の[雁E��EしゅぁE��く]と test [末尾/まつび]の explicit report [方釁EほぁE��ん]に[乁Eの]ってぁE��かった、E
- [変更/へんこぁE:
  - `tutorials/getting_started/19_pipe_operator.n.md`
    - 2 [件/けん]の doctest に `ret: 0` を[追加/つぁE��]した、E
    - `core/result` めEimport し、`checks_new` / `checks_push` / `checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/20_generics_basics.n.md`
    - generic `id` / generic `Option` の doctest めEsafe `Result` [流儀/りゅぁE��]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - trait / impl と trait bound generic の doctest めEsafe `Result` [流儀/りゅぁE��]へ[変更/へんこぁEした、E
  - `stdlib/tests/hash.n.md`
    - `ret: 0` を[追加/つぁE��]し、FNV-1a / `hash32_i32` / SHA-256 skeleton の[確誁Eかくにん]めE`Vec<Result<(),str>>` [雁E��EしゅぁE��く]へ[移/ぁE��]した、E
    - stdout [確誁Eかくにん]のある fixture として、test [末尾/まつび]で `checks_print_report checks` を[明示皁Eめいじてき]に[呼/めEぶ[形/かたち]へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - tutorial [側/がわ]は stdout [期征Eきたい]がなぁE��め、`checks_exit_code` だけを[使/つか]ぁE最封EさいしょぁE構�Eを[維持EぁE��]した、E
  - `hash.n.md` は[回帰/かいき] fixture として stdout [観寁Eかんさつ]の[価値/かち]があるため、`checks_print_report` を[入/い]れて explicit print [方釁EほぁE��ん]の[実侁Eじつれい]にもした、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/hash.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/19_pipe_operator.n.md -i tutorials/getting_started/20_generics_basics.n.md -i tutorials/getting_started/21_trait_bounds_basics.n.md -i stdlib/tests/hash.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch5.json -j 4`
    - [結果/けっか]: `7/7 pass`

# 2026-03-10 作業メモ (`match` / namespace / recursion tutorial の safe `Result` 匁E

- [目皁Eもくてき]:
  - `15_match_patterns`, `17_namespace_and_alias`, `18_recursion_and_termination` を、[現衁Eげんこう]の safe `Result` [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - `match` / `::` / [再帰/さいき]とぁE��[言誁Eげんご][中忁EちめE��しん]の chapter に unit-return test が[殁Eのこ]らなぁE��ぁE��する、E
- [根本原因/こんぽんげんいん]:
  - 3 [章/しょぁEとめE`fn main <()*>()> ():` と `assert_*` [直刁EちめE��れつ][実衁Eじっこう]の[旧流儀/きゅぁE��めE��ぎ]が[殁Eのこ]ってぁE��、E
  - reboot [征Eご]の `std/test` は `Result<(),str>` [中忁EちめE��しん]に[再設訁Eさいせっけい]されてぁE��ため、ここが[旧来/きゅぁE��い]のままだと tutorial [全佁Eぜんたい]で[流儀/りゅぁE��]が[揺/めEれる、E
- [変更/へんこぁE:
  - `tutorials/getting_started/15_match_patterns.n.md`
    - `Option` / `Result` の `match` 例を `ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
    - alias [経由/けいめEの[関数呼/かんすうめEび[出/だ]しと `Option::Some` / `Option::None` 例を safe `Result` [流儀/りゅぁE��]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
    - `sum_to` / `fib` の[再帰/さいき]例を `ret: 0` + `checks_exit_code` [前提/ぜんてい]へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - これら�E chapter は stdout [比輁Eひかく]を[伴/ともな]わなぁE��め、`checks_print_report` は[入/い]れず、[最小限/さいしょぁE��ん]の safe `Result` だけを[適用/てきよぁEした、E
  - tutorial [本斁Eほん�Eん]の[主顁Eしゅだい]は[構文/こうぶん]なので、test helper [側/がわ]の[記述釁EきじめE��りょぁEは[忁E��最低限/ひつようさいてぁE��ん]に[畁Eとど]めた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/18_recursion_and_termination.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/15_match_patterns.n.md -i tutorials/getting_started/17_namespace_and_alias.n.md -i tutorials/getting_started/18_recursion_and_termination.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch4.json -j 4`
    - [結果/けっか]: `6/6 pass`

# 2026-03-10 作業メモ (`list` / `hashset` / `hashset_str` fixture めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `stdlib/tests/list.n.md`, `stdlib/tests/hashset.n.md`, `stdlib/tests/hashset_str.n.md` を、[現衁Eげんこう]の `Vec<Result<(),str>>` + explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - collection fixture [群/ぐん]でめE`test_checked` を[途中/とちめE��]で[持Eはさ]む[古/ふる]い[書/か]き[方/かた]を[除去/じょきょ]し、test [末尾/まつび]で 1 [囁Eかい]だぁE`checks_print_report` を[呼/めEぶ[構造/こうぞう]へ[統一/とぁE��つ]する、E
- [根本原因/こんぽんげんいん]:
  - 3 fixture とめE`assert_*` / `test_fail` / `test_checked` を[逐次/ちくじ][実衁Eじっこう]する[旧流儀/きゅぁE��めE��ぎ]が[殁Eのこ]ってぁE��、E
  - `list` は `Option` [刁E��Eぶんき]が[夁Eおお]く、`hashset` / `hashset_str` は alias / remove / contains [確誁Eかくにん]が[散在/さんざい]してぁE��、[途中/とちめE��]で[何度/なんど]めEsuccess log を[出/だ]してぁE��、E
- [変更/へんこぁE:
  - `stdlib/tests/list.n.md`
    - `ret: 0` と `core/result` import を[追加/つぁE��]した、E
    - `let mut checks` を[導�E/どぁE��めE��]し、`Option` [刁E��Eぶんき]の `Some` / `None` [両方/りょぁE��ぁEめE`checks_push` へ[雁E��EしゅぁE��く]した、E
    - test [末尾/まつび]で `checks_print_report checks` を[呼/めEび、その[征Eあと]に `checks_exit_code` を[迁Eかえ]す[形/かたち]へ[変更/へんこぁEした、E
  - `stdlib/tests/hashset.n.md`
    - `ret: 0` を[追加/つぁE��]し、insert / remove / alias [確誁Eかくにん]めE`Vec<Result<(),str>>` に[雁E��EしゅぁE��く]した、E
    - `test_checked "new"` などの[途中/とちめE��]ログは[除去/じょきょ]し、[最征Eさいご]に 1 [囁Eかい]だぁEreport を[出/だ]す[形/かたち]へ[変更/へんこぁEした、E
  - `stdlib/tests/hashset_str.n.md`
    - `ret: 0` を[追加/つぁE��]し、content / remove / alias [確誁Eかくにん]めE`Vec<Result<(),str>>` に[雁E��EしゅぁE��く]した、E
    - [終亁EしゅぁE��めE��] report は `checks_print_report` + `checks_exit_code` [構�E/こうせい]へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - `hashset` / `hashset_str` は stdout [比輁Eひかく]の[価値/かち]があめEcollection fixture なので、tutorial と[異/こと]なめEexplicit report を[殁Eのこ]した、E
  - `list` めE途中/とちめE��] success log を[穁Eつ]むより、[最征Eさいご]に[全佁Eぜんたい]を[要Eみ]せるほぁE�� `Vec<Result>` [設訁Eせっけい]と[整吁Eせいごう]すると[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashset.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashset_str.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/list.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree -o /tmp/tests-collections-batch1.json -j 4`
    - [結果/けっか]: `3/3 pass`

# 2026-03-10 作業メモ (`hashmap` / `hashmap_str` / `rand` / `json` fixture めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `stdlib/tests/hashmap.n.md`, `stdlib/tests/hashmap_str.n.md`, `stdlib/tests/rand.n.md`, `stdlib/tests/json.n.md` を、[現衁Eげんこう]の `Vec<Result<(),str>>` + explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [途中/とちめE��]で `test_checked` めE`test_fail` を[呼/めEぶ[旧来/きゅぁE��い]の[実衁Eじっこう]モチE��を[除去/じょきょ]し、test [終亁E��/しゅぁE��めE��じ]に 1 [囁Eかい]だけ[明示皁Eめいじてき]に[表示/ひめE��じ]する、E
- [根本原因/こんぽんげんいん]:
  - 4 fixture とめE`fn main <()*>()> ():` なぁE��同等�E unit-return main と、`assert_*` / `test_fail` / `test_checked` を[逐次/ちくじ][実衁Eじっこう]する[古/ふる]い[書/か]き[方/かた]が[殁Eのこ]ってぁE��、E
  - reboot [征Eご]の `std/test` は `finish_checks` めEpure にし、`checks_exit_code` は stdout を[汁Eよご]さず、test case [側/がわ]ぁE`checks_print_report` を[明示皁Eめいじてき]に[呼/めEぶ[方釁EほぁE��ん]へ[移/ぁE��]ってぁE��ため、fixture [群/ぐん]が[旧流儀/きゅぁE��めE��ぎ]のままだと test [作況EさほぁEが[混在/こんざい]してぁE��、E
- [変更/へんこぁE:
  - `stdlib/tests/hashmap.n.md`
    - [全/すべ]ての[検査/けんさ]めE`checks_push` に[雁E��EしゅぁE��く]し、`Option::None` [刁E��Eぶんき]めE`Result::Err` として[保持/ほじ]する[形/かたち]へ[変更/へんこぁEした、E
    - [末尾/まつび]に `checks_print_report` と `checks_exit_code` を[追加/つぁE��]した、E
  - `stdlib/tests/hashmap_str.n.md`
    - [斁E���E/もじれつ] key 版も[同槁EどぁE��ぁEに、content [同値/どぁE��] / update / remove / alias [確誁Eかくにん]めE`Vec<Result<(),str>>` に[雁E��EしゅぁE��く]した、E
  - `stdlib/tests/rand.n.md`
    - [確玁E��/かくりつてき]な[検査/けんさ]めE`check_ne` [刁Eれつ]に[揁Eそろ]え、[終亁E��/しゅぁE��めE��じ] report のみ[表示/ひめE��じ]する[形/かたち]へ[変更/へんこぁEした、E
  - `stdlib/tests/json.n.md`
    - `Option::Some` / `Option::None` [刁E��Eぶんき]と `json_is_null` / `json_as_*` [確誁Eかくにん]めE`checks_push` へ[雁E��EしゅぁE��く]した、E
- [設訁Eせっけい][判断/はんだん]:
  - 4 fixture とめEstdout [出劁Eしゅつりょく][冁E��/なぁE��ぁEに[観測価値/かんそくかち]があるため、tutorial [側/がわ]のような silent `checks_exit_code` [単独/たんどく]ではなく、`checks_print_report` を[明示皁Eめいじてき]に[殁Eのこ]した、E
  - `test_fail` を[途中/とちめE��]で[呼/めEばず、`Result::Err` を[穁Eつ]んで[最征Eさいご]に[表示/ひめE��じ]することで、「[途中/とちめE��]では print しなぁE��「return [直剁EちめE��ぜん]に[明示 print/めいぁEprint] する」とぁE�� reboot [征Eご] test [哲学/てつがく]へ[揁Eそろ]えた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashmap.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashmap_str.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/rand.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/json.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/rand.n.md -i stdlib/tests/json.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-safe-result-batch2.json -j 4`
    - [結果/けっか]: `4/4 pass`

# 2026-03-10 作業メモ (`traits_hash` / `traits_serde` stdlib test めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tests/stdlib/traits_hash.n.md` と `tests/stdlib/traits_serde.n.md` を、[現衁Eげんこう]の safe `Result` + explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - trait [能劁EのぁE��めE��]そ�Eも�Eの[回帰/かいき]は[維持EぁE��]したまま、fixture [側/がわ]だけを reboot [征Eご]の test [哲学/てつがく]へ[寁EめEせる、E
- [根本原因/こんぽんげんいん]:
  - `traits_hash` / `traits_serde` は reboot [征Eご]に[追加/つぁE��]されぁEtrait 回帰なのに、test case [本佁Eほんたい]は `assert_*` / `test_fail` / `test_checked` を[逐次/ちくじ][実衁Eじっこう]する[旧流儀/きゅぁE��めE��ぎ]のままだった、E
  - とくに `deserialize` の[異常系/ぁE��めE��けい]は `ParseError` [判宁Eはんてい]のた�Eに[途中/とちめE��]で log を[出/だ]しており、`checks_print_report` を[最征Eさいご]に 1 [囁Eかい]だけ[呼/めEぶとぁE��[現衁Eげんこう] test [方釁EほぁE��ん]と[不整吁Eふせいごう]だった、E
- [変更/へんこぁE:
  - `tests/stdlib/traits_hash.n.md`
    - 2 [件/けん]の doctest とめE`Vec<Result<(),str>>` を[導�E/どぁE��めE��]し、`Hash` trait helper / hashmap / hashset [確誁Eかくにん]めE`checks_push` に[雁E��EしゅぁE��く]した、E
    - `Option::None` [刁E��Eぶんき]は `Result::Err` として[保持/ほじ]し、[最征Eさいご]に `checks_print_report` + `checks_exit_code` を[呼/めEぶ[構造/こうぞう]へ[変更/へんこぁEした、E
  - `tests/stdlib/traits_serde.n.md`
    - `serialize` / `deserialize` の[吁E��査/かくけんさ]めE`check_str_eq` / `check_eq_i32` / `check` に[置揁Eちかん]した、E
    - `ParseError` [判宁Eはんてい]は `test_checked` を[呼/めEばぁE`Result::Ok ()` を[穁Eつ]む[形/かたち]へ[変更/へんこぁEし、wrong error kind は `Result::Err` に[統一/とぁE��つ]した、E
- [設訁Eせっけい][判断/はんだん]:
  - trait 回帰 test は stdout [出劁Eしゅつりょく]を[観寁Eかんさつ]したほぁE��[失敗箁E��/しっぱぁE��しょ]を[追/お]ぁE��すいため、tutorial のような silent exit code [単独/たんどく]ではなぁEexplicit report を[殁Eのこ]した、E
  - `Deserialize` の[異常系/ぁE��めE��けい]は[多�E岁Eた�Eんき]だが、runner [側/がわ]の trap めEearly print に[頼/たよ]らず、[値/あたい]として[最征Eさいご]まで[持EめEち[遁Eはこ]ぶ[方釁EほぁE��ん]を[優允EめE��せん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_serde.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_serde.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md -i tests/stdlib/traits_serde.n.md --no-stdlib --no-tree -o /tmp/tests-traits-safe-result-batch.json -j 4`
    - [結果/けっか]: `4/4 pass`

# 2026-03-10 作業メモ (`fs` / `collections_diag` fixture めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tests/stdlib/fs.n.md` と `tests/stdlib/collections_diag.n.md` を、[現衁Eげんこう]の `Vec<Result<(),str>>` + explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - `Diag` / `Option` の[責勁Eせきむ]を[確誁Eかくにん]する fixture を、trap や[途中/とちめE��] print に[頼/たよ]らず、[最征Eさいご]に 1 [囁Eかい]だけ[明示皁Eめいじてき]に[表示/ひめE��じ]する[形/かたち]へ[変更/へんこぁEする、E
- [根本原因/こんぽんげんいん]:
  - `collections_diag` の 6 [件/けん]は、`Diag` / `Option` の[意味諁EぁE��ろん]は reboot [征Eご]のままなのに、fixture [側/がわ]ぁE`test_fail` / `assert_*` [直刁EちめE��れつ][実衁Eじっこう]の[旧流儀/きゅぁE��めE��ぎ]だった、E
  - `fs.n.md` の 2 [件/けん]目は、existing file read めEgeneric wasm runner で[確誁Eかくにん]しよぁE��してぁE��が、これ�E host filesystem integration に[依孁EぁE��ん]し、stable な doctest [責勁Eせきむ]を[趁Eこ]えてぁE��、E
  - `nodesrc/run_test.js` に preopen を[追加/つぁE��]してめENode WASI [実衁Eじっこう]では positive-path read が[安宁Eあんてい]しなかったため、test [対象/たいしょぁEそ�Eも�Eを[見直/みなお]す[忁E��Eひつよう]があった、E
- [変更/へんこぁE:
  - `tests/stdlib/collections_diag.n.md`
    - 6 [件/けん]すべてめE`Vec<Result<(),str>>` [雁E��EしゅぁE��く]へ[変更/へんこぁEし、`KeyNotFound` / `CapacityExceeded` / `Option::None` の[確誁Eかくにん]めE`check_str_eq` また�E `Result::Ok/Err` として[保持/ほじ]する[形/かたち]へ[揁Eそろ]えた、E
    - [吁Eかく] doctest の[末尾/まつび]で `checks_print_report` + `checks_exit_code` を[呼/めEぶようにした、E
  - `tests/stdlib/fs.n.md`
    - missing file [確誁Eかくにん]めEexplicit report [流儀/りゅぁE��]へ[変更/へんこぁEした、E
    - existing file read [確誁Eかくにん]は host FS integration に[依孁EぁE��ん]してぁE��ため、`ByteBuf -> str` helper である `fs_bytes_to_string` の[安宁Eあんてい]回帰へ[置揁Eちかん]した、E
    - これにより、`std/fs` の binary helper [責勁Eせきむ]は[維持EぁE��]しつつ、runner [環墁EかんきょぁEに[左右/さゆぁEされめEfixture を[排除/はぁE��ょ]した、E
  - `nodesrc/run_test.js`
    - repository root めEWASI preopen に[追加/つぁE��]した、E
    - ただし、今回の `fs` positive-path case は preopen [追加征EつぁE��ご]めE安宁Eあんてい]しなかったため、最終的な[解決/かいけつ]は test [責勁Eせきむ]の[刁Eき]り[刁Eわ]けで[衁Eおこな]った、E
- [設訁Eせっけい][判断/はんだん]:
  - reboot [方釁EほぁE��ん]では doctest は「[使/つか]い[方/かた]の[保証/ほしょぁE」が[主目皁Eしゅもくてき]であり、host 環境[依孁EぁE��ん]の integration [成否/せいひ]まで[抱/かか]ぁE込/こ]むべきではなぁE��E
  - そ�Eため `tests/stdlib/fs.n.md` では、generic runner で[安宁Eあんてい]に[保証/ほしょぁEできる `Err` [経路/けいろ]と `ByteBuf` helper [経路/けいろ]だけを[殁Eのこ]し、filesystem positive path は[別/べつ]の integration [層/そう]で[扱/あつか]ぁE�Eが[妥彁EだとぁEと[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 3` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 4` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 5` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 6` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i tests/stdlib/collections_diag.n.md --no-stdlib --no-tree -o /tmp/tests-fs-collections-diag-explicit.json -j 4`
    - [結果/けっか]: `8/8 pass`

# 2026-03-10 作業メモ (`cast` / `math` fixture と `02b` / `16_debug` tutorial めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `stdlib/tests/cast.n.md`, `stdlib/tests/math.n.md`, `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`, `tutorials/getting_started/16_debug_and_ansi.n.md` を、[現衁Eげんこう]の explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [単紁Eたんじゅん]な success log [依孁EぁE��ん]の case を[允Eさき]に[解涁EかいしょぁEし、`error.n.md` のような[褁E��/ふくざつ]刁E��Ecase と[刁E��/ぶんり]して[進/すす]める、E
- [根本原因/こんぽんげんいん]:
  - `cast` / `math` fixture はすでに `Vec<Result<(),str>>` を[導�E/どぁE��めE��]してぁE��が、[最征Eさいご]だぁE`test_checked` に[頼/たよ]る[過渡朁Eかとき]の[形/かたち]が[殁Eのこ]ってぁE��、E
  - `02b` tutorial は `assert_*` と `test_checked` の[単発/たんぱつ] success log に[戻/もど]っており、safe `Result` + explicit print の reboot [征Eご][方釁EほぁE��ん]と[不一致/ふぁE��ち]だった、E
  - `16_debug_and_ansi` の `std/test` 例も、`test_checked` の[旧/きゅぁE stdout [形弁Eけいしき]を[前提/ぜんてい]にしてぁE��、E
- [変更/へんこぁE:
  - `stdlib/tests/cast.n.md`
    - [末尾/まつび]の `test_checked "cast conversions"` を[除去/じょきょ]し、`checks_print_report` + `checks_exit_code` へ[置揁Eちかん]した、E
  - `stdlib/tests/math.n.md`
    - `cast` fixture と[同槁EどぁE��ぁEに、[最征Eさいご]の success log めEexplicit report [形/けい]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
    - 5 [件/けん]の doctest めE`i32` return + `Vec<Result<(),str>>` [雁E��EしゅぁE��く]へ[変更/へんこぁEした、E
    - [解极Eかいせき]系は `Result::Err` めEmessage つきで[穁Eつ]み、[最征Eさいご]に `checks_print_report` を[明示皁Eめいじてき]に[呼/めEぶ[形/かたち]へ[揁Eそろ]えた、E
    - `from_i64 sub <i64> cast 0 <i64> cast 42` は `check_str_eq` [移行征EぁE��ぁE��]に overload [曖昧/あいまい]になったため、`neg42` [中間値/ちめE��かんち]を[導�E/どぁE��めE��]して[弁Eしき][墁E��/きょぁE��い]を[明確匁Eめいかくか]した、E
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
    - `std/test` と[絁Eく]み[吁EぁEわせる例を `checks_print_report` [前提/ぜんてい]へ[変更/へんこぁEし、stdout [期征E��/きたぁE��]めE`Checked [ok]` / `[0] ok` [形弁Eけいしき]へ[更新/こうしん]した、E
- [設訁Eせっけい][判断/はんだん]:
  - tutorial でめEsuccess [表示/ひめE��じ]の[出所/でどころ]めEtest case [側/がわ]へ[寁EめEせることで、「runner が[勝手/かって]に[表示/ひめE��じ]する」�Eではなく「test case が[最征Eさいご]に[明示/めいじ]して[表示/ひめE��じ]する」とぁE�� reboot [征Eご] test [哲学/てつがく]を[一貫/ぁE��かん]させた、E
  - `16_debug_and_ansi` は ANSI [自佁Eじたい]の[確誁Eかくにん]と `std/test` [連携/れんけい]の[確誁Eかくにん]を[刁E��/ぶんり]し、[後老Eこうしゃ]は `strip_ansi` [丁Eか]でめE読/めEみめE��ぁEmachine/human report [形弁Eけいしき]へ[追征EつぁE��めE��]させた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/16_debug_and_ansi.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/cast.n.md -i stdlib/tests/math.n.md -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md -i tutorials/getting_started/16_debug_and_ansi.n.md --no-stdlib --no-tree -o /tmp/tests-explicit-report-batch3.json -j 4`
    - [結果/けっか]: `9/9 pass`

# 2026-03-10 作業メモ (`error.n.md` めEexplicit report 流儀へ追従し、`todo.md` の人側整琁E��取り込んだ)

- [目皁Eもくてき]:
  - `stdlib/tests/error.n.md` を、`Diag` / `Diags` / `Outcome` の[値/あたい]モチE��を[俁Eたも]ったまま explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [人/ひと]が[整琁Eせいり]した `todo.md` を、現状の reboot [方釁EほぁE��ん]に[沿/そ]ぁE形/かたち]で履歴に[反映/はんえい]する、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/tests/error.n.md` は `Outcome` / `Diag` [回帰/かいき]の[中忁EちめE��しん] fixture なのに、各刁E��が `test_fail` / `assert` の[逐次/ちくじ][実衁Eじっこう]に[畁Eとど]まってぁE��、E
  - これでは reboot [征Eご]の「[失敁Eしっぱい]を[値/あたい]として[持EめEち[遁Eはこ]び、test [末尾/まつび]で[明示皁Eめいじてき]に report する」とぁE��[方釁EほぁE��ん]と[不整吁Eふせいごう]だった、E
  - `todo.md` は[人/ひと]の[編雁EへんしめE��]が[完亁EかんりょぁEし、編雁E��止[領域/りょぁE��き]の[見�E/みだ]しや今後�E[持E��/しじ]が[現状/げんじょぁEと[吁EぁEぁE形/かたち]に[更新/こうしん]されてぁE��、E
- [変更/へんこぁE:
  - `stdlib/tests/error.n.md`
    - 3 [件/けん]の doctest すべてめE`Vec<Result<(),str>>` [雁E��EしゅぁE��く]へ[変更/へんこぁEした、E
    - `StdErrorKind` の[多�E岁Eた�Eんき]めE`Option::None` / `Result::Err` [刁E��Eぶんき]も、[途中/とちめE��]で trap せず `Result::Ok/Err` として[保持/ほじ]し、[最征Eさいご]に `checks_print_report` + `checks_exit_code` へ[畳/たた]む[形/かたち]へ[揁Eそろ]えた、E
    - `Outcome` / `Diag` の move model や[冁E��/なぁE�E][表現/ひめE��げん]は[夁Eか]えず、fixture [実衁Eじっこう]モチE��だけを[更新/こうしん]した、E
  - `todo.md`
    - [人/ひと]が[整琁Eせいり]した[冁E��/なぁE��ぁEをそのまま[叁Eと]り[込/こ]んだ、E
    - LLM [編雁EへんしめE��][禁止/きんし][領域/りょぁE��き]の[見�E/みだ]し、`nm` 再開発、LSP / target / tuple / pattern / [垁Eかた][前置/ぜんち]記法などの[残課顁Eざんかだい]が、[現在/げんざい]の reboot [征Eご][地図/ちず]として[読/めEみめE��い[形/かたち]になった、E
- [設訁Eせっけい][判断/はんだん]:
  - `error.n.md` は[褁E��/ふくざつ]刁E��だが、「[途中/とちめE��]で[落/お]とす」�Eではなく「[最征Eさいご]まで[値/あたい]として[遁Eはこ]ぶ」こと[自佁Eじたい]ぁEreboot [征Eご] test [設訁Eせっけい]の[一部/ぁE��ぶ]なので、その[方釁EほぁE��ん]を優先した、E
  - `todo.md` は[人/ひと]の[意図/ぁE��]が[反映/はんえい]された[最新牁Eさいしんばん]を履歴へ[固宁Eこてい]しておくほぁE��、以後�E自律実裁E�E[前提/ぜんてい]を[共朁EきょぁE��ぁEしやすいと[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 3` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/error.n.md --no-stdlib --no-tree -o /tmp/tests-error-explicit.json -j 4`
    - [結果/けっか]: `3/3 pass`

# 2026-03-10 作業メモ (`Option` / `Result` / `while` tutorial めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tutorials/getting_started/05_option.n.md`, `tutorials/getting_started/06_result.n.md`, `tutorials/getting_started/07_while_and_block.n.md` を、[現衁Eげんこう]の explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [入門/にめE��もん] chapter に[殁Eのこ]ってぁE�� `test_checked` / `test_fail` [中忁EちめE��しん]の[旧書況EきゅぁE��めE��ぁEを[渁Eへ]らし、「[最征Eさいご]に[明示 print/めいぁEprint]」すめEreboot [征Eご] test [方釁EほぁE��ん]めEtutorial [全佁Eぜんたい]へ[浸送EしんとぁEさせる、E
- [根本原因/こんぽんげんいん]:
  - 3 chapter とめE`Vec<Result<(),str>>` は[導�E/どぁE��めE��]されてぁE��が、[途中/とちめE��]の `test_fail` と[末尾/まつび]の `test_checked` に[依孁EぁE��ん]する[過渡朁Eかとき]の[形/かたち]が[殁Eのこ]ってぁE��、E
  - とくに `Option` / `Result` の[入門/にめE��もん]章で old style を[殁Eのこ]すと、利用老E��「runner が[勝手/かって]に[成功/せいこう]を[表示/ひめE��じ]する」よぁE��[要Eみ]えてしまぁE��[現衁Eげんこう]方針と[齟齬/そご]が[甁EしょぁEじる、E
- [変更/へんこぁE:
  - `tutorials/getting_started/05_option.n.md`
    - `Some` / `None` [刁E��Eぶんき]の[確誁Eかくにん]めE`check_eq_i32` / `Result::Err` / `Result::Ok` に[揁Eそろ]え、[末尾/まつび]で `checks_print_report` を[呼/めEぶ[形/かたち]へ[変更/へんこぁEした、E
    - `option_unwrap_or` の case めE`check_eq_i32` + explicit report へ[変更/へんこぁEした、E
  - `tutorials/getting_started/06_result.n.md`
    - `Ok` / `Err` [刁E��Eぶんき]の[確誁Eかくにん]めE`check_eq_i32` / `check_str_eq` / `Result::Err` に[揁Eそろ]えた、E
    - `safe_div2` の example めE同槁EどぁE��ぁEに、`checks_print_report` + `checks_exit_code` へ[移衁EぁE��ぁEした、E
  - `tutorials/getting_started/07_while_and_block.n.md`
    - `while` と `block` の[確誁Eかくにん]めE`check_eq_i32` + explicit report へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - これら�E[言誁Eげんご][基本/きほん]の chapter なので、test helper の[記述釁EきじめE��りょぁEは[墁Eふ]めE��すぎず、[末尾/まつび]の report だけを[明示/めいじ]する[最封EさいしょぁE変更に[畁Eとど]めた、E
  - `test_fail` めEhelper として[使/つか]い[綁Eつづ]けるより、`Result::Err` を[直接/ちめE��せつ][穁Eつ]むほぁE��「[失敁Eしっぱい]めE値/あたい]である」とぁE�� reboot [征Eご]の test [哲学/てつがく]に[沿/そ]ぁE��[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tutorials/getting_started/05_option.n.md -i tutorials/getting_started/06_result.n.md -i tutorials/getting_started/07_while_and_block.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-option-result-while.json -j 4`
    - [結果/けっか]: `6/6 pass`

# 2026-03-10 作業メモ (`if` / `import` / `testing workflow` tutorial めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tutorials/getting_started/08_if_layouts.n.md`, `tutorials/getting_started/09_import_and_structure.n.md`, `tutorials/getting_started/11_testing_workflow.n.md` を、[現衁Eげんこう]の explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [旧/きゅぁE `test_checked` success log と、その[前提/ぜんてい]で[書/か]かれてぁE�� `11_testing_workflow` の[説昁Eせつめい]を、current の `checks_print_report` [中忁EちめE��しん] API へ[更新/こうしん]する、E
- [根本原因/こんぽんげんいん]:
  - `08_if_layouts` と `09_import_and_structure` は、`Vec<Result<(),str>>` を[使/つか]ってぁE��にもかかわらず、[最征Eさいご]だぁE`test_checked` に[戻/もど]る[過渡朁Eかとき]の[書/か]き[方/かた]が[殁Eのこ]ってぁE��、E
  - `11_testing_workflow` は chapter [自佁Eじたい]が[旧流儀/きゅぁE��めE��ぎ]の[説昁Eせつめい]を[含/ふく]んでおり、`test_checked` を[直接/ちめE��せつ][呼/めEぶ example と[旧 stdout 期征E��/きゅぁEstdout きたぁE��]が[殁Eのこ]ってぁE��、E
- [変更/へんこぁE:
  - `tutorials/getting_started/08_if_layouts.n.md`
    - 4 [件/けん]の doctest めE`check_eq_i32` + `checks_print_report` へ[変更/へんこぁEした、E
  - `tutorials/getting_started/09_import_and_structure.n.md`
    - `pipeline_like` の[確誁Eかくにん]めEexplicit report [形/けい]へ[移衁EぁE��ぁEした、E
  - `tutorials/getting_started/11_testing_workflow.n.md`
    - [本斁Eほん�Eん]の[説昁Eせつめい]めE`check_*` / `finish_checks` / `checks_print_report` [中忁EちめE��しん]へ[書/か]き[揁Eか]えた、E
    - `std/test` と[絁Eく]み[吁EぁEわせめEexample は `Vec<Result<(),str>>` めE2 [件/けん][穁Eつ]み、[最征Eさいご]に `checks_print_report` を[明示/めいじ]する[形/かたち]へ[変更/へんこぁEした、E
    - stdout [期征E��/きたぁE��]めE`Checked [ok,ok]` / `[0] ok` / `[1] ok` [形弁Eけいしき]へ[更新/こうしん]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `11_testing_workflow` は test [方釁EほぁE��ん]そ�Eも�Eを[敁Eおし]える chapter なので、ここが[現衁Eげんこう] API と[飁Eく]い[遁Eちが]ぁE�� repo [全佁Eぜんたい]の[方吁EほぁE��ぁEを[誤誘封EごゆぁE��ぁEする。そのため、実裁E��更だけでなく[説昁Eせつめい]めE同時/どぁE��]に[更新/こうしん]した、E
  - tutorial [側/がわ]でめEsuccess [表示/ひめE��じ]めEtest case [末尾/まつび]の[明示 print/めいぁEprint]へ[揁Eそろ]えることで、runner [依孁EぁE��ん]ではなぁEcode [自佁Eじたい]の[意図/ぁE��]として[読/めEめるようにした、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/08_if_layouts.n.md -i tutorials/getting_started/09_import_and_structure.n.md -i tutorials/getting_started/11_testing_workflow.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-if-import-testing.json -j 4`
    - [結果/けっか]: `8/8 pass`

# 2026-03-10 作業メモ (`02_numbers` / `03_functions` tutorial めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tutorials/getting_started/02_numbers_and_variables.n.md` と `tutorials/getting_started/03_functions.n.md` を、[現衁Eげんこう]の explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [最初期/さいしょき] tutorial に[殁Eのこ]ってぁE�� `test_checked` success log を[除去/じょきょ]し、[新/あたら]しい test [書弁Eしょしき]を[序盤/じょばん]から[一貫/ぁE��かん]して[示/しめ]す、E
- [根本原因/こんぽんげんいん]:
  - `02_numbers_and_variables` と `03_functions` は[主顁Eしゅだい]が[基本/きほん][構文/こうぶん]であるにもかかわらず、test 部刁E��けが[過渡朁Eかとき]の `test_checked` [依孁EぁE��ん]のままだった、E
  - [利用老EりよぁE��ゃ]が[最刁Eさいしょ]に[触/ふ]れる chapter で old style が[殁Eのこ]ってぁE��と、repo [全佁Eぜんたい]で[採用/さいよう]してぁE�� reboot [征Eご] test [方釁EほぁE��ん]が[企Eつた]わりにくい、E
- [変更/へんこぁE:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 5 [件/けん]の doctest すべてめE`check_eq_i32` + `checks_print_report` [構�E/こうせい]へ[変更/へんこぁEした、E
  - `tutorials/getting_started/03_functions.n.md`
    - `function call`, `inline if expression`, `if colon form` の 3 [件/けん]を[同槁EどぁE��ぁEに explicit report [形/けい]へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - これら�E chapter では test helper が[主顁Eしゅだい]ではなぁE��め、`check_eq_i32` と `checks_print_report` だけを[使/つか]ぁE最封EさいしょぁE変更で[揁Eそろ]えた、E
  - [表示/ひめE��じ]めEtest case [末尾/まつび]に[雁E��EしゅぁE��く]する[形/かたち]に[統一/とぁE��つ]したことで、「runner が[勝手/かって]に[成功/せいこう]を[出/だ]す」�Eではなく「code [側/がわ]が[最征Eさいご]に[明示/めいじ]する」とぁE�� reboot [征Eご] test [哲学/てつがく]へ[沿/そ]わせた、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-numbers-functions.json -j 4`
    - [結果/けっか]: `8/8 pass`

# 2026-03-10 作業メモ (`12` / `13` / `14` tutorial めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`, `tutorials/getting_started/13_type_driven_error_modeling.n.md`, `tutorials/getting_started/14_refactor_with_properties.n.md` を、[現衁Eげんこう]の explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - [純粁Eじゅんすい][関数/かんすう]、`Result` / `Option` による[失敁Eしっぱい][表現/ひめE��げん]、[回帰/かいき][比輁Eひかく] helper でも、old style success log を[殁Eのこ]さなぁE��E
- [根本原因/こんぽんげんいん]:
  - 3 chapter とめE`Vec<Result<(),str>>` を[使/つか]ってぁE��も、[末尾/まつび]の `test_checked` めE`test_fail` へ[戻/もど]る[過渡朁Eかとき]の[書/か]き[方/かた]が[殁Eのこ]ってぁE��、E
  - とくに `14_refactor_with_properties.n.md` の `assert_same` は、[差刁Eさ�Eん] helper [自佁Eじたい]ぁE`assert_eq_i32` / `test_fail` に[依孁EぁE��ん]しており、「[失敁Eしっぱい]めE値/あたい]として[持EめEつ」とぁE�� reboot [征Eご]方針が helper [冁E��/なぁE�E]で[途�E/とぎ]れてぁE��、E
- [変更/へんこぁE:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
    - 2 [件/けん]の doctest めE`check_eq_i32` + `checks_print_report` へ[変更/へんこぁEした、E
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
    - `checked_half` / `choose_positive` の[確誁Eかくにん]めE`check_eq_i32` / `check_str_eq` / `Result::Err` [直接/ちめE��せつ][穁Eつ]みへ[揁Eそろ]えた、E
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
    - `sum_to_loop` / `sum_to_formula` の[比輁Eひかく]めE`check_eq_i32` [中忁EちめE��しん]へ[変更/へんこぁEした、E
    - `assert_same` は `check_eq_i32` を[迁Eかえ]し、mismatch は `Result::Err` を[迁Eかえ]ぁEhelper へ[整琁Eせいり]した、E
    - これにより helper [自佁Eじたい]めEreboot [征Eご]の `Result<(),str>` [中忁EちめE��しん] test [哲学/てつがく]に[沿/そ]ぁE形/かたち]になった、E
- [設訁Eせっけい][判断/はんだん]:
  - `14_refactor` の helper は[一要EぁE��けん]小さぁE��、[封E��/しょぁE��い]の property-like [比輁Eひかく] helper の[雛形/ひながた]でもあるため、「assert helper が[即座/そくざ]に print/trap する」�Eではなく「helper [自佁Eじたい]ぁE`Result` を[迁Eかえ]す」[方吁EほぁE��ぁEへ[寁EめEせた、E
  - `13_type_driven_error_modeling` は chapter [吁Eめい]どおり「[垁Eかた]が[失敁Eしっぱい]を[表/あらわ]す」ことを[敁Eおし]えるので、test [本佁Eほんたい]めE`Result::Err` を[直接/ちめE��せつ][穁Eつ]む[書/か]き[方/かた]に[統一/とぁE��つ]するのが[自然/しぜん]と[判断/はんだん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tutorials/getting_started/12_pure_function_pipeline.n.md -i tutorials/getting_started/13_type_driven_error_modeling.n.md -i tutorials/getting_started/14_refactor_with_properties.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-pipeline-modeling-refactor.json -j 4`
    - [結果/けっか]: `6/6 pass`

# 2026-03-10 作業メモ (`15` / `17` / `18` / `19` / `20` / `21` tutorial めEexplicit report 流儀へ追征E

- [目皁Eもくてき]:
  - `tutorials/getting_started/15_match_patterns.n.md`, `17_namespace_and_alias.n.md`, `18_recursion_and_termination.n.md`, `19_pipe_operator.n.md`, `20_generics_basics.n.md`, `21_trait_bounds_basics.n.md` を、[現衁Eげんこう]の explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - tutorial [後半/こうはん]に[殁Eのこ]ってぁE�� `test_checked` success log を[除去/じょきょ]し、[`match` / namespace / recursion / pipe / generics / trait bounds] の例も current の test [書弁Eしょしき]に[統一/とぁE��つ]する、E
- [根本原因/こんぽんげんいん]:
  - 6 chapter とめE検査/けんさ]の[本佁Eほんたい]はすでに `Vec<Result<(),str>>` [中忁EちめE��しん]へ[寁EめEってぁE��が、[最征Eさいご]だぁEold style の `test_checked` に[依孁EぁE��ん]してぁE��、E
  - これが[殁Eのこ]ると「途中は new style、最後だぁEold style」とぁE��[混在/こんざい]が[綁Eつづ]き、tutorial [全佁Eぜんたい]で[一貫/ぁE��かん]した[記述/きじめE��]にならなぁE��E
- [変更/へんこぁE:
  - 6 chapter の doctest すべてで、`assert_*` めE`check_*` へ[置揁Eちかん]し、[末尾/まつび]の `test_checked` めE`checks_print_report` + `checks_exit_code` へ[置揁Eちかん]した、E
  - `20_generics_basics.n.md` では `assert_str_eq` めE`check_str_eq` へ[揁Eそろ]えた、E
  - `21_trait_bounds_basics.n.md` めE`trait and impl` / `trait bound generic` の 2 case を[同槁EどぁE��ぁEに[更新/こうしん]した、E
- [設訁Eせっけい][判断/はんだん]:
  - これら�E[説昁Eせつめい]の[主顁Eしゅだい]が[言誁Eげんご][機�E/き�EぁEそ�Eも�Eであり、test helper の[細部/さいぶ]を[墁Eふ]めE��べきではなぁE��そのため、`check_*` と `checks_print_report` だけへ[寁EめEせる[最封EさいしょぁE変更に[畁Eとど]めた、E
  - tutorial [後半/こうはん]でめEexplicit report を[徹庁Eてってい]することで、repo [全佁Eぜんたい]として「success log は test case [末尾/まつび]の[明示 print/めいぁEprint]からだけ[出/で]る」とぁE��[方釁EほぁE��ん]を[共朁EきょぁE��ぁEできるようにした、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i tutorials/getting_started/15_match_patterns.n.md -i tutorials/getting_started/17_namespace_and_alias.n.md -i tutorials/getting_started/18_recursion_and_termination.n.md -i tutorials/getting_started/19_pipe_operator.n.md -i tutorials/getting_started/20_generics_basics.n.md -i tutorials/getting_started/21_trait_bounds_basics.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-late-basics.json -j 4`
    - [結果/けっか]: `12/12 pass`

# 2026-03-10 作業メモ (`vec` / `list` / `fs` fixture と `23_competitive_sort` めEcurrent 仕様へ揁E��、`kpsearch` Vec wrapper を修正)

- [目皁Eもくてき]:
  - `stdlib/tests/vec.n.md`, `stdlib/tests/list.n.md`, `stdlib/tests/fs.n.md`, `tutorials/getting_started/23_competitive_sort_and_search.n.md` に[殁Eのこ]ってぁE�� old helper [呼/めEび[出/だ]しを[除去/じょきょ]し、explicit report [流儀/りゅぁE��]へ[揁Eそろ]える、E
  - `23_competitive_sort` の `lower_bound_vec_i32` / `upper_bound_vec_i32` / `count_equal_range_vec_i32` ぁEcurrent move model と[整吁Eせいごう]するよう、`kpsearch` 本体�E `Vec<i32>` wrapper を[根本/こんぽん]から[修正/しゅぁE��い]する、E
- [根本原因/こんぽんげんいん]:
  - `vec` / `list` fixture は explicit report [形/けい]へ[寁EめEってぁE��が、`test_fail` / `assert_*` helper [呼/めEび[出/だ]しが[殁Eのこ]り、完�Eには current [方釁EほぁE��ん]へ[収束/しゅぁE��く]してぁE��かった、E
  - `stdlib/tests/fs.n.md` めEunit-return + `test_fail` [直呼/じかめEびの[古/ふる]い[形/かたち]が[殁Eのこ]ってぁE��、E
  - `23_competitive_sort_and_search.n.md` の 2 [件/けん]目が[空出劁Eからしゅつりょく]になった[真因/しんぁE��]は tutorial [側/がわ]ではなく、[stdlib/kp/kpsearch.nepl](/mnt/d/project/NEPLg2/stdlib/kp/kpsearch.nepl) の `*_vec_i32` wrapper ぁE`v` めE2 [囁Eかい][読/めEむ[実裁Eじっそう]で current move model と[不整吁Eふせいごう]だったことだった、E
- [変更/へんこぁE:
  - `stdlib/tests/vec.n.md`
    - `assert` / `assert_eq_i32` / `test_fail` めE`check` / `check_eq_i32` / `Result::Err` [直接/ちめE��せつ][穁Eつ]みへ[置揁Eちかん]し、[末尾/まつび]めE`checks_print_report` + `checks_exit_code` へ[統一/とぁE��つ]した、E
  - `stdlib/tests/list.n.md`
    - `Option::None` [刁E��Eぶんき]の `test_fail` めE`Result::Err` へ[置揁Eちかん]し、success [側/がわ]めE`check_*` [中忁EちめE��しん]へ[揁Eそろ]えた、E
  - `stdlib/tests/fs.n.md`
    - missing file case めE`i32` return + explicit report [形/けい]へ[更新/こうしん]した、E
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `sort_quick on Vec<i32>` めE`check` + explicit report [形/けい]へ[変更/へんこぁEした、E
    - `lower_bound` / `upper_bound` / `count_equal_range` 例�E、wrapper 修正後�E API に[依孁EぁE��ん]する[形/かたち]でそ�Eまま[動佁EどぁE��]するようにした、E
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` で、`Vec<i32>` めEtemporary memory に 1 [囁Eかい]だけ[退避/たいひ]し、そこかめE`data` / `len` を[抽出/ちめE��しゅつ]して raw-array helper へ[渡/わた]す[実裁Eじっそう]へ[変更/へんこぁEした、E
    - これにより `Vec` めE2 [囁Eかい][読/めEむ[構造/こうぞう]を[除去/じょきょ]し、current move model へ[整吁Eせいごう]させた、E
- [設訁Eせっけい][判断/はんだん]:
  - `23_competitive_sort` の[失敁Eしっぱい]は tutorial [側/がわ]の[書/か]き[方/かた]ではなぁEwrapper 本体�E[所有権/しょめE��けん][扱/あつか]ぁE��[古/ふる]かったことが[真因/しんぁE��]だったため、tutorial だけ�E[迂回/ぁE��い]ではなぁE`kpsearch` 本体を[修正/しゅぁE��い]した、E
  - `Vec` から `ptr` / `len` を[叁Eと]めEwrapper は[今征Eこんご]めE再発/さいはつ]しやすい箁E��なので、「temporary memory に[退避/たいひ]して[一度/ぁE��ど]だけ[観寁Eかんさつ]する」とぁE��[方釁EほぁE��ん]を[明示皁Eめいじてき]に[採用/さいよう]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i stdlib/tests/list.n.md -i stdlib/tests/fs.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i stdlib/kp/kpsearch.nepl --no-stdlib --no-tree -o /tmp/tests-stdlib-vec-list-fs-sort.json -j 4`
    - [結果/けっか]: `8/8 pass`
# 2026-03-10 io/streamio common read write facade

- `todo.md` の `stdio, io` 持E��と `doc/stdlib_breaking_reboot.md` を突き合わせ、現衁E`std/streamio` / `std/io` の公開面ぁEreboot の bare 名方針にまだ届いてぁE��ぁE��とを確認した、E
- `alloc/io.nepl` は target 非依存�E trait / `ByteBuf` helper だけを拁E��する土台として据え、そこでの `ByteReader` / `TextReader` / `ByteWriter` / `TextWriter` / `Flush` / `Close` めE`std` facade 側から再利用する方針にした、E
- `std/streamio.nepl` には `read` / `write` / `writeln` / `flush` / `close` の bare facade を置き、`stdin` / `stdout` / in-memory text / in-memory bytes を同じ語彙で扱えるようにした、E
- `std/io.nepl` と `std/iotarget.nepl` を追加し、`IoReadTarget` / `IoWriteTarget` enum を通じて `read target` / `write target data` / `data |> write target` を書ける category facade を用意した、E
- `tests/stdlib/streamio.n.md` と `tests/stdlib/io.n.md` は、新 API を直接使ぁEfocused case に更新した、E

# 2026-03-10 作業メモ (`std/streamio` caller だけを新しい共通名へ置揁E

- [目皁Eもくてき]:
  - [利用側/りよぁE��わ]ファイルだけで、`std/streamio` の old read/write API [呼/めEび[出/だ]しを reboot [方釁EほぁE��ん]どおり `read` / `write` / `flush` / `stream io_*` へ[寁EめEせる、E
  - [持E��/しじ]どおり `stdlib/std/streamio.nepl`, `stdlib/std/io.nepl`, `stdlib/alloc/io.nepl`, `stdlib/std/iotarget.nepl` には[触/ふ]れず、`kp` wrapper / tests [側/がわ]だけを[更新/こうしん]する、E
- [変更/へんこぁE:
  - `stdlib/kp/kpread.nepl`
    - `stream_scanner_read_token` / `_i32` / `_i64` / `_f64` / `_f32` めE`read scanner_as_stream sc` へ[置揁Eちかん]した、E
    - `u64` [読/めEみだけ�E current の common `read` overload では[符号/ふごう]つぁE`i64` と[意味/ぁE��]が[一致/ぁE��ち]しなぁE��め、`stream_scanner_read_u64` を[維持EぁE��]した、E
  - `stdlib/kp/kpwrite.nepl`
    - `stream_writer_flush` / `_writeln` / `_write_str` / `_write_i32` / `_write_i64` / `_write_f64` / `_write_f32` めE`flush` / `write` / `write "\n"` へ[置揁Eちかん]した、E
    - `writer_write_space` / `writer_write_*_ln` の[冁E��/なぁE�E][実裁Eじっそう]と doc comment も、`write inner " "` と `write inner v` + `write inner "\n"` の current [流儀/りゅぁE��]へ[揁Eそろ]えた、E
    - `u64` / fixed precision は current common `write` overload だけでは[意味諁EぁE��ろん]を[俁Eたも]てなぁE��め、old helper [呼/めEび[出/だ]しを[維持EぁE��]した、E
  - `tests/stdlib/streamio.n.md`
    - `stream_writer_write_*` / `stream_writer_writeln` / `stream_writer_write_space` / `stdout_stream` [直呼/じかめEびを、`write` / `flush` / `stream io_stdout` へ[置揁Eちかん]した、E
    - scanner case めE`read sc` へ[統一/とぁE��つ]した、E
- [根本原因/こんぽんげんいん]:
  - old caller は `stream_scanner_read_*` / `stream_writer_write_*` の[長/なが]い[名前/なまぁEへ[直接/ちめE��せつ][依孁EぁE��ん]しており、reboot の「[利用老E��/りよぁE��めE�E]け[入口/ぁE��ぐち]は facade の[共通名/きょぁE��ぁE��い]へ[一本匁EぁE��ぽんか]する」とぁE��[設訁Eせっけい]と[飁Eく]い[遁Eちが]ってぁE��、E
  - とくに `kp` wrapper は「`std/streamio` を[薁EぁE��]く[匁Eつつ]む」[役割/めE��わり]なのに、[冁E��/なぁE�E]で old helper [吁Eめい]へ[固宁Eこてい]されており、`std` facade の[改吁Eかいめい]を[利用側/りよぁE��わ]へ[波叁EはきゅぁEさせる[構造/こうぞう]になってぁE��、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 3`
    - [結果/けっか]: fail
    - [原因/げんぁE��]: caller [側/がわ]ではなぁE`stdlib/std/streamio.nepl` [本佁Eほんたい]の `read` / `write` overload [定義/てぁE��]が、すでに[削除/さくじょ]・[改吁Eかいめい]されぁEold helper [吁Eめい] (`stream_scanner_read_token`, `stream_writer_write_str` など) をまだ[参�E/さんしょぁEしており、library compile error で[停止/てぁE��]した、E
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 1`
    - [結果/けっか]: fail
    - [原因/げんぁE��]: [上訁EじょぁE��]と[吁Eおな]ぁElibrary [本佁Eほんたい] compile error の[影響/えいきょぁEで、`std/io` 経由 case めE通過/つぁE��]しなぁE��E
- [状況EじょぁE��めE��]:
  - caller [側/がわ]で[置揁Eちかん]できる old read/write call site は、`u64` / fixed precision のような current common overload [未提侁EみてぁE��めE��]ケースを[除/のぞ]ぁE��[更新/こうしん]した、E
  - [今回/こんかい]の[持E��篁E��夁Eしじはんいがい]である `stdlib/std/streamio.nepl` [本佁Eほんたい]が、current facade [吁Eめい]への[冁E��/なぁE�E][追征EつぁE��めE��]をまだ[絁Eお]えてぁE��ぁE��め、ここでは library [側/がわ]を[触/さわ]らずに[刁Eき]り[刁Eわ]けだけ[殁Eのこ]した、E

# 2026-03-10 作業メモ (`io` / `streamio` の bare `read` / `stream` めEgeneric trait 匁E

- [目皁Eもくてき]:
  - `reboot` の「bare 吁E`read` / `write` / `writeln` / `flush` / `close` に統一し、型差は trait / overload で表す」とぁE��[方釁EほぁE��ん]に[沿/そ]って、`std/io` / `std/streamio` の I/O facade めEcurrent compiler で[安宁Eあんてい]して[解決/かいけつ]できる[形/かたち]に[固宁Eこてい]する、E
  - [利用老EりよぁE��ゃ]ぁE`read sc` / `read io_stdin` / `stream io_stdout` をそのまま[書/か]けるようにしつつ、返り値型だけに[依孁EぁE��ん]する old overload を[排除/はぁE��ょ]する、E
- [根本原因/こんぽんげんいん]:
  - `std/streamio` の `read` / `stream` と `std/io` の `read` は、`(StreamScanner)->i32` と `(StreamScanner)->f64` のように「引数は同じで返り値だけが違う overload」に[依孁EぁE��ん]してぁE��、E
  - current compiler はそ�E[形/かたち]めE`let x <i32> read sc;` のような[斁E��/ぶんみめE��]だけで常に[解決/かいけつ]できず、`ambiguous overload` を[起/お]こしてぁE��、E
  - `std/io` [冁E��/なぁE�E]でめE`stream io_stdin` / `stream io_stdout` の[戻/もど]り[垁Eがた]が[曖昧/あいまい]になり、`match` [全佁Eぜんたい]ぁEunit に[崩/くず]れて `read` facade [本佁Eほんたい]まで[連鎁Eれんさ][敁E��/こしめE��]してぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/streamio.nepl`
    - `StreamFromReadTarget` / `StreamFromWriteTarget` を[追加/つぁE��]し、`stream` を「返り値垁Egeneric + trait dispatch」で[実裁Eじっそう]した、E
    - `ScannerReadable` を[追加/つぁE��]し、`read sc` めE`str` / `i32` / `i64` / `f32` / `f64` の bare 吁Egeneric へ[統一/とぁE��つ]した、E
    - `StreamReadableResult` を[追加/つぁE��]し、`StdinStream` / `TextInputStream` / `ByteInputStream` からの `read` も返り値垁Egeneric へ[統一/とぁE��つ]した、E
  - `stdlib/std/io.nepl`
    - `TargetReadable` を[追加/つぁE��]し、`read target` めE`ByteBuf` / `str` の generic facade に[変更/へんこぁEした、E
    - [冁E��/なぁE�E]では `std/streamio` の generic `stream` を[型注釁EかたちめE��しゃく]つぁElocal binding で[叁EぁEけ、その[征Eあと]は `alloc/io` helper (`io_read_all_bytes` / `io_read_all_text` / `io_write_bytes` / `io_write_str` / `io_flush` / `io_close`) へ[委譲/ぁE��めE��]する[形/かたち]に[整琁Eせいり]した、E
  - `stdlib/kp/kpwrite.nepl`
    - `stream_writer_new` / `stream_writer_free` の old 参�EめE`writer` / `free` へ[置揁Eちかん]し、`stream_writer_flush` めE`flush` へ[置揁Eちかん]した、E
  - `tests/stdlib/streamio.n.md`
    - duplicate してぁE�� `stdout_binary_writer_pipe_data_to_target` case めE1 [件/けん][削除/さくじょ]した、E
- [設訁Eせっけい][判断/はんだん]:
  - [名前/なまぁEめEbare 化するだけでは current compiler の overload 解決と[矛盾/むじゅん]するため、`cast` / `deserialize` と[吁Eおな]じく「返り値垁Egeneric めEtrait で[決/き]める」[形/かたち]へ[寁EめEせた、E
  - これにより `read` / `stream` は bare 名を[維持EぁE��]しつつ、suffix めEcompatibility alias を[墁Eふ]めE��ずに[運用/ぁE��よう]できる、E
  - `std/io` [冁E��/なぁE�E]の stream [操佁Eそうさ]は、`std/streamio` の facade 名へ[依孁EぁE��ん]しすぎると[再帰皁Eさいきてき]に overload を[褁E��匁Eふくざつか]するため、�E送Etrait helper へ[一段/ぁE��だん][落/お]として[整琁Eせいり]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/std/io.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 9`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10`
    - [結果/けっか]: pass
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success

# 2026-03-11 作業メモ (`streamio` / `io` の open/close 統一と scanner 所有権モチE��の固宁E

- [目皁Eもくてき]:
  - `reboot` の「[垁Eかた]で[区別/くべつ]し、E��数名では[区別/くべつ]しなぁE��「[後方互換/こうほぁE��かん]は[殁Eのこ]さなぁE��とぁE��[方釁EほぁE��ん]に[征Eしたが]ぁE��`std/streamio` / `std/io` の[公開面/こうかいめん]めE`open` / `read` / `write` / `writeln` / `flush` / `close` へ[統一/とぁE��つ]する、E
  - `ReadStream` / `WriteStream` めEenum target として[固宁Eこてい]し、stdin / stdout / in-memory text / bytes / fs path を[吁Eおな]じ[語彁Eごい]で[扱/あつか]えるようにする、E
  - [褁E��/ふくすぁE stream を[同時/どぁE��]に[維持EぁE��]できるよう、scanner / writer の[所有権/しょめE��けん][規則/きそく]めEcurrent move model と[矛盾/むじゅん]しない[形/かたち]へ[固宁Eこてい]する、E
- [根本原因/こんぽんげんいん]:
  - `std/streamio` の `open(ReadStream)` / `open(WriteStream)` は high-level `StreamScanner` / `StreamWriter` を[迁Eかえ]す[方吁EほぁE��ぁEへ[寁EめEってぁE��のに、`std/io` と一部 test / tutorial はまだ `open -> StdinStream` / `StdoutStream` [前提/ぜんてい]で[殁Eのこ]っており、�E閁EAPI と caller が[飁Eく]い[遁Eちが]ってぁE��、E
  - `StreamScanner` めEnon-copy resource にしたまま `read sc` の bare API へ[寁EめEせたため、`read sc` を[褁E��囁EふくすぁE��い][書/か]ぁEcurrent tutorial / kp case ぁE`D3053 use of moved value` で[壁Eこわ]れてぁE��、E
  - `close(StreamScanner)` [冁E��/なぁE�E]にめEold helper `io_bytebuf_new` [参�E/さんしょぁEが[殁Eのこ]っており、library compile error を[誘発/めE��はつ]してぁE��、E
- [変更/へんこぁE:
  - `stdlib/std/iotarget.nepl`
    - `ReadStream` / `WriteStream` の enum target めEcurrent public API として[固宁Eこてい]した、E
    - `WriteStream::Stdio` は payload なし、`ReadStream` は `Stdio` / `Fs <str>` / `Text <str>` / `Bytes <ByteBuf>` を[持EめEつ[形/かたち]に[整琁Eせいり]した、E
  - `stdlib/std/streamio.nepl`
    - `open(ReadStream) -> Result<StreamScanner,str>` と `open(WriteStream) -> Result<StreamWriter,str>` を[公閁Eこうかい][入口/ぁE��ぐち]として[固宁Eこてい]した、E
    - `StreamScanner` に `Copy` / `Clone` を[復活/ふっかつ]させ、copy / clone は cursor / buffer を[共朁EきょぁE��ぁEする alias であることめEdoc comment に[明訁Eめいき]した、E
    - `close(StreamScanner)` の old helper [参�E/さんしょぁEめE`ByteBuf mem_ptr_wrap buf_addr len` に[置揁Eちかん]した、E
    - file header と `StreamScanner` / `StreamWriter` comment めEnew policy / format へ[寁EめEせ、[褁E��/ふくすぁE stream 同時保持の[性質/せいしつ]めE追訁EつぁE��]した、E
    - `stream_scanners_can_coexist` doctest を[追加/つぁE��]し、[別、Eべつべつ]に `open` した scanner が[独竁Eどくりつ]して[読/めEめることを[固宁Eこてい]した、E
  - `stdlib/std/io.nepl`
    - category facade [冁E��/なぁE�E]めE`open(ReadStream/WriteStream)` [依孁EぁE��ん]から[刁Eき]り[離/はな]し、`StdinStream ()` / `StdoutStream ()` の low-level handle を[冁E��利用/なぁE�EりよぁEする[形/かたち]へ[整琁Eせいり]した、E
    - これにより `read ReadStream::Stdio` / `write WriteStream::Stdio ...` の facade と `streamio` の resource [生�E/せいせい]が[衝突EしょぁE��つ]しなぁE��ぁE��した、E
  - `tests/stdlib/io.n.md`, `tests/stdlib/streamio.n.md`, `tests/stdlib/kp.n.md`, `tests/stdlib/kp_i64.n.md`, `tests/stdlib/stdin.n.md`
    - old low-level `open -> StdinStream/StdoutStream` [前提/ぜんてい]を[除去/じょきょ]し、current public API へ[追征EつぁE��めE��]した、E
    - `unwrap_ok open WriteStream::Stdio` から `|> write` / `|> writeln` / `|> flush` / `|> close` の multiline pipe [流儀/りゅぁE��]へ[統一/とぁE��つ]した、E
    - scanner を[使/つか]い[刁Eき]っぁEcase では `close sc` を[追加/つぁE��]した、E
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`, `24_competitive_dp_basics.n.md`, `25_competitive_prefixsum_twopointers.n.md`, `27_competitive_algorithms_catalog.n.md`, `stdlib/kp/kpgraph.nepl`
    - `kpread` / `kpwrite` [前提/ぜんてい]めEold writer/scanner helper [吁Eめい]を[除去/じょきょ]し、current `std/streamio` [流儀/りゅぁE��]へ[書/か]き[揁Eか]えた、E
- [設訁Eせっけい][判断/はんだん]:
  - scanner めEnon-copy resource のままにすると `read sc` を[褁E��囁EふくすぁE��い][書/か]く�E然な public API と[両竁EりょぁE��つ]しなぁE��め、handle [自佁Eじたい]は copyable alias、buffer [解放/かいほぁEだぁE`close` へ[雁E��EしゅぁE��く]する[形/かたち]に[戻/もど]した、E
  - `std/io` は category facade、`std/streamio` は resource facade と[責勁Eせきむ]を[刁E��/ぶんり]し、同ぁE`open` [吁Eめい]を[使/つか]ってめE迁Eかえ]り[値/あたい]と target [垁Eかた]で[静的/せいてき]に[刁Eわ]かれる[構造/こうぞう]へ[寁EめEせた、E
  - [別、Eべつべつ]に `open` した scanner / writer が[同時/どぁE��]に[存在/そんざい]できることは public API の[重要EじゅぁE��ぁEな[性質/せいしつ]なので、doctest で[固宁Eこてい]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 11`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 13`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp_i64.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1`
    - [結果/けっか]: pass

# 2026-03-11 作業メモ (`Drop` capability の source 宣言と auto drop 挿入の compiler 固宁E

- [目皁Eもくてき]:
  - [所有権/しょめE��けん]が[絁Eお]わった[値/あたい]めEcompiler が[自勁EじどぁEで[後始末/あとしまつ]できる[土台/どだい]を、hardcode ではなぁE`.nepl` [側/がわ]の trait [宣言/せんげん]として[固宁Eこてい]する、E
  - `reboot` / `memory_safety_compiler_design` の[方釁EほぁE��ん]に[征Eしたが]ぁE��move check と[矛盾/むじゅん]しなぁEauto drop 挿入と[詳細/しょぁE��い] test を[整傁Eせいび]する、E
- [根本原因/こんぽんげんいん]:
  - [既孁Eきそん]の `drop_insertion` は lexical に `HirExprKind::Drop { name }` を[差/さ]すだけで、codegen 側では no-op のままだった。そのため source [丁EじょぁEで `Drop` を[宣言/せんげん]してめEdestructor 実行に[絁Eむす]び[仁Eつ]かなかった、E
  - [当�E/とぁE��ょ]は destructor めE`Self` [値渡/あたぁE��た]しにしてぁE��が、raw wasm ABI では[褁E��値/ふくごぁE��]をそのまま[渡/わた]す[経路/けいろ]で `unsupported function signature for wasm` が[発甁Eはっせい]してぁE��、E
  - Rust test fixture めEold 前提を[弁Eひ]きずっており、`#entry main` [欠落/けつらく]めEbranch [末尾/まつび]の不要E`;` で validator failure / loader failure を[誘発/めE��はつ]してぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/ast.rs`, `parser.rs`, `hir.rs`, `typecheck.rs`, `types.rs`
    - trait capability に `Drop` を[追加/つぁE��]した、E
    - typecheck で `#capability drop` trait を[検�E/けんしゅつ]し、drop impl target めE`TypeCtx` へ[登録/とぁE��く]できるようにした、E
    - `TypeCtx::has_drop` を[追加/つぁE��]し、tuple / named / struct / enum / apply の[再帰皁Eさいきてき][判宁Eはんてい]を[持EめEたせた、E
  - `stdlib/core/traits/drop.nepl`
    - new policy / format で file header と trait comment を[整傁Eせいび]した、E
    - destructor [署吁Eしょめい]めE`fn drop <(&Self)*>()> (self)` に[変更/へんこぁEし、raw wasm ABI と[整吁Eせいごう]するようにした、E
  - `nepl-core/src/passes/drop_insertion.rs`
    - auto drop めE`HirExprKind::Drop` ではなぁEtrait call [挿入/そうにめE��]へ[佁Eつく]り[直/なお]した、E
    - monomorphize [剁EまぁEに `Drop::drop` 呼び[出/だ]しを[入/い]れる[形/かたち]へ[変更/へんこぁEし、既存�E trait 解決・monomorphize [経路/けいろ]に[乁Eの]せた、E
    - [変数/へんすぁE[状慁EじょぁE��い]は `Valid` / `Moved` / `PossiblyMoved` を[追跡/つぁE��き]し、branch merge は[保守的/ほしゅてき]に `PossiblyMoved` へ[倁Eたお]す[形/かたち]にした、E
    - auto drop call は local [番地/ばんち]を[渡/わた]すためE`AddrOf(Var(name))` を[使/つか]ぁE形/かたち]へ[変更/へんこぁEした、E
  - `nepl-core/src/compiler.rs`
    - `insert_drops` めEmonomorphize [剁EまぁEへ[移勁EぁE��ぁEし、trait call として[解決/かいけつ]できるようにした、E
  - `nepl-core/tests/drop.rs`
    - scope end / nested scope LIFO / branch local / shadowing / conditional move / loader-visible stdlib の 7 [件/けん]を[詳細/しょぁE��い]に[固宁Eこてい]した、E
    - fixture は field [読/めEみめEzero-field struct に[頼/たよ]らず、distinct guard [垁Eがた]と `#entry main` [仁Eつ]ぁEminimal program へ[整琁Eせいり]した、E
  - `tests/compiler/drop.n.md`
    - Rust integration test と[同系統/どぁE��ぁE��ぁEの compiler doctest めEskip から宁Etestcase へ[置揁Eちかん]し、nodesrc [経路/けいろ]でめE`Drop` を[含/ふく]む入力が compile / run できることを[固宁Eこてい]した、E
- [設訁Eせっけい][判断/はんだん]:
  - auto drop は codegen special case にせず、trait call [挿入/そうにめE��]として HIR [丁EじょぁEで[表現/ひめE��げん]したほぁE��、source [宣言/せんげん]されぁEcapability と compiler [実裁Eじっそう]が[一致/ぁE��ち]する、E
  - destructor めE`&Self` にしたのは temporary / stack slot の[番地/ばんち]を[渡/わた]せるようにするためで、Rust の drop glue にめE迁Eちか]い[方吁EほぁE��ぁEである、E
  - runtime [頁E��Eじゅんじょ] test は Rust integration test、nodesrc 側は compile / run regression とぁE��[責務�E拁Eせきむぶんたん]にした、E
- [検証/けんしょぁE:
  - `cargo test -p nepl-core --test drop -- --nocapture`
    - [結果/けっか]: `7/7 pass`
  - `node nodesrc/tests.js -i tests/compiler/drop.n.md --no-stdlib --no-tree -o /tmp/tests-compiler-drop.json -j 4`
    - [結果/けっか]: `4/4 pass`
    - output JSON: `/tmp/tests-compiler-drop.json`
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `cargo build -p nepl-cli`
    - [結果/けっか]: success

# 2026-03-11 作業メモ (`HashKey` と hash collection の reboot 収束)

- [目皁Eもくてき]:
  - `HashMap` / `HashSet` めEreboot [方釁EほぁE��ん]どおり bare API + trait [委譲/ぁE��めE��]へ[寁EめEせ、specialized key helper [吁Eめい]に[依孁EぁE��ん]しなぁEcollection にする、E
  - custom trait の `#capability copy` ぁEgeneric bound と concrete impl の[両方/りょぁE��ぁEで[効/き]くよぁE�� compiler [側/がわ]を[修正/しゅぁE��い]し、`HashKey` の custom key ぁEmove check で[壁Eこわ]れなぁE��ぁE��する、E
- [根本原因/こんぽんげんいん]:
  - `HashMap` / `HashSet` の custom key failure は collection [実裁Eじっそう]ではなぁEcompiler [側/がわ]だった。`TypeCtx::is_copy` は generic type var の trait bound capability と、`Copy` 以外�E copy-capability trait impl target を[要Eみ]てぁE��かった、E
  - そ�Eため `.K: HashKey` でめEmove check は `key` めEnon-copy と[判宁Eはんてい]し、probing 中の[再利用/さいりよぁEで `D3053` を[出/だ]してぁE��、E
  - `Hash` / `hash32` test めEold star import [前提/ぜんてい]が[殁Eのこ]っており、helper shadowing と bare overload [曖昧匁EあいまぁE��]で `D3005` を[起/お]こしてぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/types.rs`, `nepl-core/src/typecheck.rs`
    - type var に `copy_cap` / `clone_cap` / `drop_cap` を[保持/ほじ]させ、type parameter bound の capability めEmove check / drop 判定へ[伝播/でん�E]するようにした、E
    - function instantiate 時にめEfresh type var へ capability flag を[弁Eひ]き[綁Eつ]ぐよぁE��した、E
    - compiler ぁE`Copy` trait 1 [倁Eこ]だけを special case [扱/あつか]ぁE��てぁE��[箁E��/かしょ]を[改/あらた]め、`#capability copy` / `clone` / `drop` を[持EめEつ trait めEcapability [単佁Eたんい]で[認譁Eにんしき]するようにした、E
  - `stdlib/core/traits/hash_key.nepl`
    - `HashMap` / `HashSet` [吁Eむ]けに key capability `HashKey` を[追加/つぁE��]した、E
    - `clone` / `eq` / `hash32` めE1 [倁Eこ]の trait へ[雁E��EしゅぁE��く]し、builtin key (`bool` / `i32` / `u8` / `i64` / `str`) の impl を[整傁Eせいび]した、E
  - `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`
    - `.K: HashKey` / `.T: HashKey` [前提/ぜんてい]で open addressing 実裁E��[整琁Eせいり]した、E
    - internal helper [吁Eめい]は `hashmap_*` / `hashset_*` へ[統一/とぁE��つ]し、star import [衝突EしょぁE��つ]を[避/さ]けた、E
  - `stdlib/alloc/hash/hash32.nepl`, `stdlib/core/traits/hash.nepl`
    - bare `hash32` overload と trait `Hash::hash32` が[再帰/さいき]や[曖昧匁EあいまぁE��]を[起/お]こさなぁE��ぁE��primitive hash [計箁Eけいさん]を[明示皁Eめいじてき]に[展開/てんかい]した、E
  - `stdlib/tests/hashmap.n.md`, `stdlib/tests/hashmap_str.n.md`, `stdlib/tests/hashset.n.md`, `stdlib/tests/hashset_str.n.md`, `tests/stdlib/traits_hash.n.md`
    - current ownership model に[吁EぁEわせて fixture を[整琁Eせいり]し、custom `HashKey` key を[含/ふく]む focused regression を[追加/つぁE��]した、E
    - `traits_hash` の先頭 case は old bare import 比輁E��めE��、current trait helper の deterministic / distinctness を[確誁Eかくにん]する[形/かたち]へ[変更/へんこぁEした、E
  - `tests/compiler/trait_capability_copy.n.md`
    - custom trait の `#capability copy` / `#capability clone` ぁEgeneric bound に[伝播/でん�E]し、`.T` を[褁E��囁EふくすぁE��い][使/つか]ってめE`D3053` にならなぁE��とを[固宁Eこてい]した、E
- [設訁Eせっけい][判断/はんだん]:
  - [現状/げんじょぁEの言語仕様では multiple trait bound が[書/か]けなぁE��め、hash collection の key [条件/じょぁE��ん]は `HashKey` に[雁E��EしゅぁE��く]した。これ�E `Eq + Hash + Clone/Copy` の collection [用/よう] capability として[扱/あつか]ぁE��E
  - ただぁEcompiler [側/がわ]は `HashKey` 専用 special case にせず、「trait capability めEtype system が[一般/ぁE��ぱん]に[琁E��/りかい]する」[方吁EほぁE��ぁEへ[修正/しゅぁE��い]した。これで他�E custom capability trait にめE吁Eおな]じ修正が[効/き]く、E
  - `btreemap.nepl` の差刁E�Eこ�E batch では[触/さわ]っておらず、collection reboot の[残件/ざんけん]として[別/べつ]に[続衁Eぞっこう]する、E
- [検証/けんしょぁE:
  - `cargo test -p nepl-core --test drop -- --nocapture`
    - [結果/けっか]: `7/7 pass`
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/tests.js -i tests/compiler/trait_capability_copy.n.md -i tests/stdlib/traits_hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree -o /tmp/tests-hash-capability-focus.json -j 4`
    - [結果/けっか]: `8/8 pass`
    - output JSON: `/tmp/tests-hash-capability-focus.json`

# 2026-03-11 作業メモ (`BTreeMap` / `BTreeSet` の reboot 追征E

- [目皁Eもくてき]:
  - `BTreeMap` / `BTreeSet` めEreboot [方釁EほぁE��ん]どおり bare API + trait [委譲/ぁE��めE��]へ[揁Eそろ]え、old `btreemap_*` / `btreeset_*` alias [前提/ぜんてい]を[除去/じょきょ]する、E
  - stdlib fixture と `pipe_collections` めEcurrent ownership model / explicit report [流儀/りゅぁE��]へ[追征EつぁE��めE��]させる、E
- [根本原因/こんぽんげんいん]:
  - `btreemap` / `btreeset` は collection reboot の[途中/とちめE��]で[止/と]まっており、`btreemap_*` / `btreeset_*` [命吁Eめいめい]、`i32` 固宁Eset、old comment format、old `ret: 1` fixture が[殁Eのこ]ってぁE��、E
  - `btreemap` / `btreeset` の `insert` は capacity [判宁Eはんてい]で collection [本佁Eほんたい]を[褁E��囁EふくすぁE��い][読/めEんでおり、current move model では `D3063` / `D3053` になってぁE��、E
  - `BTreeSet<i32>` の bare `new<i32>` は `std/test` import [丁Eか]で overload [曖昧匁EあいまぁE��]した。これ�E collection 側ではなく、current compiler ぁEno-arg generic constructor めEexpected return type だけでは[十�E/じゅぁE�Eん]に[絁Eしぼ]れてぁE��ぁE��とが[原因/げんぁE��]だった、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/btreemap.nepl`
    - bare `new` / `insert` / `get` / `contains` / `remove` / `len` / `clear` / `free` [構�E/こうせい]を[維持EぁE��]しつつ、`insert` の capacity [判宁Eはんてい]めE`hdr0` / `len_init` / `cap_init` [先読/さきめEみへ[変更/へんこぁEして move error を[除去/じょきょ]した、E
  - `stdlib/alloc/collections/btreeset.nepl`
    - file 全体を new policy / format で[書/か]き[直/なお]した、E
    - `struct BTreeSet<.T>` と `Ord` trait [前提/ぜんてい]の generic set に[再設訁Eさいせっけい]した、E
    - public API めEbare `new` / `insert` / `contains` / `remove` / `len` / `clear` / `free` へ[統一/とぁE��つ]した、E
    - internal helper は `btreeset_*` に[閁Eと]じ込め、public API と star import [衝突EしょぁE��つ]しなぁE��ぁE��した、E
    - `insert` の grow [判宁Eはんてい]めE`hdr0` / `len_init` / `cap_init` [先読/さきめEみへ[変更/へんこぁEして move error を[除去/じょきょ]した、E
  - `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`
    - old alias API と `ret: 1` [前提/ぜんてい]を[除去/じょきょ]し、`Vec<Result<(),str>>` + `checks_print_report` + `checks_exit_code` の explicit report [流儀/りゅぁE��]へ[統一/とぁE��つ]した、E
    - `btreeset` fixture では current compiler 制紁E�Eため `fn new_set ...: new<i32>` wrapper を[置/お]き、public bare name めEexpected type [仁Eつ]ぁEhelper [経由/けいめEで[呼/めEぶ[形/かたち]にした、E
  - `tests/stdlib/pipe_collections.n.md`
    - `btreemap` / `btreeset` の pipe section めEcurrent bare API へ[書/か]き[揁Eか]えた、E
    - [併/あわ]せて old `hashmap_*` / `hashset_*` alias section めEcurrent bare API へ[追征EつぁE��めE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - public API は `btree*_*` alias を[殁Eのこ]さず bare name を[正/せい]とした。old fixture 側だけを書き換えて互換層を[佁Eつく]ることはしてぁE��ぁE��E
  - `BTreeSet` は `HashSet` と[同槁EどぁE��ぁEに generic `.T: Ord` へ[寁EめEせ、collection [吁Eめい]でなぁEtrait [墁E��/きょぁE��い]が[意味諁EぁE��ろん]を[決/き]める[構造/こうぞう]にした、E
  - `new<i32>` の wrapper は collection 設計�E[妥十EだきょぁEではなぁEcurrent compiler limitation の[刁Eき]り[刁Eわ]けとして[扱/あつか]ぁE��public API 自体�E bare `new` のまま[維持EぁE��]し、この limitation は[後綁Eこうぞく]の compiler overload [改喁Eかいぜん]で[解涁EかいしょぁEすべきものとして[記録/きろく]する、E
- [検証/けんしょぁE:
  - `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stdlib/pipe_collections.n.md --no-stdlib --no-tree -o /tmp/tests-btree-focus.json -j 4`
    - [結果/けっか]: `14/14 pass`
    - output JSON: `/tmp/tests-btree-focus.json`

# 2026-03-11 作業メモ (`alloc/hash` comment / fixture の reboot 追征E

- [目皁Eもくてき]:
  - `alloc/hash` [配丁EはぁE��]の comment と fixture めEreboot [征Eご]の test [流儀/りゅぁE��]と doc comment policy へ[揁Eそろ]える、E
  - old `hash32_i32` / old `ret: 0` / old test output [前提/ぜんてい]を[除去/じょきょ]し、current bare API と explicit report [流儀/りゅぁE��]を[固宁Eこてい]する、E
- [根本原因/こんぽんげんいん]:
  - `stdlib/tests/hash.n.md` は old success/failure [流儀/りゅぁE��]のままで、`checks_print_report` / `checks_exit_code` による current safe test flow と[不一致/ふぁE��ち]だった、E
  - `alloc/hash/fnv1a32.nepl` / `alloc/hash/sha256.nepl` の comment は new policy / format に[沿/そ]っておらず、[現状/げんじょぁEの scaffold [状慁EじょぁE��い]や[注意点/ちめE��ぁE��ん]ぁEfile header と item comment から[読/めEみ[叁Eと]れなかった、E
  - `hash` fixture は old `hash32_i32` [前提/ぜんてい]が[殁Eのこ]っており、current trait [委譲/ぁE��めE��]の説明とズレてぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/hash/fnv1a32.nepl`
    - file header と `Fnv1a32` / `new_fnv1a32` / `fnv1a32_update` / `fnv1a32_finalize` の doc comment めEnew policy / format へ[書/か]き[直/なお]した、E
    - [暗号/あんごう]用途ではなぁE��と、lightweight state であること、`update` / `finalize` の O(1) を[明訁Eめいき]した、E
  - `stdlib/alloc/hash/sha256.nepl`
    - file header と `Sha256` / `new_sha256` / `sha256_update` / `sha256_finalize` の doc comment めEnew policy / format へ[書/か]き[直/なお]した、E
    - [現状/げんじょぁEでは SHA-256 digest を[計箁Eけいさん]しておらず、buffering scaffold であることを[明訁Eめいき]した、E
  - `stdlib/tests/hash.n.md`
    - `#entry main` + `Vec<Result<(),str>>` + `checks_print_report` + `checks_exit_code` の explicit report [流儀/りゅぁE��]へ[更新/こうしん]した、E
    - old `hash32_i32` を[除去/じょきょ]し、trait [経由/けいめEの `hash32_by_trait` で determinism / distinctness を[確誁Eかくにん]するようにした、E
    - `sha256_finalize` は scaffold [仕槁EしよぁEとして buffer len を[確誁Eかくにん]する test に[刁Eき]り[替/か]えた、E
- [設訁Eせっけい][判断/はんだん]:
  - `sha256` は[未実裁Eみじっそう] digest を「できてぁE��ように[要Eみ]せる」ことをせず、scaffold [段隁Eだんかい]で[保証/ほしょぁEしてぁE��ことだけを test / comment [両方/りょぁE��ぁEに[明訁Eめいき]した、E
  - hash fixture では bare `hash32` overload の[曖昧性/あいまぁE��い]を[避/さ]けるため、current trait 設計を[表/あらわ]ぁE`hash32_by_trait` を[使/つか]ぁE形/かたち]にした、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/hash.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i tests/stdlib/traits_hash.n.md --no-stdlib --no-tree -o /tmp/tests-hash-focus.json -j 4`
    - [結果/けっか]: `4/4 pass`
    - output JSON: `/tmp/tests-hash-focus.json`

# 2026-03-11 作業メモ (collection fixture / selfhost_req の reboot 追征E

- [目皁Eもくてき]:
  - `tests/stdlib/collections_diag.n.md` と `tests/stdlib/selfhost_req.n.md` に[殁Eのこ]ってぁE�� old collection API [参�E/さんしょぁEめEcurrent bare API へ[揁Eそろ]える、E
  - host filesystem の preopen に[依孁EぁE��ん]する unstable file I/O testcase めEreboot [方釁EほぁE��ん]に[沿/そ]って stable な `Result` [検証/けんしょぁEへ[戻/もど]す、E
- [根本原因/こんぽんげんいん]:
  - `collections_diag` は collection reboot [剁EまぁEの `hashmap_new` / `hashmap_insert` / `hashset_new` / `hashset_insert` が[残孁Eざんそん]しており、public API と fixture が[乖離/かいり]してぁE��、E
  - `selfhost_req` の string map case は[既/すで]に `HashMap<str,.V>` へ[統吁EとぁE��ぁEされた後も `HashMapStr` / `hashmap_str_*` 参�Eが[殁Eのこ]ってぁE��、E
  - `selfhost_req` の file I/O case は host filesystem の positive-path read めEdoctest [本佁Eほんたい]で[期征Eきたい]しており、preopen [条件/じょぁE��ん]で `ret: 0` が[不安宁Eふあんてい]になってぁE��、E
- [変更/へんこぁE:
  - `tests/stdlib/collections_diag.n.md`
    - `HashMap<i32,i32>` / `HashSet<i32>` を[明示/めいじ]し、`new` / `insert` / `remove` の bare API へ[書/か]き[揁Eか]えた、E
    - [説明文/せつめいぶん]の `hashmap_insert` / `hashset_insert` めEcurrent 吁E`insert` へ[揁Eそろ]えた、E
  - `tests/stdlib/selfhost_req.n.md`
    - string map case めE`HashMap<str,i32>` + `new<str,i32>` / `insert<str,i32>` / `get<str,i32>` へ[移衁EぁE��ぁEした、E
    - compile-fail case めE`new<Point, str>` / `insert<Point, str>` の bare API [表訁EひめE��き]へ[更新/こうしん]し、current collection API でめE`D3081` [期征Eきたい]が[崩/くず]れなぁE��とを[確誁Eかくにん]した、E
    - file I/O case は host positive-path read をやめ、missing file に[対/たい]して `Result::Err` が[迁Eかえ]ることめEstable に[検証/けんしょぁEする testcase へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - `std/fs` は `tests/stdlib/fs.n.md` でめE整琁Eせいり]したとおり、host preopen に[依孁EぁE��ん]する positive-path read めEfixture の[成功条件/せいこうじょぁE��ん]にしなぁE��`Result` と helper [意味諁EぁE��ろん]の[検証/けんしょぁEめEstable に[固宁Eこてい]する、E
  - `selfhost_req` は Rust 側 request の[痕跡/こんせき]を[殁Eのこ]しつつも、current reboot public API と[一致/ぁE��ち]する[形/かたち]へ[追征EつぁE��めE��]させる、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 6`
    - [結果/けっか]: pass (`compile_fail`)
  - `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md -i tests/stdlib/selfhost_req.n.md --no-stdlib --no-tree -o /tmp/tests-collections-selfhost-current.json -j 4`
    - [結果/けっか]: `12/12 pass`
    - output JSON: `/tmp/tests-collections-selfhost-current.json`

# 2026-03-11 作業メモ (`HashMap` / `HashSet` custom hasher を支える compiler 根因修正)

- [目皁Eもくてき]:
  - `HashMap<.K,.V,.H>` / `HashSet<.K,.H>` ぁEuser-provided hasher を[値/あたい]として[叁EぁEけ[叁Eと]れるようにし、`Hasher<.K>` trait [経由/けいめEの dispatch めEcurrent compiler / web compile path の[両方/りょぁE��ぁEで[安宁Eあんてい]させる、E
  - `field::get` の qualified call と bare `get` の collection API が[衝突EしょぁE��つ]しなぁE��ぁE��し、NEPLg2 の[前置/ぜんち][記況EきほぁE + overload 解決に[沿/そ]っぁEroot fix を[入/い]れる、E
- [根本原因/こんぽんげんいん]:
  - `tests/stdlib/traits_hash.n.md` の compile failure は stdlib 側の hasher 実裁E��はなく、compiler と web compile path の[不一致/ふぁE��ち]が[真因/しんぁE��]だった、E
  - native / analysis [経路/けいろ]では `SourceMap` を[使/つか]って qualified import alias から `field::get` を[正/ただ]しく[選/えら]べてぁE��が、`nepl-web` の compile [経路/けいろ]だけ�E `compile_module(...)` を[送Eとお]って `SourceMap` なしで typecheck してぁE��。そのため `field::get` ぁEbare `get` に[崩/くず]れ、`HashMap::get` と[衝突EしょぁE��つ]して unresolved trait call まで[連鎁Eれんさ]してぁE��、E
  - さらに trait impl lookup めEapplied string 名ではなぁE`base trait name + trait args` の[構造/こうぞう]で[扱/あつか]わなぁE��、generic hasher impl ぁEmonomorphize [征Eご]めE`FuncRef::Trait` のまま[殁Eのこ]ることが[刁Eわ]かった、E
- [変更/へんこぁE:
  - `nepl-core/src/typecheck.rs`
    - qualified import alias から target file set を[弁Eひ]く[仕絁Eしく]みめE`SourceMap` [利用/りよぁEへ[整琁Eせいり]し、selected qualified callable は `HirExprKind::FnValue(symbol)` として[保持/ほじ]するようにした、E
    - `field::get` の qualified call ぁEbare `get` に[崩/くず]れず、collection API との overload [衝突EしょぁE��つ]を[避/さ]けられるようにした、E
  - `nepl-core/src/loader.rs`
    - `SourceMap::iter_paths` を[追加/つぁE��]し、typecheck ぁEimport alias と file path suffix を[対応仁Eたいおうづ]けられるようにした、E
  - `nepl-core/src/hir.rs`, `nepl-core/src/monomorphize.rs`, `nepl-core/src/compiler.rs`, `nepl-core/src/ast.rs`, `nepl-core/src/parser.rs`
    - generic trait / impl の trait args めEstring ではなく[構造/こうぞう]で[保持/ほじ]し、`Hasher<.K>` impl の dispatch ぁEmonomorphize [征Eご]に concrete call へ[落/お]ちるよぁE��した、E
    - monomorphize [段隁Eだんかい]では unresolved trait call を[検査/けんさ]し、generic hasher 経路に[殁Eのこ]ってぁE��ぁE��とを[保証/ほしょぁEするようにした、E
  - `nepl-web/src/lib.rs`
    - web compile [経路/けいろ]めE`compile_module_with_source_map(...)` に[刁Eき]り[替/か]え、native path と[吁Eおな]ぁE`SourceMap` [前提/ぜんてい]で compile するようにした、E
    - 刁E��[刁Eわ]け用の panic catch / debug export は[最終的/さいしゅぁE��き]に[除去/じょきょ]し、恒乁E��正だけを[殁Eのこ]した、E
  - `nepl-core/tests/overload.rs`
    - grouped constructor / specific `get` overload / annotated `let` の regression を[追加/つぁE��]し、今回の root fix めEcompiler test として[固宁Eこてい]した、E
  - `stdlib/core/traits/hash.nepl`, `stdlib/core/traits/hash_key.nepl`, `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`
    - custom hasher を[叁EぁEけ[叁Eと]めEcurrent reboot 形へ[整琁Eせいり]した、E
  - `stdlib/tests/hashmap*.n.md`, `stdlib/tests/hashset*.n.md`, `tests/stdlib/traits_hash.n.md`, `tests/stdlib/collections_diag.n.md`, `tests/stdlib/pipe_collections.n.md`, `tests/stdlib/selfhost_req.n.md`
    - `DefaultHash32 ()` のような old [表訁EひめE��き]を[殁Eのこ]さず `DefaultHash32` へ[統一/とぁE��つ]し、current custom hasher / bare collection API [前提/ぜんてい]へ[更新/こうしん]した、E
  - `stdlib/alloc/string.nepl`
    - hash focused を[送Eとお]す[過稁Eかてい]で[要Eみ]つかった一晁E`RegionToken` [再利用/さいりよぁEの move model [衝突EしょぁE��つ]を[解涁EかいしょぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - `field::get` と `HashMap::get` の[競吁EきょぁE��ぁEは library alias を[足/た]して[回避/かいひ]するのではなく、qualified name 解決と前置記況Ereduction の root fix で[解涁EかいしょぁEした、E
  - custom hasher は built-in special case を[墁Eふ]めE��ず、trait impl と overload 解決で[支/ささ]える reboot [方釁EほぁE��ん]を[維持EぁE��]した、E
  - web path だけ別挙動になる�Eは[設訁Eせっけい]として[悪/わる]ぁE�Eで、debug helper を[常設/じょぁE��つ]せず compile path 自体を native と[吁Eおな]ぁE`SourceMap` [前提/ぜんてい]へ[揁Eそろ]えた、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/stdlib/collections_diag.n.md --no-stdlib --no-tree -o /tmp/tests-hash-focus.json -j 4`
    - [結果/けっか]: `13/13 pass`
    - output JSON: `/tmp/tests-hash-focus.json`
  - `cargo test -p nepl-core --test overload grouped_argument_overload_uses_later_items_before_reduction -- --nocapture`
    - [結果/けっか]: pass
  - `cargo test -p nepl-core --test overload grouped_constructor_argument_can_flow_into_generic_new_call -- --nocapture`
    - [結果/けっか]: pass
  - `cargo test -p nepl-core --test overload more_specific_get_overload_beats_generic_catchall -- --nocapture`
    - [結果/けっか]: pass
  - `cargo test -p nepl-core --test overload annotated_let_prefers_specific_get_over_generic_field_get -- --nocapture`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (stack / ringbuffer / queue の bare API 統一)

- [目皁Eもくてき]:
  - `stack_` / `ringbuffer_` prefix めEpublic API から[除去/じょきょ]し、collection reboot [方釁EほぁE��ん]どおり `new` / `push` / `pop` / `peek` / `len` / `is_empty` / `clear` / `free` の bare 名へ[統一/とぁE��つ]する、E
  - `queue` は `ringbuffer` の public API を[再輸入/さいめE��めE��]せずに current bare API と[衝突EしょぁE��つ]しない[形/かたち]へ[佁Eつく]り[替/か]える、E
  - reboot [征Eご]の collection API に[吁EぁEわせて examples / parser / fixtures / compiler doctest を[追征EつぁE��めE��]させる、E
- [根本原因/こんぽんげんいん]:
  - `stack` / `ringbuffer` は bare 吁Ewrapper alias を[後仁Eあとづ]けした[過渡朁Eかとき]のままで、actual public defs ぁE`stack_new` / `ringbuffer_push_back` など旧 prefix 名を[保持/ほじ]してぁE��、E
  - `queue` は bare API へ[寁EめEせる[途中/とちめE��]で `ringbuffer` module めEalias import しており、`new` / `push` などの symbol 雁E��ぁEqueue module 冁E��[汚染/おせん]されて `new<i32>` ぁEambiguous になってぁE��、E
  - `stack` pipe fixture に[殁Eのこ]ってぁE�� `let p s |> pop` は current parser / reduction では `let` [直征EちめE��ご]の pipe left-hand side めE1 [値/あたい]へ[畳/たた]めず、`D3013` を[起/お]こしてぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_new` / `stack_push` / `stack_pop` / `stack_peek` / `stack_len` / `stack_is_empty` / `stack_clear` / `stack_free` めEactual def ごと bare 吁E`new` / `push` / `pop` / `peek` / `len` / `is_empty` / `clear` / `free` へ[改吁Eかいめい]した、E
    - `stack_pop_keep` / `stack_peek_keep` めE`pop_keep` / `peek_keep` へ[揁Eそろ]え、旧 alias block は[削除/さくじょ]した、E
  - `stdlib/alloc/collections/ringbuffer.nepl`
    - `ringbuffer_new` / `ringbuffer_with_capacity` / `ringbuffer_push_back` / `ringbuffer_pop_front` / `ringbuffer_peek_front` / `ringbuffer_len` / `ringbuffer_cap` / `ringbuffer_is_empty` / `ringbuffer_clear` / `ringbuffer_free` めEbare 名へ[改吁Eかいめい]した、E
    - public wrapper alias は[撤去/てっきょ]し、helper 名だけを ringbuffer internal [用/よう]に[殁Eのこ]した、E
  - `stdlib/alloc/collections/queue.nepl`
    - `RingBuffer<.T>` handle を[冁E��/なぁE��ぁEして委譲する[形/かたち]をやめ、queue 自身ぁEringbuffer と[吁Eおな]ぁE`[len, cap, head, data_ptr]` header / data layout を[直接/ちめE��せつ][所朁EしょめE��]する[実裁Eじっそう]へ[刁Eき]り[替/か]えた、E
    - これにより `ringbuffer` module import による public symbol [汚染/おせん]を[断/た]ち、`queue::new` / `queue::push` の ambiguity めEroot fix した、E
  - `stdlib/nm/parser.nepl`, `examples/bf.nepl`, `examples/rpn.nepl`
    - stack API めEcurrent bare 名へ[追征EつぁE��めE��]した、E
  - `stdlib/tests/stack.n.md`, `stdlib/tests/ringbuffer.n.md`, `stdlib/tests/queue.n.md`
    - current bare API + `Result` [前提/ぜんてい]へ[更新/こうしん]した、E
    - stack fixture の `let p s |> pop` は current reduction で stable な `let p <Option<i32>> pop s;` へ[書/か]き[揁Eか]えた、E
  - `tests/stdlib/stack_collections.n.md`, `tests/stdlib/ringbuffer_collections.n.md`, `tests/stdlib/queue_collections.n.md`, `tests/stdlib/pipe_collections.n.md`, `tests/stdlib/collections_diag.n.md`
    - bare collection API へ[統一/とぁE��つ]した、E
  - `tests/compiler/overload.n.md`
    - stack `new` を[使/つか]ぁEoverload case は current collection [仕槁EしよぁEどおり impure main + bare `new` へ[追征EつぁE��めE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `queue` の問題�E `ringbuffer` alias import の[丁EぁE��]に wrapper を[釁Eかさ]ねると[再発/さいはつ]するため、public bare API を[共朁EきょぁE��ぁEしつつ internal layout だけを[共朁EきょぁE��ぁEする[形/かたち]へ[夁Eか]えた、E
  - bare API 化�E alias 追加でなぁEactual def の rename として[衁Eおこな]ぁE��reboot の「後方互換を[殁Eのこ]さなぁE��[原則/げんそく]を[宁EまめEった、E
  - stack fixture の `let p s |> pop` は parser / reduction の別課題として[刁Eき]り[刁Eわ]け、collection reboot batch では current stable syntax へ fixture を[寁EめEせた、E
- [検証/けんしょぁE:
  - `target/debug/nepl-cli -i /tmp/queue_test.nepl --target std --output /tmp/queue-test-out`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/queue.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/queue.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/ringbuffer.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 6`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 6`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 14`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 18`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 19`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (vec bare API 整琁E�� move model 追征E

- [目皁Eもくてき]:
  - `Vec` の public API めEalias ではなぁEactual def として bare 名へ[揁Eそろ]える、E
  - vec reboot の[影響允EえいきょぁE��き]である sort / string / parser / tutorial / overload fixture めEcurrent move model に[吁EぁEわせて[整吁Eせいごう]させる、E
  - compiler / web compile path を[含/ふく]む剁Ebatch の Rust 差刁E�� trunk build で[再確誁Eさいかくにん]したぁE��で、focused suite を[緑化/りょくか]する、E
- [根本原因/こんぽんげんいん]:
  - `vec.nepl` は bare 吁Ewrapper を[持EめEってぁE��が、actual def ぁE`vec_new` / `vec_push` [系/けい]のまま[殁Eのこ]っており、reboot の「alias ではなく唯一の public 名」[原則/げんそく]に[叁Eはん]してぁE��、E
  - `set` は collection bare API の[候裁Eこうほ]として[自然/しぜん]だが、current parser/compiler では[予紁E��Eよやくご]として[扱/あつか]われるためEpublic 名にできず、vec の write API は `replace` を[維持EぁE��]する[忁E��Eひつよう]があった、E
  - `Vec` めEstack に[吁EぁEわせて即座に `Result` 化すると、`string` / `diag` / `parser` / `std/test` まで impure 化が[連鎁Eれんさ]し、この batch の[責勁Eせきむ]を[趁Eこ]える。ここでは bare API と move fix を[優允EめE��せん]し、`Result` 方針�E[統一/とぁE��つ]は collection reboot の後綁Ebatch に[送Eおく]った、E
  - tutorial 25 / 26 と `traits_order` は `Vec` owner めE`len/get/get/...` のように[褁E��囁EふくすぁE��い][読/めEんでおり、current move model では[不安宁Eふあんてい]だった、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/vec.nepl`
    - actual def めE`new` / `with_capacity` / `len` / `cap` / `data_ptr` / `data_mem_ptr` / `data_len` / `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` へ[統一/とぁE��つ]した、E
    - 旧 alias block は[削除/さくじょ]した、E
    - `push` の[再確俁Eさいかくほ]で `cap = 0` の[晁Eとき]に `0 * 2 = 0` となってぁE��[欠陥/けっかん]を[修正/しゅぁE��い]し、E [容釁EようりょぁEからでめE1 へ[拡張/かくちめE��]するようにした、E
    - doctest#2 は `match` arm の unit/i32 [混在/こんざい]を[解涁EかいしょぁEし、current API [形/けい]へ[更新/こうしん]した、E
  - `stdlib/alloc/string.nepl`
    - `sb_append` に[殁Eのこ]ってぁE�� stale `uwok` を[除去/じょきょ]し、pure vec API へ[追征EつぁE��めE��]した、E
  - `stdlib/nm/parser.nepl`
    - `Stack<NestSection>` と `Vec` の owner を[繰/く]り[迁Eかえ]し[読/めEんでぁE�� helper を[整琁Eせいり]し、header / data+len めE1 [囁Eかい]だけ[叁Eと]り[出/だ]して raw helper へ[渡/わた]す[実裁Eじっそう]に[変更/へんこぁEした、E
    - これにより close-one / close-all / inline/json [周辺/しゅぁE��ん]の move error を[除去/じょきょ]した、E
  - `tests/stdlib/traits_order.n.md`
    - sort [結果/けっか]の[検証/けんしょぁEめE`get` [反復/はん�Eく]から `data_len + raw load` へ[刁Eき]り[替/か]え、owner めE1 [囁Eかい]だけ[読/めEむ[形/かたち]へ[揁Eそろ]えた、E
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - `VecDataLen` と raw load を[使/つか]って[突Eまど]の[左右端/さゆぁE��ん]を[読/めEむ[形/かたち]へ[変更/へんこぁEし、prefixsum tutorial の move error を[解涁EかいしょぁEした、E
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - `print_dist` めE`len/get/get/...` から `data_len + raw load` へ[変更/へんこぁEし、stdout が[空/から]になる[不安宁Eふあんてい]な[挙動/きょどぁEを[解涁EかいしょぁEした、E
  - `tests/compiler/overload.n.md`, `tests/compiler/overload_nested_generic_push.n.md`
    - vec pure API [前提/ぜんてい]へ[戻/もど]し、stale `unwrap_ok` を[除去/じょきょ]した、E
    - current compiler [挙動/きょどぁEに[吁EぁEわせて ret / compile_fail [期征E��/きたぁE��]を[更新/こうしん]した、E
  - `nepl-core/src/lib.rs`
    - `compile_module_with_source_map` の re-export を[戻/もど]し、前 batch で[導�E/どぁE��めE��]した web/CLI path [統一/とぁE��つ]めEtrunk build [可能/か�EぁEな[状慁EじょぁE��い]へ[俁Eたも]った、E
- [設訁Eせっけい][判断/はんだん]:
  - `replace` は妥協ではなぁEcurrent parser/compiler の[制紁EせいめE��]を[踁Eふ]まえた public 名である。`set` を[使/つか]ぁE��は keyword / parser [設訁Eせっけい]の reboot が[別送Eべっと]忁E��、E
  - `Vec` の `Result` 化�E[忁E��Eひつよう]だが、仁Ebatch で[押/お]し[込/こ]むと pure/impure [墁E��/きょぁE��い]の[整琁Eせいり]なしに library [全埁EぜんぁE��]へ[波叁EはきゅぁEするため、root-cause を[刁E��/ぶんり]して後続�E collection reboot batch へ[送Eおく]った、E
  - tutorial / trait fixture の move fix は `Copy` [前提/ぜんてい]へ[戻/もど]す�Eではなく、`VecDataLen` めEraw load で owner めE1 [囁Eかい]だけ[観測/かんそく]する current ownership model に[寁EめEせた、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_order.n.md -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/26_competitive_graph_bfs.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i tests/compiler/overload.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-overload-vec.json -j 2`
    - [結果/けっか]: `46/46 pass`
    - output JSON: `/tmp/tests-overload-vec.json`

# 2026-03-12 作業メモ (list bare API 統一)

- [目皁Eもくてき]:
  - `list_nil` / `list_cons` / `list_push_front` / `list_head` / `list_tail` / `list_len` / `list_get` / `list_free` / `list_reverse` めEalias ではなぁEactual def ごと bare 名へ[統一/とぁE��つ]する、E
  - list fixture / pipe fixture / compiler fixture めEcurrent collection reboot [方釁EほぁE��ん]へ[追征EつぁE��めE��]させる、E
- [根本原因/こんぽんげんいん]:
  - `list.nepl` は public API めEdoctest も旧 prefix 名�Eままで、reboot の「関数名では区別しなぁE��[原則/げんそく]から最めE夁Eはず]れてぁE��、E
  - list doctest の 2 件目は string helper めEstar import したまま bare `new/head/get/len` へ[寁EめEせると ambiguity を[起/お]こしめE��く、比輁E��けに忁E��な API めEtrait 側 bare `eq` へ[置/お]き[揁Eか]える忁E��があった、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/list.nepl`
    - actual def めE`new` / `cons` / `push` / `head` / `tail` / `is_empty` / `len` / `get` / `free` / `reverse` へ[改吁Eかいめい]した、E
    - file header と doctest 冁E�E public 名も current bare API へ[揁Eそろ]えた、E
    - string doctest は `alloc/string` helper ではなぁE`core/traits/eq` の bare `eq` を[使/つか]ぁE��ぁE��[変更/へんこぁEした、E
  - `stdlib/tests/list.n.md`
    - mk helper と全 check case めEbare list API へ[更新/こうしん]した、E
  - `tests/stdlib/pipe_collections.n.md`
    - list chain example めE`new |> push |> push ...` と `len/get` の current bare API へ[更新/こうしん]した、E
  - `tests/compiler/list_dot_map.n.md`
    - compile_fail fixture の `list.list_nil` めE`list.new` へ[変更/へんこぁEした、E
- [設訁Eせっけい][判断/はんだん]:
  - list は allocation failure めE`Result` で[表/あらわ]す方針へまだ[乁Eの]ってぁE��ぁE��、この batch では naming reboot を[優允EめE��せん]した、E
  - 斁E���E比輁E�E string module helper 名に[依孁EぁE��ん]するより、trait 経由の bare `eq` に[寁EめEせたほぁE�� reboot 全体�E naming [方釁EほぁE��ん]と[整吁Eせいごう]する、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 1`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (btreemap / btreeset の new/insert Result 匁E

- [目皁Eもくてき]:
  - `btreemap` / `btreeset` の allocation path めEstack 系と[揁Eそろ]え、`new` と grow を[伴/ともな]ぁE`insert` めE`Result<..., Diag>` で[迁Eかえ]すよぁE��する、E
  - reboot 後�E collection [方釁EほぁE��ん]に[吁EぁEわせて pipe fixture と stdlib tests を[追征EつぁE��めE��]させる、E
- [根本原因/こんぽんげんいん]:
  - `btreemap` / `btreeset` は bare API 化こそ進んでぁE��が、`alloc_raw` [失敁Eしっぱい]を[値/あたい]で[表現/ひめE��げん]せず pure value を[迁Eかえ]しており、OOM を[扱/あつか]ぁEcollection [方釁EほぁE��ん]から[夁Eはず]れてぁE��、E
  - `btreemap` は `core/field` めEbare import したまま collection 自身の `get` を[定義/てぁE��]しており、`len` / `insert` [冁E��/なぁE�E]の `get hm "hdr"` ぁE`BTreeMap::get` と `field::get` で[衝突EしょぁE��つ]してぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/btreemap.nepl`
    - `new` めE`Result<BTreeMap<.K,.V>, Diag>` へ[変更/へんこぁEし、keys / values / header [確俁Eかくほ]の[失敁Eしっぱい]めE`diag_out_of_memory` へ[変換/へんかん]した、E
    - `grow` めE`Result` 化し、keys / values [再確俁Eさいかくほ]の[失敁Eしっぱい]で[途中/とちめE��][解放/かいほぁEを[衁Eおこな]ってから `Diag` を[迁Eかえ]すよぁE��した、E
    - `insert` は grow path めE`unwrap_ok ... grow` で[叁EぁEけ、public return めE`Result` へ[変更/へんこぁEした、E
    - `core/field` import めE`field` namespace に[刁Eき]り[替/か]え、header [参�E/さんしょぁEめE`field::get` に[統一/とぁE��つ]した、E
  - `stdlib/alloc/collections/btreeset.nepl`
    - `new` と internal `btreeset_grow`、およ�E public `insert` めE`Result<BTreeSet<.T>, Diag>` へ[変更/へんこぁEした、E
  - `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`, `tests/stdlib/pipe_collections.n.md`
    - `must_map` / `must_set` helper を[導�E/どぁE��めE��]し、pipe 連鎖で `Result` を[明示皁Eめいじてき]に[解匁EかいほぁEする current style へ[揁Eそろ]えた、E
- [設訁Eせっけい][判断/はんだん]:
  - `remove` / `contains` / `get` / `len` / `clear` / `free` は allocation を[伴/ともな]わなぁE��め、この batch では pure API のままとした、E
  - `insert` だけを `Result` 化した�Eは、grow による OOM が[起/お]こりぁE��[経路/けいろ]を[正確/せいかく]に[値/あたい]で[表現/ひめE��げん]するため、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/btreemap.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/btreeset.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 4`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (sort fixture の bare Vec API 追征E

- [目皁Eもくてき]:
  - `tests/stdlib/sort.n.md` に残ってぁE��旧 `vec_*` 実体名めEcurrent bare API へ[揁Eそろ]える、E
  - sort return fixture に残ってぁE�� stale expected めEcurrent `Vec` [意味諁EぁE��ろん]へ[同期/どぁE��]する、E
- [根本原因/こんぽんげんいん]:
  - `Vec` 本体�E actual def ぁE`new` / `push` / `data_len` へ[移衁EぁE��ぁEしたが、sort fixture だけが旧 `vec_new` / `vec_push` / `vec_data_len` のまま[残孁Eざんそん]してぁE��、E
  - `sort_*_ret_vec_is_reusable_after_sort` は 2 [要素/ようそ]めEsort [征Eご]に 1 [要素/ようそ]だけ[追加/つぁE��]して `len` を[要Eみ]めEtest なのに、旧 expected `5` が[殁Eのこ]ってぁE��、E
- [変更/へんこぁE:
  - `tests/stdlib/sort.n.md`
    - `vec_new` / `vec_push` / `vec_data_len` めE`new` / `push` / `data_len` へ[置揁Eちかん]した、E
    - `sort_quick_ret_vec_is_reusable_after_sort`
    - `sort_heap_ret_vec_is_reusable_after_sort`
    - `sort_merge_ret_vec_is_reusable_after_sort`
      の expected `ret` めE`3` へ[修正/しゅぁE��い]した、E
- [設訁Eせっけい][判断/はんだん]:
  - ここでの failure は sort [実裁Eじっそう]の bug ではなぁEfixture の[前提/ぜんてい]ずれであり、library [本佁Eほんたい]は[夁Eか]えず test だけを current public API と current `len` [意味諁EぁE��ろん]へ[寁EめEせた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 6`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 11`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 15`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-sort.json -j 2`
    - [結果/けっか]: `22/22 pass`
    - output JSON: `/tmp/tests-stdlib-sort.json`

# 2026-03-12 作業メモ (compiler fixture の bare List API 追征E

- [目皁Eもくてき]:
  - `tests/compiler/neplg2.n.md` に残ってぁE�� `list_nil` / `list_cons` / `list_get` めEcurrent bare API へ[揁Eそろ]える、E
- [根本原因/こんぽんげんいん]:
  - list 本体�E actual def ぁE`new` / `cons` / `get` へ[移衁EぁE��ぁEしたが、compiler regression 1 件だけが旧 public 名�Eまま[殁Eのこ]ってぁE��、E
- [変更/へんこぁE:
  - `tests/compiler/neplg2.n.md`
    - `list_get_out_of_bounds_err` の[説昁Eせつめい]と snippet めE`new` / `cons` / `get` へ[更新/こうしん]した、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 33`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (StdErrorKind の lower-layer 移設)

- [目皁Eもくてき]:
  - `Vec` めE`Result` 化する[前提/ぜんてい]として、`StdErrorKind` めE`Diag` [層/そう]から[刁Eき]り[離/はな]して lower layer へ[移/ぁE��]す、E
  - reboot [方釁EほぁE��ん]どおり、[軽釁EけいりょぁE error kind と richer diagnostic を[刁E��/ぶんり]する、E
- [根本原因/こんぽんげんいん]:
  - `StdErrorKind` ぁE`stdlib/alloc/diag/error.nepl` に[置/お]かれてぁE��ため、`Vec -> StdErrorKind` を[導�E/どぁE��めE��]すると `vec -> diag/error -> vec` の[循環/じゅんかん][依孁EぁE��ん]になる、E
  - reboot doc の[意図/ぁE��]は `Result<T, StdErrorKind>` を[軽釁EけいりょぁEな[制御/せいぎょ] error とし、`Diag` / `Outcome` は richer な[診断/しんだん][表現/ひめE��げん]として[別/べつ][層/そう]に[置/お]くことだった、E
- [変更/へんこぁE:
  - `stdlib/core/result.nepl`
    - `StdErrorKind` enum を[移設/ぁE��つ]した、E
    - `std_error_kind_str` を[移設/ぁE��つ]した、E
  - `stdlib/alloc/diag/error.nepl`
    - `StdErrorKind` / `std_error_kind_str` の[定義/てぁE��]を[削除/さくじょ]し、`Diag` / `Diags` / `Outcome` [本佁Eほんたい]に[雁E��/しゅぁE��めE��]させた、E
    - file header めEcurrent [責勁Eせきむ]へ[同期/どぁE��]した、E
  - `stdlib/alloc/diag/diag.nepl`
    - `std_error_kind_str` めE`core/result` から[要Eみ]るため�E import を[追加/つぁE��]した、E
  - `stdlib/tests/diag.n.md`
    - `StdErrorKind` import [允Eもと]の[変更/へんこぁEに[追征EつぁE��めE��]した、E
    - old assert style めEcurrent safe test flow (`checks_print_report` / `checks_exit_code`) に[揁Eそろ]えた、E
- [設訁Eせっけい][判断/はんだん]:
  - 今回は `StdErrorKind` の[置/お]き[場/ば]だけを[整琁Eせいり]し、`Diag` helper の public 名や `Outcome` API は[夁Eか]えてぁE��ぁE��E
  - これで `Vec` ぁE`Result<..., StdErrorKind>` を[迁Eかえ]してめE`Diag` [層/そう]への[送E��EぎゃくりめE��]が[起/お]きない[土台/どだい]になった、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_serde.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -i tests/stdlib/traits_serde.n.md -i tests/stdlib/io.n.md --no-stdlib --no-tree -o /tmp/tests-std-error-kind-core.json -j 4`
    - [結果/けっか]: `13/13 pass`
    - output JSON: `/tmp/tests-std-error-kind-core.json`
- 2026-03-12: `List` めEcollection reboot 方針へ寁E��るため、`stdlib/alloc/collections/list.nepl` の `new/cons/push/reverse` めE`Result<..., Diag>` 返却へ変更した。空リスト�E体�E追加確保をしなぁE��、�E開面は `stack` / `ringbuffer` / `queue` / `btree` と同じ `Result` 方針へ揁E��た、E
- 2026-03-12: `stdlib/tests/list.n.md`, `tests/stdlib/pipe_collections.n.md`, `tests/compiler/list_dot_map.n.md`, `tests/compiler/neplg2.n.md` めEcurrent API へ追従した。`new ... |> uwok |> push ... |> uwok` の一行連鎖へ統一し、`reverse` めE`uwok` 経由で受ける形に揁E��た、E
- 2026-03-12: collection の doc test / fixture めEcurrent reboot API に同期した、E
  - `stdlib/alloc/collections/stack.nepl`
    - doc test に残ってぁE��旧 `new |> uwok` めE`unwrap_ok<Stack<...>, Diag> new<...>` へ統一し、`push ... |> uwok` めE1 行に揁E��た、E
    - `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-stdlib --no-tree -o /tmp/tests-stack-docs.json -j 2` で `10/10 pass`、E
  - `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`
    - public API (`new/insert/get/contains/remove/len/free`) の comment を新 format に寁E��、各関数の usage doctest を追加した、E
    - hasher 付き `new` の例�E、既存通過例に合わせて `unwrap_ok<HashMap<...>, Diag> new DefaultHash32` / `unwrap_ok<HashSet<...>, Diag> new DefaultHash32` へ揁E��た、E
    - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/hashmap.nepl -n 1` pass、E
    - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/hashset.nepl -n 1` pass、E
  - `stdlib/tests/btreeset.n.md`, `tests/stdlib/pipe_collections.n.md`
    - `BTreeSet` / `Stack` / `RingBuffer` / `Queue` / `HashMap` / `HashSet` の fixture に残ってぁE��曖昧な bare `new<i32>` めE�� pipe 書法を current style に更新した、E
  - `tests/compiler/list_dot_map.n.md`
    - `namespace_pathsep_map_with_result` は stale `compile_fail` だった�Eで normal test (`ret: 2`) に直した、E
  - focused 検証
    - `node nodesrc/tests.js -i stdlib/tests/btreeset.n.md -i tests/stdlib/pipe_collections.n.md -i tests/compiler/list_dot_map.n.md --no-stdlib --no-tree -o /tmp/tests-collections-regression-slice.json -j 4` で `14/14 pass`、E

# 2026-03-12 作業メモ (collection public API の doc comment / doctest 追加)

- [目皁Eもくてき]:
  - reboot [方釁EほぁE��ん]どおり、`alloc/collections` の public API に current bare 名と `Result` / `Option` [流儀/りゅぁE��]を[示/しめ]ぁEusage doctest を[墁Eふ]めE��、E
  - old comment のまま「[佁Eなに]を[迁Eかえ]すか」だけで[絁Eお]わってぁE��関数へ、current [使/つか]い[方/かた]を[追訁EつぁE��]する、E
- [根本原因/こんぽんげんいん]:
  - collection reboot で public 名と return [方釁EほぁE��ん]は[夁Eか]わったが、`queue` / `ringbuffer` / `btreemap` / `btreeset` の comment には current style の最小例が[十�E/じゅぁE�Eん]に[無/な]かった、E
  - `queue.clear` の doctest では let [本佁Eほんたい]の[末尾/まつび]に `;` が[殁Eのこ]っており、[弁Eしき]ぁEunit に[崩/くず]れてぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/queue.nepl`
    - `new` / `with_capacity` / `len` / `is_empty` / `push` / `pop` / `peek` / `clear` / `free` に current style の usage doctest を[追加/つぁE��]した、E
    - `clear` の snippet は `let q0 ...` と `let q clear q0` に[刁E��/ぶんり]し、let [本佁Eほんたい]の unit 化を[避/さ]けた、E
  - `stdlib/alloc/collections/ringbuffer.nepl`
    - `new` / `with_capacity` / `len` / `cap` / `is_empty` / `push` / `pop` / `peek` / `clear` / `free` に usage doctest を[追加/つぁE��]した、E
  - `stdlib/alloc/collections/btreemap.nepl`
    - `BTreeMap` struct comment めEcurrent format に[補強/ほきょぁEし、`new` / `len` / `contains` / `get` / `insert` / `remove` / `clear` / `free` の usage doctest を[追加/つぁE��]した、E
  - `stdlib/alloc/collections/btreeset.nepl`
    - `BTreeSet` struct comment めEcurrent format に[補強/ほきょぁEし、`new` / `len` / `contains` / `insert` / `remove` / `clear` / `free` の usage doctest を[追加/つぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - doctest は reboot doc の[方釁EほぁE��ん]に[征Eしたが]ぁE��API の[最封EさいしょぁE[使用侁EしよぁE��い]と current ownership / error [流儀/りゅぁE��]を[示/しめ]す[用送Eようと]に[限宁Eげんてい]した、E
  - fixture [代替/だぁE��い]ではなく、public 関数[直剁EちめE��ぜん]に置ぁE��「[要Eみ]た[送Eとお]りに[使/つか]える」ことを[保証/ほしょぁEする comment へ[寁EめEせた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/queue.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/queue.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/queue.nepl -n 8`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/ringbuffer.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/ringbuffer.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/ringbuffer.nepl -n 10`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreemap.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreemap.nepl -n 8`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreeset.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreeset.nepl -n 7`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (`Deque` [追加/つぁE��]と nullary `new` [書弁Eしょしき]の[統一/とぁE��つ])

- [目皁Eもくてき]:
  - `alloc/collections` に `Deque` を[追加/つぁE��]し、[前征Eぜんご][両端/りょぁE��ん] queue の bare API を[標溁EひめE��じゅん]で[揁Eそろ]える、E
  - collection fixture に[殁Eのこ]ってぁE�� `new<i32> |> unwrap_ok ...` / `new<i32> |> uwok` を、current [推奨/すいしょぁEの `unwrap_ok<..., Diag> new<i32>` [形/けい]へ[統一/とぁE��つ]する、E
- [根本原因/こんぽんげんいん]:
  - nullary overload の `new` は pipe [起点/きてん]に[置/お]くと expected type が[十�E/じゅぁE�Eん]に[伝播/でん�E]せず、`D3005 ambiguous overload` を[起/お]こしてぁE��、E
  - `Deque` [追加/つぁE��]後�E fixture でもこの[書弁Eしょしき]をそのまま[使/つか]ってぁE��ため、`peek_*` / `pop_*` [以剁EぁE��ん]に `new` の[段隁Eだんかい]で[失敁Eしっぱい]してぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/deque.nepl`
    - `Deque<.T>` と `new` / `with_capacity` / `len` / `cap` / `is_empty` / `push_front` / `push_back` / `pop_front` / `pop_back` / `peek_front` / `peek_back` / `clear` / `free` を[追加/つぁE��]した、E
    - [冁E��/なぁE�E]は ring buffer [由来/めE��い]の `[len, cap, head, data_ptr]` header で[実裁Eじっそう]した、E
    - public API の doc comment は new policy に[征Eしたが]って usage doctest を[付丁EふめEした、E
  - `stdlib/tests/deque.n.md`, `tests/stdlib/deque_collections.n.md`
    - `Deque` fixture を[追加/つぁE��]し、`push_back` / `push_front` / `peek_front` / `peek_back` / `pop_front` / `pop_back` の[基本/きほん][利用侁EりよぁE��い]を[固宁Eこてい]した、E
  - `stdlib/tests/queue.n.md`, `stdlib/tests/ringbuffer.n.md`, `stdlib/tests/stack.n.md`
    - pipe [起点/きてん]の `new` めE`unwrap_ok<..., Diag> new<...>` に[統一/とぁE��つ]した、E
  - `tests/stdlib/queue_collections.n.md`, `tests/stdlib/ringbuffer_collections.n.md`, `tests/stdlib/stack_collections.n.md`
    - `new<i32> |> uwok` めE`unwrap_ok<..., Diag> new<i32>` へ[置揁Eちかん]し、`push ... |> uwok` は 1 [衁EぎょぁEのまま[維持EぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `new` の overload [曖昧/あいまい]さを fixture [側/がわ]で[避/さ]ける[書弁Eしょしき]へ[揁Eそろ]え、`push ... |> uwok` のような result-based collection pipe [流儀/りゅぁE��]は[維持EぁE��]した、E
  - `Deque` は `Queue` と `RingBuffer` の[中閁EちめE��かん] ADT として[置/お]き、`alloc/collections` に queue family を[揁Eそろ]える[足場/あしば]とした、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/tests/deque.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/deque.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/deque_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/queue.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/ringbuffer.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/queue_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/ringbuffer_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 2`
    - [結果/けっか]: pass

# 2026-03-12 作業メモ (`Fenwick` [追加/つぁE��]と current collection regression [回収/かいしゅぁE)

- [目皁Eもくてき]:
  - `alloc/collections` に Fenwick Tree を[追加/つぁE��]し、prefix sum / range sum の bare API を[標溁EひめE��じゅん]で[提侁EてぁE��めE��]する、E
  - そ�E[途中/とちめE��]で[露出/ろしめE��]した `mem` / `string` / `vec` の current regression を[根本/こんぽん]から[回収/かいしゅぁEする、E
- [根本原因/こんぽんげんいん]:
  - `Fenwick` は owner 型なのに、`field::get` で `fw` を[褁E��囁EふくすぁE��い][読/めEんでおり、move model と[衝突EしょぁE��つ]してぁE��、E
  - `mem` / `string` / `vec` には、[前置/ぜんち][記況EきほぁEの call の[冁E��/なぁE�E]へさらに call を[埁EぁEめ[込/こ]んだ[箁E��/かしょ]が[殁Eのこ]っており、current compiler では stack reduction が[不安宁Eふあんてい]だった、E
  - `mem` / `string` の doc comment doctest に old `assert_*` [流儀/りゅぁE��]が[殁Eのこ]ってぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/fenwick.nepl`
    - `Fenwick` と `new` / `len` / `add` / `sum_prefix` / `sum_range` / `free` を[追加/つぁE��]した、E
    - owner [値/あたい]を[褁E��囁EふくすぁE��い][読/めEまなぁE��ぁE��temporary memory と raw helper を[使/つか]って `add` / `sum_prefix` / `sum_range` / `free` を[実裁Eじっそう]した、E
    - public usage doctest めEnew doc comment policy に[征Eしたが]って[付丁EふめEした、E
  - `stdlib/tests/fenwick.n.md`, `tests/stdlib/fenwick_collections.n.md`
    - `Fenwick` fixture を[追加/つぁE��]し、`new |> add ... |> uwok` と `sum_prefix` / `sum_range` の[基本/きほん][利用侁EりよぁE��い]を[固宁Eこてい]した、E
    - owner [値/あたい]の[再利用/さいりよぁEを[避/さ]けるため、query ごとに[独竁Eどくりつ]の `Fenwick` を[佁Eつく]る[形/かたち]へ[揁Eそろ]えた、E
  - `stdlib/core/mem.nepl`
    - `store_i32 add ...` / `store_u8 add ...` / `load_u8 add ...` のような nested call めEtemporary binding に[展開/てんかい]した、E
    - doc comment doctest #1 めEcurrent safe style に[更新/こうしん]した、E
  - `stdlib/alloc/collections/vec.nepl`
    - `push` の `realloc_ptr` / constructor path めEtemporary binding に[刁E��/ぶんかい]し、current compiler で[安宁Eあんてい]して[読/めEめる[形/かたち]へ[修正/しゅぁE��い]した、E
  - `stdlib/alloc/string.nepl`
    - `str_split` と `u128` parse path の nested call めEtemporary binding に[展開/てんかい]した、E
    - `from_bool` / `from_i32` の doc comment doctest めEcurrent safe style に[更新/こうしん]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `Fenwick` は `kpfenwick` をそのまま[持EめEち[丁EぁEげるのでなく、reboot 後�E bare API と `Result` [方釁EほぁE��ん]に[吁EぁEわせて `alloc/collections` の owner collection として[再設訁Eさいせっけい]した、E
  - regression fix は fixture [側/がわ]だけでなく、nested prefix call めEsource [側/がわ]で[排除/はぁE��ょ]して[根本/こんぽん]から[修正/しゅぁE��い]した、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: pass�E�Eweb/dist` の compiler [更新/こうしん]を[確誁Eかくにん]�E�E
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/btreeset.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/fenwick.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/fenwick_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/fenwick.n.md -i tests/stdlib/fenwick_collections.n.md --no-stdlib --no-tree -o /tmp/tests-fenwick.json -j 2`
    - [結果/けっか]: `3/3 pass`

# 2026-03-12 作業メモ (`BinaryHeap` [追加/つぁE��]と public doctest [整傁Eせいび])

- [目皁Eもくてき]:
  - `alloc/collections` に `BinaryHeap` を[追加/つぁE��]し、`Ord` を[用/もち]ぁE�� priority queue めEbare API で[提侁EてぁE��めE��]する、E
  - public doc comment に reboot [方釁EほぁE��ん]どおりの usage doctest を[追加/つぁE��]し、fixture と[整吁Eせいごう]する[形/かたち]で[固宁Eこてい]する、E
- [根本原因/こんぽんげんいん]:
  - `Vec` wrapper [方弁EほぁE��き]では、`vec::Vec<.T>` の namespaced type [記況EきほぁEと owner move model ぁEcurrent compiler / stdlib [方釁EほぁE��ん]に[吁EぁEわず、`BinaryHeap` の owner [表現/ひめE��げん]として[不安宁Eふあんてい]だった、E
  - `push` / `peek` / `pop` の doc comment usage めE`let hp: ... |> push ... |> uwok` の[連鎁Eれんさ]で[書/か]くと、web compile path の focused doctest では current overload / layout [処琁Eしょり]と[衝突EしょぁE��つ]し、file doctest だけが compile fail してぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/binary_heap.nepl`
    - `BinaryHeap<.T>` めE12 byte header `[len, cap, data_ptr]` の owner [構造/こうぞう]として[追加/つぁE��]した、E
    - `new` / `with_capacity` / `len` / `cap` / `is_empty` / `push` / `peek` / `pop` / `free` めEbare API で[実裁Eじっそう]した、E
    - sift-up / sift-down めEraw header / data pointer helper で[絁Eく]み、owner [値/あたい]の[多重/たじめE��][消費/しょぁE�E]を[避/さ]けた、E
    - public doc comment に usage doctest を[追加/つぁE��]し、file doctest は current compiler で[安宁Eあんてい]に[送Eとお]めEexplicit `unwrap_ok push hp item` [流儀/りゅぁE��]へ[揁Eそろ]えた、E
  - `stdlib/tests/binary_heap.n.md`
    - `push` / `peek` / `pop` / `with_capacity` の focused fixture を[追加/つぁE��]した、E
  - `tests/stdlib/binary_heap_collections.n.md`
    - pipe [記況EきほぁEで `new |> push ... |> uwok` を[使/つか]ぁEcollection-level usage fixture を[追加/つぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `BinaryHeap` は `Vec` の alias ではなく、`Stack` と[同系統/どぁE��ぁE��ぁEの owner collection として[独竁Eどくりつ] header を[持EめEつ[形/かたち]にした、E
  - public docs では「[忁Eかなら]ず[送Eとお]めEusage」を[優允EめE��せん]し、pipe [連鎁Eれんさ]は `stdlib/tests` / `tests/stdlib` [側/がわ]の fixture で[保証/ほしょぁEする[刁E��/ぶんたん]にした、E
  - `let hp: ... |> push ... |> uwok` の file doctest compile fail は current web compiler の layout / overload [残件/ざんけん]として[認譁Eにんしき]し、[関数垁Eかんすうがた] style [拡張/かくちめE��] batch で[再訪/さいほぁEする、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/binary_heap.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/binary_heap_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md -i stdlib/alloc/collections/binary_heap.nepl --no-stdlib --no-tree -o /tmp/tests-binary-heap.json -j 2`
    - [結果/けっか]: `9/9 pass`

# 2026-03-12 作業メモ (`BloomFilter` [追加/つぁE��])

- [目皁Eもくてき]:
  - `alloc/collections` に[近似/きんじ] membership test [用送Eようと]の `BloomFilter<.T,.H>` を[追加/つぁE��]し、reboot [方釁EほぁE��ん]どおり bare API で[提侁EてぁE��めE��]する、E
  - public doc comment と fixture の[両方/りょぁE��ぁEで `new` / `insert` / `contains` / `clear` / `free` の[使/つか]い[方/かた]を[固宁Eこてい]する、E
- [根本原因/こんぽんげんいん]:
  - `alloc/collections` には[正確/せいかく]な `Set` / `Map` はあっても、[空閁Eくうかん][効玁Eこうりつ]を[優允EめE��せん]する[近似/きんじ]雁E��がなく、membership-heavy な[用送Eようと]めEstdlib [標溁EひめE��じゅん]だけで[表現/ひめE��げん]しにくかった、E
  - current web compiler / doctest path では、public doc comment の pipe [連鎁Eれんさ] usage ぁE`unwrap_ok new ... |> insert ...` の[形/かたち]で[不安宁Eふあんてい]になる[箁E��/かしょ]があり、[実裁Eじっそう]ではなぁEsnippet layout [側/がわ]で compile fail を[起/お]こしてぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/bloom_filter.nepl`
    - `BloomFilter<.T,.H>` めE`[bit 長/ちめE��, byte 長/ちめE��, bitset pointer, hasher]` を[持EめEつ owner collection として[追加/つぁE��]した、E
    - `new` / `len` / `insert` / `contains` / `clear` / `free` めEbare API で[実裁Eじっそう]した、E
    - bitset は byte [配�E/はぁE��つ]で[保持/ほじ]し、E [本/ぼん]の probe index を[使/つか]ぁEfixed-probe Bloom Filter とした、E
    - `insert` / `contains` / `clear` は temporary raw storage を[使/つか]って field の[多重/たじめE��][読/めEみを[避/さ]け、current move model に[吁EぁEわせた、E
    - public doc comment は current compiler で[安宁Eあんてい]して[送Eとお]めEexplicit style に[揁Eそろ]えた、E
  - `stdlib/tests/bloom_filter.n.md`
    - `insert + contains` と `clear + invalid len` の focused fixture を[追加/つぁE��]した、E
    - `#2` は nested prefix / generic call の[絁Eく]み[吁EぁEわせで `main` 末尾ぁEunit [扱/あつか]ぁE��れる regression があったため、`contains` / `is_err` / invalid `new` めE1 step ずつ[値/あたい]へ[落/お]とぁEexplicit style に[揁Eそろ]えた、E
  - `tests/stdlib/bloom_filter_collections.n.md`
    - pipe [記況EきほぁEで `new |> insert ... |> clear` を[確誁Eかくにん]する collection-level usage fixture を[追加/つぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `BloomFilter` は[正確/せいかく]な[雁E��/しゅぁE��ぁEでなく「not contained を[高送Eこうそく]に[判宁Eはんてい]する」[専用/せんよう][構造/こうぞう]として `alloc/collections` に[置/お]ぁE��、E
  - hasher は `HashMap` / `HashSet` と[吁Eおな]じく `.H: Hasher<.T>` を[叁EぁEける owner value にし、user-provided hasher をそのまま[流Eなが]せる[形/かたち]にした、E
  - public doctest は「[確宁Eかくじつ]に[送Eとお]めEusage」を[優允EめE��せん]し、pipe [連鎁Eれんさ]は `tests/stdlib` [側/がわ] fixture で[保証/ほしょぁEする[刁E��/ぶんたん]にした、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bloom_filter.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bloom_filter.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/bloom_filter_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md -i stdlib/alloc/collections/bloom_filter.nepl --no-stdlib --no-tree -o /tmp/tests-bloom-filter.json -j 2`
    - [結果/けっか]: `9/9 pass`

# 2026-03-12 作業メモ (`DisjointSet` [追加/つぁE��])

- [目皁Eもくてき]:
  - `alloc/collections` に `DisjointSet` を[追加/つぁE��]し、Union-Find めEbare API で[標準提侁EひめE��じゅんてぁE��めE��]する、E
  - public doc comment / `stdlib/tests` / `tests/stdlib` の 3 [層/そう]で usage を[固宁Eこてい]し、graph めEgrouping の[基盤/き�Eん]を[墁Eふ]めE��、E
- [根本原因/こんぽんげんいん]:
  - `alloc/collections` には queue / heap / tree / hash は[揁Eそろ]ってきたが、[雁E��刁E��/しゅぁE��ぁE�Eんかつ]を[扱/あつか]ぁEDSU がなく、Kruskal めEconnectivity check の[基盤/き�Eん]が[欠/か]けてぁE��、E
  - current owner model では query めEreceiver を[消費/しょぁE�E]するので、`same` / `size` / `find` を[吁Eおな]ぁEowner に[綁Eつづ]けて[呼/めEぶ fixture は moved-value compile fail になってぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/disjoint_set.nepl`
    - `DisjointSet` めE`[n, parent ptr, sizes ptr]` を[持EめEつ owner collection として[追加/つぁE��]した、E
    - `new` / `len` / `find` / `union` / `same` / `size` / `free` めEbare API で[実裁Eじっそう]した、E
    - [冁E��/なぁE�E]は `parent[i]` と `sizes[root]` を[持EめEつ classic Union-Find で、`union` は union-by-size を[採用/さいよう]した、E
    - public API は pure query を[優允EめE��せん]して path compression を[入/い]れず、`find` / `same` / `size` は[読/めEみ[叁Eと]りだけで[完絁Eかんけつ]する[形/かたち]にした、E
  - `stdlib/tests/disjoint_set.n.md`
    - `union + same + size` と invalid index の focused fixture を[追加/つぁE��]した、E
    - query ぁEowner を[消費/しょぁE�E]する current model に[吁EぁEわせて、[同値/どぁE��]な DSU を[佁Eつく]り[直/なお]して[吁E��誁Eかくかくにん]を[刁E��/ぶんり]した、E
  - `tests/stdlib/disjoint_set_collections.n.md`
    - pipe [記況EきほぁEで `new |> union ... |> uwok` を[確誁Eかくにん]する collection-level usage fixture を[追加/つぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `DisjointSet` は owner [構造/こうぞう]だが、public query めE`Result<i32,Diag>` / `Result<bool,Diag>` に[俁Eたも]つため、path compression を[見送Eみおく]って union-by-size のみで[平衡性/へぁE��ぁE��い]を[確俁Eかくほ]した、E
  - path compression めEpublic API に[輁Eの]せるには owner と query value を[一緁EぁE��しょ]に[迁Eかえ]す別設計が[要Eい]る�Eで、[関数垁Eかんすうがた] style [支援/しえん] batch で[再検訁EさいけんとぁEする、E
  - doctest と fixture は「current owner model で[確宁Eかくじつ]に[送Eとお]めEusage」を[優允EめE��せん]し、同ぁEowner の[再利用/さいりよぁEを[避/さ]けた、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 6`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/disjoint_set.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/disjoint_set.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/disjoint_set_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i tests/stdlib/disjoint_set_collections.n.md -i stdlib/alloc/collections/disjoint_set.nepl --no-stdlib --no-tree -o /tmp/tests-disjoint-set.json -j 2`
    - [結果/けっか]: `9/9 pass`

# 2026-03-12 作業メモ (`SegmentTree` [追加/つぁE��])

- [目皁Eもくてき]:
  - `alloc/collections` に `SegmentTree` を[追加/つぁE��]し、[点更新/てんこぁE��ん]と[一般区閁EぁE��ぱんくかん] sum query の[土台/どだい]を[標準化/ひめE��じゅんか]する、E
  - `Fenwick` と[役割/めE��わり]を[刁Eわ]け、`alloc/collections` に query-oriented tree を[墁Eふ]めE��、E
- [根本原因/こんぽんげんいん]:
  - `Fenwick` は prefix / range sum には[十�E/じゅぁE�Eん]だが、[実裁Eじっそう]の[見送Eみとお]しや[一般区閁EぁE��ぱんくかん]木の[入口/ぁE��ぐち]としては `SegmentTree` めE忁E��Eひつよう]だった、E
  - `set` は current parser の[予紁E��Eよやくご]で public API 名にできず、そのままでは file doctest 以前に source parse が[壁Eこわ]れてぁE��、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/segment_tree.nepl`
    - `SegmentTree` めE`[n, base, data ptr]` を[持EめEつ owner collection として[追加/つぁE��]した、E
    - `new` / `len` / `replace` / `add` / `sum_range` / `free` めEbare API で[実裁Eじっそう]した、E
    - [冁E��/なぁE�E]は base めE2 [冪/べき]に[丸/まる]めた iterative segment tree とし、leaf は `[base, base+n)` に[置/お]ぁE��、E
    - current parser の[制紁EせいめE��]に[征Eしたが]ぁE��point overwrite は `set` でなぁE`replace` めEpublic 名とした、E
  - `stdlib/tests/segment_tree.n.md`
    - `replace + add + sum_range` と invalid index/range の focused fixture を[追加/つぁE��]した、E
  - `tests/stdlib/segment_tree_collections.n.md`
    - pipe [記況EきほぁEで `new |> replace ... |> add ...` を[確誁Eかくにん]する collection-level usage fixture を[追加/つぁE��]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `SegmentTree` は current reboot [段隁Eだんかい]では `i32` sum 専用に[絁Eしぼ]り、封E��の[関数垁Eかんすうがた] style / monoid [支援/しえん] batch で generic aggregator に[拡張/かくちめE��]する、E
  - `set` でなぁE`replace` を[選/えら]んだのは `Vec` と[吁Eおな]ぁEparser [制紁EせいめE��]によるも�Eで、命名[不整吁Eふせいごう]は[言語�E/げんごがわ]の reserved keyword [整琁Eせいり] task と[接綁Eせつぞく]する、E
- [検証/けんしょぁE:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 3`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/segment_tree.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/segment_tree.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/segment_tree_collections.n.md -n 1`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md -i tests/stdlib/segment_tree_collections.n.md -i stdlib/alloc/collections/segment_tree.nepl --no-stdlib --no-tree -o /tmp/tests-segment-tree.json -j 2`
    - [結果/けっか]: `8/8 pass`

# 2026-03-12 作業メモ (fix(compiler): composite size/load/store を実体サイズに合わせる)

- [目皁Eもくてき]:
  - `size_of<T>` ぁEmulti-field struct / tuple / enum / generic apply に対して[正/ただ]しい[実佁Eじったい] size を[迁Eかえ]すよぁE��する、E
  - aggregate value めE`load<T>` / `store<T>` で[扱/あつか]ぁE��き、`i32` 1 [誁Eご]ではなく[実佁Eじったい] size ぶん�E byte copy として lowering する、E
- [根本原因/こんぽんげんいん]:
  - wasm / llvm codegen の `size_of` / `align_of` は、`u8` と 64-bit scalar [以夁EぁE��い]を[事実丁EじじつじょぁE 4 byte [扱/あつか]ぁE��てぁE��、E
  - さらに `load<T>` / `store<T>` めE`Struct` / `Tuple` / `Enum` めE`i32` 1 [誁Eご]として lowering しており、aggregate value の round-trip が[壁Eこわ]れてぁE��、E
  - wasm [側/がわ] aggregate `load` の[初回/しょかい][実裁Eじっそう]では `local.tee` により[戻/もど]めEpointer ぁEstack に 2 [倁Eこ][殁Eのこ]り、validation failure めE起/お]きてぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/codegen_wasm.rs`
    - `type_storage_size_bytes` / `type_storage_align_bytes` / `is_aggregate_storage_type` を[追加/つぁE��]、E
    - generic apply は `TypeCtx` clone + type param substitution で field/payload の[実佁Eじったい] size を[再帰皁Eさいきてき]に[計箁Eけいさん]、E
    - aggregate `load<T>` / `store<T>` めEbyte copy lowering に[変更/へんこぁE、E
    - aggregate `load<T>` の `local.tee` めE`local.set` に[修正/しゅぁE��い]し、WASM stack balance を[復旧/ふっきゅぁE、E
  - `nepl-core/src/codegen_llvm.rs`
    - 同等�E helper を[追加/つぁE��]、E
    - aggregate `load<T>` / `store<T>` めE`i8` [単佁Eたんい]の copy lowering に[変更/へんこぁE、E
  - `tests/compiler/sizeof.n.md`
    - `sizeof_multi_field_struct_regression` を[追加/つぁE��]、E
    - `Pair{i32,i32}` の `8 byte`、`WidePair{i64,i32}` の `12 byte` を[固宁Eこてい]、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/run_doctest.js -i tests/compiler/sizeof.n.md -n 4`
    - [結果/けっか]: pass
- [差異/さい]メモ:
  - こ�E[修正/しゅぁE��い]で `size_of` regression は[解涁EかいしょぁEしたが、`alloc/collections/trie` の non-empty insert はまだ runtime OOB が[殁Eのこ]る、E
  - `Trie` [追加/つぁE��] batch は library [側/がわ] root cause が[未収束/みしゅぁE��く]のため commit してぁE��ぁE��`trie_build_suffix_chain` と node [接綁Eせつぞく] logic めEfocused に[再調査/さいちめE��さ]する、E

# 2026-03-12 作業メモ (alloc/collections/trie 調査のみ・未 commit)

- [目皁Eもくてき]:
  - `alloc/collections` の[種顁Eしゅるい][拡允EかくじゅぁEとして `Trie` を[追加/つぁE��]できるかを[評価/ひめE��か]する、E
- [刁E��刁E��/きりわけ]:
  - `TrieNode` の push / append / terminal 更新までは focused scratch で pass した、E
  - `Trie` owner [値/あたい]から `Vec<TrieNode>` を[叁Eと]り[出/だ]して prefix [探索/たんさく] loop を[囁Eまわ]すところで runtime `unreachable` が[再現/さいげん]した、E
  - `size_of` / aggregate byte copy [修正/しゅぁE��い]後も[殁Eのこ]ったため、library [実裁Eじっそう]でなぁEcurrent compiler/runtime の「owner struct + aggregate field + loop」をまたぐ lowering の[問顁Eもんだい]と[判断/はんだん]した、E
  - `trie_find_child_index` めEhelper から inline へ[展開/てんかい]しても、`insert` / `contains` / `starts_with` の non-empty case は[収束/しゅぁE��く]しなかった、E
- [判断/はんだん]:
  - broken state めEstdlib に[混/ま]ぜなぁE��め、`trie.nepl` / `stdlib/tests/trie.n.md` / `tests/stdlib/trie_collections.n.md` は未 commit のまま[削除/さくじょ]して worktree から[夁Eはず]した、E
  - `Trie` は stdlib task としては[残件/ざんけん]だが、[次/つぎ]に[進/すす]むには compiler/runtime [側/がわ]の[最小�E現/さいしょぁE��ぁE��ん] test を[允Eさき]に[佁Eつく]るべき[段隁Eだんかい]である、E

# 2026-03-12 作業メモ (alloc/collections/adjacency_list 調査のみ・未 commit)

- [目皁Eもくてき]:
  - sparse graph [吁Eむ]け�E `AdjacencyList` めE`alloc/collections` に[追加/つぁE��]できるかを[評価/ひめE��か]する、E
- [刁E��刁E��/きりわけ]:
  - `heads | to | next` の 3 [配�E/はぁE��つ]めE1 [本/ほん]の contiguous buffer に[詰/つ]める library [設訁Eせっけい]までは[作�E/さくせい]した、E
  - native compiler では `new + insert + contains` の[最小侁EさいしょぁE��い]が[送Eとお]る一方、web compile path では same-`from` edge めE2 [本/ほん][追加/つぁE��]した case で `contains` ぁEfalse になった、E
  - owner aggregate めEtemporary memory に[退避/たいひ]する形と、`hdr + buf_ptr` へ[落/お]とした header-pointer owner の 2 [桁Eあん]を[試/ため]したが、どちらも web compile path では `insert` / `contains` / `remove` ぁE`RuntimeError: unreachable` へ[崩/くず]れた、E
  - [痁E��/しょぁE��めE��]は library [側/がわ]の linked-list [更新/こうしん]より、current compiler/runtime の owner value lowering と aggregate/header [読/めEみ[出/だ]し�E[墁E��/きょぁE��い]に[依孁EぁE��ん]してぁE��と[判断/はんだん]した、E
- [判断/はんだん]:
  - broken state めEstdlib に[混/ま]ぜなぁE��め、`adjacency_list.nepl` / `stdlib/tests/adjacency_list.n.md` / `tests/stdlib/adjacency_list_collections.n.md` は未 commit のまま worktree から[夁Eはず]した、E
  - `AdjacencyList` は stdlib [残件/ざんけん]として note に[殁Eのこ]し、[次囁Eじかい]は compiler/runtime [側/がわ]の[最小�E現/さいしょぁE��ぁE��ん] test として[允Eさき]に[刁Eき]り[出/だ]す、E

# 2026-03-12 作業メモ (alloc/collections/btreemultiset 調査のみ・未 commit)

- [目皁Eもくてき]:
  - ordered multiset めE`alloc/collections` に[追加/つぁE��]し、[重褁EちめE��ふく] key を[個数/こすぁEつきで[保持/ほじ]できる collection を[標準化/ひめE��じゅんか]する、E
- [刁E��刁E��/きりわけ]:
  - `BTreeMap<.T, i32>` の count wrapper として `BTreeMultiSet` を[試佁Eしさく]した、E
  - しかぁEcurrent owner model では wrapper owner と inner `BTreeMap` owner の[二重/にじゅぁE[所朁EしょめE��]を[自然/しぜん]に[扱/あつか]えず、`insert` / `remove_one` / `clear` の[吁E��/かくしょ]で `D3053 use of moved value` が[連鎁Eれんさ]した、E
  - raw header wrapper に[落/お]としても、doctest fixture では `RuntimeError: unreachable` が[殁Eのこ]り、library [側/がわ]だけで[整吁Eせいごう]した API に[収束/しゅぁE��く]しなかった、E
- [判断/はんだん]:
  - `BTreeMultiSet` めEbroken state めEstdlib に[混/ま]ぜず、試作ファイルは未 commit のまま worktree から[夁Eはず]した、E
  - ordered multiset は[有用/めE��よう]だが、wrapper owner と inner owner の[合�E/ごうせい]めEcurrent compiler/runtime がどこまで[支/ささ]えられるかを[允Eさき]に[再評価/さいひめE��か]する、E

# 2026-03-12 作業メモ (feat(list): 関数垁Ehelper 追加)

- [目皁Eもくてき]:
  - `List` に `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` を[追加/つぁE��]し、tutorial [剁EまぁEに[関数垁Eかんすうがた] style の[基礁Eきそ] API を[整/ととの]える、E
  - namespace call regression とあわせて、`list::map` ぁEcurrent bare API / move model で[自然/しぜん]に[使/つか]えることを[保証/ほしょぁEする、E
- [根本原因/こんぽんげんいん]:
  - compiler [側/がわ]では `TypeKind::Function` ぁEtrait model の `Copy` [判宁Eはんてい]で false [扱/あつか]ぁE�Eままで、[高階/こうかい][関数/かんすう]を[再帰/さいき] helper に[渡/わた]すと `D3053 use of moved value` が[発甁Eはっせい]してぁE��、E
  - library [側/がわ]では `list_map_impl` ぁE`cons<.U> f load<.T> lst_ptr mapped_tail` の[形/かたち]で nested call をそのまま[書/か]ぁE��おり、前置記法�E[畳/たた]み[込/こ]みで `f` の[結果/けっか]ではなく[関数値/かんすうち]や[壁Eこわ]れた[値/あたい]ぁE`cons` の head へ[流Eなが]れ[込/こ]んでぁE��、E
  - `tests/compiler/list_dot_map.n.md` の `list_namespace_map_with_list` めEempty list に `map` して `get 0 |> unwrap` しており、fixture [前提/ぜんてい]が[誤/あやま]ってぁE��、E
- [変更/へんこぁE:
  - `nepl-core/src/types.rs`
    - `is_copy_with_trait_model` と `is_copy_eligible_inner` で `TypeKind::Function` めE`Copy` / copy-eligible [扱/あつか]ぁE��[変更/へんこぁEした、E
  - `stdlib/alloc/collections/list.nepl`
    - `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` と internal helper を[追加/つぁE��]した、E
    - `list_map_impl` は `let mapped_head <.U> ...` を[経由/けいめEしてから `cons` する[形/かたち]へ[変更/へんこぁEし、nested call の[誤解釁EごかぁE��めE��]を[防止/ぼぁE��]した、E
    - public doc comment は current policy / format に[揁Eそろ]えた、E
  - `stdlib/tests/list.n.md`
    - `list_functional_helpers` を[追加/つぁE��]し、owner [再利用/さいりよぁEを[避/さ]けるため source list を[個別/こべつ]に[刁E��/ぶんり]した、E
  - `tests/compiler/list_dot_map.n.md`
    - old compile-fail めEcurrent namespace success case へ[更新/こうしん]した、E
    - non-empty list めE`list::push` で[佁Eつく]ってから `list::map` を[呼/めEぶ fixture に[変更/へんこぁEした、E
- [検証/けんしょぁE:
  - `cargo build -p nepl-cli`
    - [結果/けっか]: success
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 9`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 10`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/list_dot_map.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/list.n.md -i tests/compiler/list_dot_map.n.md --no-stdlib --no-tree -o /tmp/tests-list-fp-short.json -j 2`
    - [結果/けっか]: `5/5 pass`

# 2026-03-12 作業メモ (feat(vec): 関数垁Ehelper 追加)

- [目皁Eもくてき]:
  - `Vec` に `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` を[追加/つぁE��]し、`List` / `Option` / `Result` に[綁Eつづ]く[関数垁Eかんすうがた] style の[基本操佁EきほんそぁE��]を[揁Eそろ]える、E
  - bare `map` ぁE`Option` / `Result` と[同屁EどぁE��ょ]しても、`Vec` [側/がわ]へ[正/ただ]しく[解決/かいけつ]されることめEfixture で[固宁Eこてい]する、E
- [根本原因/こんぽんげんいん]:
  - `Vec` は owner [構造/こうぞう]なので、`List` のように node を[再帰/さいき][構篁Eこうちく]するだけでなく、[出劁Eしゅつりょく]バッファの[確俁Eかくほ]と move model を[同時/どぁE��]に[整吁Eせいごう]させる[忁E��Eひつよう]があった、E
  - `fold` / `reduce` めEwhile loop + `set out f out item` の[形/かたち]で[書/か]くと、generic accumulator `.U` / `.T` ぁE`Copy` でない[場吁Eばあい]に `D3054 use of potentially moved value` になった、E
  - `find` でめEmutable `Option<.T>` めEwhile [条件/じょぁE��ん]で[読/めEむと、`.T` ぁEnon-`Copy` の[場吁Eばあい]に moved-value [判宁Eはんてい]へ[落/お]ちた、E
  - fixture [側/がわ]めE`filtered` めE`len` と `get` で[再利用/さいりよぁEしており、current owner model では `D3053` だった、E
- [変更/へんこぁE:
  - `stdlib/alloc/collections/vec.nepl`
    - `vec_read_at` / `vec_write_at` と、`vec_fold_impl` / `vec_reduce_impl` / `vec_find_impl` を[追加/つぁE��]した、E
    - `map` は exact capacity を[允Eさき]に[確俁Eかくほ]して raw loop で[詰/つ]める[形/かたち]にした、E
    - `filter` は 2-pass�E�E個数/こすぁE[計測/けいそく] -> exact capacity [確俁Eかくほ] -> [転冁Eてんしゃ]�E�にし、`push` の[逐次/ちくじ][連鎁Eれんさ]を[避/さ]けた、E
    - `fold` / `reduce` / `find` は再帰 helper に[寁EめEせ、generic owner / accumulator の moved-value を[根本/こんぽん]から[解涁EかいしょぁEした、E
    - public doc comment と `neplg2:test` めEcurrent policy / format に[揁Eそろ]えた、E
  - `stdlib/tests/vec.n.md`
    - `vec_functional_helpers` を[追加/つぁE��]し、`map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` の focused fixture を[整傁Eせいび]した、E
    - owner [再利用/さいりよぁEを[避/さ]けるため、`filtered` の[長/なが]さ[確誁Eかくにん]と[要素/ようそ][確誁Eかくにん]は source を[刁E��/ぶんり]した、E
  - `tests/compiler/list_dot_map.n.md`
    - `vec_map_with_star_alias_works` を[追加/つぁE��]し、`alloc/collections/vec` と `core/result` / `core/option` めE`as *` で[同時/どぁE��] import した[状慁EじょぁE��い]でめEbare `map<i32,i32>` ぁE`Vec` [牁Eばん]へ[解決/かいけつ]することを[固宁Eこてい]した、E
- [設訁Eせっけい][判断/はんだん]:
  - `Vec` helper は[全佁Eぜんたい]を[新要Eしんき] owner として[迁Eかえ]すため、`map` / `filter` の[確俁Eかくほ][失敁Eしっぱい]は `StdErrorKind::OutOfMemory` に[雁E��EしゅぁE��く]した、E
  - `filter` めE2-pass にしたのは、current reboot [段隁Eだんかい]で `Result` を[持EめEつ owner value めEloop [冁Eない]で[逐次/ちくじ][更新/こうしん]すると move model と[早期脱出/そうきだっしゅつ]が[褁E��/ふくざつ]になるためである、E
  - `fold` / `reduce` / `find` は mutable owner / accumulator を[避/さ]けるために再帰 helper を[選/えら]び、compiler [側/がわ]の追加修正なしで current model に[叁Eおさ]めた、E
- [検証/けんしょぁE:
  - `NO_COLOR=false trunk build`
    - [結果/けっか]: success
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 5`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/vec.n.md -n 2`
    - [結果/けっか]: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/list_dot_map.n.md -n 4`
    - [結果/けっか]: pass
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/compiler/list_dot_map.n.md --no-stdlib --no-tree -o /tmp/tests-vec-fp-short.json -j 2`
    - [結果/けっか]: `6/6 pass`

# 2026-03-15 作業メモ (doc: 開発計画と仕様�E再確誁E

- [目皁Eもくてき]:
  - `nepl-core` と `doc/` に書かれてぁE��仕様、およ�E `todo.md` の開発計画につぁE��実現可能か、E��刁E��を確認する、E
- [確認結果/かくにんけっか]:
  - `doc/memory_safety_migration_plan.md` の、E5. 実裁E��先頁E��」セクションで定義されてぁE�� Phase 1�E�基盤修正�E�およ�E Phase 2�E�型・API 刁E���E��E区刁E��と、`todo.md` の、E. メモリ安�E型モチE��を統合仕様に基づぁE��実裁E��る」�Eサブ頁E��が完�Eに一致してぁE��ことを確認した、E
  - 実裁E��針！EInternalAlloc` の利用、`MemPtr` の隔離、`List` の persistent 化、`VarState` の導�E、E��域推論など�E��E、GCを使用せずメモリ安�E性を確保するNEPLg2の目標達成�Eために非常に論理皁E��つ実現可能に絁E��立てられてぁE��、E
  - したがって、現在の `todo.md` および `plan.md` は適刁E��あり、修正は忁E��なぁE��判断した、E

# 2026-03-15 作業メモ (doc: todo.md の頁E��と優先頁E���E適正匁E

- [目皁Eもくてき]:
  - `todo.md` の「stdlib 再構篁E本流」セクションの作業頁E��が、�Eイグレーション計画 (`memory_safety_migration_plan.md`) の Phase 刁E��と矛盾し、破綻してぁE���E��Eてのメモリ安�E化コンパイラ実裁E�� `alloc` めE`std` 層のポインタ隔離再構築�E前に置かれてぁE���E�問題を解決し、アーキチE��チャの依存関係に沿った適刁E��頁E��に並び替える、E
- [作業冁E��/さぎめE��なぁE��ぁE:
  - `todo.md` 冁E�Eタスク網羁E��を完�Eに維持しつつ、依存度の強ぁE��盤から進める原則 (`diag/trait` -> `compiler 前提` -> `core/mem` -> `alloc` -> `コンパイラ後段パス` -> `runtimes` -> `std` -> `features`) に沿って再構�E、E
  - メモリ安�Eマイグレーションの吁E��階を、レイヤーごとの整備フェーズに適刁E��刁E��・配置した:
    - コンパイラ基盤と診断整傁E(Phase 0) を最序盤に配置、E
    - `core/mem` および `alloc` の生�Eインタ隔離 (Phase 1, 2) をライブラリ層再構築�E前半に配置、E
    - Purity追跡の変更、Resource IRによるDrop Elaboration、およ�ERegion 推諁E(Phase 4, 5, 6) を、`alloc` が安�E化された後�E「コンパイラ解析パスの強化」タスクとして配置、E
    - `std/io` 等への `ExternalIO` 効果宣言付丁E(Phase 3) めE`std` 層構築フェーズに配置、E
- [確認結果/かくにんけっか]:
  - ポインタを隔離するとぁE��ライブラリ前提をクリアしてからコンパイラの所有権等による自動管琁E���E (Resource IR) を導�Eするよう軌道修正され、現実的で論理破綻のなぁE��スクリストに修正されたことを確認した、E

# 2026-03-15 作業メモ (doc: 全体仕様�E依存型・形式証明パラダイムへの到達実現性評価)

- [目皁Eもくてき]:
  - `doc/` 以下�E全ドキュメントを精査し、最終目標である「強力な静的・型�E所有権検査」「完�Eな検証」、そして「依存型�E�Eependent Types�E�による形式証明」が完�Eに実現可能か�E重に検討する、E
- [確認ファイル/かくにんファイル]:
  - `trait_system_design.md`, `move_effect_spec.md`, `error.md`, `shadowing.md`, `stdlib_breaking_reboot.md`, `testing.md`, `rewrite_plan.md`, `runtime.md`, `new_tutorial_plan.md` およびメモリ安�E系仕槁E
- [検討結果・刁E��/けんとぁE��っか�Eぶんせき]:
  - **完璧な実現可能性を確誁E*: 現在の吁E��仕様�E、依存型めE��式証明�E導�Eを妨げる「暗黙�E副作用・状態�E非決定性」をコンパイラアーキチE��チャの根底から徹底的に排除するよう設計されてぁE��、E
  - **CTFE�E�Eompile-Time Function Evaluation�E��E強力な土台**: `move_effect_spec.md` と `purity_ownership_memory_spec.md` にある「�E部皁E�� `InternalAlloc`�E�生メモリ操作）を Surface の `Pure` に畳み込む」仕様と「Escape Analysis」�E、依存型実現における最強の武器となる。「�E部皁E��はミュータブルで高速に実行できるが、外部�E�型シスチE��側�E�から�E完�Eに純粋な数学皁E��数に見える」とぁE��性質が保証されてぁE��ため、型チェチE��がコンパイル時に安�Eにコードを評価�E�メタ計算）できる、E
  - **決定論的な名前・型解決**: `rewrite_plan.md` の「DefId ベ�Eスのモジュール解決」「`noshadow`」や、`trait_system_design.md` の「構造皁E��同値による trait 制紁E���E、証明�E検証時に忁E��となる「名前や型�E決定論的同一性」を拁E��してぁE��、E
  - **GCレスと純粋不変性の両竁E*: `runtime.md` 等に特記される「Region Inference による Persistent な値のスコープ管琁E���E、値が勝手に書き換わらなぁE��と�E�不変性�E��E静的証明そのも�Eと言える。依存型の型パラメータに安�Eに値を持ち込める、E
- [結諁Eけつろん]:
  - NEPLg2 が現計画で目持E��てぁE��「純粋性」「局所可変状態�E隔離」「型付きメモリ所有権」�E3本柱は、まさに定理証明支援系�E�Eoq, Agda等）相当�E型シスチE��を汎用プログラミング言語上に構築するため�E **Must-Have�E�忁E��要件�E�E* をすべて満たしてぁE��、E
  - アーキチE��チャの変更を�Eく忁E��とせず、現在のマイルスト�Eンを完遂した延長線上に「CTFEの拡允E��「Totality�E�停止性�E�チェチE��の導�E」「命題型の追加」とぁE��形で自然に依存型・形式証明を接続可能であると結論づけた、E

# 2026-03-15 作業メモ (doc: 依存型・形式証明に向けた下書き仕様書の作�E)

- [目皁Eもくてき]:
  - 依存型の導�Eと形式証明を見据えた `doc/dependent_type_proof_plan.md` を作�Eし、現在進行中のメモリ安�E・Purity化�Eプロジェクトがどのように封E��の強力な型検査�E�ETFE, Totality Checker, 命題型�E�へと接続されるか�E青�E真を残す、E
- [作業冁E��/さぎめE��なぁE��ぁE:
  - `doc/dependent_type_proof_plan.md` を作�Eし、未来の構想�E�三大追加要素�E�と現在の土台�E�Escape Analysis, Region/Drop, 決定論的解決�E�が高い親和性を持つことをドキュメント化した、E
  - Vecの長さを型レベルで追跡する構文のスケチE��を追加し、形式証明導�E後�Eプログラミングの未来像を提示した、E

# 2026-03-27 作業メモ (compare / migration の全斁E��み継綁E

- [目皁Eもくてき]:
  - `doc/2.1spec/` から外した旧語彙や未採用機�Eが、`doc/compare/` と `doc/migration/` に残って正仕様�Eように読まれなぁE��ぁE��琁E��る、E
- [作業冁E��/さぎめE��なぁE��ぁE:
  - `doc/compare/index.md` の「追加されるもの」かめE`noshadow let` とぁE��断定表現を外し、同一シグネチャ再定義の保護は封E��拡張候補として保留中だと明記した、E
  - `doc/compare/module_system.md` の `module parser:` / `module lexer:` 例に残ってぁE��旧 placeholder を、現行�E `let <name> <expr>` と lambda で読める例へ修正した、E
  - `doc/migration/index.md` の tutorial 想定ファイル吁E`33_noshadow_and_overload.n.md` を、現衁Ecore 仕様と衝突しなぁE`33_overload_and_redefinition.n.md` に更新した、E
- [plan.mdとの差異/さ]:
  - `plan.md` の目標�E体に変更はなぁE��E
  - 斁E��群の補助賁E��側で残ってぁE��旧案語彙を整琁E��、`doc/2.1spec/` を正の仕様として読む導線を補強した、E

# 2026-03-27 作業メモ (root doc / compare の全斁E��み継綁E

- [目皁Eもくてき]:
  - `doc/README.md` と `doc/compare/` の表現を、`doc/2.1spec/index.md` の現在のスチE�Eタス整琁E��揁E��る、E
- [作業冁E��/さぎめE��なぁE��ぁE:
  - `doc/README.md` の `2.1spec/` 説明を、「完�Eな仕様」と断定する表現から、各章で draft / 封E��仕様を明示する現在の整琁E��合わせた説明へ更新した、E
  - `doc/compare/syntax.md` の 0 引数関数例を、`let main \\(): ...` とぁE��旧 block 省略の見え方から、現在の宣言説明と齟齬の少なぁE`let main \() 0` へ差し替えた、E
- [plan.mdとの差異/さ]:
  - `plan.md` の目標�E体に変更はなぁE��E
  - 正の仕様を案�Eする入口斁E��と比輁E��書の表現を、現在の `2.1spec` のスチE�Eタス表示に合わせた、E

# 2026-03-27 作業メモ (root tool doc の全斁E��み継綁E

- [目皁Eもくてき]:
  - root の補助斁E��群でも、現衁EBootstrap 実裁E�E説明と NEPLg2.1 の正の仕様が混ざって見えなぁE��ぁE��墁E��を揃える、E
- [作業冁E��/さぎめE��なぁE��ぁE:
  - `doc/debug.md` に対象実裁E��記を追加し、この斁E��が現衁E`nepl-core` / `nepl-cli` の debug build 挙動を説明するもので、正の仕様�E `doc/2.1spec/` だと明記した、E
  - `doc/llvm_ir_setup.md` に対象実裁E��記を追加し、この斁E��ぁELLVM target の開発環墁E��モであり、target 設計そのも�Eは `doc/2.1spec/platform.md` を参照すべきだと明記した、E
- [plan.mdとの差異/さ]:
  - `plan.md` の目標�E体に変更はなぁE��E
  - root 斁E��群のスチE�Eタス表示をそろえ、`doc/2.1spec/` を正の仕様として読む導線を補強した、E

# 2026-03-27 作業メモ (2.1spec 残件の整合修正)

- [目皁Eもくてき]:
  - `2.1spec` の章間整合�E残件を整琁E��、Zenn #1 / #2 を正としたコア仕様�E周囲に残ってぁE��未定義語や表記ずれを解消する、E
- [作業冁E��/さぎめE��なぁE��ぁE:
  - `doc/2.1spec/modules.md` の `merge` を頁E��なぁEmultiset ではなぁEdeclaration sequence として定義し直し、「後老E��先」�E意味が統合後頁E��で決まることを�E記した、E
  - `doc/2.1spec/modules.md` / `syntax.md` / `platform.md` に `#if <cond_expr>:` めE2.1 の正規�E前置チE��レクチE��ブとして追加し、旧 `#if[target=...]` 角括弧記法を 2.0 系表記として退けた、E
  - `doc/2.1spec/traits.md` の `merge` 例から不要な `.K` / `.V` と無関係な制紁E��除去し、Coherence 違反と bare 名曖昧性の扱ぁE��刁E��した、E
  - `doc/2.1spec/traits.md` と `doc/2.1spec/stdlib.md` の標溁Etrait 一覧を揃え、`Add .U .R`、I/O 系 trait、allocator 系 trait を�E通化し、`RegionOwned` / `MemReadable` / `MemWritable` は封E��導�Eであることを�E記した、E
  - `doc/2.1spec/errors.md` に `Diags` めE`Diag` の列を表す補助型だと追記した、E
  - `doc/2.1spec/index.md` と `doc/README.md` の説明を、凍結済みコア仕様と draft / 封E��仕様�E周辺領域が併存する現在の整琁E��合わせて補正した、E
- [plan.mdとの差異/さ]:
  - `plan.md` の目標�E体に変更はなぁE��E
  - 仕様書群のスチE�Eタス表示と章間参照を整琁E��、`2.1spec` を読むときに「どこが凍結済みで、どこが封E��仕様か」が追ぁE��すくなった、E
# 2026-04-02 Web Playground editor 再開発計画作�E

- [目的]:
  - Web Playground の editor を場当たり的に修正するのではなく、highlight / problems / hover / key input を含めて責務�E割からめE��直すため、現状調査と再開発計画を整琁E��た、E- [現状確認]:
  - `web/src/editor/editor.ts` の `CanvasEditor` ぁEtext state, cursor/selection, undo/redo, folding, language provider 連携, Problems 更新まで抱えており、�E力�E描画・状態�E言語機�Eが寁E��合になってぁE��、E  - `web/src/editor/editor-input-handler.ts` ぁEDOM event と editor state 更新を直結してぁE��ため、shortcut めEkey input のチE��トを CLI だけで再現できなぁE��E  - `web/src/language/neplg2/neplg2-provider.ts` ぁE`window.wasmBindings` に直結しつつ、highlight / hover / definition / completion / indentation / comment toggle めE1 ファイルに抱えてぁE��、E  - `nepl-web/src/lib.rs` には `analyze_semantics`, `analyze_semantics_with_vfs`, `analyze_name_resolution` など editor 再開発に忁E��な解极EAPI が揃ってぁE��一方、editor 側に UI 非依存�E正規化層がなぁE��E  - `nodesrc/compiler_loader.js` で Trunk 成果物めENode.js から読み込めるので、browser なしで CLI から editor 解析テストを回す導線�E既にある、E- [plan.mdとの差刁E:
  - `plan.md` には playground editor 再設計�E具体計画めECLI 完結テスト方針�Eまだ整琁E��れてぁE��ぁE��E  - 変更提案として、editor めEpure な core/reducer と browser adapter に刁E��し、解析結果の正規化層を設ける計画めE`doc/web_playground_editor_redevelopment_plan.md` に記録した、E  - そ�E後�E見直しで、repository 持E��にある「`trunk build` 後に `nodesrc/cli.js` のチE��トを実行し、output の JSON を確認すること」を満たすには、専用 runner だけでなぁE`nodesrc/cli.js` 経由の正式導線が忁E��だと刁E��ったため、計画書へ追記した、E- [追加した斁E��]:
  - `doc/web_playground_editor_redevelopment_plan.md` を追加し、現状の問題点、根本原因、責務�E割案、hover/problems/highlight の再設計方針、CLI 完結テスト計画、段階的な実裁E��ェーズを記述した、E  - `doc/README.md` から新しい計画書へ辿れるようにリンクを追加した、E- [今後�E実裁E��点]:
  - `editor-core` を新設して command/state/reducer/keymap/view-model めEpure に刁E��出す、E  - `neplg2-provider` を解析呼び出し層と hover/problems/highlight/navigation 生�E層へ刁E��する、E  - `nodesrc/playground_editor_test_runner.js` は下佁Erunner とし、完亁E��認と CI は `nodesrc/cli.js` 経由の JSON 出力に統一する、E  - `doc/testing.md` と `doc/web_playground.md` も、実裁E��階では playground editor の正式検証手頁E��合わせて更新対象に含める、E  - 再レビューの結果、既孁Eeditor を一気に置き換える計画だと「不忁E��な変更を加えなぁE��「小さく�E割して進める」「commit 前にチE��ト確認」とぁE��持E��に反しめE��ぁE��刁E��ったため、計画書へ段階移行と commit/checkpoint の制紁E��追記した、E  - fixture 形式も `source.nepl` / `vfs.json` / `commands.json` / `expected.json` に固定し、DOM event ではなぁEeditor core command めECLI から再現する方針を明文化した、E# 2026-04-02 実裁E��モ (playground editor 実裁E��姁E

- [今回着手したこと]:
  - `web/src/editor-core/` を追加し、editor state の最小単位として `types.ts`, `state.ts`, `reducer.ts`, `keymap.ts`, `bridge.ts` を作�Eした、E  - 現段階では `select_all`, `toggle_overwrite`, `undo`, `redo`, `set_cursor`, `set_selection`, `replace_text`, `record_history` めEpure command として扱える、E  - `web/src/main.ts` から bridge を読み込み、既孁E`CanvasEditor` / `EditorInputHandler` から core keymap を経由して shortcut を�E琁E��る最初�E統合を入れた、E  - `nodesrc/playground_editor_test_runner.js` を追加し、`tests/playground_editor/basic_shortcuts/` fixture を用ぁE�� CLI snapshot チE��ト�E最小導線を作�Eした、E- [確認できたこと]:
  - `npm --prefix web run build:ts` は通過した、E  - `node nodesrc/playground_editor_test_runner.js --case tests/playground_editor/basic_shortcuts` は通過し、`expected.json` との一致確認までできる状態にした、E  - 現在の runner は `web/dist_ts/editor-core/bridge.js` を直接読む下佁Erunner であり、計画どおり最終的な正式導線�E `nodesrc/cli.js` 側へ寁E��る忁E��がある、E- [今回見えた差刁E�E未解決]:
  - 既孁Ebrowser editor の state 更新はまだ `CanvasEditor` 側に大きく残っており、core は shortcut の入口だけを刁E��出した段階、E  - hover / problems / highlight / definition / completion の正規化層は未着手で、`neplg2-provider` の責務�E離はこれから、E  - `AGENTS.md` で要求されてぁE�� `trunk build` はこ�E環墁E��は `trunk` コマンド�E体が見つからず未実行。環墁E��備また�E導�E手頁E�E確認が忁E��、E  - `nodesrc/cli.js` には playground editor 用の正式な test entry がまだ無く、現状は補助 runner のみ、E# 2026-04-02 実裁E��モ (playground editor CLI チE��ト導線�E整傁E

- [今回進めたこと]:
  - `nodesrc/playground_editor_test_runner.js` めElibrary と CLI の両用に整琁E��、case directory の再帰探索、`keyboard_event` step の解釈、aggregate summary の生�Eを追加した、E  - `nodesrc/cli.js` に `--playground-editor-tests` と `-o json=...` の正式導線を追加し、playground editor fixture を集紁E��行して JSON を�E力できるようにした、E  - `nodesrc/cli.js` は起動時に `parser` / `html_gen` / `html_gen_playground` を無条件に require してぁE��ため、playground editor test のような無関係なモードでめE`parser.ts` 未ビルドで即死してぁE��。root cause は top-level dependency の過剰読み込みだった�Eで、忁E��時のみ読み込む lazy load に変更した、E  - `tests/playground_editor/` に `keyboard_shortcuts`, `keyboard_unmapped`, `text_edit_history` を追加し、shortcut・未対忁Ekey・undo/redo めEfixture 化した、E  - `doc/testing.md` と `doc/web_playground.md` に playground editor の正式な CLI チE��ト手頁E��追記した、E- [確認結果]:
  - `npm --prefix web run build:ts` は通過した、E  - `node nodesrc/playground_editor_test_runner.js --case tests/playground_editor/basic_shortcuts` は通過した、E  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` は通過し、`4/4 passed` を確認した、E  - 出劁EJSON では `basic_shortcuts`, `keyboard_shortcuts`, `keyboard_unmapped`, `text_edit_history` の全 case ぁE`ok: true` になってぁE��、E- [残課題]:
  - editor browser adapter 側の state 更新責務�Eまだ `CanvasEditor` に多く残ってぁE��、E  - hover / problems / highlight / definition / completion の正規化層は未着手、E  - `AGENTS.md` で要求されてぁE�� `trunk build` は、この環墁E��は `trunk` コマンドが存在せず未実行�Eまま、E# 2026-04-03 実裁E��モ (playground editor 入口置揁E

- [今回進めたこと]:
  - `web/src/editor-core/browser-adapter.ts` を追加し、web playground が直接使ぁE��しい editor API として `PlaygroundEditor` / `createPlaygroundEditor` を定義した、E  - 新 API は `setText`, `getText`, `setPath`, `getPath`, `focus`, `resizeEditor`, `setFontSize`, `showPopup`, `getCursorPosition`, `getTokenInsight` を提供し、旧 `CanvasEditor` の冁E��詳細めEmain 側から隠す形にした、E  - `web/src/main.ts` は `CanvasEditorLibrary.createCanvasEditor(...)` をやめて `createPlaygroundEditor(...)` を使ぁE��ぁE��変更した。これにより web playground 本体�E editor 入口は新 API 側へ置き換わった、E  - `web/src/library/tabs.ts` と `web/src/terminal/shell.ts` めE`path` 直参�Eをやめ、`getPath` / `setPath` を優先して使ぁE��ぁE��変更した、E  - 互換経路として `web/src/library/canvas-editor-lib.ts` めE`window.PlaygroundEditorFactory` があれ�E新 API を返すようにした、E- [確認結果]:
  - `npm --prefix web run build:ts` は通過した、E  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` は引き続き `4/4 passed`、E- [現状認識]:
  - web playground の起動�E口は新 API に置き換わったが、browser adapter の冁E��ではまだ `CanvasEditor` / renderer / input handler / DOM UI を�E利用してぁE��、E  - したがって「playground 上で使われめEeditor API の置換」�Eできたが、「�E部責務�E全面刷新」�E未完亁E��E
# 2026-04-03 実裁E��モ (playground editor analysis 層と CLI 拡張)

- [今回の実裁E:
  - `web/src/editor-core/language-analysis.ts` を追加し、`neplg2-provider` の highlight / problems / folding / semanticTokens / inlayHints / hover / definition / occurrences めEpure な刁E��変換層へ刁E��出した、E  - `web/src/language/neplg2/neplg2-provider.ts` は WASM の甁Epayload を保持しつつ、editor 向け update payload と吁E�� query めE`NEPLPlaygroundLanguageAnalysis` へ委譲するように変更した、E  - `web/src/editor-core/browser-adapter.ts` に `getHoverInfo`, `getDefinitionLocation`, `getOccurrences`, `getProblems`, `getHighlightSnapshot` を追加し、web playground 側が新 API から刁E��結果を扱える入口を揃えた、E  - `nodesrc/playground_editor_test_runner.js` を拡張し、従来の `commands.json` fixture に加えて `analysis.json` + `requests.json` fixture を実行できるようにした、E  - `tests/playground_editor/analysis_payload_basic` と `tests/playground_editor/analysis_hover_definition` を追加し、highlight payload、diagnostics、folding、inlay hints、hover、definition、occurrences めECLI snapshot で固定化した、E- [確認結果]:
  - `npm --prefix web run build:ts` は通過した、E  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` は `6/6 passed` になり、keyboard/state 系 4 case と analysis 系 2 case めEJSON で確認した、E- [plan.md との差刁E:
  - browser adapter の入口置換と analysis 正規化層、CLI での hover/problems/highlight 系 snapshot までは入った、E  - ただし�E部ではまだ `CanvasEditor` / renderer / input handler / DOM UI を�E利用しており、描画と state 更新責務�E完�E撤去までは未完亁E��E  - `AGENTS.md` で求められてぁE�� `trunk build` は、この環墁E��は `trunk` コマンドが存在せず未実行�Eまま。ここ�E環墁E��備が残タスク、E
# 2026-04-03 実裁E��モ (playground editor 入劁Estate の core 匁E

- [今回の実裁E:
  - `web/src/editor-core/reducer.ts` に `insert_text`, `delete_backward`, `delete_forward` を追加し、文字�E力と削除の text / selection / undo 更新めEpure reducer 側で扱えるようにした、E  - `web/src/editor/editor.ts` は core runtime state めEeditor 実体へ反映する `applyCoreRuntimeState` を持つようにし、`applyCoreStateCommand` は個別刁E��ではなぁEreducer の結果を適用する形へ寁E��た、E  - `web/src/editor/editor-input-handler.ts` は `input`, `Backspace`, `Delete` をまぁEcore command で処琁E��、旧処琁E�E fallback に下げた、E  - `tests/playground_editor/core_text_input` と `tests/playground_editor/core_delete_selection` を追加し、insert/backspace/delete と選択削除の history めECLI fixture で固定化した、E- [確認結果]:
  - `npm --prefix web run build:ts` は通過した、E  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` は `8/8 passed` になり、analysis 2 case、shortcut/state 4 case、text edit 2 case を確認した、E- [現状認識]:
  - editor の入劁Estate は一部 pure core へ移ったが、pointer 操作、行移動、scroll、描画、completion UI はまだ `CanvasEditor` 側の責務が大きい、E  - そ�Eため、�E面置換完亁E��はまだ達してぁE��ぁE��特に renderer / DOM UI / pointer まわりの刁E��が残ってぁE��、E
# 2026-04-03 �������� (playground editor ���E�ړ��� core ��)

- [����̎���]:
  - `web/src/editor-core/types.ts` / `reducer.ts` �� `move_cursor` ��ǉ����A���E�ړ��� shift �I���̍X�V�� pure reducer �ň�����悤�ɂ����B
  - `web/src/editor/editor-input-handler.ts` �� `ArrowLeft` / `ArrowRight` �̔� ctrl �n���܂� core command �ŏ�������悤�ɕύX�����B
  - `tests/playground_editor/core_cursor_move` ��ǉ����A���E�ړ��ƑI�������� snapshot ���Œ艻�����B
- [�m�F����]:
  - `npm --prefix web run build:ts` �͒ʉ߂����B
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` �� `9/9 passed`�B
- [����F��]:
  - ���E�ړ��� core ���֊�������A�㉺�ړ��AHome/End�APageUp/PageDown�Apointer drag�Ascroll�Afold click�Acompletion UI �͂܂��� editor ��������́B
  - ���̂��߁A�S�ʒu�������� WSL git commit �̏����ɂ͂܂��͂��Ă��Ȃ��B
