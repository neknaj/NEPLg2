import init, * as nepl_web from '/nepl-web.js';


document.addEventListener('DOMContentLoaded', async () => {
    await init();
    // --- DOM Elements ---
    const editorCanvas = document.getElementById('editor-canvas');
    const editorTextarea = document.getElementById('editor-hidden-input');
    const editorStatus = document.getElementById('editor-status');
    const completionList = document.getElementById('completion-list');
    const generalPopup = document.getElementById('general-popup');
    const terminalCanvas = document.getElementById('terminal-canvas');
    const terminalTextarea = document.getElementById('terminal-hidden-input');
    const terminalStatus = document.getElementById('terminal-status');
    const exampleSelect = document.getElementById('example-select');
    // --- Editor Setup ---
    const neplProvider = new NeplLanguageProvider();
    neplProvider.setCompiler(nepl_web);
    const editor = new CanvasEditor(editorCanvas, editorTextarea, {
        popup: generalPopup,
        completionList: completionList,
    });
    editor.registerLanguageProvider('nepl', neplProvider);
    // --- Terminal Setup ---
    const terminal = new CanvasEditor(terminalCanvas, terminalTextarea, {
        popup: generalPopup,
        completionList: null, // No completions in terminal
    });
    terminal.isReadOnly = true;
    terminal.setText("NEPL Playground Terminal\n> ");
    // --- Example Loading Logic ---
    async function loadExamples() {
        try {
            const response = await fetch('/examples/manifest.json');
            if (!response.ok) {
                console.error("Failed to load examples manifest.");
                return;
            }
            const exampleFiles = await response.json();
            exampleSelect.innerHTML = '<option value="" disabled selected>Select an example...</option>';
            for (const file of exampleFiles) {
                const option = document.createElement('option');
                option.value = file;
                option.textContent = file;
                exampleSelect.appendChild(option);
            }
        } catch (error) {
            console.error("Error loading examples:", error);
        }
    async function loadSelectedExample() {
        const selectedFile = exampleSelect.value;
        if (!selectedFile) return;

            const response = await fetch(`/examples/${selectedFile}`);
            if (!response.ok) {
                editor.setText(`// Failed to load ${selectedFile}`);
                return;
            }
            const text = await response.text();
            editor.setText(text);
            editorStatus.textContent = `examples/${selectedFile}`;
        }
    // --- Event Listeners ---
    exampleSelect.addEventListener('change', loadSelectedExample);

    // --- Initialization ---
    loadExamples();

    // Make editor and terminal globally available for debugging
    window.editor = editor;
    window.terminal = terminal;

    // Initial resize and focus
    editor.resizeEditor();
    terminal.resizeEditor();
    editor.focus();
});