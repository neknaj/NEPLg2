export class WASI {
    constructor(args, env, vfs, terminal) {
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
                random_get: (bufPtr, bufLen) => {
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

    setMemory(memory) {
        this.memory = memory;
    }

    // System Calls

    fd_write(fd, iovs, iovs_len, nwritten) {
        if (!this.memory) return 5; // EIO
        const view = new DataView(this.memory.buffer);
        let written = 0;

        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);

            const buffer = new Uint8Array(this.memory.buffer, ptr, len);
            const str = new TextDecoder().decode(buffer); // UTF-8 decode

            if (fd === 1 || fd === 2) { // stdout || stderr
                // We print immediately for now. 
                // Ideally this should be buffered if line-based, but for playground simple print is okay
                this.terminal.print(str.replace(/\n$/, '')); // Remove trailing newline as print adds it? No, print adds one.
                // Logic refinement: terminal.print adds \n. If str has \n, we might double newline.
                // Better to just write raw if possible or let terminal handle it.
                // Terminal uses `insertText`. 
                // Reverting terminal access: direct insert
                // But `this.terminal` is the CanvasTerminal instance...
                // Let's just use `terminal.print` but be careful about newlines.
            }
            written += len;
        }

        view.setUint32(nwritten, written, true);
        return 0; // Success
    }

    fd_read(fd, iovs, iovs_len, nread) {
        // Stdin mock
        if (fd === 0) {
            // Implementation requires blocking or async, which fd_read in valid WASM is synchronous usually?
            // Usually difficult in browser JS loop. Returning 0 implies EOF or no data.
            const view = new DataView(this.memory.buffer);
            view.setUint32(nread, 0, true);
            return 0;
        }
        return 0;
    }

    fd_fdstat_get(fd, stat) {
        // Mock
        return 0;
    }

    // Args & Env
    args_sizes_get(argc, argv_buf_size) {
        const view = new DataView(this.memory.buffer);
        view.setUint32(argc, this.args.length, true);
        const size = this.args.reduce((acc, arg) => acc + new TextEncoder().encode(arg).length + 1, 0);
        view.setUint32(argv_buf_size, size, true);
        return 0;
    }

    args_get(argv, argv_buf) {
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

    environ_sizes_get(count, buf_size) {
        // Empty env for now
        const view = new DataView(this.memory.buffer);
        view.setUint32(count, 0, true);
        view.setUint32(buf_size, 0, true);
        return 0;
    }

    environ_get(environ, environ_buf) {
        return 0;
    }

    proc_exit(rval) {
        throw new Error(`Exited with code ${rval}`);
    }
}
