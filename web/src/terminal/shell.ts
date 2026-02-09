import { VFS } from '../runtime/vfs.js';

export class Shell {
    terminal: any;
    editor: any;
    vfs: VFS;
    env: Map<string, string>;
    history: string[];
    historyIndex: number;

    constructor(terminal: any, vfs: VFS) {
        this.terminal = terminal;
        this.vfs = vfs || new VFS();
        this.editor = null;
        this.env = new Map([
            ["USER", "nepl"],
            ["PATH", "/usr/bin:/bin"],
            ["SHELL", "nepl-shell"]
        ]);
        this.history = [];
        this.historyIndex = 0;
    }

    async executeLine(line: string) {
        const trimmed = line.trim();
        if (!trimmed) return;

        this.history.push(trimmed);
        this.historyIndex = this.history.length;

        const parts = trimmed.split(/\s+/);
        const cmd = parts[0];
        const args = parts.slice(1);

        let result: any;
        try {
            switch (cmd) {
                case 'help':
                    result = this.cmdHelp();
                    break;
                case 'ls':
                    result = this.cmdLs(args);
                    break;
                case 'cat':
                    result = this.cmdCat(args);
                    break;
                case 'pwd':
                    result = "/";
                    break;
                case 'echo':
                    result = args.join(' ');
                    break;
                case 'clear':
                    this.terminal.clear();
                    return;
                case 'neplg2':
                    result = await this.cmdNeplg2(args, null);
                    break;
                case 'wasmi':
                    result = await this.cmdWasmi(args, null);
                    break;
                case 'tree':
                    result = this.renderTree(args[0] || '/');
                    break;
                default:
                    result = `Command not found: ${cmd}`;
            }
        } catch (e: any) {
            result = `Error: ${e.message}`;
        }

        if (result !== undefined && result !== null) {
            this.terminal.print(result);
        }
    }

    cmdHelp() {
        return `Available commands:
  help          - Show this help
  ls [path]     - List directory contents
  cat <file>    - Display file contents
  pwd           - Print working directory
  clear         - Clear the terminal
  neplg2 [run|build] [-i input] [-o output] [--emit wasm|wat|wat-min|all] [--attach-source]
                - Compile NEPLg2 code (WASM/WAT) and optionally run (WASM)
  wasmi <file>  - Run a WASM file using the wasmi runtime
  tree [path]   - Show directory tree structure
  echo [text]   - Display text`;
    }

    cmdLs(args: string[]) {
        const path = args[0] || '/';
        try {
            const entries = this.vfs.listDir(path);
            return entries.join('  ');
        } catch (e: any) {
            return `ls: ${path}: ${e.message}`;
        }
    }

    cmdCat(args: string[]) {
        if (args.length === 0) return "cat: missing file";
        const path = args[0];
        try {
            const content = this.vfs.readFile(path);
            if (content instanceof Uint8Array) {
                return `cat: ${path}: Binary file`;
            }
            return content;
        } catch (e: any) {
            return `cat: ${path}: ${e.message}`;
        }
    }

