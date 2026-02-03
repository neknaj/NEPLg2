import { CanvasEditor } from '../editor/editor.js';
import { Shell } from './shell.js';

export class CanvasTerminal extends CanvasEditor {
    constructor(canvas, textarea, domElements) {
        // Reuse Editor infrastructure
        super(canvas, textarea, domElements, { bindEvents: false }); // We'll bind custom events or override

        // Custom Terminal State
        this.shell = new Shell(this, null); // VFS null for now
        this.promptStr = "neplg2> ";
        this.promptLen = this.promptStr.length;

        // We override the text structure:
        // Text is one big string, but we track where the "editable" part starts.
        this.readOnlyLength = 0;

        this.setText(this.promptStr);
        this.readOnlyLength = this.promptStr.length;
        this.setCursor(this.readOnlyLength);

        // Override bindEvents to intercept specific keys
        this.inputHandler.bindEvents = this.bindTerminalEvents.bind(this);
        this.init(); // Re-init to bind new events
    }

    bindTerminalEvents() {
        // Call original bindEvents to get mouse/focus logic
        // But we need to wrap onKV/onInput to enforce Read-Only
        const superBind = Object.getPrototypeOf(this.inputHandler).bindEvents.bind(this.inputHandler);
        superBind();

        // Remove original keydown/input listeners and replace (a bit hacky, but feasible if we saved references, 
        // essentially we are monkey-patching the inputHandler instance for this subclass)

        // Easier: Override the methods on the instance
        this.inputHandler.onKeydown = this.onTerminalKeydown.bind(this);
        this.inputHandler.onInput = this.onTerminalInput.bind(this);
        this.inputHandler.onPaste = this.onTerminalPaste.bind(this);
        this.inputHandler.onCut = (e) => e.preventDefault(); // No cut from history

        // Force color for terminal
        this.colors.background = '#0d1117'; // Slightly darker
        this.font = '14px "JetBrains Mono", monospace';
    }

    async onTerminalKeydown(e) {
        if (this.isComposing) return;

        // Prevent editing before readOnlyLength
        const isEditKey = e.key.length === 1 || e.key === 'Backspace' || e.key === 'Delete' || e.key === 'Enter';
        if (isEditKey && this.cursor < this.readOnlyLength && e.key !== 'Enter') {
            // Allow navigation but not editing
            // If trying to type in RO area, move cursor to end?
            this.setCursor(this.text.length);
        }

        switch (e.key) {
            case 'Enter':
                e.preventDefault();
                await this.handleEnter();
                break;
            case 'ArrowUp':
                e.preventDefault();
                this.navigateHistory(-1);
                break;
            case 'ArrowDown':
                e.preventDefault();
                this.navigateHistory(1);
                break;
            case 'Backspace':
                if (this.cursor <= this.readOnlyLength) {
                    e.preventDefault();
                    return;
                }
                // Call super logic
                Object.getPrototypeOf(this.inputHandler).onKeydown.call(this.inputHandler, e);
                break;
            case 'usage': // ... Handle copy/paste properly
            default:
                // Call super logic
                Object.getPrototypeOf(this.inputHandler).onKeydown.call(this.inputHandler, e);
        }
    }

    onTerminalInput(e) {
        if (this.cursor < this.readOnlyLength) {
            this.setCursor(this.text.length);
        }
        Object.getPrototypeOf(this.inputHandler).onInput.call(this.inputHandler, e);
    }

    onTerminalPaste(e) {
        if (this.cursor < this.readOnlyLength) {
            this.setCursor(this.text.length);
        }
        Object.getPrototypeOf(this.inputHandler).onPaste.call(this.inputHandler, e);
    }

    async handleEnter() {
        // Extract command
        const line = this.text.substring(this.readOnlyLength);
        this.insertText('\n'); // Visual newline

        // Execute
        await this.shell.executeLine(line);

        // New Prompt
        this.printPrompt();
    }

    print(output) {
        // Ensure ends with newline
        let str = output;
        if (!str.endsWith('\n')) str += '\n';

        this.insertText(str);
        // We do typically reset readOnlyLength AFTER the prompt is printed
        // but since insertText moves cursor, we just update RO length at the end of handling
    }

    printError(err) {
        // Could use ANSI colors if we implemented them
        this.print(err);
    }

    printPrompt() {
        this.insertText(this.promptStr);
        this.readOnlyLength = this.text.length;
        this.setCursor(this.readOnlyLength);
        this.scrollToCursor();
    }

    clear() {
        this.text = "";
        this.printPrompt();
    }

    navigateHistory(dir) {
        // TODO: Implement history cycling
        // const cmd = this.shell.history[...]
    }
}
