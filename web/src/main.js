import { CanvasEditor } from './editor/editor.js';
import { NeplLanguageProvider } from './language/nepl-provider.js';

// --- Initialization ---

// Editor Setup
const editorCanvas = document.getElementById('editor-canvas');
const editorTextarea = document.getElementById('editor-hidden-input');
const editorCompletionList = document.getElementById('completion-list');
// Problems panel left out of index.html for simplicity, passing null
// Popup left out of this specific container in index.html, using global one or implementing one
const globalPopup = document.getElementById('general-popup');

const editor = new CanvasEditor(
    editorCanvas,
    editorTextarea,
    {
        popup: globalPopup,
        problemsPanel: null,
        completionList: editorCompletionList
    }
);

const neplProvider = new NeplLanguageProvider();
editor.registerLanguageProvider(neplProvider);

// --- Example Loading ---
const exampleSelect = document.getElementById('example-select');
const examples = ['rpn.nepl', 'fib.nepl', 'counter.nepl', 'helloworld.nepl', 'stdio.nepl'];

examples.forEach(ex => {
    const opt = document.createElement('option');
    opt.value = ex;
    opt.textContent = ex;
    exampleSelect.appendChild(opt);
});

exampleSelect.addEventListener('change', async (e) => {
    const file = e.target.value;
    if (!file) return;
    try {
        const res = await fetch(`examples/${file}`);
        if (!res.ok) throw new Error("Failed to load");
        const text = await res.text();
        editor.setText(text);
        document.getElementById('editor-status').textContent = `examples/${file}`;
    } catch (err) {
        console.error(err);
        editor.setText("// Error loading example: " + file);
    }
});

// Load default (RPN)
exampleSelect.value = 'rpn.nepl';
exampleSelect.dispatchEvent(new Event('change'));


import { CanvasTerminal } from './terminal/terminal.js';
import { VFS } from './runtime/vfs.js';

// VFS Setup
const vfs = new VFS();

// Terminal Setup
const terminalCanvas = document.getElementById('terminal-canvas');
const terminalTextarea = document.getElementById('terminal-hidden-input');

const terminal = new CanvasTerminal(terminalCanvas, terminalTextarea, {
    popup: globalPopup,
    problemsPanel: null,
    completionList: null // No completion in terminal for now
});
terminal.shell.vfs = vfs; // Inject VFS


// --- UI Logic ---

// Split Pane Resizer
const resizer = document.getElementById('workspace-resizer');
const workspace = document.querySelector('.workspace');
const editorPane = document.querySelector('.editor-pane');
const terminalPane = document.querySelector('.terminal-pane');

let isResizing = false;

resizer.addEventListener('mousedown', (e) => {
    isResizing = true;
    resizer.classList.add('dragging');
    document.body.style.cursor = 'col-resize';
});

document.addEventListener('mousemove', (e) => {
    if (!isResizing) return;
    const containerRect = workspace.getBoundingClientRect();
    const x = e.clientX - containerRect.left;

    // Limits
    const minWidth = 100;
    if (x < minWidth || x > containerRect.width - minWidth) return;

    const percentage = (x / containerRect.width) * 100;
    editorPane.style.width = `${percentage}%`;
    terminalPane.style.width = `${100 - percentage}%`;

    editor.resizeEditor();
    terminal.resizeEditor();
});

document.addEventListener('mouseup', () => {
    if (isResizing) {
        isResizing = false;
        resizer.classList.remove('dragging');
        document.body.style.cursor = '';
    }
});

// Window Resize Handling
window.addEventListener('resize', () => {
    editor.resizeEditor();
    terminal.resizeEditor();
});

console.log("NEPLg2 Playground initialized.");
