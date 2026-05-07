#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/platforms/wasix/tui.nepl',
    'stdlib/platforms/wasix/tui/tty.nepl',
    'stdlib/platforms/wasix/tui/ansi.nepl',
    'stdlib/platforms/wasix/tui/text.nepl',
    'stdlib/platforms/wasix/tui/style.nepl',
    'stdlib/platforms/wasix/tui/box.nepl',
    'stdlib/platforms/wasix/tui/buffer.nepl',
];

const sources = Object.fromEntries(relPaths.map((relPath) => [
    relPath,
    fs.readFileSync(path.join(repoRoot, relPath), 'utf8'),
]));

const stripComments = (src) => src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const rootRelPath = 'stdlib/platforms/wasix/tui.nepl';
const ttyRelPath = 'stdlib/platforms/wasix/tui/tty.nepl';
const ansiRelPath = 'stdlib/platforms/wasix/tui/ansi.nepl';
const textRelPath = 'stdlib/platforms/wasix/tui/text.nepl';
const styleRelPath = 'stdlib/platforms/wasix/tui/style.nepl';
const boxRelPath = 'stdlib/platforms/wasix/tui/box.nepl';
const bufferRelPath = 'stdlib/platforms/wasix/tui/buffer.nepl';

const codeByPath = Object.fromEntries(Object.entries(sources).map(([relPath, src]) => [relPath, stripComments(src)]));
const code = Object.values(codeByPath).join('\n');
const rootCode = codeByPath[rootRelPath];
const ttyCode = codeByPath[ttyRelPath];
const ansiCode = codeByPath[ansiRelPath];
const textCode = codeByPath[textRelPath];
const styleCode = codeByPath[styleRelPath];
const boxCode = codeByPath[boxRelPath];

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    for (const [relPath, fileCode] of Object.entries(codeByPath)) {
        assert.doesNotMatch(fileCode, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
    }
}

for (const submodule of ['tty', 'ansi', 'text', 'style', 'box', 'buffer']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"platforms/wasix/tui/${submodule}"\\s+as\\s+@merge`),
        `tui root facade must re-export ${submodule} submodule`,
    );
}
assert.doesNotMatch(rootCode, /\b(fn|struct|enum)\s+\w+/, 'tui root facade must not regain implementation bodies');

assert.match(textCode, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'wasix tui text module must qualify implementation Vec allocation calls');
assert.match(ttyCode, /fn\s+get_tty_state_result\s+<\(\)\*>Result<i32,i32>>\s+\(\):[\s\S]*Result<i32,i32>::Ok\s+ptr[\s\S]*Result<i32,i32>::Err\s+errno/, 'TTY state acquisition must return Result instead of a freed-pointer sentinel');
assert.doesNotMatch(ttyCode, /fn\s+get_tty_state\s+<\(\)\*>i32>/, 'TTY state acquisition must not return raw i32 sentinel values');
assert.match(ansiCode, /fn\s+set_fg_color\s+<\(AnsiColor\)\*>\(\)>\s+\(color\):[\s\S]*print\s+ansi_color_code\s+color/, 'set_fg_color must use typed AnsiColor conversion');
assert.match(ansiCode, /fn\s+set_bg_color\s+<\(AnsiColor\)\*>\(\)>\s+\(color\):[\s\S]*print\s+ansi_background_color_code\s+color/, 'set_bg_color must use typed AnsiColor conversion');
assert.match(styleCode, /fn\s+style_text\s+<\(AnsiTextStyle,str\)\*>str>\s+\(style,\s*s\):[\s\S]*ansi_text_style_code\s+style[\s\S]*ansi_reset_code/, 'style_text must use typed AnsiTextStyle conversion');
assert.match(boxCode, /fn\s+line_box_styled\s+<\(AnsiTextStyle,str,i32\)\*>str>\s+\(style,\s*content,\s*cols\):[\s\S]*style_text\s+style\s+body/, 'line_box_styled must accept typed AnsiTextStyle instead of numeric color codes');
assert.doesNotMatch(code, /fn\s+(?:set_fg_color|set_bg_color)\s+<\(i32\)\*>/, 'TUI color setters must not accept raw i32 ANSI color codes');
assert.doesNotMatch(code, /fn\s+style_text\s+<\(i32,i32,str\)\*>/, 'style_text must not accept raw i32 foreground/background codes');
assert.doesNotMatch(code, /fn\s+line_box_styled\s+<\(i32,i32,str,i32\)\*>/, 'line_box_styled must not accept raw i32 foreground/background codes');
assert.match(textCode, /fn\s+tui_empty_str_vec\s+<\(\)->Vec<str>>\s+\(\):\s+v::vec_empty<str>/, 'text_wrap_lines allocation fallback must use typed empty Vec storage');
assert.match(textCode, /fn\s+tui_push_str\s+<\(Vec<str>,str\)->TuiStrPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<str>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*TuiStrPushRes\s+tui_empty_str_vec\s+false/, 'text_wrap_lines push must convert grow failure to ok=false');
assert.match(textCode, /fn\s+text_wrap_lines\s+<\(str,i32\)\*>Vec<str>>\s+\(text,\s*cols\):[\s\S]*match\s+v::new<str>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'text_wrap_lines must handle Vec allocation failure');
assert.match(textCode, /while\s+and\s+lt\s+i\s+n\s+not\s+failed:/, 'text_wrap_lines must stop scanning after line accumulation failure');
assert.match(textCode, /let\s+pushed_tail\s+<TuiStrPushRes>\s+tui_push_str\s+out\s+tail/, 'text_wrap_lines tail accumulation must go through checked push');

console.log('stdlib wasix tui unsafe unwrap regression passed');