    async cmdNeplg2(args: string[], stdin: any): Promise<any> {
        const parsed = this.parseFlags(args);

        if (args.includes('run') || args.includes('build')) {
            this.terminal.print("Compiling...");

            // Sync current editor state to VFS via TabManager if available
            if ((this as any).tabManager) {
                (this as any).tabManager.saveCurrentTab();
            }

            let inputFile: string | boolean | undefined = parsed.flags['-i'] || parsed.flags['--input'];
            if (!inputFile || inputFile === true) {
                const lastPos = parsed.positional[parsed.positional.length - 1];
                if (lastPos && lastPos !== 'run' && lastPos !== 'build') {
                    inputFile = lastPos;
                } else {
                    inputFile = undefined;
                }
            }

            let source = "";
            let inputPath = "editor";

            // If we have an active editor, try to use its content if it matches the input file or no input file given
            if (this.editor) {
                const editorPath = (this.editor as any).path;
                const editorText = typeof this.editor.getText === 'function' ? this.editor.getText() : (this.editor as any).text;
                
                if (editorText !== undefined) {
                    const isTargetFile = typeof inputFile === 'string' && (inputFile === editorPath || (inputFile.startsWith('/') && inputFile === (editorPath.startsWith('/') ? editorPath : '/' + editorPath)));
                    
                    if (!inputFile || isTargetFile) {
                        source = editorText;
                        inputPath = editorPath || "editor";
                        if (editorPath) {
                            this.vfs.writeFile(editorPath, editorText);
                            this.terminal.print(`(Using synced editor content for ${editorPath})`);
                        } else {
                            this.terminal.print("(Using editor content)");
                        }
                    }
                }
            }

            if (!source) {
                if (typeof inputFile === 'string') {
                    if (!this.vfs.exists(inputFile)) {
                        const slashed = inputFile.startsWith('/') ? inputFile : '/' + inputFile;
                        if (this.vfs.exists(slashed)) {
                            inputFile = slashed;
                        } else {
                            return `Error: File not found '${inputFile}'`;
                        }
                    }
                    source = this.vfs.readFile(inputFile) as string;
                    inputPath = inputFile;
                } else if (this.editor) {
                    // Fallback to editor if source still not found and editor exists
                    source = typeof this.editor.getText === 'function' ? this.editor.getText() : (this.editor as any).text;
                    inputPath = (this.editor as any).path || "editor";
                } else {
                    return "Error: No input file and editor not connected";
                }
            }

            this.terminal.print(`Source: ${inputPath}`);

            if (!(window as any).wasmBindings) return "Error: Compiler (WASM) not loaded yet.";
            const wasmBindings = (window as any).wasmBindings;

            try {
                // --emit は "wasm,wat,wat-min" のようにカンマ区切りを許可する
                const wantsRun = args.includes('run');
                const emitValues = this.normalizeEmit(parsed.flags['--emit']);
                if (wantsRun && !emitValues.includes('wasm')) {
                    emitValues.push('wasm');
                }

                const attachSource = Boolean(parsed.flags['--attach-source'] || parsed.flags['--attach_source']);

                const outArg = (parsed.flags['-o'] as any) || (parsed.flags['--output'] as any);
                const outBase = this.outputBaseFromArg(typeof outArg === 'string' ? outArg : 'out');

                const emitJs: any = emitValues.length === 1 ? emitValues[0] : emitValues;
                const outputs = wasmBindings.compile_outputs_with_vfs(
                    inputPath,
                    source,
                    this.vfs.serialize(),
                    emitJs,
                    attachSource
                );

                let wasmOutFile: string | null = null;

                if (outputs.wasm) {
                    const p = this.outputPath(outBase, 'wasm');
                    this.vfs.writeFile(p, outputs.wasm);
                    wasmOutFile = p;
                    this.terminal.print(`Generated ${p}`);
                }
                if (outputs.wat) {
                    const p = this.outputPath(outBase, 'wat');
                    this.vfs.writeFile(p, outputs.wat);
                    this.terminal.print(`Generated ${p}`);
                }
                if (outputs['wat-min']) {
                    const p = this.outputPath(outBase, 'wat-min');
                    this.vfs.writeFile(p, outputs['wat-min']);
                    this.terminal.print(`Generated ${p}`);
                }

                this.terminal.print("Compilation finished.");

                if (wantsRun) {
                    if (!wasmOutFile) {
                        return "Error: --emit does not include wasm, so it cannot be run.";
                    }
                    return await this.cmdWasmi([wasmOutFile], stdin);
                }
                return "Build complete.";
            } catch (e: any) {
                if (e.message && (e.message.includes("wasmi") || e.message.includes("Worker") || e.message.includes("Program") || e.message.includes("Execution"))) {
                    return `Execution Failed: ${e.message}`;
                }
                return `Compilation Failed: ${e}`;
            }
        }
        return "Unknown neplg2 command.";
    }


    normalizeEmit(flagValue: any): string[] {
        const raw: string[] = [];
        if (typeof flagValue === 'string') raw.push(flagValue);
        if (Array.isArray(flagValue)) raw.push(...flagValue.map((v) => String(v)));
        if (raw.length === 0) raw.push('wasm');

        const expanded: string[] = [];
        for (const item of raw) {
            for (const part of item.split(',')) {
                const v = part.trim();
                if (!v) continue;
                if (v === 'all') {
                    expanded.push('wasm', 'wat', 'wat-min');
                } else {
                    expanded.push(v);
                }
            }
        }

        // 重複除去（順序は維持）
        const seen = new Set<string>();
        const out: string[] = [];
        for (const v of expanded) {
            if (!seen.has(v)) {
                seen.add(v);
                out.push(v);
            }
        }
        return out;
    }

    outputBaseFromArg(output: string): string {
        if (output.endsWith('.min.wat')) return output.slice(0, -'.min.wat'.length);
        if (output.endsWith('.wasm')) return output.slice(0, -'.wasm'.length);
        if (output.endsWith('.wat')) return output.slice(0, -'.wat'.length);
        return output;
    }

    outputPath(base: string, emit: 'wasm' | 'wat' | 'wat-min'): string {
        if (emit === 'wasm') return `${base}.wasm`;
        if (emit === 'wat') return `${base}.wat`;
        // wat-min
        return `${base}.min.wat`;
    }

