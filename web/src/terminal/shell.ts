import { resolveCompilerAssets, type CompilerAssetUrls } from '../runtime/compiler-assets.js';
import { VFS } from '../runtime/vfs.js';

type WorkerStdoutMessage = {
    type: 'stdout';
    fd: number;
    data: number[];
};

type WorkerCompileResultMessage = {
    type: 'compile_result';
    outputs: Record<string, string | Uint8Array>;
};

type WorkerExitMessage = {
    type: 'exit';
    code: number;
};

type WorkerErrorMessage = {
    type: 'error';
    message: string;
};

type WorkerMessage =
    | WorkerStdoutMessage
    | WorkerCompileResultMessage
    | WorkerExitMessage
    | WorkerErrorMessage
    | { type: 'stdin_request' };

type RunWasmWorkerRequest = {
    type: 'run-wasm';
    bin: Uint8Array;
    args: string[];
    env: Record<string, string>;
    vfsData: Record<string, string | Uint8Array>;
    sab: SharedArrayBuffer | null;
};

type ExecuteNeplg2WorkerRequest = {
    type: 'execute-neplg2';
    compiler: CompilerAssetUrls;
    entryPath: string;
    source: string;
    vfsData: Record<string, string | Uint8Array>;
    emitValues: string[];
    attachSource: boolean;
    runAfterBuild: boolean;
    runArgs: string[];
    env: Record<string, string>;
    sab: SharedArrayBuffer | null;
};

type WorkerRequest = RunWasmWorkerRequest | ExecuteNeplg2WorkerRequest;

type WorkerProcessOptions = {
    onCompileResult?: (outputs: Record<string, string | Uint8Array>) => void;
};

export class Shell {
    terminal: any;
    editor: any;
    vfs: VFS;
    env: Map<string, string>;
    history: string[];
    historyIndex: number;
    private activeWorker: Worker | null;
    private sab: SharedArrayBuffer | null;
    private stdinBuffer: Int32Array | null;
    private stdinData: Uint8Array | null;
    private currentProcessReject: ((reason?: any) => void) | null;

    constructor(terminal: any, vfs: VFS) {
        this.terminal = terminal;
        this.vfs = vfs || new VFS();
        this.editor = null;
        this.env = new Map([
            ['USER', 'nepl'],
            ['PATH', '/usr/bin:/bin'],
            ['SHELL', 'nepl-shell'],
        ]);
        this.history = [];
        this.historyIndex = 0;
        this.activeWorker = null;
        this.sab = null;
        this.stdinBuffer = null;
        this.stdinData = null;
        this.currentProcessReject = null;
    }

