import init, { compile_source, compile_test, list_tests } from "./nepl_web.js";

const frame = document.getElementById("editor-frame");
const fallback = document.getElementById("fallback");
const fallbackEditor = document.getElementById("fallback-editor");
const status = document.getElementById("editor-status");

const terminalStatus = document.getElementById("terminal-status");
const terminalOutput = document.getElementById("terminal-output");
const terminalCommand = document.getElementById("terminal-command");
const terminalStdin = document.getElementById("terminal-stdin");
const terminalRun = document.getElementById("terminal-run");
const terminalTest = document.getElementById("terminal-test");
const terminalClear = document.getElementById("terminal-clear");

let wasmReady = false;
let testList = [];

function showFallback() {
    fallback.hidden = false;
    status.textContent = "エディタが見つかりません";
    status.classList.remove("status-success");
    status.classList.add("status-warning");
}

function markReady() {
    status.textContent = "エディタを読み込みました";
    status.classList.remove("status-warning");
    status.classList.add("status-success");
}

function setTerminalStatus(message, ok) {
    terminalStatus.textContent = message;
    terminalStatus.classList.remove("status-warning", "status-success");
    terminalStatus.classList.add(ok ? "status-success" : "status-warning");
}

function appendOutput(text) {
    terminalOutput.textContent += text;
    terminalOutput.scrollTop = terminalOutput.scrollHeight;
}

function appendLine(text) {
    appendOutput(`${text}\n`);
}

function clearOutput() {
    terminalOutput.textContent = "";
}

function getEditorSource() {
    if (!fallback.hidden) {
        return fallbackEditor.value;
    }
    try {
        const win = frame.contentWindow;
        if (win && win.editor && typeof win.editor.getValue === "function") {
            return win.editor.getValue();
        }
        if (win && typeof win.getNeplSource === "function") {
            return win.getNeplSource();
        }
        const doc = frame.contentDocument;
        if (doc) {
            const textarea = doc.querySelector("textarea");
            if (textarea) {
                return textarea.value;
            }
        }
    } catch (_error) {
        return fallbackEditor.value;
    }
    return fallbackEditor.value;
}

async function ensureWasmReady() {
    if (wasmReady) {
        return true;
    }
    setTerminalStatus("WASM が初期化されていません", false);
    appendLine("WASM の初期化に失敗しています。ページを再読み込みしてください。");
    return false;
}

async function runProgram(source, stdin) {
    let wasmBytes;
    try {
        wasmBytes = compile_source(source);
    } catch (error) {
        appendLine(String(error));
        return;
    }
    const result = await runWasm(wasmBytes, stdin);
    if (result.trap) {
        appendLine(`trap: ${result.trap}`);
    }
    if (result.stdout) {
        appendOutput(result.stdout);
        if (!result.stdout.endsWith("\n")) {
            appendOutput("\n");
        }
    }
    if (result.stderr) {
        appendOutput(result.stderr);
        if (!result.stderr.endsWith("\n")) {
            appendOutput("\n");
        }
    }
    appendLine(`exit code: ${result.exitCode}`);
}

async function runTests() {
    if (testList.length === 0) {
        appendLine("テストが見つかりません。");
        return;
    }
    let failures = 0;
    for (const name of testList) {
        appendLine(`test ${name} ...`);
        let wasmBytes;
        try {
            wasmBytes = compile_test(name);
        } catch (error) {
            failures += 1;
            appendLine(String(error));
            continue;
        }
        const result = await runWasm(wasmBytes, "");
        if (result.trap || result.exitCode !== 0) {
            failures += 1;
            appendLine(`FAILED (${result.trap || `exit ${result.exitCode}`})`);
        } else {
            appendLine("ok");
        }
    }
    appendLine(`tests: ${testList.length}, failed: ${failures}`);
}

async function handleRun() {
    if (!(await ensureWasmReady())) {
        return;
    }
    const source = getEditorSource().trim();
    if (!source) {
        appendLine("ソースが空です。");
        return;
    }
    appendLine("$ run");
    await runProgram(source, terminalStdin.value);
}

async function handleTest() {
    if (!(await ensureWasmReady())) {
        return;
    }
    appendLine("$ test");
    await runTests();
}

function handleHelp() {
    appendLine("commands:");
    appendLine("  run   - compile and run the current source");
    appendLine("  test  - run stdlib tests");
    appendLine("  clear - clear terminal output");
    appendLine("  help  - show this help");
}

