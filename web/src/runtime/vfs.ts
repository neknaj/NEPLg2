export class VFS {
    files: Map<string, string | Uint8Array>;
    readOnlyFiles: Set<string>;

    constructor() {
        this.files = new Map();
        this.readOnlyFiles = new Set();
        // Populated by main.js
    }

    writeFile(path: string, content: string | Uint8Array, options: { force?: boolean } = {}) {
        if (!path.startsWith('/')) path = '/' + path;
        if (!options.force && this.readOnlyFiles.has(path)) {
            throw new Error(`File is read-only: ${path}`);
        }
        this.files.set(path, content);
    }

    readFile(path: string): string | Uint8Array {
        if (!path.startsWith('/')) path = '/' + path;
        if (!this.files.has(path)) {
            throw new Error(`File not found: ${path}`);
        }
        return this.files.get(path)!;
    }

    exists(path: string): boolean {
        if (!path.startsWith('/')) path = '/' + path;
        return this.files.has(path);
    }

    setReadOnly(path: string, readOnly: boolean = true) {
        if (!path.startsWith('/')) path = '/' + path;
        if (readOnly) {
            this.readOnlyFiles.add(path);
        } else {
            this.readOnlyFiles.delete(path);
        }
    }

    isReadOnly(path: string): boolean {
        if (!path.startsWith('/')) path = '/' + path;
        return this.readOnlyFiles.has(path);
    }

    isEditable(path: string): boolean {
        if (!path.startsWith('/')) path = '/' + path;
        if (this.isReadOnly(path)) return false;
        const content = this.files.get(path);
        return typeof content === 'string';
    }

    isDir(path: string): boolean {
        if (!path.startsWith('/')) path = '/' + path;
        if (path === '/') return true;
        const prefix = path.endsWith('/') ? path : path + '/';
        for (const key of this.files.keys()) {
            if (key.startsWith(prefix)) return true;
        }
        return false;
    }

    listDir(dirPath: string): string[] {
        if (!dirPath.startsWith('/')) dirPath = '/' + dirPath;
        if (!dirPath.endsWith('/')) dirPath += '/';

        const results = new Set<string>();
        for (const path of this.files.keys()) {
            if (path.startsWith(dirPath)) {
                const relative = path.substring(dirPath.length);
                const firstSegment = relative.split('/')[0];
                if (firstSegment) {
                    results.add(firstSegment);
                }
            }
        }
        return Array.from(results).sort();
    }

    getAllFiles(): Map<string, string | Uint8Array> {
        return this.files;
    }

    serialize(): Record<string, string | Uint8Array> {
        const obj: Record<string, string | Uint8Array> = {};
        for (const [path, content] of this.files.entries()) {
            obj[path] = content;
        }
        return obj;
    }

    /**
     * Compile 用の VFS overlay は、利用者が編集した NEPL source module だけを渡す。
     * bundled stdlib は CompilerSession が保持しているため read-only file を混ぜず、
     * `.txt` や WASM binary などの runtime-only data も source module として扱わない。
     */
    serializeForCompile(): Record<string, string> {
        const obj: Record<string, string> = {};
        for (const [path, content] of this.files.entries()) {
            if (this.readOnlyFiles.has(path)) {
                continue;
            }
            if (!path.endsWith('.nepl')) {
                continue;
            }
            if (typeof content !== 'string') {
                continue;
            }
            obj[path] = content;
        }
        return obj;
    }

    deserialize(data: Record<string, string | Uint8Array>) {
        for (const [path, content] of Object.entries(data)) {
            this.files.set(path, content);
        }
    }

    deleteFile(path: string): boolean {
        if (!path.startsWith('/')) path = '/' + path;
        return this.files.delete(path);
    }
}
