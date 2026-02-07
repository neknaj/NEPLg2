import { CanvasTerminal } from '../terminal/terminal.js';
import { VFS } from './vfs.js';

export class WASI {
    args: string[];
    env: Map<string, string>;
    vfs: VFS;
    terminal: CanvasTerminal;
    imports: any;
    memory: WebAssembly.Memory | null;

    constructor(args: string[], env: Map<string, string>, vfs: VFS, terminal: CanvasTerminal) {
        this.args = args;
        this.env = env;
        this.vfs = vfs;
        this.terminal = terminal;
        this.imports = {
            wasi_snapshot_preview1: {
                fd_write: this.fd_write.bind(this),
                fd_read: this.fd_read.bind(this),
                fd_close: () => 0,
                fd_seek: () => 0,
                fd_fdstat_get: this.fd_fdstat_get.bind(this),
                environ_get: this.environ_get.bind(this),
                environ_sizes_get: this.environ_sizes_get.bind(this),
                args_get: this.args_get.bind(this),
                args_sizes_get: this.args_sizes_get.bind(this),
                proc_exit: this.proc_exit.bind(this),
                clock_time_get: () => 0, // Mock
                random_get: (bufPtr: number, bufLen: number) => {
                    if (!this.memory) return 5;
                    const mem = new Uint8Array(this.memory.buffer);
                    for (let i = 0; i < bufLen; i++) {
                        mem[bufPtr + i] = Math.floor(Math.random() * 256);
                    }
                    return 0;
                }
            }
        };
        this.memory = null;
    }

    setMemory(memory: WebAssembly.Memory) {
        this.memory = memory;
    }

    // System Calls

    fd_write(fd: number, iovs: number, iovs_len: number, nwritten: number): number {
        if (!this.memory) return 5; // EIO
        const view = new DataView(this.memory.buffer);
        let written = 0;

        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);

            const buffer = new Uint8Array(this.memory.buffer, ptr, len);
            const str = new TextDecoder().decode(buffer); // UTF-8 decode

            if (fd === 1 || fd === 2) { // stdout || stderr
                this.terminal.write(str);
            }
            written += len;
        }

        view.setUint32(nwritten, written, true);
        return 0; // Success
    }

    fd_read(fd: number, iovs: number, iovs_len: number, nread: number): number {
        if (!this.memory) return 5;
        // Stdin mock
        if (fd === 0) {
            const view = new DataView(this.memory.buffer);
            view.setUint32(nread, 0, true);
            return 0;
        }
        return 0;
    }

    fd_fdstat_get(fd: number, stat: number): number {
        // Mock
        return 0;
    }

    // Args & Env
    args_sizes_get(argc: number, argv_buf_size: number): number {
        if (!this.memory) return 5;
        const view = new DataView(this.memory.buffer);
        view.setUint32(argc, this.args.length, true);
        const size = this.args.reduce((acc, arg) => acc + new TextEncoder().encode(arg).length + 1, 0);
        view.setUint32(argv_buf_size, size, true);
        return 0;
    }

    args_get(argv: number, argv_buf: number): number {
        if (!this.memory) return 5;
        const view = new DataView(this.memory.buffer);
        const memory = new Uint8Array(this.memory.buffer);
        let currentPtr = argv_buf;

        for (let i = 0; i < this.args.length; i++) {
            view.setUint32(argv + i * 4, currentPtr, true);
            const arg = new TextEncoder().encode(this.args[i]);
            memory.set(arg, currentPtr);
            memory[currentPtr + arg.length] = 0; // null terminator
            currentPtr += arg.length + 1;
        }
        return 0;
    }

    environ_sizes_get(count: number, buf_size: number): number {
        if (!this.memory) return 5;
        const view = new DataView(this.memory.buffer);
        view.setUint32(count, 0, true);
        view.setUint32(buf_size, 0, true);
        return 0;
    }

    environ_get(environ: number, environ_buf: number): number {
        return 0;
    }

    proc_exit(rval: number) {
        throw new Error(`Exited with code ${rval}`);
    }
}
