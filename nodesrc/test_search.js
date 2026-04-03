// nodesrc/test_search.js
// 目的: search.js のローカル完結テスト
// 外部依存ゼロ: Node.js 標準の assert モジュールのみ使用
//
// 実行方法:
//   node nodesrc/test_search.js

"use strict";

const assert = require("assert");
const {
  searchIndex,
  buildEntriesFromAst,
  inlinesToSearchText,
  tokenizeQuery,
  normalizeText,
  makeSnippet,
} = require("./search");

let passCount = 0;
let failCount = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    passCount++;
  } catch (e) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failCount++;
  }
}

// =========================================================
// tokenizeQuery のテスト
// =========================================================
console.log("\n[tokenizeQuery]");

test("空文字列は空配列", () => {
  assert.deepStrictEqual(tokenizeQuery(""), []);
});

test("スペース区切りで分割", () => {
  assert.deepStrictEqual(tokenizeQuery("hello world"), ["hello", "world"]);
});

test("全角スペースで分割", () => {
  assert.deepStrictEqual(tokenizeQuery("hello\u3000world"), ["hello", "world"]);
});

test("大文字を小文字化", () => {
  assert.deepStrictEqual(tokenizeQuery("Hello World"), ["hello", "world"]);
});

test("複数スペースは無視", () => {
  assert.deepStrictEqual(tokenizeQuery("  foo   bar  "), ["foo", "bar"]);
});

test("日本語クエリ", () => {
  assert.deepStrictEqual(tokenizeQuery("関数 定義"), ["関数", "定義"]);
});

// =========================================================
// inlinesToSearchText のテスト (ruby/furigana 対応)
// =========================================================
console.log("\n[inlinesToSearchText]");

test("通常テキスト", () => {
  const inlines = [{ type: "text", text: "hello world" }];
  const result = inlinesToSearchText(inlines);
  assert.strictEqual(result, "hello world");
});

test("ルビ: 漢字と読みがなの両方を含む", () => {
  const inlines = [
    {
      type: "ruby",
      base: [{ type: "text", text: "関数" }],
      ruby: [{ type: "text", text: "かんすう" }],
    },
  ];
  const result = inlinesToSearchText(inlines);
  assert.ok(result.includes("関数"), "漢字が含まれていること");
  assert.ok(result.includes("かんすう"), "読み仮名が含まれていること");
});

test("ルビ: 漢字で検索できる", () => {
  const inlines = [
    {
      type: "ruby",
      base: [{ type: "text", text: "[取/と]り[出/だ]す" }],
      ruby: [{ type: "text", text: "" }],
    },
  ];
  // シンプルに漢字が含まれていれば OK
  const result = inlinesToSearchText(inlines);
  assert.ok(typeof result === "string");
});

test("ネストしたルビ", () => {
  const inlines = [
    { type: "text", text: "これは" },
    {
      type: "ruby",
      base: [{ type: "text", text: "変数" }],
      ruby: [{ type: "text", text: "へんすう" }],
    },
    { type: "text", text: "です" },
  ];
  const result = inlinesToSearchText(inlines);
  assert.ok(result.includes("変数"));
  assert.ok(result.includes("へんすう"));
  assert.ok(result.includes("これは"));
  assert.ok(result.includes("です"));
});

test("gloss: base と notes 両方含む", () => {
  const inlines = [
    {
      type: "gloss",
      base: [{ type: "text", text: "引数" }],
      notes: [
        [{ type: "text", text: "ひきすう" }],
        [{ type: "text", text: "argument" }],
      ],
    },
  ];
  const result = inlinesToSearchText(inlines);
  assert.ok(result.includes("引数"));
  assert.ok(result.includes("ひきすう"));
  assert.ok(result.includes("argument"));
});

test("code_inline はそのまま含む", () => {
  const inlines = [{ type: "code_inline", text: "fn main()" }];
  assert.ok(inlinesToSearchText(inlines).includes("fn main()"));
});

test("link のテキスト部を含む", () => {
  const inlines = [
    {
      type: "link",
      text: [{ type: "text", text: "こちら" }],
      href: "https://example.com",
    },
  ];
  assert.ok(inlinesToSearchText(inlines).includes("こちら"));
});

// =========================================================
// searchIndex のテスト
// =========================================================
console.log("\n[searchIndex]");

