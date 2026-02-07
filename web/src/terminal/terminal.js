import { Shell } from './shell.js';

export class CanvasTerminal {
    constructor(canvas, textarea, domElements, options = {}) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.textarea = textarea;
        this.options = options;

        // Dependencies
        this.shell = new Shell(this, null);

        // State
        this.colors = {
            background: '#0d1117',
            foreground: '#c9d1d9',
            cursor: '#58a6ff',
            input: '#f0f6fc',

            // Palette for Oh My Posh mock
            orange: '#ea5e00', // Posh orange
            white: '#ffffff',
            blue: '#41a6ff',
            gray: '#8b949e',
            green: '#56d364'
        };

        // History is now Array of Span[]
        // Span: { text: string, color: string, bg?: string }
        this.history = [];

        // Prompt State
        this.promptSpans = [];
        this.updatePrompt();

        this.currentInput = "";
        this.cursorIndex = 0;

        // Composition
        this.isComposing = false;
        this.composingText = "";

        // Styling
        this.fontSize = 14;
        this.fontFamily = '"JetBrains Mono", Consolas, monospace';
        this.lineHeight = 1.4;
        this.padding = 10;

        // Metrics
        this.charWidth = 0;
        this.charHeight = 0;

        // Cursor Blinking
        this.cursorVisible = true;
        this.blinkInterval = setInterval(() => {
            this.cursorVisible = !this.cursorVisible;
            if (!this.isComposing) this.render();
        }, 500);

