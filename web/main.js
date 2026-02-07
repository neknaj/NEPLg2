import { CanvasTerminal } from './src/terminal/terminal.js';
import { VFS } from './src/runtime/vfs.js';

console.log("main.js loaded");
let start_flag = false;

window.addEventListener("TrunkApplicationStarted", start_app);
window.setTimeout(start_app, 1000);

function start_app() {
    if (start_flag) return;
    start_flag = true;

    // --- Core Dependencies ---
    console.log("[Playground] Initializing VFS...");
    const vfs = new VFS();

    let wasm;
    try {
        wasm = window.wasmBindings
    }
    catch (e) {
        console.error("[Playground] WASM bindings not found, retrying in 1 second...", e);
        start_flag = false; // Allow retry
        window.setTimeout(start_app, 1000);
        return;
    }
    console.log("[Playground] Trunk application started. Initializing...");

    console.log("[Playground] WASM bindings:", wasm);
    if (wasm && wasm.initSync) {
        try {
            wasm.initSync();
            console.log("[Playground] WASM initSync complete.");

            // Mount stdlib into VFS
            const stdlibFiles = wasm.get_stdlib_files();
            if (stdlibFiles && Array.isArray(stdlibFiles)) {
                console.log(`[Playground] Mounting ${stdlibFiles.length} stdlib files...`);
                for (const [path, content] of stdlibFiles) {
                    vfs.writeFile('/stdlib/' + path, content);
                }
                console.log("[Playground] stdlib mounting complete.");
            }
        } catch (e) {
            console.error("[Playground] WASM initSync failed:", e);
        }
    }

    // --- DOM Elements ---
    const editorCanvas = document.getElementById('editor-canvas');
    const editorTextarea = document.getElementById('editor-hidden-input');
    const editorStatus = document.getElementById('editor-status');
    const completionList = document.getElementById('completion-list');
    const generalPopup = document.getElementById('general-popup');
    const terminalCanvas = document.getElementById('terminal-canvas');
    const terminalTextarea = document.getElementById('terminal-hidden-input');
    const exampleSelect = document.getElementById('example-select');

    // --- Editor Setup ---
    console.log("[Playground] Setting up CanvasEditor...");
    const neplProvider = new NEPLg2LanguageProvider();
    const { editor } = CanvasEditorLibrary.createCanvasEditor({
        canvas: editorCanvas,
        textarea: editorTextarea,
        popup: generalPopup,
        completionList: completionList,
        languageProviders: {
            nepl: neplProvider
        },
        initialLanguage: 'nepl'
    });
    console.log("[Playground] Editor setup complete.");

    // --- Terminal Setup ---
    console.log("[Playground] Setting up CanvasTerminal...");
    const terminal = new CanvasTerminal(terminalCanvas, terminalTextarea, null, {});

    // Inject dependencies into shell
    if (terminal.shell) {
        terminal.shell.editor = editor;
        terminal.shell.vfs = vfs;
        console.log("[Playground] Shell dependencies injected.");
    }

    // --- Simple Commands for Buttons ---
    function executeCommand(cmd) {
        console.log(`[Playground] Executing command: ${cmd}`);
        // This simulates user typing the command
        terminal.currentInput = cmd;
        terminal.execute();
    }

    // --- Example Loading Logic ---
    async function loadExamples() {
        console.log("[Playground] Loading examples list from VFS...");

        // Ensure VFS is populated before listing
        const examples = vfs.listDir('/examples');
        console.log("[Playground] Examples found in VFS:", examples);

        exampleSelect.innerHTML = '<option value="" disabled selected>Select an example...</option>';

        for (const file of examples) {
            const option = document.createElement('option');
            option.value = file;
            option.textContent = file;
            exampleSelect.appendChild(option);
        }

        console.log("[Playground] Setting default example...");
        if (examples.includes('rpn.nepl')) {
            await loadExample('rpn.nepl');
        } else if (examples.length > 0) {
            await loadExample(examples[0]);
        } else {
            console.warn("[Playground] No examples found in VFS. Fallback to helloworld?");
            // If VFS is empty, it might be a mounting issue.
        }
    }

    async function loadExample(filename) {
        console.log(`[Playground] Loading example from VFS: ${filename}`);
        try {
            const path = '/examples/' + filename;
            if (!vfs.exists(path)) {
                console.error(`[Playground] Example ${filename} not found in VFS`);
                return;
            }
            const text = vfs.readFile(path);
            editor.setText(text);
            editorStatus.textContent = path.startsWith('/') ? path.substring(1) : path;
            terminal.print([
                { text: "Loaded ", color: "#56d364" },
                { text: filename, color: "#58a6ff" }
            ]);
            // Update select to match
            exampleSelect.value = filename;
            console.log(`[Playground] Example ${filename} loaded successfully.`);
        } catch (error) {
            console.error(`[Playground] Error loading example ${filename}:`, error);
            editor.setText(`// Error loading ${filename}: ${error}`);
            terminal.printError(`Error loading ${filename}: ${error}`);
        }
    }

    async function loadSelectedExample() {
        const selectedFile = exampleSelect.value;
        if (!selectedFile) return;
        console.log(`[Playground] User selected example: ${selectedFile}`);
        await loadExample(selectedFile);
    }

    // --- Event Listeners ---
    exampleSelect.addEventListener('change', loadSelectedExample);

    window.addEventListener('resize', () => {
        editor.resizeEditor();
        terminal.resizeEditor();
    });

    // --- Initialization ---
    loadExamples();

    // Make globally available
    window.editor = editor;
    window.terminal = terminal;
    window.executeCommand = executeCommand;

    // Initial resize and focus
    console.log("[Playground] Performing initial layout...");
    setTimeout(() => {
        editor.resizeEditor();
        terminal.resizeEditor();
        editor.focus();
        console.log("[Playground] Initial layout and focus complete. Terminal visible?", !!terminalCanvas.offsetParent);
    }, 100);
}