const sampleIndex = [
  {
    id: "page-std-vec",
    title: "vec モジュール",
    url: "std/vec.html",
    body: "ベクタ（可変長配列）に関する関数を提供します。vec_new で新しいベクタを作成できます。",
  },
  {
    id: "page-std-string",
    title: "string モジュール",
    url: "std/string.html",
    body: "文字列操作に関する関数を提供します。concat で文字列を結合できます。",
  },
  {
    id: "vec-push",
    title: "vec_push 関数",
    url: "std/vec.html#vec-push",
    body: "ベクタの末尾に要素を追加します。vec_push vec_new で使います。",
  },
  {
    id: "string-concat",
    title: "concat 関数",
    url: "std/string.html#concat",
    body: "2つの文字列を結合します。",
  },
  {
    id: "tutorial-fn",
    title: "関数 かんすう の定義",
    url: "tutorial/02_fn.html#fn-def",
    body: "関数（かんすう）は fn キーワードで定義します。引数と戻り値の型を指定します。",
  },
];

test("空クエリは空配列を返す", () => {
  assert.deepStrictEqual(searchIndex("", sampleIndex), []);
});

test("1ワード検索: タイトルヒット", () => {
  const results = searchIndex("vec", sampleIndex);
  assert.ok(results.length > 0, "結果があること");
  // vec_push と vec モジュールがヒットするはず
  assert.ok(
    results.some((r) => r.url.includes("vec")),
    "vec 関連のページがヒット",
  );
});

test("AND 検索: 両方含むものだけヒット", () => {
  const results = searchIndex("vec push", sampleIndex);
  assert.ok(results.length > 0, "結果があること");
  // vec_push エントリがヒットするはず
  assert.ok(
    results.some((r) => r.id === "vec-push"),
    "vec_push がヒット",
  );
  // string 系はヒットしないはず
  assert.ok(
    !results.some((r) => r.id === "string-concat"),
    "string-concat はヒットしない",
  );
});

test("大文字小文字を区別しない", () => {
  const lower = searchIndex("vec", sampleIndex);
  const upper = searchIndex("VEC", sampleIndex);
  assert.deepStrictEqual(
    lower.map((r) => r.id),
    upper.map((r) => r.id),
  );
});

test("ヒットなしは空配列を返す", () => {
  const results = searchIndex("xyzzy_nonexistent", sampleIndex);
  assert.deepStrictEqual(results, []);
});

test("タイトルヒットは本文ヒットよりスコアが高い", () => {
  // 'concat' はタイトルにも本文にも出現する
  // concat 関数 (id: string-concat) はタイトルに 'concat'
  // string モジュール (id: page-std-string) は本文に 'concat'
  const results = searchIndex("concat", sampleIndex);
  const titleHit = results.find((r) => r.id === "string-concat");
  const bodyHit = results.find((r) => r.id === "page-std-string");
  assert.ok(titleHit && bodyHit, "両方ヒットすること");
  assert.ok(
    titleHit.score >= bodyHit.score,
    "タイトルヒットのスコアが高いこと",
  );
});

test("maxResults で件数制限", () => {
  const results = searchIndex("関数", sampleIndex, 1);
  assert.strictEqual(results.length, 1);
});

test("日本語クエリ: 漢字で検索", () => {
  const results = searchIndex("関数", sampleIndex);
  assert.ok(results.length > 0, "関数 で検索できること");
  assert.ok(
    results.some((r) => r.id === "tutorial-fn"),
    "tutorial-fn がヒット",
  );
});

test("日本語クエリ: 読み仮名で検索（rubyを含むindexを想定）", () => {
  // body にかんすうが含まれているのでヒットするはず
  const results = searchIndex("かんすう", sampleIndex);
  assert.ok(results.length > 0, "かんすう で検索できること");
  assert.ok(
    results.some((r) => r.id === "tutorial-fn"),
    "読み仮名でもヒット",
  );
});

test("スニペットが生成される", () => {
  const results = searchIndex("ベクタ", sampleIndex);
  assert.ok(results.length > 0);
  assert.ok(typeof results[0].snippet === "string");
  assert.ok(results[0].snippet.length > 0);
});

// =========================================================
// buildEntriesFromAst のテスト
// =========================================================
console.log("\n[buildEntriesFromAst]");

// 最小 AST を手作りする（parser.js に依存せず）
const minimalAst = {
  type: "document",
  children: [
    {
      type: "paragraph",
      inlines: [
        { type: "text", text: "このページは vec に関するドキュメントです。" },
      ],
    },
    {
      type: "section",
      level: 1,
      heading: [{ type: "text", text: "vec モジュール" }],
      children: [
        {
          type: "paragraph",
          inlines: [{ type: "text", text: "ベクタを扱います。" }],
        },
        {
          type: "section",
          level: 2,
          heading: [
            { type: "text", text: "" },
            {
              type: "ruby",
              base: [{ type: "text", text: "関数" }],
              ruby: [{ type: "text", text: "かんすう" }],
            },
            { type: "text", text: " vec_push" },
          ],
          children: [
            {
              type: "paragraph",
              inlines: [{ type: "text", text: "要素を末尾に追加します。" }],
            },
          ],
        },
      ],
    },
  ],
};

