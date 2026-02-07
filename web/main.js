import { CanvasTerminal } from './src/terminal/terminal.js';

addEventListener("TrunkApplicationStarted", () => {
    const wasm = window.wasmBindings;

    console.log("WASM bindings:", wasm);
    if (wasm && wasm.initSync) {
        wasm.initSync();
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

    // --- Terminal Setup ---
    console.log("Creating CanvasTerminal with:", terminalCanvas, terminalTextarea);
    let terminal;
    try {
        terminal = new CanvasTerminal(terminalCanvas, terminalTextarea, null, {});
        console.log("Terminal created successfully:", terminal);
    } catch (error) {
        console.error("Failed to create terminal:", error);
        // Fallback: create a simple mock terminal
        terminal = {
            print: (msg) => console.log("Terminal:", msg),
            printError: (msg) => console.error("Terminal:", msg),
            shell: { editor: null }
        };
    }

    // Inject editor reference into shell
    if (terminal.shell) {
        terminal.shell.editor = editor;
    }

    // --- Simple Commands for Buttons ---
    function executeCommand(cmd) {
        if (!terminal.execute) {
            console.error("Terminal does not have execute method");
            return;
        }
        // This simulates user typing the command
        terminal.currentInput = cmd;
        terminal.execute();
    }

    // --- Example Loading Logic ---
    async function loadExamples() {
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
                const response = await fetch(`/examples/${file}`, { method: 'HEAD' });
                if (response.ok) {
                    const option = document.createElement('option');
                    option.value = file;
                    option.textContent = file;
                    exampleSelect.appendChild(option);
                }
            } catch (error) {
                console.log(`Example ${file} not found, skipping`);
            }
        }

        // Load default example (rpn.nepl)
        await loadExample('rpn.nepl');
    }

    async function loadExample(filename) {
        try {
            const response = await fetch(`/examples/${filename}`);
            if (!response.ok) {
                editor.setText(`// Failed to load ${filename}`);
                if (terminal.printError) {
                    terminal.printError(`Failed to load ${filename}`);
                }
                return;
            }
            const text = await response.text();
            editor.setText(text);
            editorStatus.textContent = `examples/${filename}`;
            if (terminal.print) {
                terminal.print([
                    { text: "Loaded ", color: "#56d364" },
                    { text: filename, color: "#58a6ff" }
                ]);
            }
            // Update select to match
            exampleSelect.value = filename;
        } catch (error) {
            editor.setText(`// Error loading ${filename}: ${error}`);
            if (terminal.printError) {
                terminal.printError(`Error loading ${filename}: ${error}`);
            }
        }
    }

    async function loadSelectedExample() {
        const selectedFile = exampleSelect.value;
        if (!selectedFile) return;
        await loadExample(selectedFile);
    }

    // --- Event Listeners ---
    exampleSelect.addEventListener('change', loadSelectedExample);

    // --- Initialization ---
    loadExamples();

    // Make globally available
    window.editor = editor;
    window.terminal = terminal;
    window.executeCommand = executeCommand;

    // Initial resize and focus
    editor.resizeEditor();
    if (terminal.resizeEditor) {
        terminal.resizeEditor();
    }
    editor.focus();
});