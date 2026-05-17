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
    'stdlib/platforms/wasix/tui/text/repeat.nepl',
    'stdlib/platforms/wasix/tui/text/width.nepl',
    'stdlib/platforms/wasix/tui/text/line.nepl',
    'stdlib/platforms/wasix/tui/text/wrap.nepl',
    'stdlib/platforms/wasix/tui/style.nepl',
    'stdlib/platforms/wasix/tui/box.nepl',
    'stdlib/platforms/wasix/tui/buffer.nepl',
    'stdlib/platforms/wasix/tui/buffer/storage.nepl',
    'stdlib/platforms/wasix/tui/buffer/wrap.nepl',
    'stdlib/platforms/wasix/tui/buffer/present.nepl',
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
const textRepeatRelPath = 'stdlib/platforms/wasix/tui/text/repeat.nepl';
const textWidthRelPath = 'stdlib/platforms/wasix/tui/text/width.nepl';
const textLineRelPath = 'stdlib/platforms/wasix/tui/text/line.nepl';
const textWrapRelPath = 'stdlib/platforms/wasix/tui/text/wrap.nepl';
const styleRelPath = 'stdlib/platforms/wasix/tui/style.nepl';
const boxRelPath = 'stdlib/platforms/wasix/tui/box.nepl';
const bufferRelPath = 'stdlib/platforms/wasix/tui/buffer.nepl';
const bufferStorageRelPath = 'stdlib/platforms/wasix/tui/buffer/storage.nepl';
const bufferWrapRelPath = 'stdlib/platforms/wasix/tui/buffer/wrap.nepl';
const bufferPresentRelPath = 'stdlib/platforms/wasix/tui/buffer/present.nepl';

const codeByPath = Object.fromEntries(Object.entries(sources).map(([relPath, src]) => [relPath, stripComments(src)]));
const code = Object.values(codeByPath).join('\n');
const rootCode = codeByPath[rootRelPath];
const ttyCode = codeByPath[ttyRelPath];
const ansiCode = codeByPath[ansiRelPath];
const textCode = codeByPath[textRelPath];
const textRepeatCode = codeByPath[textRepeatRelPath];
const textWidthCode = codeByPath[textWidthRelPath];
const textLineCode = codeByPath[textLineRelPath];
const textWrapCode = codeByPath[textWrapRelPath];
const textFamilyCode = [textCode, textRepeatCode, textWidthCode, textLineCode, textWrapCode].join('\n');
const styleCode = codeByPath[styleRelPath];
const boxCode = codeByPath[boxRelPath];
const bufferCode = codeByPath[bufferRelPath];
const bufferStorageCode = codeByPath[bufferStorageRelPath];
const bufferWrapCode = codeByPath[bufferWrapRelPath];
const bufferPresentCode = codeByPath[bufferPresentRelPath];

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

