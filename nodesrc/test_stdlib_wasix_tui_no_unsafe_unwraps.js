#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/platforms/wasix/tui.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'wasix tui must qualify implementation Vec allocation calls');
assert.match(code, /fn\s+get_tty_state_result\s+<\(\)\*>Result<i32,i32>>\s+\(\):[\s\S]*Result<i32,i32>::Ok\s+ptr[\s\S]*Result<i32,i32>::Err\s+errno/, 'TTY state acquisition must return Result instead of a freed-pointer sentinel');
assert.doesNotMatch(code, /fn\s+get_tty_state\s+<\(\)\*>i32>/, 'TTY state acquisition must not return raw i32 sentinel values');
assert.match(code, /fn\s+set_fg_color\s+<\(AnsiColor\)\*>\(\)>\s+\(color\):[\s\S]*print\s+ansi_color_code\s+color/, 'set_fg_color must use typed AnsiColor conversion');
assert.match(code, /fn\s+set_bg_color\s+<\(AnsiColor\)\*>\(\)>\s+\(color\):[\s\S]*print\s+ansi_background_color_code\s+color/, 'set_bg_color must use typed AnsiColor conversion');
assert.match(code, /fn\s+style_text\s+<\(AnsiTextStyle,str\)\*>str>\s+\(style,\s*s\):[\s\S]*ansi_text_style_code\s+style[\s\S]*ansi_reset_code/, 'style_text must use typed AnsiTextStyle conversion');
assert.match(code, /fn\s+line_box_styled\s+<\(AnsiTextStyle,str,i32\)\*>str>\s+\(style,\s*content,\s*cols\):[\s\S]*style_text\s+style\s+body/, 'line_box_styled must accept typed AnsiTextStyle instead of numeric color codes');
assert.doesNotMatch(code, /fn\s+(?:set_fg_color|set_bg_color)\s+<\(i32\)\*>/, 'TUI color setters must not accept raw i32 ANSI color codes');
assert.doesNotMatch(code, /fn\s+style_text\s+<\(i32,i32,str\)\*>/, 'style_text must not accept raw i32 foreground/background codes');
assert.doesNotMatch(code, /fn\s+line_box_styled\s+<\(i32,i32,str,i32\)\*>/, 'line_box_styled must not accept raw i32 foreground/background codes');
assert.match(code, /fn\s+tui_empty_str_vec\s+<\(\)->Vec<str>>\s+\(\):\s+v::vec_empty<str>/, 'text_wrap_lines allocation fallback must use typed empty Vec storage');
assert.match(code, /fn\s+tui_push_str\s+<\(Vec<str>,str\)->TuiStrPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<str>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*TuiStrPushRes\s+tui_empty_str_vec\s+false/, 'text_wrap_lines push must convert grow failure to ok=false');
assert.match(code, /fn\s+text_wrap_lines\s+<\(str,i32\)\*>Vec<str>>\s+\(text,\s*cols\):[\s\S]*match\s+v::new<str>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'text_wrap_lines must handle Vec allocation failure');
assert.match(code, /while\s+and\s+lt\s+i\s+n\s+not\s+failed:/, 'text_wrap_lines must stop scanning after line accumulation failure');
assert.match(code, /let\s+pushed_tail\s+<TuiStrPushRes>\s+tui_push_str\s+out\s+tail/, 'text_wrap_lines tail accumulation must go through checked push');

console.log('stdlib wasix tui unsafe unwrap regression passed');
