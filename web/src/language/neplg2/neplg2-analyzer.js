// neplg2-analyzer.js

const KEYWORDS = new Set(['let', 'mut', 'fn', 'if', 'then', 'else', 'cond', 'while', 'do', 'block', 'set', 'break', 'return', 'mlstr', 'Tuple', 'Vec', 'List', 'Set']);
const DIRECTIVES = new Set(['#entry', '#indent', '#target', '#import', '#use', '#if', '#wasm']);
const TYPES = new Set(['i32', 'i64', 'f32', 'f64', 'bool', 'str', 'String', '()']);
const BOOLEANS = new Set(['true', 'false']);
const OPERATORS = new Set(['->', '*>', '<', '>', '|>', '@']);
const PUNCTUATION = new Set(['(', ')', ':', ';', ',', '.']);

const SNIPPETS = [
    { label: 'let', type: 'snippet', insertText: 'let $0', detail: 'let binding' },
    { label: 'fn', type: 'snippet', insertText: 'fn name $0:\n    ()', detail: 'function declaration' },
    { label: 'if', type: 'snippet', insertText: 'if:\n    cond $0\n    then ', detail: 'if statement' },
    { label: 'while', type: 'snippet', insertText: 'while:\n    cond $0\n    do ', detail: 'while loop' },
];

class NEPLg2Analyzer {
    constructor(text) {
        this.text = text;
        this.tokens = [];
        this.diagnostics = [];
        this.declarations = new Map();
        this.wordBoundaries = [];
        this.foldingRanges = [];
        this.offset = 0;
    }

    analyze() {
        this.tokenize();
        this.parse();
        this.findDiagnostics();
        this.computeWordBoundaries();
        this.computeFoldingRanges();
    }

    tokenize() {
        this.offset = 0; this.tokens = [];
        while (!this.isAtEnd()) {
            this.offset = this.skipWhitespace();
            if (this.isAtEnd()) break;
            const char = this.peek();

            // Directives (#entry, #import, etc.)
            if (char === '#') {
                this.tokens.push(this.scanDirective());
            }
            // Comments
            else if (char === '/' && this.peekNext() === '/') {
                this.tokens.push(this.scanComment());
            }
            // Strings
            else if (char === '"') {
                this.tokens.push(this.scanString());
            }
            // Numbers
            else if (this.isDigit(char)) {
                this.tokens.push(this.scanNumber());
            }
            // Identifiers and keywords
            else if (this.isAlpha(char) || char === '_') {
                this.tokens.push(this.scanIdentifier());
            }
            // Type annotations <T>
            else if (char === '<' && this.isTypeAnnotationStart()) {
                this.tokens.push(this.scanTypeAnnotation());
            }
            // Operators and arrows
            else if (this.isOperatorStart(char)) {
                this.tokens.push(this.scanOperator());
            }
            // Punctuation
            else if (PUNCTUATION.has(char)) {
                this.tokens.push(this.scanPunctuation());
            }
            else {
                this.advance();
            }
        }
    }

    createToken(type, start, end) {
        return { startIndex: start, endIndex: end, type };
    }

    scanDirective() {
        const start = this.offset;
        this.advance(); // Skip #
        while (!this.isAtEnd() && this.isAlphaNumeric(this.peek())) {
            this.advance();
        }
        // Continue to end of line for full directive
        while (!this.isAtEnd() && this.peek() !== '\n') {
            this.advance();
        }
        return this.createToken('keyword', start, this.offset);
    }

    scanComment() {
        const start = this.offset;
        while (!this.isAtEnd() && this.peek() !== '\n') {
            this.advance();
        }
        return this.createToken('comment', start, this.offset);
    }

    scanString() {
        const start = this.offset;
        this.advance(); // Skip opening "
        while (!this.isAtEnd() && this.peek() !== '"') {
            if (this.peek() === '\\\\') {
                this.advance();
            }
            this.advance();
        }
        if (!this.isAtEnd()) {
            this.advance(); // Skip closing "
        }
        return this.createToken('string', start, this.offset);
    }

    scanNumber() {
        const start = this.offset;
        while (!this.isAtEnd() && this.isDigit(this.peek())) {
            this.advance();
        }
        // Check for decimal point
        if (this.peek() === '.' && this.isDigit(this.peekNext())) {
            this.advance(); // Skip .
            while (!this.isAtEnd() && this.isDigit(this.peek())) {
                this.advance();
            }
        }
        return this.createToken('number', start, this.offset);
    }

