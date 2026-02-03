import { WASI } from '../runtime/wasi.js';

export class Shell {
    constructor(terminal, vfs) {
        this.terminal = terminal;
        this.vfs = vfs; // Virtual File System
        this.env = new Map();
        this.history = [];
        this.historyIndex = 0;
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
                // simple echo that ignores stdin if args present, else prints stdin
                if (args.length > 0) return args.join(' ');
                return stdin || "";

            case 'clear':
            case 'cls':
                this.terminal.clear();
                return null;

            case 'help':
                return "Available commands: echo, clear, help, neplg2, wasmi, ls";

            case 'neplg2':
                return await this.cmdNeplg2(args, stdin);

            case 'wasmi':
                return await this.cmdWasmi(args, stdin);

            case 'ls':
                return this.vfs.listFiles().join('\n');

            case 'cat':
                if (args.length === 0) return "cat: missing operand";
                try {
                    const content = this.vfs.readFile(args[0]);
                    // Handle binary content if needed (convert to hex or string)
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

        if (args.includes('run')) {
            this.terminal.print("Compiling...");

            // Get Input
            const inputFile = parsed.positional[parsed.positional.length - 1] || 'editor_source';

            if (inputFile === 'editor_source' || !this.vfs.exists(inputFile)) {
                // Mock: "Compiling [Content from Editor]..."
            }

            // Simulating output generation
            if (parsed.flags['--emit'] && parsed.flags['--emit'].includes('wat')) {
                this.vfs.writeFile('out.wat', `(module (func $main (export "main") (result i32) i32.const 42))`);
                this.terminal.print("Generated out.wat");
            }

            // Generate a dummy valid WASM for testing 'wasmi' (Minimal valid module)
            // Magic: \0asm, Version: 1
            const wasmPlaceholder = new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);

            const outFile = parsed.flags['-o'] || 'out.wasm';
            this.vfs.writeFile(outFile, wasmPlaceholder);
            this.terminal.print(`Compilation finished. Output to ${outFile}`);

            if (parsed.flags['--target'] === 'wasi') {
                // Auto-run if 'run' was the command? Usually 'run' implies execute.
                // But logic says: compile to target, THEN run? 
                // If the command is just 'run', we execute the result.
                return await this.cmdWasmi([outFile], stdin);
            }
            return "Build complete.";
        }
        return "Unknown neplg2 command.";
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
        if (bin instanceof Uint8Array && bin.length === 8) {
            return "wasmi: [Mock] Executing placeholder WASM (Success)";
        }

        if (!(bin instanceof Uint8Array)) {
            return "wasmi: invalid binary format (must be Uint8Array)";
        }

        this.terminal.print(`Executing ${filename} ...`);

        try {
            // Basic imports for standard modules
            // We pass 'wasi' instance which creates the import object
            const wasi = new WASI(args, this.env, this.vfs, this.terminal);

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
                return e.message;
            }
            return `wasmi error: ${e}`;
        }

        return "Program exited.";
    }
}
