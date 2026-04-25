const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..');
const sourceDir = path.join(repoRoot, 'examples');
const targetDir = path.join(repoRoot, 'web', 'examples');

function listNeplFiles(dir) {
    return fs.readdirSync(dir)
        .filter((name) => name.endsWith('.nepl'))
        .sort();
}

fs.rmSync(targetDir, { recursive: true, force: true });
fs.mkdirSync(targetDir, { recursive: true });

for (const name of listNeplFiles(sourceDir)) {
    fs.copyFileSync(path.join(sourceDir, name), path.join(targetDir, name));
}

console.log(`synced ${listNeplFiles(sourceDir).length} example files to web/examples`);
