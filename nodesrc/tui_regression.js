#!/usr/bin/env node
// nodesrc/tui_regression.js
// 目的:
// - WASIX TUI の回帰確認を自動化する。
// - nepl-cli で wasm を生成し、wasmer run にキー入力を送って
//   クラッシュしないこと・終了できること・基本描画が出ることを検証する。

const { spawn, spawnSync } = require('node:child_process');
const path = require('node:path');
const os = require('node:os');
const fs = require('node:fs');

function parseArgs(argv) {
    const opts = {
        input: 'examples/tui_editor/main.nepl',
        outputBase: 'tmp/editor',
        wasmer: process.env.WASMER_BIN || 'wasmer',
        neplCli: 'target/debug/nepl-cli',
        timeoutMs: 8000,
    };
    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '--input' && i + 1 < argv.length) opts.input = argv[++i];
        else if (a === '--output-base' && i + 1 < argv.length) opts.outputBase = argv[++i];
        else if (a === '--wasmer' && i + 1 < argv.length) opts.wasmer = argv[++i];
        else if (a === '--nepl-cli' && i + 1 < argv.length) opts.neplCli = argv[++i];
        else if (a === '--timeout-ms' && i + 1 < argv.length) opts.timeoutMs = Number(argv[++i]);
        else if (a === '-h' || a === '--help') {
            return { ...opts, help: true };
        }
    }
    return { ...opts, help: false };
}

function stripAnsi(s) {
    return String(s)
        .replace(/\x1b\[[0-9;?]*[ -/]*[@-~]/g, '')
        .replace(/\x1b\][^\x07]*(\x07|\x1b\\)/g, '');
}

function assertCond(cond, msg) {
    if (!cond) throw new Error(msg);
}

function countContains(haystack, needle) {
    let c = 0;
    let i = 0;
    while (true) {
        const j = haystack.indexOf(needle, i);
        if (j < 0) return c;
        c += 1;
        i = j + needle.length;
    }
}

function assertHasColorSpan(output, text, fg, bg, label) {
    const esc = '\x1b[' + String(fg) + ';' + String(bg) + 'm' + text + '\x1b[0m';
    assertCond(output.includes(esc), label || `missing color span: ${text}`);
}

function writeFixtures(baseDir, fixtures) {
    if (!fixtures) return;
    for (const [rel, content] of Object.entries(fixtures)) {
        const p = path.join(baseDir, rel);
        fs.mkdirSync(path.dirname(p), { recursive: true });
        fs.writeFileSync(p, String(content), 'utf8');
    }
}

function assertFileEquals(baseDir, relPath, expected) {
    const p = path.join(baseDir, relPath);
    const actual = fs.readFileSync(p, 'utf8');
    if (actual !== String(expected)) {
        throw new Error(
            `file mismatch: ${relPath}\n--- expected ---\n${String(expected)}\n--- actual ---\n${actual}`
        );
    }
}

function makeVfsSnapshot() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-tui-vfs-'));
    const srcTmp = path.resolve('tmp');
    const dstTmp = path.join(root, 'tmp');
    fs.mkdirSync(dstTmp, { recursive: true });
    if (fs.existsSync(srcTmp) && fs.statSync(srcTmp).isDirectory()) {
        fs.cpSync(srcTmp, dstTmp, { recursive: true });
    }
    return root;
}

function cleanupVfsSnapshot(vfsRoot) {
    try {
        fs.rmSync(vfsRoot, { recursive: true, force: true });
    } catch {}
}

function runCompile(opts) {
    const outBase = path.resolve(opts.outputBase);
    const cp = spawnSync(
        opts.neplCli,
        ['-i', opts.input, '--target', 'wasix', '--output', outBase],
        { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }
    );
    if (cp.status !== 0) {
        const detail = [cp.stdout || '', cp.stderr || ''].join('\n').trim();
        throw new Error(`compile failed:\n${detail}`);
    }
    return `${outBase}.wasm`;
}

