export class VFS {
    files: Map<string, string | Uint8Array>;

    constructor() {
        this.files = new Map();
        // Populated by main.js
    }

    writeFile(path: string, content: string | Uint8Array) {
        if (!path.startsWith('/')) path = '/' + path;
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