    scanIdentifier() {
        const start = this.offset;
        while (!this.isAtEnd() && (this.isAlphaNumeric(this.peek()) || this.peek() === '_')) {
            this.advance();
        }
        const text = this.text.substring(start, this.offset);

        let type = 'variable';
        if (KEYWORDS.has(text)) {
            type = 'keyword';
        } else if (TYPES.has(text)) {
            type = 'keyword'; // Treat types as keywords for now
        } else if (BOOLEANS.has(text)) {
            type = 'boolean';
        } else {
            // Check if next non-whitespace is ( or : to determine if it's a function
            const tempOffset = this.skipWhitespace(this.offset);
            if (this.text[tempOffset] === '(' || this.text[tempOffset] === ':') {
                type = 'function';
            }
        }

        return this.createToken(type, start, this.offset);
    }

    isTypeAnnotationStart() {
        // Simple heuristic: < followed by alphanumeric
        return this.peek() === '<' && this.isAlpha(this.peekNext());
    }

    scanTypeAnnotation() {
        const start = this.offset;
        this.advance(); // Skip <
        let depth = 1;
        while (!this.isAtEnd() && depth > 0) {
            if (this.peek() === '<') depth++;
            else if (this.peek() === '>') depth--;
            this.advance();
        }
        return this.createToken('keyword', start, this.offset);
    }

    isOperatorStart(char) {
        return OPERATORS.has(char) || (char === '-' && this.peekNext() === '>') ||
            (char === '*' && this.peekNext() === '>') ||
            (char === '|' && this.peekNext() === '>');
    }

    scanOperator() {
        const start = this.offset;
        // Handle multi-character operators
        if ((this.peek() === '-' || this.peek() === '*' || this.peek() === '|') && this.peekNext() === '>') {
            this.advance();
            this.advance();
        } else {
            this.advance();
        }
        return this.createToken('operator', start, this.offset);
    }

    scanPunctuation() {
        const start = this.offset;
        this.advance();
        return this.createToken('punctuation', start, this.offset);
    }

    parse() {
        this.declarations.clear();
        for (let i = 0; i < this.tokens.length; i++) {
            const token = this.tokens[i];
            const tokenText = this.text.substring(token.startIndex, token.endIndex);

            // Look for let, fn declarations
            if (token.type === 'keyword' && (tokenText === 'let' || tokenText === 'fn')) {
                const nextToken = this.findNextMeaningfulToken(i);
                if (nextToken && (nextToken.type === 'variable' || nextToken.type === 'function' || nextToken.type === 'keyword')) {
                    const name = this.text.substring(nextToken.startIndex, nextToken.endIndex);
                    if (name !== 'mut') { // Skip 'mut' keyword
                        if (!this.declarations.has(name)) {
                            this.declarations.set(name, { index: nextToken.startIndex, type: tokenText });
                        }
                    }
                }
            }
        }
    }

    findDiagnostics() {
        this.diagnostics = [];
        // Add any specific diagnostics for NEPLg2 here
    }

    getLineAndCol(index) {
        const lines = this.text.substring(0, index).split('\n');
        const line = lines.length - 1;
        const col = lines[line].length;
        return { line, col };
    }