function runScenario(opts, wasmPath, scenario, vfsRoot) {
    return new Promise((resolve, reject) => {
        const child = spawn(opts.wasmer, ['run', `--volume=${vfsRoot}:${vfsRoot}`, wasmPath], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });
        let stdout = '';
        let stderr = '';
        let done = false;

        const finish = (err, result) => {
            if (done) return;
            done = true;
            clearTimeout(killTimer);
            if (err) reject(err);
            else resolve(result);
        };

        child.stdout.on('data', (d) => {
            stdout += d.toString('utf8');
        });
        child.stderr.on('data', (d) => {
            stderr += d.toString('utf8');
        });
        child.on('error', (e) => finish(e));
        child.on('close', (code, signal) => {
            finish(null, { code, signal, stdout, stderr });
        });

        let at = 0;
        for (const step of scenario.steps) {
            at += step.delayMs;
            setTimeout(() => {
                if (done) return;
                try {
                    child.stdin.write(step.bytes, 'binary');
                } catch {}
            }, at);
        }
        setTimeout(() => {
            if (!done) {
                try { child.stdin.end(); } catch {}
            }
        }, at + 20);

        const killTimer = setTimeout(() => {
            if (done) return;
            try { child.kill('SIGKILL'); } catch {}
            finish(new Error(`scenario timeout: ${scenario.name}`));
        }, opts.timeoutMs);
    });
}

