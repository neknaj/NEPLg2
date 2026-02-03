export class VFS {
    constructor() {
        this.files = new Map();
        // Initial state
        this.files.set('README.txt', 'Welcome to NEPLg2 Playground!\n\nThis is a virtual file system.\nYou can save compiled binaries here.');
    }

    writeFile(path, content) {
        this.files.set(path, content);
    }

    readFile(path) {
        if (!this.files.has(path)) {
            throw new Error(`File not found: ${path}`);
        }
        return this.files.get(path);
    }

    exists(path) {
        return this.files.has(path);
    }

    listFiles() {
        return Array.from(this.files.keys());
    }

    deleteFile(path) {
        return this.files.delete(path);
    }
}