for (const submodule of ['repeat', 'width', 'line', 'wrap']) {
    assert.match(
        textCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/text\\/${submodule}"\\s+as\\s+@merge`),
        `wasix tui text facade must re-export text/${submodule}`,
    );
}
assert.doesNotMatch(textCode, /\b(fn|struct|enum)\s+\w+/, 'wasix tui text facade must not regain implementation bodies');
assert.match(textWrapCode, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'wasix tui text wrap module must qualify implementation Vec allocation calls');
for (const submodule of ['storage', 'wrap', 'present']) {
    assert.match(
        bufferCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/buffer\\/${submodule}"\\s+as\\s+@merge`),
        `wasix tui buffer facade must re-export buffer/${submodule}`,
    );
}
assert.doesNotMatch(bufferCode, /\b(fn|struct|enum)\s+\w+/, 'wasix tui buffer facade must not regain implementation bodies');
assert.match(ttyCode, /pub\s+struct\s+TtyState:[\s\S]*region\s+<RegionToken<u8>>/, 'TTY state must carry its free obligation as a RegionToken owner');
assert.match(ttyCode, /fn\s+get_tty_state_result\s+<\(\)\*>Result<TtyState,i32>>\s+\(\):[\s\S]*alloc_region_bytes<u8>\s+24[\s\S]*Result<TtyState,i32>::Ok\s+state[\s\S]*Result<TtyState,i32>::Err\s+errno/, 'TTY state acquisition must return a typed owner instead of a raw pointer sentinel');
assert.match(ttyCode, /fn\s+tty_state_raw\s+<\(&TtyState\)->i32>\s+\(state\):\s+mem_ptr_addr\s+tty_state_ptr\s+state/, 'TTY raw address extraction must stay inside the TTY boundary helper');
assert.match(ttyCode, /pub\s+fn\s+enter_raw_mode\s+<\(\)\*>Result<TtyState,i32>>\s+\(\):/, 'enter_raw_mode must return a typed TtyState owner on success');
assert.match(ttyCode, /pub\s+fn\s+restore_mode\s+<\(TtyState\)\*>i32>\s+\(old_state\):/, 'restore_mode must consume the typed TtyState owner');
assert.doesNotMatch(ttyCode, /\b(?:alloc_raw|dealloc_raw)\b/, 'TTY state owner allocation must not use raw i32 allocation APIs');
assert.doesNotMatch(ttyCode, /fn\s+get_tty_state\s+<\(\)\*>i32>/, 'TTY state acquisition must not return raw i32 sentinel values');
assert.doesNotMatch(ttyCode, /pub\s+fn\s+enter_raw_mode\s+<\(\)\*>i32>|pub\s+fn\s+restore_mode\s+<\(i32\)\*>/, 'TTY raw mode APIs must not expose raw i32 state owners');
assert.match(ansiCode, /fn\s+set_fg_color\s+<\(AnsiColor\)\*>\(\)>\s+\(color\):[\s\S]*print\s+ansi_color_code\s+color/, 'set_fg_color must use typed AnsiColor conversion');
assert.match(ansiCode, /fn\s+set_bg_color\s+<\(AnsiColor\)\*>\(\)>\s+\(color\):[\s\S]*print\s+ansi_background_color_code\s+color/, 'set_bg_color must use typed AnsiColor conversion');
assert.match(styleCode, /fn\s+style_text\s+<\(AnsiTextStyle,str\)\*>str>\s+\(style,\s*s\):[\s\S]*ansi_text_style_code\s+style[\s\S]*ansi_reset_code/, 'style_text must use typed AnsiTextStyle conversion');
assert.match(boxCode, /fn\s+line_box_styled\s+<\(AnsiTextStyle,str,i32\)\*>str>\s+\(style,\s*content,\s*cols\):[\s\S]*style_text\s+style\s+body/, 'line_box_styled must accept typed AnsiTextStyle instead of numeric color codes');
assert.doesNotMatch(code, /fn\s+(?:set_fg_color|set_bg_color)\s+<\(i32\)\*>/, 'TUI color setters must not accept raw i32 ANSI color codes');
assert.doesNotMatch(code, /fn\s+style_text\s+<\(i32,i32,str\)\*>/, 'style_text must not accept raw i32 foreground/background codes');
assert.doesNotMatch(code, /fn\s+line_box_styled\s+<\(i32,i32,str,i32\)\*>/, 'line_box_styled must not accept raw i32 foreground/background codes');
assert.match(textFamilyCode, /enum\s+TuiTextByteKind:[\s\S]*Ascii[\s\S]*Utf8Len2[\s\S]*Utf8Len3[\s\S]*Utf8Len4[\s\S]*Invalid/, 'TUI text width scanning must classify UTF-8 byte kind as an enum');
assert.match(textWidthCode, /fn\s+tui_text_byte_len\s+<\(TuiTextByteKind\)->i32>\s+\(kind\):[\s\S]*match\s+kind:/, 'TUI text byte length must branch through enum match');
assert.match(textWidthCode, /fn\s+tui_text_byte_width\s+<\(TuiTextByteKind\)->i32>\s+\(kind\):[\s\S]*match\s+kind:/, 'TUI text byte width must branch through enum match');
assert.match(textWidthCode, /fn\s+str_display_width\s+<\(str\)\*>i32>\s+\(s\):[\s\S]*tui_skip_escape_sequence[\s\S]*tui_text_byte_kind/, 'str_display_width must share escape skipping and byte classification helpers');
assert.match(textLineCode, /fn\s+line_clip_to_cols\s+<\(str,i32\)\*>str>\s+\(s,\s*cols\):[\s\S]*tui_text_byte_kind[\s\S]*tui_text_byte_len[\s\S]*tui_text_byte_width/, 'line clipping must use shared TUI byte classification helpers');
assert.match(bufferStorageCode, /fn\s+buffer_new\s+<\(i32,i32\)\*>i32>\s+\(cols,\s*rows\):[\s\S]*alloc_raw[\s\S]*store<str>/, 'TUI buffer storage module must own raw slot allocation');
assert.match(bufferStorageCode, /fn\s+buffer_set_line\s+<\(i32,i32,str\)\*>\(\)>\s+\(b,\s*row,\s*line\):[\s\S]*store<str>/, 'TUI buffer storage module must own raw slot writes');
assert.match(bufferWrapCode, /fn\s+buffer_set_wrapped_text\s+<\(i32,i32,i32,i32,str\)\*>\(\)>\s+\(b,\s*start_row,\s*cols,\s*height,\s*text\):[\s\S]*tui_text_byte_kind[\s\S]*tui_text_byte_len[\s\S]*tui_text_byte_width/, 'TUI buffer wrapped text must use shared TUI byte classification helpers');
assert.match(bufferWrapCode, /fn\s+buffer_set_wrapped_text[\s\S]*buffer_set_line/, 'TUI buffer wrap module must delegate raw slot updates to storage');
assert.match(bufferPresentCode, /fn\s+buffer_present_diff\s+<\(i32\)\*>\(\)>\s+\(b\):[\s\S]*move_cursor[\s\S]*print/, 'TUI buffer present module must own cursor output policy');
assert.match(textWrapCode, /fn\s+tui_empty_str_vec\s+<\(\)->Vec<str>>\s+\(\):\s+v::vec_empty<str>/, 'text_wrap_lines allocation fallback must use typed empty Vec storage');
assert.match(textWrapCode, /fn\s+tui_push_str\s+<\(Vec<str>,str\)->TuiStrPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<str>\s+items\s+item:[\s\S]*Result::Err\s+e:[\s\S]*TuiStrPushRes\s+v::vec_push_error_vec<str>\s+e\s+false/, 'text_wrap_lines push must preserve the Vec owner and convert grow failure to ok=false');
assert.match(textWrapCode, /fn\s+text_wrap_lines\s+<\(str,i32\)\*>Vec<str>>\s+\(text,\s*cols\):[\s\S]*match\s+v::new<str>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'text_wrap_lines must handle Vec allocation failure');
assert.match(textWrapCode, /while\s+and\s+lt\s+i\s+n\s+not\s+failed:/, 'text_wrap_lines must stop scanning after line accumulation failure');
assert.match(textWrapCode, /let\s+pushed_tail\s+<TuiStrPushRes>\s+tui_push_str\s+out\s+tail/, 'text_wrap_lines tail accumulation must go through checked push');

console.log('stdlib wasix tui unsafe unwrap regression passed');
