#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), 'utf8');
}

const stringSrc = read('stdlib/alloc/string.nepl');
const stringSearchSrc = read('stdlib/alloc/string/search.nepl');
const stringSearchCompareSrc = read('stdlib/alloc/string/search/compare.nepl');
const tokenSrc = read('stdlib/neplg2/core/syntax/token.nepl');
const lexerSrc = read('stdlib/neplg2/core/syntax/lexer.nepl');
const importSpecSrc = read('stdlib/neplg2/core/module/import_spec.nepl');
const moduleGraphSrc = read('stdlib/neplg2/core/module/graph.nepl');
const stdlibMapSrc = read('stdlib/neplg2/core/module/stdlib_map.nepl');

assert.match(
    stringSrc,
    /pub\s+#import\s+"\.\/string\/search"\s+as\s+\*/,
    'alloc/string.nepl must re-export string/search for offset-based scanners',
);

assert.match(
    stringSearchSrc,
    /pub\s+#import\s+"\.\/search\/compare"\s+as\s+\*/,
    'alloc/string/search.nepl must re-export compare helpers for offset-based scanners',
);

assert.match(
    stringSearchCompareSrc,
    /\bfn\s+str_starts_with_at\s+<\(str,i32,str\)->bool>/,
    'alloc/string/search/compare.nepl must own str_starts_with_at for offset-based scanners',
);

assert.match(
    stringSearchCompareSrc,
    /\bstr_eq_at\s+s\s+prefix\s+start\s+lp\s+0\b/,
    'str_starts_with_at must centralize the internal str_eq_at loop-index argument',
);

assert.doesNotMatch(
    lexerSrc,
    /\blet\s+ok_hash\b|\blet\s+ok_i\b|\blet\s+ok_n2\b/,
    'lexer must not hand-roll #indent byte comparisons',
);

assert.match(
    lexerSrc,
    /str_starts_with_at\s+source\s+start\s+directive/,
    'lex_starts_with_indent_directive must use str_starts_with_at',
);

assert.match(
    lexerSrc,
    /str_starts_with_at\s+source\s+start\s+"#if\[target="/,
    'lexer must use str_starts_with_at for #if[target=',
);

assert.match(
    lexerSrc,
    /str_starts_with_at\s+source\s+start\s+"#if\[profile="/,
    'lexer must use str_starts_with_at for #if[profile=',
);

assert.doesNotMatch(
    lexerSrc,
    /string::str_eq_at/,
    'self-host lexer must not call internal-style str_eq_at directly',
);

assert.match(
    tokenSrc,
    /struct\s+SelfhostToken:[\s\S]*kind\s+<TokenKind>[\s\S]*span\s+<SelfhostSourceSpan>/,
    'SelfhostToken must keep only copyable token identity and source span',
);

assert.doesNotMatch(
    tokenSrc,
    /struct\s+SelfhostToken:[\s\S]*\n\s+lexeme\s+<str>/,
    'SelfhostToken must not store owned lexeme strings in Vec<SelfhostToken>',
);

assert.match(
    tokenSrc,
    /fn\s+selfhost_token_lexeme\s+<\(str,SelfhostToken\)->str>[\s\S]*string_slice::str_slice\s+source\s+field::get\s+span\s+"start"\s+field::get\s+span\s+"end"/,
    'token lexeme extraction must slice from source text at the owner boundary',
);

assert.match(
    lexerSrc,
    /fn\s+lex_ident_or_keyword_token\s+<\(str,i32,i32,i32\)->SelfhostToken>[\s\S]*let\s+lexeme\s+<str>\s+string_slice::str_slice\s+source\s+start\s+end[\s\S]*lex_consume_temp_str\s+lexeme;[\s\S]*lex_token_slice\s+kind\s+source\s+file_id\s+start\s+end/,
    'identifier keyword classification must consume temporary lexeme strings and store only token ranges',
);

assert.doesNotMatch(
    lexerSrc,
    /selfhost_token_new\s+kind\s+source_span_new\s+file_id\s+start\s+end\s+string_slice::str_slice/,
    'lexer must not pass owned source slices into SelfhostToken construction',
);

assert.match(
    lexerSrc,
    /fn\s+lex_stack_drop_top[\s\S]*let\s+stack_storage\s+<VecStorageState>[\s\S]*let\s+stack_data\s+<MemPtr<i32>>\s+field::get\s+stack\s+"data"[\s\S]*Vec<i32>\s+sub\s+stack_len\s+1\s+stack_cap\s+stack_storage\s+stack_data/,
    'lex_stack_drop_top must preserve Vec storage state and move the data owner into the returned Vec',
);

assert.doesNotMatch(
    lexerSrc,
    /Vec<i32>\s+sub\s+stack_len\s+1\s+stack_cap\s+stack_data/,
    'lex_stack_drop_top must not use the obsolete four-field Vec constructor',
);

assert.match(
    importSpecSrc,
    /str_starts_with_at\s+s\s+idx\s+"as"/,
    'import spec parser must use str_starts_with_at for the as keyword',
);

assert.match(
    importSpecSrc,
    /struct\s+SelfhostImportSpec:[\s\S]*item_index\s+<i32>[\s\S]*path_start\s+<i32>[\s\S]*path_end\s+<i32>[\s\S]*alias_start\s+<i32>[\s\S]*alias_end\s+<i32>/,
    'SelfhostImportSpec must store copy-only lexeme ranges instead of owned path/alias strings',
);

assert.doesNotMatch(
    importSpecSrc,
    /struct\s+SelfhostImportSpec:[\s\S]*\n\s+(path|alias)\s+<str>/,
    'SelfhostImportSpec must not store owned str fields in Vec<SelfhostImportSpec>',
);

assert.match(
    importSpecSrc,
    /fn\s+selfhost_import_spec_path\s+<\(str,SelfhostImportSpec\)->str>[\s\S]*string_slice::str_slice\s+lexeme\s+spec\.path_start\s+spec\.path_end/,
    'import spec path extraction must slice from the source lexeme at the owner boundary',
);

assert.match(
    importSpecSrc,
    /fn\s+selfhost_import_spec_alias\s+<\(str,SelfhostImportSpec\)->str>[\s\S]*string_slice::str_slice\s+lexeme\s+spec\.alias_start\s+spec\.alias_end/,
    'import spec alias extraction must slice from the source lexeme at the owner boundary',
);

assert.match(
    moduleGraphSrc,
    /fn\s+selfhost_module_graph_visit_imports\s+<\(&SelfhostVirtualFileSystem,str,&SelfhostModuleAst,&Vec<SelfhostImportSpec>/,
    'module graph import traversal must keep the AST alive while resolving range-only import specs',
);

assert.match(
    moduleGraphSrc,
    /selfhost_module_graph_visit_imports\s+vfs\s+path\s+&ast\s+&imports/,
    'module graph must pass the AST into import traversal instead of freeing it before path slicing',
);

assert.match(
    moduleGraphSrc,
    /selfhost_import_spec_path\s+item\.lexeme\s+spec/,
    'module graph must slice import paths from the original module item lexeme',
);

assert.match(
    stdlibMapSrc,
    /fn\s+selfhost_module_path_resolve_import_spec\s+<\(&SelfhostModulePathMap,str,str,SelfhostImportSpec\)/,
    'stdlib path map import-spec resolver must receive the source lexeme for range-only specs',
);

console.log('selfhost string helper boundary regression passed');