        this.init();
    }

    init() {
        this.updateMetrics();
        this.bindEvents();
        this.resize();
        this.focus();

        this.print([
            { text: "Welcome to ", color: this.colors.gray },
            { text: "NEPLg2 Playground", color: this.colors.blue },
            { text: " (Web Terminal)", color: this.colors.gray }
        ]);
        this.print([{ text: "Type 'help' for commands.", color: this.colors.green }]);
    }

    updatePrompt() {
        // Mocking an Oh My Posh style prompt
        // [nepl] [main] [path]>
        this.promptSpans = [
            { text: "[", color: this.colors.orange },
            { text: "nepl", color: this.colors.white }, // User
            { text: "]", color: this.colors.orange },

            { text: "[", color: this.colors.orange },
            { text: "\uE0A0 main", color: this.colors.blue }, // Git branch (using basic char if font missing)
            { text: "]", color: this.colors.orange },

            { text: "[", color: this.colors.orange },
            { text: "/web", color: this.colors.green }, // Path
            { text: "]> ", color: this.colors.orange },
        ];
    }

    updateMetrics() {
        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        const metrics = this.ctx.measureText('M');
        this.charWidth = metrics.width;
        this.charHeight = this.fontSize * this.lineHeight;
    }

    bindEvents() {
        this.textarea.addEventListener('input', this.handleInput.bind(this));
        this.textarea.addEventListener('keydown', this.handleKeydown.bind(this));
        this.textarea.addEventListener('compositionstart', this.handleCompositionStart.bind(this));
        this.textarea.addEventListener('compositionupdate', this.handleCompositionUpdate.bind(this));
        this.textarea.addEventListener('compositionend', this.handleCompositionEnd.bind(this));

        this.canvas.addEventListener('mousedown', (e) => {
            this.focus();
            e.preventDefault();
        });
    }

    resize() {
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = rect.width;
        this.canvas.height = rect.height;
        this.updateMetrics();
        this.render();
    }

    resizeEditor() {
        this.resize();
    }

    focus() {
        this.textarea.focus();
    }

    // --- Input Handling ---

    handleInput(e) {
        if (this.isComposing) return;
        if (e.data) this.insertText(e.data);
        this.textarea.value = '';
        this.render();
    }

    handleKeydown(e) {
        if (this.isComposing) return;

        switch (e.key) {
            case 'Enter':
                this.execute();
                break;
            case 'Backspace':
                this.deleteBack();
                break;
            case 'ArrowLeft':
                if (this.cursorIndex > 0) this.cursorIndex--;
                break;
            case 'ArrowRight':
                if (this.cursorIndex < this.currentInput.length) this.cursorIndex++;
                break;
            case 'c':
                if (e.ctrlKey) {
                    // Print ^C with colors
                    const cancelledState = [
                        ...this.promptSpans,
                        { text: this.currentInput + "^C", color: this.colors.input }
                    ];
                    this.history.push(cancelledState);

                    this.currentInput = "";
                    this.cursorIndex = 0;
                    this.render();
                    e.preventDefault();
                }
                break;
        }

        this.restartBlink();
        this.updateInputPosition();
        this.render();
    }

    handleCompositionStart() {
        this.isComposing = true;
    }

    handleCompositionUpdate(e) {
        this.composingText = e.data;
        this.render();
        this.updateInputPosition();
    }

    handleCompositionEnd(e) {
        this.isComposing = false;
        this.insertText(e.data);
        this.composingText = "";
        this.textarea.value = "";
        this.render();
    }

    insertText(text) {
        const pre = this.currentInput.slice(0, this.cursorIndex);
        const post = this.currentInput.slice(this.cursorIndex);
        this.currentInput = pre + text + post;
        this.cursorIndex += text.length;
    }

    deleteBack() {
        if (this.cursorIndex > 0) {
            const pre = this.currentInput.slice(0, this.cursorIndex - 1);
            const post = this.currentInput.slice(this.cursorIndex);
            this.currentInput = pre + post;
            this.cursorIndex--;
        }
    }

    async execute() {
        const cmd = this.currentInput;

        // Push Command Line to History
        const cmdLine = [
            ...this.promptSpans,
            { text: cmd, color: this.colors.input } // User input color
        ];
        this.history.push(cmdLine);

        this.currentInput = "";
        this.cursorIndex = 0;
        this.isComposing = false;

        if (cmd.trim()) {
            await this.shell.executeLine(cmd);
        }

        this.render();
    }

    /**
     * Print text to the terminal.
     * @param {string|Span[]} content - String or Array of Spans
     */
    print(content) {
        if (typeof content === 'string') {
            // Split by newline and wrap in span
            const lines = content.split('\n');
            for (const line of lines) {
                this.history.push([{ text: line, color: this.colors.foreground }]);
            }
        } else if (Array.isArray(content)) {
            // content is Span[]. Assume single line for now? 
            // Or support multiline arrays? Let's assume input is one line if array.
            this.history.push(content);
        }
        this.render();
    }

    printError(text) {
        this.print([{ text: text, color: '#ff7b72' }]); // Red color for errors
    }

    clear() {
        this.history = [];
        this.render();
    }

    // --- Rendering ---

    render() {
        // Background
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        this.ctx.textBaseline = 'top';

        const maxVisibleLines = Math.floor((this.canvas.height - this.padding * 2) / this.charHeight);
        const totalLines = this.history.length + 1; // +1 input

        // Auto-scroll
        let startLine = 0;
        if (totalLines > maxVisibleLines) {
            startLine = totalLines - maxVisibleLines;
        }

        let y = this.padding;

        // Helper to draw a line of spans
        const drawLine = (spans, startX) => {
            let x = startX;
            for (const span of spans) {
                this.ctx.fillStyle = span.color || this.colors.foreground;
                this.ctx.fillText(span.text, x, y);
                x += this.ctx.measureText(span.text).width;
            }
            return x;
        };

        // Render History
        for (let i = startLine; i < this.history.length; i++) {
            drawLine(this.history[i], this.padding);
            y += this.charHeight;
        }

        // Render Current Input Line
        if (this.history.length >= startLine) {
            // Draw Prompt
            let x = drawLine(this.promptSpans, this.padding);

            // Draw Input Text
            const preInput = this.currentInput.slice(0, this.cursorIndex);

            // Text before cursor
            this.ctx.fillStyle = this.colors.input;
            this.ctx.fillText(preInput, x, y);
            let inputX = x + this.ctx.measureText(preInput).width;

            // Composition Text (if any)
            if (this.isComposing) {
                this.ctx.fillStyle = this.colors.foreground; // or specific compose color
                this.ctx.fillText(this.composingText, inputX, y);

                // Underline
                const compWidth = this.ctx.measureText(this.composingText).width;
                this.ctx.beginPath();
                this.ctx.moveTo(inputX, y + this.charHeight - 2);
                this.ctx.lineTo(inputX + compWidth, y + this.charHeight - 2);
                this.ctx.strokeStyle = this.colors.cursor;
                this.ctx.stroke();

                inputX += compWidth;
            }

            // Post Input (text after cursor)
            // Cursor is drawn AT inputX (before post input)
            // Draw Cursor first or on top?

            // Draw Cursor
            if (this.cursorVisible && !this.isComposing) {
                this.ctx.fillStyle = this.colors.cursor;
                this.ctx.fillRect(inputX, y, 2, this.charHeight); // Thin cursor
            }

            // Text after cursor
            const postInput = this.currentInput.slice(this.cursorIndex);
            this.ctx.fillStyle = this.colors.input;
            this.ctx.fillText(postInput, inputX, y);
        }
    }

    restartBlink() {
        this.cursorVisible = true;
        clearInterval(this.blinkInterval);
        this.blinkInterval = setInterval(() => {
            this.cursorVisible = !this.cursorVisible;
            if (!this.isComposing) this.render();
        }, 500);
    }

    updateInputPosition() {
        // Calculate pixel position for hidden textarea
        // Needs prompt width + preInput width
        // Simplified: just render off-screen if we don't need candidate window EXACTLY positioned
        // But for IME, exact position is nice.

        let promptWidth = 0;
        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        for (const span of this.promptSpans) {
            promptWidth += this.ctx.measureText(span.text).width;
        }

        const preText = this.currentInput.slice(0, this.cursorIndex) + (this.isComposing ? this.composingText : "");
        const inputWidth = this.ctx.measureText(preText).width;

        const x = this.padding + promptWidth + inputWidth;

        const maxVisibleLines = Math.floor((this.canvas.height - this.padding * 2) / this.charHeight);
        const totalLines = this.history.length + 1;
        let visualRow = totalLines - 1;
        if (totalLines > maxVisibleLines) visualRow = maxVisibleLines - 1;

        const y = this.padding + visualRow * this.charHeight;

        this.textarea.style.left = `${this.canvas.offsetLeft + x}px`;
        this.textarea.style.top = `${this.canvas.offsetTop + y}px`;
        this.textarea.style.height = `${this.charHeight}px`;
    }
}
