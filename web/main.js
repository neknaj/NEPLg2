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
        console.log("[Playground] Loading examples list...");
        const knownExamples = [
            'helloworld.nepl',
            'counter.nepl',
            'fib.nepl',
            'stdio.nepl',
            'rpn.nepl',
            'abc086_a.tmp.nepl'
        ];

        exampleSelect.innerHTML = '<option value="" disabled selected>Select an example...</option>';

        for (const file of knownExamples) {
            try {
                // Use GET instead of HEAD for better compatibility
                const response = await fetch(`/examples/${file}`);
                if (response.ok) {
                    const option = document.createElement('option');
                    option.value = file;
                    option.textContent = file;
                    exampleSelect.appendChild(option);
                }
            } catch (error) {
                console.log(`[Playground] Example ${file} not found, skipping`);
            }
        }

        console.log("[Playground] Setting default example...");
        // Load default example (rpn.nepl if available, else helloworld)
        if (knownExamples.includes('rpn.nepl')) {
            await loadExample('rpn.nepl');
        } else {
            await loadExample('helloworld.nepl');
        }
    }

    async function loadExample(filename) {
        console.log(`[Playground] Loading example: ${filename}`);
        try {
            // Append timestamp to bust cache
            const response = await fetch(`/examples/${filename}?t=${Date.now()}`);
            if (!response.ok) {
                console.error(`[Playground] Failed to fetch example ${filename}: ${response.statusText}`);
                editor.setText(`// Failed to load ${filename}`);
                terminal.printError(`Failed to load ${filename}`);
                return;
            }
            const text = await response.text();
            editor.setText(text);
            editorStatus.textContent = `examples/${filename}`;
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