import { VFS } from './runtime/vfs.js';
import { readCompilerAssetsFromDocument } from './runtime/compiler-assets.js';
import {
    mountBundledGuiFontResources,
    type GuiFontResourceMountError,
    type GuiFontResourceMountResult,
} from './gui-font/font-resource-vfs.js';
import './editor-core/bridge.js';
import './editor-core/language-analysis.js';
import { PlaygroundPanelManager } from './workspace/panel-manager.js';

declare const NEPLg2LanguageProvider: any;

console.log('[Playground] main.js loaded (panel workspace)');
let startFlag = false;

window.addEventListener('TrunkApplicationStarted', startApp);
window.setTimeout(startApp, 1000);

function startApp() {
    if (startFlag) {
        return;
    }
    startFlag = true;

    const vfs = new VFS();
    const guiFontResourceMountPromise: Promise<GuiFontResourceMountResult> = mountBundledGuiFontResources(vfs);
    const mountTextFile = (path: string, content: unknown, options: { readOnly?: boolean } = {}) => {
        const normalizedPath = String(path);
        const text = String(content ?? '').replace(/\r\n?/g, '\n');
        vfs.writeFile(normalizedPath, text, { force: true });
        vfs.setReadOnly(normalizedPath, Boolean(options.readOnly));
    };

    let wasm: any;
    try {
        wasm = (window as any).wasmBindings;
    } catch (error) {
        console.error('[Playground] WASM bindings not found, retrying...', error);
        startFlag = false;
        window.setTimeout(startApp, 1000);
        return;
    }

    if (wasm && wasm.initSync) {
        try {
            wasm.initSync();
            if (wasm.get_bundled_stdlib_vfs) {
                const stdlibVfs = wasm.get_bundled_stdlib_vfs();
                if (stdlibVfs && typeof stdlibVfs === 'object') {
                    for (const [path, content] of Object.entries(stdlibVfs)) {
                        mountTextFile(String(path), content, { readOnly: true });
                    }
                }
            } else if (wasm.get_stdlib_files) {
                const stdlibFiles = wasm.get_stdlib_files();
                if (stdlibFiles && Array.isArray(stdlibFiles)) {
                    for (const [path, content] of stdlibFiles) {
                        mountTextFile(`/stdlib/${path}`, content, { readOnly: true });
                    }
                }
            }

            if (wasm.get_example_files) {
                const exampleFiles = wasm.get_example_files();
                if (exampleFiles && Array.isArray(exampleFiles)) {
                    for (const [path, content] of exampleFiles) {
                        mountTextFile(`/examples/${path}`, content, { readOnly: false });
                    }
                }
            }

            if (wasm.get_readme) {
                mountTextFile('/README', wasm.get_readme(), { readOnly: true });
            }
        } catch (error) {
            console.error('[Playground] WASM initSync failed:', error);
        }
    }

    const workspaceRoot = document.getElementById('workspace-root') as HTMLElement;
    const guiWindowLayer = document.getElementById('gui-window-layer') as HTMLElement;
    const popup = document.getElementById('general-popup') as HTMLElement;
    const fontSizeSelect = document.getElementById('font-size-select') as HTMLSelectElement;
    const compilerModeSelect = document.getElementById('compiler-mode-select') as HTMLSelectElement;
    const runBtn = document.getElementById('run-button') as HTMLButtonElement;
    const compileBtn = document.getElementById('compile-button') as HTMLButtonElement;
    const helpBtn = document.getElementById('help-button') as HTMLButtonElement;
    const editorHelpBtn = document.getElementById('editor-help-button') as HTMLButtonElement;
    const resetLayoutBtn = document.getElementById('reset-layout-button') as HTMLButtonElement;
    const clearBtn = document.getElementById('clear-button') as HTMLButtonElement;
    const stopBtn = document.getElementById('stop-button') as HTMLButtonElement;
    const cursorSpan = document.getElementById('cursor-pos') as HTMLElement;
    const analysisSpan = document.createElement('span');
    analysisSpan.id = 'analysis-info';
    analysisSpan.style.opacity = '0.9';
    analysisSpan.textContent = '';
    document.querySelector('.status-left')?.appendChild(analysisSpan);
    const terminalStatusSpan = document.getElementById('terminal-status') as HTMLElement;
    let compilerMode = normalizeCompilerMode(compilerModeSelect?.value);
    (window as any).NEPLg2CompilerAssets = readCompilerAssetsFromDocument(document);

    const panelManager = new PlaygroundPanelManager({
        root: workspaceRoot,
        guiWindowLayer,
        popup,
        vfs,
        createNeplProvider: () => new NEPLg2LanguageProvider({ vfs }),
        getCompilerMode: () => compilerMode,
        beforeWasmExecution: guiFontResourceExecutionPreflight,
        cursorSpan,
        analysisSpan,
        terminalStatusSpan,
    });
    panelManager.redraw();
    guiFontResourceMountPromise.then((result) => {
        if (!result.ok) {
            terminalStatusSpan.textContent = `gui-font:${result.error.kind}`;
        }
    });

    const openInitialDocument = () => {
        const initialPath = vfs.exists('/examples/rpn.nepl') ? '/examples/rpn.nepl' : '/README';
        panelManager.openFileInFocusedEditor(initialPath);
    };

    if (!panelManager.getActiveEditorTabPath()) {
        openInitialDocument();
    }

    function executeCommand(command: string) {
        panelManager.saveFocusedEditorTab();
        panelManager.executeInFocusedTerminal(command);
    }

    async function runCurrentFile() {
        panelManager.saveFocusedEditorTab();
        if (!(await ensureGuiFontResourcesForRun())) {
            return;
        }
        const activePath = panelManager.getActiveEditorTabPath() || '/README';
        executeCommand(`neplg2 run -i ${activePath}`);
    }

    function compileCurrentFile() {
        panelManager.saveFocusedEditorTab();
        const activePath = panelManager.getActiveEditorTabPath() || '/README';
        executeCommand(`neplg2 build --emit wat -i ${activePath}`);
    }

    function updateFontSize() {
        panelManager.setFontSize(parseInt(fontSizeSelect.value, 10));
    }

    function normalizeCompilerMode(value: string | null | undefined): 'rust' | 'selfhost' {
        return value === 'selfhost' ? 'selfhost' : 'rust';
    }

    function updateCompilerMode() {
        compilerMode = normalizeCompilerMode(compilerModeSelect?.value);
        terminalStatusSpan.textContent = `wasi-target:${compilerMode}`;
    }

    async function ensureGuiFontResourcesForRun(): Promise<boolean> {
        const preflightMessage = await guiFontResourceExecutionPreflight();
        if (!preflightMessage) {
            return true;
        }
        const result = await guiFontResourceMountPromise;
        if (result.ok) {
            return true;
        }

        panelManager.ensureTerminalLeaf();
        const terminal = panelManager.getFocusedTerminalRuntime();
        if (terminal) {
            terminal.terminal.print([
                { text: 'error[gui.font_resource.mount]', color: '#ff7b72' },
                { text: ': ', color: '#c9d1d9' },
                { text: preflightMessage, color: '#c9d1d9' },
            ]);
        } else {
            console.error('[Playground] GUI font resource mount failed:', result.error);
        }
        terminalStatusSpan.textContent = `gui-font:${result.error.kind}`;
        return false;
    }

    async function guiFontResourceExecutionPreflight(): Promise<string | null> {
        const result = await guiFontResourceMountPromise;
        if (result.ok) {
            return null;
        }
        return formatGuiFontResourceMountError(result.error);
    }

    function formatGuiFontResourceMountError(error: GuiFontResourceMountError): string {
        switch (error.kind) {
            case 'FetchUnavailable':
                return `FetchUnavailable: fetch unavailable for ${error.resourcePath}`;
            case 'InvalidResourcePath':
                return `InvalidResourcePath: invalid resource path ${error.resourcePath}: ${error.reason}`;
            case 'NetworkError':
                return `NetworkError: network error for ${error.resourcePath}: ${error.message}`;
            case 'HttpError':
                return `HttpError: http ${error.status} for ${error.resourcePath}`;
            case 'InvalidBytes':
                return `InvalidBytes: invalid binary resource ${error.resourcePath}: ${error.message}`;
            case 'InvalidText':
                return `InvalidText: invalid text resource ${error.resourcePath}: ${error.message}`;
            case 'VfsWriteFailed':
                return `VfsWriteFailed: VFS write failed for ${error.vfsPath}: ${error.message}`;
        }
    }

    runBtn.addEventListener('click', () => {
        void runCurrentFile();
    });
    compileBtn.addEventListener('click', compileCurrentFile);
    helpBtn.addEventListener('click', () => executeCommand('help'));
    editorHelpBtn.addEventListener('click', (event) => {
        event.stopPropagation();
        panelManager.showEditorHelp(editorHelpBtn.getBoundingClientRect());
    });
    resetLayoutBtn.addEventListener('click', () => {
        panelManager.resetWorkspaceLayout();
        if (!panelManager.getActiveEditorTabPath()) {
            openInitialDocument();
        }
        panelManager.setFontSize(parseInt(fontSizeSelect.value, 10));
        panelManager.focusDefaultEditor();
    });
    clearBtn.addEventListener('click', () => {
        const terminal = panelManager.getFocusedTerminalRuntime();
        if (terminal) {
            terminal.terminal.clear();
        }
    });
    stopBtn.addEventListener('click', () => {
        panelManager.stopActiveProcess();
    });
    fontSizeSelect.addEventListener('change', updateFontSize);
    compilerModeSelect.addEventListener('change', updateCompilerMode);

    window.addEventListener('resize', () => panelManager.resizeAll());

    (window as any).executeCommand = executeCommand;
    (window as any).panelManager = panelManager;

    setTimeout(() => {
        updateFontSize();
        updateCompilerMode();
        panelManager.focusDefaultEditor();
    }, 100);
}