    parseFlags(args: string[]) {
        const flags: Record<string, string | boolean> = {};
        const positional: string[] = [];

        for (let i = 0; i < args.length; i++) {
            const a = args[i];

            if (a.startsWith('-')) {
                // `--key=value` / `-k=value` 形式をサポートする
                const eq = a.indexOf('=');
                if (eq !== -1) {
                    const key = a.slice(0, eq);
                    const raw = a.slice(eq + 1);

                    if (raw === '' || raw === 'true') {
                        flags[key] = true;
                    } else if (raw === 'false') {
                        flags[key] = false;
                    } else {
                        flags[key] = raw;
                    }
                    continue;
                }

                // `--key value` 形式
                if (i + 1 < args.length && !args[i + 1].startsWith('-')) {
                    flags[a] = args[i + 1];
                    i++;
                } else {
                    flags[a] = true;
                }
            } else {
                positional.push(a);
            }
        }

        return { flags, positional };
    }

    private activeWorker: Worker | null = null;
    private sab: SharedArrayBuffer | null = null;
    private stdinBuffer: Int32Array | null = null;
    private stdinData: Uint8Array | null = null;
    private currentProcessReject: ((reason?: any) => void) | null = null;

    interrupt() {
        if (this.activeWorker) {
            console.log("Interrupting worker...");
            this.activeWorker.terminate();
            this.activeWorker = null;
            this.terminal.printError("\nProcess interrupted.");
            if (this.stdinBuffer) {
                Atomics.store(this.stdinBuffer, 0, -1);
                Atomics.notify(this.stdinBuffer, 0);
            }
            if (this.currentProcessReject) {
                this.currentProcessReject(new Error("Process interrupted"));
                this.currentProcessReject = null;
            }
        }
    }

    async cmdWasmi(args: string[], stdin: any): Promise<any> {
        if (args.length === 0) return "wasmi: missing file";
        const filename = args[0];
        if (!this.vfs.exists(filename)) return `wasmi: file not found: ${filename}`;

        const bin = this.vfs.readFile(filename);
        if (!(bin instanceof Uint8Array)) return "wasmi: invalid binary format";

        this.terminal.print(`Executing ${filename} ...`);

        if (!this.sab) {
            try {
                if (typeof SharedArrayBuffer !== 'undefined') {
                    console.log("Creating SharedArrayBuffer for stdin...");
                    this.sab = new SharedArrayBuffer(1024 * 64);
                    this.stdinBuffer = new Int32Array(this.sab, 0, 1);
                    this.stdinData = new Uint8Array(this.sab, 4);
                }
            } catch (e) {
                console.warn("SharedArrayBuffer restriction:", e);
                this.sab = null;
            }
        }

        if (this.stdinBuffer) {
            Atomics.store(this.stdinBuffer, 0, 0);
        }

        return new Promise((resolve, reject) => {
            const worker = new Worker(new URL('../runtime/worker.js', import.meta.url), { type: 'module' });
            this.activeWorker = worker;
            this.currentProcessReject = reject;

            worker.onmessage = (e) => {
                const { type, data, code, message } = e.data;
                switch (type) {
                    case 'stdout':
                        const text = new TextDecoder().decode(new Uint8Array(data));
                        this.terminal.write(text);
                        break;
                    case 'exit':
                        this.activeWorker = null;
                        this.currentProcessReject = null;
                        worker.terminate();
                        resolve(code === 0 ? null : `Program exited with code ${code}`);
                        break;
                    case 'error':
                        this.activeWorker = null;
                        this.currentProcessReject = null;
                        worker.terminate();
                        reject(new Error(message));
                        break;
                }
            };

            worker.onerror = (e) => {
                this.activeWorker = null;
                this.currentProcessReject = null;
                worker.terminate();
                reject(new Error("Worker error: " + e.message));
            };

            worker.postMessage({
                type: 'run',
                bin,
                args,
                env: Object.fromEntries(this.env),
                vfsData: this.vfs.serialize(),
                sab: this.sab
            });
        });
    }

    handleStdin(text: string | null) {
        if (this.stdinBuffer && this.stdinData) {
            if (text === null) {
                Atomics.store(this.stdinBuffer, 0, -1);
            } else {
                const encoded = new TextEncoder().encode(text);
                this.stdinData.set(encoded);
                Atomics.store(this.stdinBuffer, 0, encoded.length);
            }
            Atomics.notify(this.stdinBuffer, 0);
        }
    }

    get isRunning() {
        return this.activeWorker !== null;
    }

    renderTree(rootPath: string) {
        if (!rootPath.startsWith('/')) rootPath = '/' + rootPath;
        const results: string[] = [];
        results.push(rootPath);

        const build = (path: string, prefix: string) => {
            const entries = this.vfs.listDir(path);
            for (let i = 0; i < entries.length; i++) {
                const entry = entries[i];
                const isLast = i === entries.length - 1;
                const fullPath = (path.endsWith('/') ? path : path + '/') + entry;
                const isDir = this.vfs.isDir(fullPath);

                results.push(`${prefix}${isLast ? '└── ' : '├── '}${(isDir ? entry + '/' : entry)}`);

                if (isDir) {
                    build(fullPath, prefix + (isLast ? '    ' : '│   '));
                }
            }
        };

        build(rootPath, '');
        return results.join('\n');
    }
}