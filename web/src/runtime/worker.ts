import { WASI } from './wasi.js';
import { VFS } from './vfs.js';

// Worker-side WASI that handles blocking I/O
class WorkerWASI extends WASI {
    stdinBuffer: Int32Array | null = null;
    stdinData: Uint8Array | null = null;
    private stdinOffset = 0;
    private stdinTotal = 0;

    constructor(args: string[], env: Map<string, string>, vfs: VFS, buffer: SharedArrayBuffer) {
        super(args, env, vfs, null as any); // Terminal is null here, we use postMessage
        if (buffer) {
            this.stdinBuffer = new Int32Array(buffer, 0, 1);
            this.stdinData = new Uint8Array(buffer, 4);
        }
    }

    fd_write(fd: number, iovs: number, iovs_len: number, nwritten: number): number {
        if (!this.memory) return 5;
        const view = new DataView(this.memory.buffer);
        let totalWritten = 0;

        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);
            const buffer = new Uint8Array(this.memory.buffer, ptr, len);

            // Send output to main thread
            self.postMessage({ type: 'stdout', fd, data: Array.from(buffer) });
            totalWritten += len;
        }

        view.setUint32(nwritten, totalWritten, true);
        return 0;
    }

    fd_read(fd: number, iovs: number, iovs_len: number, nread: number): number {
        if (fd !== 0) return super.fd_read(fd, iovs, iovs_len, nread);
        if (!this.memory || !this.stdinBuffer || !this.stdinData) {
            // console.warn("stdin not available");
            return 5;
        }

        const view = new DataView(this.memory.buffer);

        // If no data is currently in our local offset, wait for more
        if (this.stdinOffset >= this.stdinTotal) {
            this.stdinOffset = 0;
            this.stdinTotal = 0;
            // Signal that we are waiting
            self.postMessage({ type: 'stdin_request' });

            try {
                // Wait while the value at index 0 is 0
                // console.log("Worker waiting for stdin (Atomics.wait)...");
                const res = Atomics.wait(this.stdinBuffer, 0, 0);
                // console.log("Atomics.wait returned:", res);
            } catch (e) {
                console.error("Atomics.wait failed (isolation might be missing):", e);
                view.setUint32(nread, 0, true);
                return 0; // EOF fallback
            }

            this.stdinTotal = Atomics.load(this.stdinBuffer, 0);
            // console.log("Worker woke up, stdinTotal:", this.stdinTotal);
            if (this.stdinTotal < 0) {
                // Interrupted
                view.setUint32(nread, 0, true);
                return 0;
            }
        }

        let bytesRead = 0;
        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);

            const remaining = this.stdinTotal - this.stdinOffset;
            const toRead = Math.min(len, remaining);

            if (toRead > 0) {
                const mem = new Uint8Array(this.memory.buffer, ptr, toRead);
                mem.set(this.stdinData.subarray(this.stdinOffset, this.stdinOffset + toRead));
                this.stdinOffset += toRead;
                bytesRead += toRead;
            }
        }

        // If we consumed everything, reset the buffer signal for the main thread
        if (this.stdinOffset >= this.stdinTotal) {
            Atomics.store(this.stdinBuffer, 0, 0);
        }

        view.setUint32(nread, bytesRead, true);
        return 0;
    }
}

self.onmessage = async (e) => {
    const { type, bin, args, env, vfsData, sab } = e.data;
    if (type === 'run') {
        const vfs = new VFS();
        vfs.deserialize(vfsData);

        const envMap = new Map<string, string>();
        for (const [k, v] of Object.entries(env || {})) {
            envMap.set(k, v as string);
        }

        const wasi = new WorkerWASI(args, envMap, vfs, sab);
        try {
            const { instance } = await WebAssembly.instantiate(bin, wasi.imports);
            wasi.setMemory(instance.exports.memory as WebAssembly.Memory);

            if (instance.exports._start) {
                (instance.exports._start as Function)();
            } else if (instance.exports.main) {
                (instance.exports.main as Function)();
            }
            self.postMessage({ type: 'exit', code: 0 });
        } catch (err: any) {
            self.postMessage({ type: 'error', message: err.message });
        }
    }
};