function defaultScenarios() {
    return [
        {
            name: 'smoke_quit',
            steps: [{ delayMs: 220, bytes: 'q\n' }],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `smoke_quit exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('NEPLg2 TUI Editor / Tab'), 'title not rendered');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'keys_edit_and_nav',
            steps: [
                { delayMs: 80, bytes: 'a' },
                { delayMs: 80, bytes: '\x1b[D' },
                { delayMs: 80, bytes: '\x1b[C' },
                { delayMs: 80, bytes: '\x1b[A' },
                { delayMs: 80, bytes: '\x1b[B' },
                { delayMs: 80, bytes: '1' },
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: '3' },
                { delayMs: 100, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `keys_edit_and_nav exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('Tabs: [1][2][3]'), 'tabs line missing');
                assertCond(plain.includes('Col: 2') || plain.includes('Col: 3'), 'cursor column did not move');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'syntax_color_ansi',
            steps: [{ delayMs: 120, bytes: 'q\n' }],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `syntax_color_ansi exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                assertHasColorSpan(r.stdout, '#entry', 33, 40, 'directive color span missing');
                assertHasColorSpan(r.stdout, '#indent', 33, 40, 'directive span missing (#indent)');
                assertHasColorSpan(r.stdout, '4', 35, 40, 'number color span missing');
                assertHasColorSpan(r.stdout, 'fn', 36, 40, 'keyword color span missing');
            },
        },
        {
            name: 'editor_features',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'a' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 220, bytes: 'g' },
                { delayMs: 200, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `editor_features exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('tmp/nepl_tab2.nepl'), 'tab file path missing');
                assertCond(plain.includes('open: tmp/nepl_tab2.nepl'), 'tab switch open message missing');
                assertCond(plain.includes('mode: INSERT'), 'insert mode message missing');
                assertCond(plain.includes('mode: NORMAL'), 'normal mode message missing');
                assertCond(plain.includes('edited'), 'edit message missing');
                assertCond(plain.includes('type:'), 'type inspect message missing');
                assertCond(plain.includes('jump:'), 'jump message missing');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'insert_mode_no_shortcut_interference',
            steps: [
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'q' },
                { delayMs: 80, bytes: 's' },
                { delayMs: 80, bytes: 'r' },
                { delayMs: 80, bytes: '1' },
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: '3' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `insert_mode_no_shortcut_interference exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('mode: INSERT'), 'insert mode message missing');
                assertCond(plain.includes('edited'), 'insert edit message missing');
                assertCond(!plain.includes('open: tmp/nepl_tab2.nepl'), 'tab shortcut fired in insert mode');
                assertCond(!plain.includes('open: tmp/nepl_tab3.nepl'), 'tab shortcut fired in insert mode');
                assertCond(!plain.includes('saved'), 'save shortcut fired in insert mode');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'normal_mode_word_motion_wb',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'w' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 80, bytes: 'b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `normal_mode_word_motion_wb exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(countContains(plain, 'type:') >= 1, 'word motion type messages missing');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'normal_mode_line_motion_0_dollar',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'l' },
                { delayMs: 80, bytes: 'l' },
                { delayMs: 80, bytes: '$' },
                { delayMs: 80, bytes: '0' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `normal_mode_line_motion_0_dollar exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('type:'), 'line-start motion failed');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'normal_mode_0_g_do_not_edit',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: '0' },
                { delayMs: 80, bytes: 'g' },
                { delayMs: 80, bytes: '0' },
                { delayMs: 80, bytes: 'g' },
                { delayMs: 80, bytes: '0' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `normal_mode_0_g_do_not_edit exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(!plain.includes('edited'), 'normal mode navigation edited text');
                assertCond(plain.includes('type:'), 'normal mode cursor context unexpected');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'normal_mode_gg_G_navigation',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'G' },
                { delayMs: 80, bytes: '0' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 80, bytes: 'g' },
                { delayMs: 80, bytes: 'g' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `normal_mode_gg_G_navigation exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(countContains(plain, 'type:') >= 1, 'G/gg navigation type messages missing');
                assertCond(!plain.includes('edited'), 'normal mode navigation edited text');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'line_number_gutter_visible',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `line_number_gutter_visible exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('1 fn helper'), 'line number gutter missing');
                assertCond(plain.includes('2     add x 1'), 'line number second line missing');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'normal_mode_x_delete_char',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'x' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `normal_mode_x_delete_char exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('edited'), 'x delete did not update text');
                assertCond(plain.includes('type:'), 'x delete result unexpected');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'normal_mode_A_I_insert_positions',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'A' },
                { delayMs: 80, bytes: 'Z' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: '0' },
                { delayMs: 80, bytes: 'I' },
                { delayMs: 80, bytes: 'Q' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `normal_mode_A_I_insert_positions exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('edited'), 'A/I did not edit text');
                assertCond(plain.includes('type:'), 'I insert at line start failed');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'edit_at_eof_regression',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'G' },
                { delayMs: 80, bytes: 'A' },
                { delayMs: 80, bytes: 'X' },
                { delayMs: 80, bytes: '\n' },
                { delayMs: 80, bytes: 'Y' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `edit_at_eof_regression exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('mode: INSERT'), 'EOF edit did not enter insert mode');
                assertCond(plain.includes('edited'), 'EOF edit did not modify text');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'save_exact_tab2_insert_prefix',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'z' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `save_exact_tab2_insert_prefix exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('type:'), 'edited source check failed (tab2)');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'save_exact_tab3_multiline',
            steps: [
                { delayMs: 80, bytes: '3' },
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'A' },
                { delayMs: 80, bytes: '\n' },
                { delayMs: 80, bytes: 'B' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `save_exact_tab3_multiline exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('type:'), 'edited source check failed (tab3)');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'arrow_csi',
            steps: [
                { delayMs: 80, bytes: '\x1b[D' },
                { delayMs: 80, bytes: '\x1b[C' },
                { delayMs: 80, bytes: '\x1b[A' },
                { delayMs: 80, bytes: '\x1b[B' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `arrow_csi exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
    ];
}

async function main() {
    const opts = parseArgs(process.argv.slice(2));
    if (opts.help) {
        console.log('Usage: node nodesrc/tui_regression.js [--input <file.nepl>] [--output-base <tmp/editor>] [--wasmer <bin>] [--nepl-cli <path>] [--timeout-ms <n>]');
        process.exit(0);
    }

    const wasmPath = runCompile(opts);
    const scenarios = defaultScenarios();
    const results = [];
    for (const s of scenarios) {
        const vfsRoot = makeVfsSnapshot();
        try {
            writeFixtures(vfsRoot, s.fixtures || null);
            const r = await runScenario(opts, wasmPath, s, vfsRoot);
            s.expect(r);
            results.push({ name: s.name, exit_code: r.code, stdout_len: r.stdout.length, stderr_len: r.stderr.length });
        } finally {
            cleanupVfsSnapshot(vfsRoot);
        }
    }

    console.log(JSON.stringify({
        ok: true,
        input: opts.input,
        wasm: wasmPath,
        scenarios: results,
    }, null, 2));
}

main().catch((e) => {
    console.error(String(e && e.stack ? e.stack : e));
    process.exit(1);
});
