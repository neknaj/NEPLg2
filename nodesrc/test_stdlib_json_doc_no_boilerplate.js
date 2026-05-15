#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const files = {
    facade: 'stdlib/alloc/encoding/json.nepl',
    types: 'stdlib/alloc/encoding/json/types.nepl',
    builders: 'stdlib/alloc/encoding/json/builders.nepl',
    access: 'stdlib/alloc/encoding/json/access.nepl',
    escape: 'stdlib/alloc/encoding/json/escape.nepl',
    serialize: 'stdlib/alloc/encoding/json/serialize.nepl',
};

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const srcByFile = Object.fromEntries(Object.entries(files).map(([key, relPath]) => [key, read(relPath)]));
const combinedSrc = Object.values(srcByFile).join('\n');
const stdTestReportSrc = read('stdlib/std/test/report.nepl');

const forbiddenPhrases = [
    ['generic main-use title', '\u4e3b\u306a\u7528\u9014'],
    ['predefined-process placeholder', '\u5b9a\u7fa9\u6e08\u307f\u51e6\u7406'],
    ['thin-wrapper placeholder', '\u8584\u3044\u30e9\u30c3\u30d1'],
    ['move-and-rebind placeholder', '\u518d\u5229\u7528\u6642\u306f\u675f\u7e1b\u3057\u76f4'],
    ['generic enum overview', '\u5217\u6319\u578b\u306e\u6982\u8981'],
];

for (const [label, phrase] of forbiddenPhrases) {
    assert.equal(combinedSrc.includes(phrase), false, `json modules must not contain generated doc boilerplate: ${label}`);
}

const requiredPhrases = [
    ['JsonValue variant contract', 'JsonValue: JSON \u306e\u5024\u8868\u73fe'],
    ['null constructor contract', 'json_null: JSON null \u5024\u3092\u4f5c\u308b'],
    ['bool constructor contract', 'json_bool: bool \u5024\u3092 JSON Bool \u306b\u5909\u63db\u3059\u308b'],
    ['array fragment copy contract', '`arr` \u306f Copy \u306a value fragment \u3067\u3059'],
    ['object fragment copy contract', '`obj` \u306f Copy \u306a value fragment \u3067\u3059'],
    ['bool accessor contract', 'json_as_bool: JSON Bool payload \u3092\u53d6\u308a\u51fa\u3059'],
    ['number accessor contract', 'json_as_number: JSON Number payload \u3092\u53d6\u308a\u51fa\u3059'],
    ['string accessor contract', 'json_as_string: JSON String payload \u3092\u53d6\u308a\u51fa\u3059'],
];

for (const [label, phrase] of requiredPhrases) {
    assert.equal(combinedSrc.includes(phrase), true, `json modules must document ${label}`);
}

assert.match(
    srcByFile.facade,
    /pub #import "alloc\/encoding\/json\/types" as \*[\s\S]*pub #import "alloc\/encoding\/json\/builders" as \*[\s\S]*pub #import "alloc\/encoding\/json\/access" as \*[\s\S]*pub #import "alloc\/encoding\/json\/escape" as \*[\s\S]*pub #import "alloc\/encoding\/json\/serialize" as \*/,
    `${files.facade} must stay a public facade over the responsibility modules`
);
assert.doesNotMatch(srcByFile.facade, /^(?:fn|enum|struct)\s/m, `${files.facade} must not reintroduce implementation bodies`);

for (const [key, src] of Object.entries(srcByFile)) {
    const relPath = files[key];
    const lineCount = implementationLineCount(src);
    if (key === 'facade') {
        assert.ok(lineCount <= 80, `${relPath} facade must stay small`);
    } else {
        assert.ok(lineCount <= 240, `${relPath} must stay below the stdlib split review limit`);
    }
}

for (const key of ['facade', 'types', 'builders', 'access', 'escape', 'serialize']) {
    assert.doesNotMatch(
        srcByFile[key],
        /\bmem_ptr_addr\b|\bload<[^>]+>/,
        `${files[key]} must not contain the raw Vec payload traversal boundary`
    );
}
for (const key of ['escape', 'serialize']) {
    assert.doesNotMatch(
        srcByFile[key],
        /#import "alloc\/string\/[^"]+" as \*/,
        `${files[key]} must not leak string helper overloads through the json facade`
    );
}
assert.match(
    srcByFile.serialize,
    /json_serialize_array[\s\S]*concat3 "\[" get items "items" "\]"/,
    `${files.serialize} must serialize arrays by wrapping the typed JsonArray fragment`,
);
assert.match(
    srcByFile.serialize,
    /json_serialize_object[\s\S]*concat3 "\{" get members "members" "\}"/,
    `${files.serialize} must serialize objects by wrapping the typed JsonObject fragment`,
);
assert.match(
    stdTestReportSrc,
    /#import "alloc\/encoding\/json\/escape" as json/,
    'std/test/report must import the exact JSON escape module for qualified json::json_quote_string calls'
);

console.log('stdlib json doc boilerplate regression passed');
