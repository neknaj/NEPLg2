const frame = document.getElementById("editor-frame");
const fallback = document.getElementById("fallback");
const fallbackEditor = document.getElementById("fallback-editor");
const statusBadge = document.getElementById("editor-status");
const docsLink = document.getElementById("docs-link");

const compileButton = document.getElementById("compile");
const runButton = document.getElementById("run");
const testButton = document.getElementById("test");
const clearButton = document.getElementById("clear");
const statusElement = document.getElementById("status");
const watOutput = document.getElementById("wat");

const terminalCommand = document.getElementById("terminal-command");
const terminalStdin = document.getElementById("terminal-stdin");
const terminalOutput = document.getElementById("terminal-output");

let wasmBindings = null;
let wasmReady = false;
let testList = [];

function showFallback() {
    fallback.hidden = false;
    statusBadge.textContent = "エディタが見つかりません";
    statusBadge.classList.remove("status-success");
    statusBadge.classList.add("status-warning");
}

function markReady() {
    statusBadge.textContent = "エディタを読み込みました";
    statusBadge.classList.remove("status-warning");
    statusBadge.classList.add("status-success");
}

function setStatus(message, isError = false) {
    statusElement.textContent = message;
    statusElement.classList.toggle("status--error", isError);
}

function readBindings() {
    if (wasmBindings) {
        return wasmBindings;
    }
    if (window.wasmBindings) {
        wasmBindings = window.wasmBindings;
    }
    return wasmBindings;
}

function initializeWasmBindings() {
    if (wasmReady) {
        return;
    }
    const bindings = readBindings();
    if (!bindings) {
        return;
    }
    wasmReady = true;
    try {
        testList = bindings.list_tests().split("\n").filter((name) => name.length > 0);
    } catch (error) {
        setStatus(`テスト一覧の取得に失敗しました: ${error}`, true);
        testList = [];
    }
    setStatus("WASM の初期化が完了しました。");
    appendLine("type 'help' for available commands");
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
    if (!wasmReady) {
        initializeWasmBindings();
    }
    if (wasmReady) {
        return true;
    }
    setStatus("WASM が初期化されていません。読み込みを待っています。", true);
    return false;
}

async function runProgram(source, stdin) {
    const bindings = readBindings();
    if (!bindings) {
        appendLine("WASM が初期化されていません。");
        return;
    }
    let wasmBytes;
    try {
        wasmBytes = bindings.compile_source(source);
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
    const bindings = readBindings();
    if (!bindings) {
        appendLine("WASM が初期化されていません。");
        return;
    }
    if (testList.length === 0) {
        appendLine("テストが見つかりません。");
        return;
    }
    let failures = 0;
    for (const name of testList) {
        appendLine(`test ${name} ...`);
        let wasmBytes;
        try {
            wasmBytes = bindings.compile_test(name);
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

async function handleCompile() {
    if (!(await ensureWasmReady())) {
        return;
    }
    const source = getEditorSource().trim();
    if (!source) {
        setStatus("ソースが空です。", true);
        return;
    }
    try {
        const bindings = readBindings();
        if (!bindings) {
            setStatus("WASM が初期化されていません。", true);
            return;
        }
        const wat = bindings.compile_to_wat(source);
        watOutput.value = wat;
        setStatus("WAT を生成しました。");
    } catch (error) {
        setStatus(`コンパイルに失敗しました: ${error}`, true);
    }
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
    setStatus("実行中...");
    await runProgram(source, terminalStdin.value);
    setStatus("実行が完了しました。");
}

async function handleTest() {
    if (!(await ensureWasmReady())) {
        return;
    }
    appendLine("$ test");
    setStatus("テストを実行中...");
    await runTests();
    setStatus("テストが完了しました。");
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
    const cmd = raw.split(/\s+/)[0];
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
    let instance;
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

let fallbackTimer = window.setTimeout(showFallback, 3000);

function clearFallbackTimer() {
    if (fallbackTimer) {
        window.clearTimeout(fallbackTimer);
        fallbackTimer = null;
    }
}

async function initEditorFrame() {
    if (!frame) {
        showFallback();
        return;
    }
    const rawSrc = frame.dataset.src;
    if (!rawSrc) {
        showFallback();
        return;
    }
    const resolved = new URL(rawSrc, window.location.href).toString();
    try {
        const response = await fetch(resolved, { method: "GET" });
        if (!response.ok) {
            showFallback();
            return;
        }
        frame.src = resolved;
        return;
    } catch (_error) {
        showFallback();
    }
}

frame.addEventListener("load", () => {
    clearFallbackTimer();
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

compileButton.addEventListener("click", handleCompile);
runButton.addEventListener("click", handleRun);
testButton.addEventListener("click", handleTest);
clearButton.addEventListener("click", clearOutput);
terminalCommand.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
        event.preventDefault();
        handleCommand();
    }
});

if (docsLink) {
    const fallbackUrl = docsLink.dataset.docsUrl
        ? new URL(docsLink.dataset.docsUrl, window.location.href).toString()
        : new URL("../doc/", window.location.href).toString();
    docsLink.href = fallbackUrl;
    docsLink.rel = "noopener noreferrer";
}
initializeWasmBindings();
window.addEventListener("TrunkApplicationStarted", () => {
    initializeWasmBindings();
});
initEditorFrame();
