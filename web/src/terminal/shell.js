import { WASI } from '../runtime/wasi.js';

export class Shell {
    constructor(terminal, vfs) {
        this.terminal = terminal;
        this.vfs = vfs; // Virtual File System
        this.env = new Map();
        this.history = [];
        this.historyIndex = 0;
        this.editor = null; // To be injected
    }

    async executeLine(line) {
        if (!line.trim()) return;
        this.history.push(line);
        this.historyIndex = this.history.length;

        // Simple pipe parsing
        const segments = line.split('|').map(s => s.trim());

        let inputData = null; // Stdin for the next command

        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];
            const args = this.parseArgs(segment);
            if (args.length === 0) continue;

            const cmd = args[0];
            const cmdArgs = args.slice(1);

            try {
                // Determine if this is the last command in chain (stdout goes to terminal)
                const isLast = (i === segments.length - 1);

                // Execute
                const output = await this.runCommand(cmd, cmdArgs, inputData);

                if (isLast) {
                    if (output) this.terminal.print(output);
                } else {
                    inputData = output; // Pass to next
                }
            } catch (e) {
                this.terminal.printError(`Error: ${e.message}`);
                break;
            }
        }
    }

    parseArgs(segment) {
        // Very basic quote aware parsing
        const args = [];
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

    async runCommand(cmd, args, stdin) {
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
                    "  cat       Display file content",
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

            case 'compile':
                return await this.cmdNeplg2(['build', '--emit', 'wat'], stdin);

            case 'test':
                // Assuming test.nepl or similar exists for testing
                return await this.cmdNeplg2(['run', 'stdlib/std/test.nepl'], stdin);

            case 'neplg2':
                return await this.cmdNeplg2(args, stdin);

            case 'wasmi':
                return await this.cmdWasmi(args, stdin);

            case 'ls':
                return this.vfs ? this.vfs.listFiles().join('\n') : "No VFS";

            case 'cat':
                if (args.length === 0) return "cat: missing operand";
                try {
                    const content = this.vfs.readFile(args[0]);
                    if (typeof content === 'string') return content;
                    return "[Binary content]";
                } catch (e) {
                    return e.message;
                }

            case 'vfs_debug':
                console.log(this.vfs);
                return "Dumped to console";

            default:
                throw new Error(`Unknown command: ${cmd}`);
        }
    }

    async cmdNeplg2(args, stdin) {
        // Usage: neplg2 run --target wasi -o out --emit wat,mini-wat input.nepl
        const parsed = this.parseFlags(args);

        if (args.includes('run') || args.includes('build')) {
            this.terminal.print("Compiling...");

            // Get Input
            const inputFile = parsed.positional[parsed.positional.length - 1];
            const useEditor = !inputFile || inputFile === 'run' || inputFile === 'build';

            let source = "";
            if (useEditor) {
                if (this.editor) {
                    source = this.editor.getText();
                    this.terminal.print("(Using editor content)");
                } else {
                    return "Error: Editor not connected";
                }
            } else {
                if (!this.vfs.exists(inputFile)) return `Error: File not found '${inputFile}'`;
                source = this.vfs.readFile(inputFile);
            }

            // Compilation
            if (!window.wasmBindings) {
                return "Error: Compiler (WASM) not loaded yet. Please wait.";
            }

            try {
                // Check if we need to emit WAT
                if (parsed.flags['--emit'] && parsed.flags['--emit'].includes('wat')) {
                    const wat = window.wasmBindings.compile_to_wat({ source });
                    this.vfs.writeFile('out.wat', wat);
                    this.terminal.print("Generated out.wat");
                }

                // Compile to WASM using the new binding
                // The binding might expect just 'source' string.
                const wasm = window.wasmBindings.compile_source(source);
                // wasm is a Uint8Array (Vec<u8>)

                const outFile = parsed.flags['-o'] || 'out.wasm';
                this.vfs.writeFile(outFile, wasm);
                this.terminal.print(`Compilation finished. Output to ${outFile}`);

                if (args.includes('run')) {
                    return await this.cmdWasmi([outFile], stdin);
                }
                return "Build complete.";
            } catch (e) {
                console.error(e);
                return `Compilation Failed: ${e}`;
            }
        }
        return "Unknown neplg2 command (try 'run').";
    }

    parseFlags(args) {
        const flags = {};
        const positional = [];
        for (let i = 0; i < args.length; i++) {
            if (args[i].startsWith('-')) {
                if (i + 1 < args.length && !args[i + 1].startsWith('-')) {
                    flags[args[i]] = args[i + 1];
                    i++;
                } else {
                    flags[args[i]] = true; // bool flag
                }
            } else {
                positional.push(args[i]);
            }
        }
        return { flags, positional };
    }

    async cmdWasmi(args, stdin) {
        if (args.length === 0) return "wasmi: missing file";
        const filename = args[0];

        if (!this.vfs.exists(filename)) return `wasmi: file not found: ${filename}`;

        const bin = this.vfs.readFile(filename);
        if (typeof bin === 'string' && bin.startsWith('BINARY')) {
            return "wasmi: [Mock] Executing placeholder WASM (Success)";
        }

        if (!(bin instanceof Uint8Array)) {
            return "wasmi: invalid binary format (must be Uint8Array)";
        }

        this.terminal.print(`Executing ${filename} ...`);

        try {
            // We pass 'wasi' instance which creates the import object
            const wasi = new WASI(args, this.env, this.vfs, this.terminal);

            // WebAssembly.instantiate matches the browser API
            const { instance } = await WebAssembly.instantiate(bin, wasi.imports);
            wasi.setMemory(instance.exports.memory);

            if (instance.exports._start) {
                instance.exports._start();
            } else if (instance.exports.main) {
                const res = instance.exports.main();
                return `Exited with ${res}`;
            } else {
                return "wasmi: no entry point (_start or main) found";
            }
        } catch (e) {
            if (e.message && e.message.includes("Exited with code")) {
                return e.message; // Just the exit message
            }
            return `wasmi error: ${e}`;
        }

        return "Program exited.";
    }
}