    computeFoldingRanges() {
        this.foldingRanges = [];
        const lines = this.text.split('\n');
        const stack = [];

        for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
            const line = lines[lineIndex];
            const trimmed = line.trim();

            // Check for block start (lines ending with :)
            if (trimmed.endsWith(':')) {
                stack.push(lineIndex);
            }

            // Check for dedent to close blocks
            if (stack.length > 0 && trimmed.length > 0) {
                const currentIndent = line.match(/^\s*/)[0].length;
                while (stack.length > 0) {
                    const startLineIndex = stack[stack.length - 1];
                    const startIndent = lines[startLineIndex].match(/^\s*/)[0].length;

                    if (currentIndent <= startIndent) {
                        const start = stack.pop();
                        if (lineIndex > start + 1) {
                            this.foldingRanges.push({
                                startLine: start,
                                endLine: lineIndex - 1,
                                placeholder: '...'
                            });
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    getCharType(char) {
        if (/\s/.test(char)) return 'space';
        if (/[\w$]/.test(char)) return 'word';
        return 'symbol';
    }

    computeWordBoundaries() {
        this.wordBoundaries = [0];
        if (this.text.length === 0) return;
        let lastType = this.getCharType(this.text[0]);
        for (let i = 1; i < this.text.length; i++) {
            const currentType = this.getCharType(this.text[i]);
            if (currentType !== lastType) {
                this.wordBoundaries.push(i);
                lastType = currentType;
            }
        }
    }

    getNextWordBoundary(index, direction) {
        if (direction === 'right') {
            for (const boundary of this.wordBoundaries) {
                if (boundary > index) return boundary;
            }
            return this.text.length;
        } else {
            for (let i = this.wordBoundaries.length - 1; i >= 0; i--) {
                const boundary = this.wordBoundaries[i];
                if (boundary < index) return boundary;
            }
            return 0;
        }
    }

    getHoverInfoAt(index) {
        const wordInfo = this.findWordAt(index);
        if (!wordInfo) return null;

        if (KEYWORDS.has(wordInfo.word)) {
            return { content: `NEPLg2 keyword: ${wordInfo.word}` };
        }

        const declaration = this.declarations.get(wordInfo.word);
        if (declaration) {
            return { content: `${declaration.type} ${wordInfo.word}` };
        }

        return null;
    }

    getDefinitionLocationAt(index) {
        const wordInfo = this.findWordAt(index);
        if (!wordInfo) return null;
        const declaration = this.declarations.get(wordInfo.word);
        if (declaration) return { targetIndex: declaration.index };
        return null;
    }

    getOccurrencesAt(index) {
        const wordInfo = this.findWordAt(index);
        if (!wordInfo || KEYWORDS.has(wordInfo.word)) return [];
        const occurrences = [];
        const wordRegex = new RegExp(`\\b${wordInfo.word}\\b`, 'g');
        let match;
        while ((match = wordRegex.exec(this.text))) {
            occurrences.push({
                startIndex: match.index,
                endIndex: match.index + match[0].length
            });
        }
        return occurrences;
    }

    getCompletions(index) {
        let suggestions = [];
        const prefixInfo = this.getPrefixAt(index);
        const prefix = prefixInfo ? prefixInfo.prefix.toLowerCase() : '';

        KEYWORDS.forEach(kw => suggestions.push({ label: kw, type: 'keyword' }));
        this.declarations.forEach((val, key) => suggestions.push({
            label: key,
            type: val.type,
            detail: val.type
        }));
        SNIPPETS.forEach(snip => suggestions.push(snip));

        if (!prefix) return suggestions;
        return suggestions.filter(s => {
            const label = s.label.toLowerCase();
            if (s.type === 'snippet') {
                const text = (s.insertText || '').toLowerCase();
                return label.startsWith(prefix) || text.startsWith(prefix);
            }
            return label.startsWith(prefix);
        });
    }

    getIndentationAt(index) {
        const lineStart = this.text.lastIndexOf('\n', index - 1) + 1;
        const line = this.text.substring(lineStart, index);
        const currentIndent = line.match(/^\s*/)[0];
        const trimmedLine = line.trim();

        // If line ends with :, increase indent
        if (trimmedLine.endsWith(':')) {
            return {
                textToInsert: '\n' + currentIndent + '    ',
                cursorOffset: currentIndent.length + 5
            };
        }

        return {
            textToInsert: '\n' + currentIndent,
            cursorOffset: currentIndent.length + 1
        };
    }

    adjustIndentationAt(selectionStart, selectionEnd, isOutdent) {
        const indentUnit = '    ';
        const oldLines = this.text.split('\n');
        const newLines = [...oldLines];

        let startLineIndex, endLineIndex;
        let charIndex = 0;

        for (let i = 0; i < oldLines.length; i++) {
            const line = oldLines[i];
            const lineEndIndex = charIndex + line.length;

            if (startLineIndex === undefined && selectionStart <= lineEndIndex) {
                startLineIndex = i;
            }

            if (startLineIndex !== undefined) {
                if (selectionEnd === charIndex && selectionStart < selectionEnd) {
                    endLineIndex = i - 1;
                    break;
                }
                if (selectionEnd <= lineEndIndex) {
                    endLineIndex = i;
                    break;
                }
            }
            charIndex += line.length + 1;
        }

        if (startLineIndex === undefined) return {
            newText: this.text,
            newSelectionStart: selectionStart,
            newSelectionEnd: selectionEnd
        };
        if (endLineIndex === undefined) endLineIndex = oldLines.length - 1;

        let firstLineDiff = 0;
        let totalDiff = 0;

        for (let i = startLineIndex; i <= endLineIndex; i++) {
            if (newLines[i].length === 0 && (i !== startLineIndex || startLineIndex !== endLineIndex)) continue;

            let diff = 0;
            if (isOutdent) {
                const leadingWhitespace = newLines[i].match(/^\s*/)[0];
                const removeCount = Math.min(leadingWhitespace.length, indentUnit.length);
                if (removeCount > 0) {
                    newLines[i] = newLines[i].substring(removeCount);
                    diff = -removeCount;
                }
            } else {
                newLines[i] = indentUnit + newLines[i];
                diff = indentUnit.length;
            }

            if (i === startLineIndex) {
                firstLineDiff = diff;
            }
            totalDiff += diff;
        }

        const newText = newLines.join('\n');
        const newSelectionStart = selectionStart + firstLineDiff;
        const newSelectionEnd = selectionEnd + totalDiff;

        return { newText, newSelectionStart, newSelectionEnd };
    }

    toggleCommentAt(selectionStart, selectionEnd) {
        const lineStartIndex = this.text.lastIndexOf('\n', selectionStart - 1) + 1;
        let lineEndIndex = this.text.indexOf('\n', selectionEnd);
        if (lineEndIndex === -1) lineEndIndex = this.text.length;

        const selectedLinesText = this.text.substring(lineStartIndex, lineEndIndex);
        const lines = selectedLinesText.split('\n');
        const isAllCommented = lines.filter(line => line.trim() !== '').every(line => line.trim().startsWith('//'));

        let newLines, selectionDelta = 0;
        if (isAllCommented) {
            newLines = lines.map(line => {
                const match = line.match(/^(\s*)\/\/\s?(.*)/);
                if (match) {
                    selectionDelta -= 3;
                    return match[1] + match[2];
                }
                return line;
            });
        } else {
            const minIndent = Math.min(...lines.filter(l => l.trim()).map(l => l.match(/^\s*/)[0].length));
            const indent = ' '.repeat(minIndent);
            newLines = lines.map(line => {
                if (line.trim() === '') return line;
                selectionDelta += 2;
                return indent + '//' + line.substring(minIndent);
            });
        }

        const newText = this.text.substring(0, lineStartIndex) + newLines.join('\n') + this.text.substring(lineEndIndex);
        return { newText, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd + selectionDelta };
    }

    getBracketMatchAt(index) {
        // Simplified bracket matching for NEPLg2
        return null;
    }

    isAtEnd(offset = this.offset) { return offset >= this.text.length; }
    peek(offset = this.offset) { return this.isAtEnd(offset) ? '\0' : this.text.charAt(offset); }
    peekNext(offset = this.offset) { return this.isAtEnd(offset + 1) ? '\0' : this.text.charAt(offset + 1); }
    advance() { this.offset++; return this.text.charAt(this.offset - 1); }
    isAlpha(char) { return (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z'); }
    isDigit(char) { return char >= '0' && char <= '9'; }
    isAlphaNumeric(char) { return this.isAlpha(char) || this.isDigit(char); }
    skipWhitespace(offset = this.offset) {
        while (offset < this.text.length && /\s/.test(this.peek(offset))) offset++;
        return offset;
    }

    findNextMeaningfulToken(currentIndex) {
        for (let i = currentIndex + 1; i < this.tokens.length; i++) {
            if (this.tokens[i].type !== 'comment') {
                return this.tokens[i];
            }
        }
        return null;
    }

    findWordAt(index) {
        const wordRegex = /[\w$]+/g;
        let match;
        while ((match = wordRegex.exec(this.text))) {
            if (index >= match.index && index <= match.index + match[0].length) {
                return {
                    word: match[0],
                    startIndex: match.index,
                    endIndex: match.index + match[0].length
                };
            }
        }
        return null;
    }

    getPrefixAt(index) {
        let start = index;
        while (start > 0 && /[\w$]/.test(this.text[start - 1])) start--;
        if (start === index) return null;
        return {
            prefix: this.text.substring(start, index),
            startIndex: start,
            endIndex: index
        };
    }
}