function handleCommand() {
    const raw = terminalCommand.value.trim();
    if (!raw) {
        return;
    }
    terminalCommand.value = "";
    const parts = raw.split(/\s+/);
    const cmd = parts[0];
    if (cmd === "run") {
        handleRun();
        return;
    }
    if (cmd === "test") {
        handleTest();
        return;
    }
    if (cmd === "clear") {
        clearOutput();
        return;
    }
    if (cmd === "help") {
        handleHelp();
        return;
    }
    appendLine(`unknown command: ${cmd}`);
    handleHelp();
}

async function runWasm(wasmBytes, stdinText) {
    const stdinBytes = new TextEncoder().encode(stdinText || "");
    let stdinPos = 0;
    let stdout = "";
    let stderr = "";
    const decoder = new TextDecoder("utf-8");
    const imports = {
        wasi_snapshot_preview1: {
            fd_write(fd, iovs, iovsLen, nwritten) {
                const memory = instance.exports.memory;
                const view = new DataView(memory.buffer);
                const bytes = new Uint8Array(memory.buffer);
                let total = 0;
                const count = Math.max(0, iovsLen | 0);
                for (let i = 0; i < count; i += 1) {
                    const ptr = view.getUint32(iovs + i * 8, true);
                    const len = view.getUint32(iovs + i * 8 + 4, true);
                    const slice = bytes.subarray(ptr, ptr + len);
                    const chunk = decoder.decode(slice, { stream: true });
                    if (fd === 2) {
                        stderr += chunk;
                    } else {
                        stdout += chunk;
                    }
                    total += len;
                }
                if (nwritten) {
                    view.setUint32(nwritten, total, true);
                }
                return 0;
            },
            fd_read(fd, iovs, iovsLen, nread) {
                if (fd !== 0) {
                    return 8;
                }
                const memory = instance.exports.memory;
                const view = new DataView(memory.buffer);
                const bytes = new Uint8Array(memory.buffer);
                let total = 0;
                const count = Math.max(0, iovsLen | 0);
                for (let i = 0; i < count; i += 1) {
                    const ptr = view.getUint32(iovs + i * 8, true);
                    const len = view.getUint32(iovs + i * 8 + 4, true);
                    if (stdinPos >= stdinBytes.length) {
                        break;
                    }
                    const take = Math.min(len, stdinBytes.length - stdinPos);
                    bytes.set(stdinBytes.subarray(stdinPos, stdinPos + take), ptr);
                    stdinPos += take;
                    total += take;
                    if (take < len) {
                        break;
                    }
                }
                if (nread) {
                    view.setUint32(nread, total, true);
                }
                return 0;
            },
        },
    };
    let instance;
    try {
        const result = await WebAssembly.instantiate(wasmBytes, imports);
        instance = result.instance;
    } catch (error) {
        return { stdout, stderr, exitCode: 1, trap: String(error) };
    }
    let exitCode = 0;
    try {
        if (instance.exports._start) {
            instance.exports._start();
        } else if (instance.exports.main) {
            const result = instance.exports.main();
            if (typeof result === "number") {
                exitCode = result | 0;
            }
        } else {
            return { stdout, stderr, exitCode: 1, trap: "entry not found" };
        }
    } catch (error) {
        decoder.decode();
        return { stdout, stderr, exitCode: 1, trap: String(error) };
    }
    decoder.decode();
    return { stdout, stderr, exitCode };
}

const timeout = window.setTimeout(showFallback, 1500);

frame.addEventListener("load", () => {
    window.clearTimeout(timeout);
    try {
        const doc = frame.contentDocument;
        if (!doc || doc.body.children.length === 0) {
            showFallback();
            return;
        }
    } catch (_error) {
        showFallback();
        return;
    }
    fallback.hidden = true;
    markReady();
});

terminalRun.addEventListener("click", handleRun);
terminalTest.addEventListener("click", handleTest);
terminalClear.addEventListener("click", clearOutput);
terminalCommand.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
        event.preventDefault();
        handleCommand();
    }
});

(async () => {
    try {
        await init();
        wasmReady = true;
        const rawList = list_tests();
        testList = rawList.split("\n").filter((name) => name.length > 0);
        setTerminalStatus("WASM 準備完了", true);
        appendLine("type 'help' for available commands");
    } catch (error) {
        wasmReady = false;
        setTerminalStatus("WASM 初期化に失敗しました", false);
        appendLine(String(error));
    }
})();
