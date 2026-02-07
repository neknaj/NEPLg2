import { WASI } from '../runtime/wasi.js';
import { CanvasTerminal } from './terminal.js';
import { VFS } from '../runtime/vfs.js';

export class Shell {
    terminal: CanvasTerminal;
    vfs: VFS;
    env: Map<string, string>;
    history: string[];
    historyIndex: number;
    editor: any;

    constructor(terminal: CanvasTerminal, vfs: VFS) {
        this.terminal = terminal;
        this.vfs = vfs;
        this.env = new Map();
        this.history = [];
        this.historyIndex = 0;
        this.editor = null;
    }

    async executeLine(line: string) {
        if (!line.trim()) return;
        this.history.push(line);
        this.historyIndex = this.history.length;

        const segments = line.split('|').map(s => s.trim());
        let inputData: any = null;

        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];
            const args = this.parseArgs(segment);
            if (args.length === 0) continue;

            const cmd = args[0];
            const cmdArgs = args.slice(1);

            try {
                const isLast = (i === segments.length - 1);
                const output = await this.runCommand(cmd, cmdArgs, inputData);

                if (isLast) {
                    if (output) this.terminal.print(output);
                } else {
                    inputData = output;
                }
            } catch (e: any) {
                this.terminal.printError(`Error: ${e.message}`);
                break;
            }
        }
    }

    parseArgs(segment: string) {
        const args: string[] = [];
        let current = '';
        let inQuote = false;

        for (let i = 0; i < segment.length; i++) {
            const char = segment[i];
            if (char === '"') {
                inQuote = !inQuote;
            } else if (char === ' ' && !inQuote) {
                if (current) {
                    args.push(current);
                    current = '';
                }
            } else {
                current += char;
            }
        }
        if (current) args.push(current);
        return args;
    }

    async runCommand(cmd: string, args: string[], stdin: any): Promise<any> {
        switch (cmd) {
            case 'echo':
                if (args.length > 0) return args.join(' ');
                return stdin || "";

            case 'clear':
            case 'cls':
                this.terminal.clear();
                return null;

            case 'help':
                return [
                    "Available commands:",
                    "  help      Show this help message",
                    "  clear     Clear the terminal screen",
                    "  echo      Print arguments to stdout",
                    "  ls        List files in the virtual file system",
                    "  tree      Recursive directory listing",
                    "  cat       Display file content",
                    "  copy      Copy terminal buffer to clipboard",
                    "  neplg2    NEPLg2 Compiler & Toolchain",
                    "    run     Compile and run the current editor content",
                    "    build   Compile the editor or a file",
                    "  run       Alias for 'neplg2 run'",
                    "  compile   Alias for 'neplg2 build --emit wat'",
                    "  test      Alias for 'neplg2 run stdlib/test.nepl' (if applicable)",
                    "  wasmi     Run a .wasm file"
                ].join('\n');

            case 'run':
                return await this.cmdNeplg2(['run'], stdin);

            case 'build':
                return await this.cmdNeplg2(['build'], stdin);

            case 'compile':
                return await this.cmdNeplg2(['build', '--emit', 'wat'], stdin);

            case 'test':
                return await this.cmdNeplg2(['run', 'stdlib/std/test.nepl'], stdin);

            case 'neplg2':
                return await this.cmdNeplg2(args, stdin);

            case 'wasmi':
                return await this.cmdWasmi(args, stdin);

            case 'ls':
                const lsPath = args[0] || '/';
                try {
                    if (this.vfs.isDir(lsPath)) {
                        const entries = this.vfs.listDir(lsPath);
                        const mapped = entries.map(entry => {
                            const fullPath = (lsPath.endsWith('/') ? lsPath : lsPath + '/') + entry;
                            return this.vfs.isDir(fullPath) ? entry + '/' : entry;
                        });
                        return mapped.join('\n');
                    }
                    return this.vfs.exists(lsPath) ? args[0] : `ls: no such file or directory: ${lsPath}`;
                } catch (e: any) {
                    return `ls: ${e.message}`;
                }

            case 'tree':
                const treePath = args[0] || '/';
                try {
                    return this.renderTree(treePath);
                } catch (e: any) {
                    return `tree: ${e.message}`;
                }

            case 'cat':
                if (args.length === 0) return "cat: missing operand";
                const catPath = args[0];
                try {
                    if (this.vfs.isDir(catPath)) return `cat: ${catPath}: Is a directory`;
                    const content = this.vfs.readFile(catPath);
                    if (typeof content === 'string') return content;
                    return "[Binary content]";
                } catch (e: any) {
                    return `cat: ${catPath}: No such file`;
                }

            case 'copy':
                this.terminal.copyAll();
                return null;

            case 'vfs_debug':
                console.log(this.vfs);
                return "Dumped to console";

            default:
                throw new Error(`Unknown command: ${cmd}`);
        }
    }

    async cmdNeplg2(args: string[], stdin: any): Promise<any> {
        const parsed = this.parseFlags(args);

        if (args.includes('run') || args.includes('build')) {
            this.terminal.print("Compiling...");

            let inputFile = parsed.flags['-i'] || parsed.flags['--input'];
            if (!inputFile) {
                const lastPos = parsed.positional[parsed.positional.length - 1];
                if (lastPos && lastPos !== 'run' && lastPos !== 'build') {
                    inputFile = lastPos;
                }
            }

            let source = "";
            let inputPath = "editor";

            if (inputFile) {
                if (typeof inputFile !== 'string') return "Error: Invalid input file path";
                if (!this.vfs.exists(inputFile)) return `Error: File not found '${inputFile}'`;
                source = this.vfs.readFile(inputFile) as string;
                inputPath = inputFile;
            } else {
                if (this.editor) {
                    if (typeof this.editor.getText === 'function') {
                        source = this.editor.getText();
                    } else if (this.editor.text !== undefined) {
                        source = this.editor.text;
                    } else {
                        return "Error: Could not retrieve text from editor";
                    }
                    this.terminal.print("(Using editor content)");
                } else {
                    return "Error: Editor not connected";
                }
            }
            this.terminal.print(`Source: ${inputPath}`);

            if (!window.wasmBindings) return "Error: Compiler (WASM) not loaded yet.";

            try {
                if (parsed.flags['--emit'] && (parsed.flags['--emit'] as string).includes('wat')) {
                    const wat = window.wasmBindings.compile_to_wat(source);
                    this.vfs.writeFile('out.wat', wat);
                    this.terminal.print("Generated out.wat");
                }

                const wasm = window.wasmBindings.compile_source(source);
                const outFile = (parsed.flags['-o'] as string) || 'out.wasm';
                this.vfs.writeFile(outFile, wasm);
                this.terminal.print(`Compilation finished. Output to ${outFile}`);

                if (args.includes('run')) {
                    return await this.cmdWasmi([outFile], stdin);
                }
                return "Build complete.";
            } catch (e) {
                return `Compilation Failed: ${e}`;
            }
        }
        return "Unknown neplg2 command.";
    }

    parseFlags(args: string[]) {
        const flags: Record<string, string | boolean> = {};
        const positional: string[] = [];
        for (let i = 0; i < args.length; i++) {
            if (args[i].startsWith('-')) {
                if (i + 1 < args.length && !args[i + 1].startsWith('-')) {
                    flags[args[i]] = args[i + 1];
                    i++;
                } else {
                    flags[args[i]] = true;
                }
            } else {
                positional.push(args[i]);
            }
        }
        return { flags, positional };
    }

    private activeWorker: Worker | null = null;
    private sab: SharedArrayBuffer | null = null;
    private stdinBuffer: Int32Array | null = null;
    private stdinData: Uint8Array | null = null;

    interrupt() {
        if (this.activeWorker) {
            this.activeWorker.terminate();
            this.activeWorker = null;
            this.terminal.printError("\nProcess interrupted.");
            if (this.stdinBuffer) {
                Atomics.store(this.stdinBuffer, 0, -1);
                Atomics.notify(this.stdinBuffer, 0);
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
                this.sab = new SharedArrayBuffer(1024 * 64); // 64KB stdin buffer
                this.stdinBuffer = new Int32Array(this.sab, 0, 1);
                this.stdinData = new Uint8Array(this.sab, 4);
            } catch (e) {
                console.warn("SharedArrayBuffer not supported, falling back to non-blocking (stdin will not work)");
            }
        }

        return new Promise((resolve, reject) => {
            const worker = new Worker(new URL('../runtime/worker.js', import.meta.url), { type: 'module' });
            this.activeWorker = worker;

            worker.onmessage = (e) => {
                const { type, data, code, message } = e.data;
                switch (type) {
                    case 'stdout':
                        const text = new TextDecoder().decode(new Uint8Array(data));
                        this.terminal.write(text);
                        break;
                    case 'stdin_request':
                        // In a real terminal, we might switch mode here
                        // For now we assume terminal handles input and calls handleStdin
                        break;
                    case 'exit':
                        this.activeWorker = null;
                        worker.terminate();
                        resolve(`Program exited with code ${code}`);
                        break;
                    case 'error':
                        this.activeWorker = null;
                        worker.terminate();
                        reject(new Error(message));
                        break;
                }
            };

            worker.onerror = (e) => {
                this.activeWorker = null;
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

    handleStdin(text: string) {
        if (this.stdinBuffer && this.stdinData) {
            const encoded = new TextEncoder().encode(text);
            this.stdinData.set(encoded);
            Atomics.store(this.stdinBuffer, 0, encoded.length);
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