    async executeLine(line: string) {
        const trimmed = line.trim();
        if (!trimmed) {
            return;
        }

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
                    result = '/';
                    break;
                case 'echo':
                    result = args.join(' ');
                    break;
                case 'clear':
                    this.terminal.clear();
                    return;
                case 'neplg2':
                    result = await this.cmdNeplg2(args);
                    break;
                case 'wasmi':
                    result = await this.cmdWasmi(args);
                    break;
                case 'tree':
                    result = this.renderTree(args[0] || '/');
                    break;
                default:
                    result = `Command not found: ${cmd}`;
            }
        } catch (error: any) {
            result = `Error: ${error.message}`;
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
        } catch (error: any) {
            return `ls: ${path}: ${error.message}`;
        }
    }

    cmdCat(args: string[]) {
        if (args.length === 0) {
            return 'cat: missing file';
        }
        const path = args[0];
        try {
            const content = this.vfs.readFile(path);
            if (content instanceof Uint8Array) {
                return `cat: ${path}: Binary file`;
            }
            return content;
        } catch (error: any) {
            return `cat: ${path}: ${error.message}`;
        }
    }

    async cmdNeplg2(args: string[]): Promise<any> {
        const parsed = this.parseFlags(args);
        const wantsRun = args.includes('run');
        const wantsBuild = args.includes('build');
        if (!wantsRun && !wantsBuild) {
            return 'Unknown neplg2 command.';
        }

        this.terminal.print('Compiling...');
        this.syncCurrentEditorToVfs();

        const sourceInput = this.resolveCompileInput(parsed);
        if (typeof sourceInput === 'string') {
            return sourceInput;
        }

        this.terminal.print(`Source: ${sourceInput.inputPath}`);

        const compiler = this.resolveCompilerAssetUrls();
        if (!compiler) {
            return 'Error: Compiler assets are not ready yet.';
        }

        const emitValues = this.normalizeEmit(parsed.flags['--emit']);
        if (wantsRun && !emitValues.includes('wasm')) {
            emitValues.push('wasm');
        }
        const attachSource = Boolean(parsed.flags['--attach-source'] || parsed.flags['--attach_source']);
        const outArg = (parsed.flags['-o'] as any) || (parsed.flags['--output'] as any);
        const outBase = this.outputBaseFromArg(typeof outArg === 'string' ? outArg : 'out');
        const wasmOutFile = emitValues.includes('wasm') ? this.outputPath(outBase, 'wasm') : null;

        const request: ExecuteNeplg2WorkerRequest = {
            type: 'execute-neplg2',
            compiler,
            entryPath: sourceInput.inputPath,
            source: sourceInput.source,
            vfsData: this.vfs.serialize(),
            emitValues,
            attachSource,
            runAfterBuild: wantsRun,
            runArgs: wasmOutFile ? [wasmOutFile] : [],
            env: Object.fromEntries(this.env),
            sab: this.ensureStdinBuffer(),
        };

        try {
            const result = await this.runWorkerProcess(request, {
                onCompileResult: (outputs) => {
                    this.persistCompileOutputs(outBase, outputs);
                    this.terminal.print('Compilation finished.');
                },
            });
            if (!wantsRun) {
                return 'Build complete.';
            }
            return result;
        } catch (error: any) {
            return error?.message ? `Compilation Failed: ${error.message}` : `Compilation Failed: ${error}`;
        }
    }

    async cmdWasmi(args: string[]): Promise<any> {
        if (args.length === 0) {
            return 'wasmi: missing file';
        }
        const filename = args[0];
        if (!this.vfs.exists(filename)) {
            return `wasmi: file not found: ${filename}`;
        }

        const bin = this.vfs.readFile(filename);
        if (!(bin instanceof Uint8Array)) {
            return 'wasmi: invalid binary format';
        }

        this.terminal.print(`Executing ${filename} ...`);

        try {
            return await this.runWorkerProcess({
                type: 'run-wasm',
                bin,
                args,
                env: Object.fromEntries(this.env),
                vfsData: this.vfs.serialize(),
                sab: this.ensureStdinBuffer(),
            });
        } catch (error: any) {
            return `Execution Failed: ${error?.message ? error.message : error}`;
        }
    }

    private syncCurrentEditorToVfs() {
        if (!(this as any).tabManager) {
            return;
        }
        (this as any).tabManager.saveCurrentTab();
    }

    private resolveCompileInput(parsed: { flags: Record<string, string | boolean>; positional: string[] }) {
        let inputFile: string | boolean | undefined = parsed.flags['-i'] || parsed.flags['--input'];
        if (!inputFile || inputFile === true) {
            const lastPositional = parsed.positional[parsed.positional.length - 1];
            if (lastPositional && lastPositional !== 'run' && lastPositional !== 'build') {
                inputFile = lastPositional;
            } else {
                inputFile = undefined;
            }
        }

        let source = '';
        let inputPath = 'editor';

        if (this.editor) {
            const editorPath = typeof this.editor.getPath === 'function'
                ? this.editor.getPath()
                : (this.editor as any).path;
            const editorText = typeof this.editor.getText === 'function'
                ? this.editor.getText()
                : (this.editor as any).text;

            if (editorText !== undefined) {
                const normalizedEditorPath = editorPath && editorPath.startsWith('/') ? editorPath : editorPath ? `/${editorPath}` : editorPath;
                const normalizedInputFile = typeof inputFile === 'string'
                    ? (inputFile.startsWith('/') ? inputFile : `/${inputFile}`)
                    : null;
                const isTargetFile = normalizedInputFile && normalizedEditorPath === normalizedInputFile;
                const editorEditable = typeof this.editor.getEditable === 'function'
                    ? this.editor.getEditable()
                    : true;

                if (!inputFile || isTargetFile) {
                    source = editorText;
                    inputPath = editorPath || 'editor';
                    if (editorPath && editorEditable) {
                        this.vfs.writeFile(editorPath, editorText);
                        this.terminal.print(`(Using synced editor content for ${editorPath})`);
                    } else if (editorPath) {
                        this.terminal.print(`(Using read-only editor view for ${editorPath})`);
                    } else {
                        this.terminal.print('(Using editor content)');
                    }
                }
            }
        }

        if (!source) {
            if (typeof inputFile === 'string') {
                let resolvedInputFile = inputFile;
                if (!this.vfs.exists(resolvedInputFile)) {
                    const withSlash = resolvedInputFile.startsWith('/') ? resolvedInputFile : `/${resolvedInputFile}`;
                    if (this.vfs.exists(withSlash)) {
                        resolvedInputFile = withSlash;
                    } else {
                        return `Error: File not found '${inputFile}'`;
                    }
                }
                source = this.vfs.readFile(resolvedInputFile) as string;
                inputPath = resolvedInputFile;
            } else if (this.editor) {
                source = typeof this.editor.getText === 'function'
                    ? this.editor.getText()
                    : (this.editor as any).text;
                inputPath = (typeof this.editor.getPath === 'function'
                    ? this.editor.getPath()
                    : (this.editor as any).path) || 'editor';
            } else {
                return 'Error: No input file and editor not connected';
            }
        }

        return { source, inputPath };
    }

    private resolveCompilerAssetUrls(): CompilerAssetUrls | null {
        return resolveCompilerAssets(window as any, typeof document !== 'undefined' ? document : null);
    }

    private ensureStdinBuffer(): SharedArrayBuffer | null {
        if (this.sab) {
            if (this.stdinBuffer) {
                Atomics.store(this.stdinBuffer, 0, 0);
            }
            return this.sab;
        }

        try {
            if (typeof SharedArrayBuffer !== 'undefined') {
                this.sab = new SharedArrayBuffer(1024 * 64);
                this.stdinBuffer = new Int32Array(this.sab, 0, 1);
                this.stdinData = new Uint8Array(this.sab, 4);
                Atomics.store(this.stdinBuffer, 0, 0);
            }
        } catch (error) {
            console.warn('SharedArrayBuffer restriction:', error);
            this.sab = null;
        }

        return this.sab;
    }

    private createWorker(): Worker {
        return new Worker(new URL('../runtime/worker.js', import.meta.url), { type: 'module' });
    }

    private async runWorkerProcess(request: WorkerRequest, options: WorkerProcessOptions = {}): Promise<any> {
        if (this.stdinBuffer) {
            Atomics.store(this.stdinBuffer, 0, 0);
        }

        return new Promise((resolve, reject) => {
            const worker = this.createWorker();
            this.activeWorker = worker;
            this.currentProcessReject = reject;

            const finish = () => {
                this.activeWorker = null;
                this.currentProcessReject = null;
                worker.terminate();
            };

            worker.onmessage = (event: MessageEvent<WorkerMessage>) => {
                const message = event.data;
                switch (message.type) {
                    case 'stdout': {
                        const text = new TextDecoder().decode(new Uint8Array(message.data));
                        this.terminal.write(text);
                        break;
                    }
                    case 'compile_result':
                        options.onCompileResult?.(message.outputs);
                        break;
                    case 'exit':
                        finish();
                        resolve(message.code === 0 ? null : `Program exited with code ${message.code}`);
                        break;
                    case 'error':
                        finish();
                        reject(new Error(message.message));
                        break;
                    case 'stdin_request':
                        break;
                }
            };

            worker.onerror = (event) => {
                finish();
                reject(new Error(`Worker error: ${event.message}`));
            };

            worker.postMessage(request);
        });
    }

    private persistCompileOutputs(outBase: string, outputs: Record<string, string | Uint8Array>) {
        if (outputs.wasm instanceof Uint8Array) {
            const wasmPath = this.outputPath(outBase, 'wasm');
            this.vfs.writeFile(wasmPath, outputs.wasm);
            this.terminal.print(`Generated ${wasmPath}`);
        }
        if (typeof outputs.wat === 'string') {
            const watPath = this.outputPath(outBase, 'wat');
            this.vfs.writeFile(watPath, outputs.wat);
            this.terminal.print(`Generated ${watPath}`);
        }
        if (typeof outputs['wat-min'] === 'string') {
            const watMinPath = this.outputPath(outBase, 'wat-min');
            this.vfs.writeFile(watMinPath, outputs['wat-min']);
            this.terminal.print(`Generated ${watMinPath}`);
        }
    }

    normalizeEmit(flagValue: any): string[] {
        const raw: string[] = [];
        if (typeof flagValue === 'string') {
            raw.push(flagValue);
        }
        if (Array.isArray(flagValue)) {
            raw.push(...flagValue.map((value) => String(value)));
        }
        if (raw.length === 0) {
            raw.push('wasm');
        }

        const expanded: string[] = [];
        for (const item of raw) {
            for (const part of item.split(',')) {
                const value = part.trim();
                if (!value) {
                    continue;
                }
                if (value === 'all') {
                    expanded.push('wasm', 'wat', 'wat-min');
                } else {
                    expanded.push(value);
                }
            }
        }

        const seen = new Set<string>();
        const normalized: string[] = [];
        for (const value of expanded) {
            if (!seen.has(value)) {
                seen.add(value);
                normalized.push(value);
            }
        }
        return normalized;
    }

    outputBaseFromArg(output: string): string {
        if (output.endsWith('.min.wat')) {
            return output.slice(0, -'.min.wat'.length);
        }
        if (output.endsWith('.wasm')) {
            return output.slice(0, -'.wasm'.length);
        }
        if (output.endsWith('.wat')) {
            return output.slice(0, -'.wat'.length);
        }
        return output;
    }

    outputPath(base: string, emit: 'wasm' | 'wat' | 'wat-min'): string {
        if (emit === 'wasm') {
            return `${base}.wasm`;
        }
        if (emit === 'wat') {
            return `${base}.wat`;
        }
        return `${base}.min.wat`;
    }

    parseFlags(args: string[]) {
        const flags: Record<string, string | boolean> = {};
        const positional: string[] = [];

        for (let index = 0; index < args.length; index++) {
            const token = args[index];
            if (token.startsWith('-')) {
                const eqIndex = token.indexOf('=');
                if (eqIndex !== -1) {
                    const key = token.slice(0, eqIndex);
                    const rawValue = token.slice(eqIndex + 1);
                    if (rawValue === '' || rawValue === 'true') {
                        flags[key] = true;
                    } else if (rawValue === 'false') {
                        flags[key] = false;
                    } else {
                        flags[key] = rawValue;
                    }
                    continue;
                }

                if (index + 1 < args.length && !args[index + 1].startsWith('-')) {
                    flags[token] = args[index + 1];
                    index++;
                } else {
                    flags[token] = true;
                }
            } else {
                positional.push(token);
            }
        }

        return { flags, positional };
    }

    interrupt() {
        if (!this.activeWorker) {
            return;
        }
        this.activeWorker.terminate();
        this.activeWorker = null;
        this.terminal.printError('\nProcess interrupted.');
        if (this.stdinBuffer) {
            Atomics.store(this.stdinBuffer, 0, -1);
            Atomics.notify(this.stdinBuffer, 0);
        }
        if (this.currentProcessReject) {
            this.currentProcessReject(new Error('Process interrupted'));
            this.currentProcessReject = null;
        }
    }

    handleStdin(text: string | null) {
        if (!this.stdinBuffer || !this.stdinData) {
            return;
        }
        if (text === null) {
            Atomics.store(this.stdinBuffer, 0, -1);
        } else {
            const encoded = new TextEncoder().encode(text);
            this.stdinData.fill(0);
            this.stdinData.set(encoded.subarray(0, this.stdinData.length));
            Atomics.store(this.stdinBuffer, 0, Math.min(encoded.length, this.stdinData.length));
        }
        Atomics.notify(this.stdinBuffer, 0);
    }

    get isRunning() {
        return this.activeWorker !== null;
    }

    renderTree(rootPath: string) {
        let normalizedRoot = rootPath;
        if (!normalizedRoot.startsWith('/')) {
            normalizedRoot = `/${normalizedRoot}`;
        }
        const results: string[] = [];
        results.push(normalizedRoot);

        const build = (path: string, prefix: string) => {
            const entries = this.vfs.listDir(path);
            for (let index = 0; index < entries.length; index++) {
                const entry = entries[index];
                const isLast = index === entries.length - 1;
                const fullPath = `${path.endsWith('/') ? path : `${path}/`}${entry}`;
                const isDir = this.vfs.isDir(fullPath);

                results.push(`${prefix}${isLast ? '└─ ' : '├─ '}${isDir ? `${entry}/` : entry}`);

                if (isDir) {
                    build(fullPath, `${prefix}${isLast ? '   ' : '│  '}`);
                }
            }
        };

        build(normalizedRoot, '');
        return results.join('\n');
    }
}