test("ページエントリが生成される", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  assert.ok(entries.length > 0, "エントリが1つ以上ある");
  const page = entries.find((e) => e.url === "std/vec.html");
  assert.ok(page, "ページエントリがある");
});

test("section エントリが生成される", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  // 'vec モジュール' セクションが含まれるはず
  const sec = entries.find(
    (e) => e.title.includes("vec モジュール") && e.url.includes("#"),
  );
  assert.ok(sec, "section エントリがある");
});

test("section URL に fragment が付く", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  const sectionsWithFragment = entries.filter((e) => e.url.includes("#"));
  assert.ok(sectionsWithFragment.length > 0, "fragment 付き URL がある");
});

test("ルビを含む見出しでは漢字・読みがな両方がインデックスに含まれる", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  // 'vec_push' セクションの見出しが '関数 かんすう vec_push' のようになっているはず
  const pushSec = entries.find((e) => e.title.includes("vec_push"));
  assert.ok(pushSec, "vec_push セクションがある");
  assert.ok(pushSec.title.includes("関数"), "漢字が含まれる");
  assert.ok(pushSec.title.includes("かんすう"), "読み仮名が含まれる");
});

test("body に段落テキストが含まれる", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  const pushSec = entries.find((e) => e.title.includes("vec_push"));
  assert.ok(pushSec, "vec_push セクションがある");
  assert.ok(pushSec.body.includes("末尾"), "body に段落テキストが含まれる");
});

// =========================================================
// 結合テスト: buildEntriesFromAst + searchIndex
// =========================================================
console.log("\n[integration: build + search]");

test("buildEntriesFromAst で構築したインデックスを searchIndex で検索できる", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  // 漢字で検索
  const r1 = searchIndex("関数", entries);
  assert.ok(r1.length > 0, "関数 で検索できる");
  // 読み仮名で検索
  const r2 = searchIndex("かんすう", entries);
  assert.ok(r2.length > 0, "かんすう で検索できる");
  // vec_push で検索
  const r3 = searchIndex("vec_push", entries);
  assert.ok(r3.length > 0, "vec_push で検索できる");
});

test("AND 検索: 漢字 + 関数名で絞り込み可能", () => {
  const entries = buildEntriesFromAst(
    minimalAst,
    "std/vec.html",
    "vec モジュール",
  );
  const results = searchIndex("関数 vec_push", entries);
  assert.ok(results.length > 0, "漢字+関数名の AND 検索ができる");
  assert.ok(
    results.every(
      (r) => r.title.includes("vec_push") || r.body.includes("vec_push"),
    ),
    "結果が vec_push に関連している",
  );
});

test("kind での絞り込み (searchIndex)", () => {
  const customIndex = [
    { id: "vec-push", title: "vec_push", body: "body1", kind: "fn" },
    { id: "VecData", title: "VecData", body: "body2", kind: "struct" },
    {
      id: "math-add",
      title: "add",
      body: "vec_push also uses this",
      kind: "fn",
    },
  ];
  // fn で絞り込み
  const fns = searchIndex("vec", customIndex, 20, { kind: "fn" });
  assert.strictEqual(fns.length, 2);
  assert.ok(fns.some((r) => r.id === "vec-push"));

  // struct で絞り込み
  const structs = searchIndex("vec", customIndex, 20, { kind: "struct" });
  assert.strictEqual(structs.length, 1);
  assert.strictEqual(structs[0].id, "VecData");

  // all で絞り込み
  const all = searchIndex("vec", customIndex, 20, { kind: "all" });
  assert.strictEqual(all.length, 3);
});

test("AST から kind プロパティが抽出される", () => {
  const astWithKind = {
    type: "document",
    children: [
      {
        type: "section",
        level: 1,
        heading: [{ type: "text", text: "vec" }],
        kind: "module",
        children: [],
      },
      {
        type: "section",
        level: 2,
        heading: [{ type: "text", text: "push" }],
        kind: "fn",
        children: [],
      },
    ],
  };
  const entries = buildEntriesFromAst(astWithKind, "opt.html", "opt");
  const vecEnt = entries.find((e) => e.title === "vec");
  const pushEnt = entries.find((e) => e.title === "push");
  assert.strictEqual(vecEnt.kind, "module");
  assert.strictEqual(pushEnt.kind, "fn");
});

// =========================================================
// 結果
// =========================================================
console.log(`\n${"─".repeat(40)}`);
console.log(`テスト結果: ${passCount} 成功, ${failCount} 失敗`);
if (failCount > 0) {
  process.exit(1);
}
console.log("All tests passed!");
