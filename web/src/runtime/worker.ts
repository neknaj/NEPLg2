import { WASI } from './wasi.js';
import { VFS } from './vfs.js';

// Worker-side WASI that handles blocking I/O
class WorkerWASI extends WASI {
    stdinBuffer: Int32Array | null = null;
    stdinData: Uint8Array | null = null;

    constructor(args: string[], env: Map<string, string>, vfs: VFS, buffer: SharedArrayBuffer) {
        super(args, env, vfs, null as any); // Terminal is null here, we use postMessage
        this.stdinBuffer = new Int32Array(buffer, 0, 1);
        this.stdinData = new Uint8Array(buffer, 4);
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
        if (!this.memory || !this.stdinBuffer || !this.stdinData) return 5;

        // Signal that we are waiting for input
        self.postMessage({ type: 'stdin_request' });

        // Wait for main thread to signal input
        // Atomics.wait returns 'ok', 'not-equal', or 'timed-out'
        Atomics.wait(this.stdinBuffer, 0, 0);

        const available = this.stdinBuffer[0];
        if (available < 0) return 0; // Interrupted or EOF

        const view = new DataView(this.memory.buffer);
        let totalRead = 0;
        let bufferOffset = 0;

        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);
            const remainingInStdin = available - bufferOffset;
            const toRead = Math.min(len, remainingInStdin);

            if (toRead > 0) {
                const mem = new Uint8Array(this.memory.buffer, ptr, toRead);
                mem.set(this.stdinData.subarray(bufferOffset, bufferOffset + toRead));
                totalRead += toRead;
                bufferOffset += toRead;
            }
        }

        // Reset buffer signal
        Atomics.store(this.stdinBuffer, 0, 0);
        view.setUint32(nread, totalRead, true);

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
