export class VFS {
    constructor() {
        this.files = new Map();
        // Initial state
        this.writeFile('/README.txt', 'Welcome to NEPLg2 Playground!\n\nThis is a virtual file system.\nYou can save compiled binaries here.');
    }

    writeFile(path, content) {
        // Ensure path starts with /
        if (!path.startsWith('/')) path = '/' + path;
        this.files.set(path, content);
    }

    readFile(path) {
        if (!path.startsWith('/')) path = '/' + path;
        if (!this.files.has(path)) {
            throw new Error(`File not found: ${path}`);
        }
        return this.files.get(path);
    }

    exists(path) {
        if (!path.startsWith('/')) path = '/' + path;
        return this.files.has(path);
    }

    isDir(path) {
        if (!path.startsWith('/')) path = '/' + path;
        if (path === '/') return true;
        const prefix = path.endsWith('/') ? path : path + '/';
        for (const key of this.files.keys()) {
            if (key.startsWith(prefix)) return true;
        }
        return false;
    }

    /**
     * Lists files and directories in a given path.
     * @param {string} dirPath The directory path to list.
     * @returns {string[]} List of names (files or directories).
     */
    listDir(dirPath) {
        if (!dirPath.startsWith('/')) dirPath = '/' + dirPath;
        if (!dirPath.endsWith('/')) dirPath += '/';

        const results = new Set();
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

    deleteFile(path) {
        if (!path.startsWith('/')) path = '/' + path;
        return this.files.delete(path);
    }
}
