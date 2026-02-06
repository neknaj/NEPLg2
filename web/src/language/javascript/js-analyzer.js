// js-analyzer.js

const KEYWORDS = new Set(['class', 'extends', 'super', 'const', 'let', 'var', 'function', 'async', 'await', 'new', 'if', 'else', 'return', 'for', 'while', 'do', 'switch', 'case', 'default', 'break', 'continue', 'try', 'catch', 'finally', 'import', 'export', 'from', 'as', 'this']);
const BOOLEANS = new Set(['true', 'false']);
const OPERATORS = new Set(['+', '-', '*', '/', '%', '<', '>', '=', '!', '&', '|', '?', ':', '.']);
const PUNCTUATION = new Set(['{', '}', '(', ')', '[', ']', ',', ';']);
const SNIPPETS = [
    { label: 'log', type: 'snippet', insertText: 'console.log($0);', detail: 'console.log(...)' },
    { label: 'for', type: 'snippet', insertText: 'for (let i = 0; i < 10; i++) {\n    $0\n}', detail: 'for loop' },
    { label: 'if', type: 'snippet', insertText: 'if ($0) {\n    \n}', detail: 'if statement' },
    { label: 'ifelse', type: 'snippet', insertText: 'if ($0) {\n    \n} else {\n    \n}', detail: 'if/else statement' },
    { label: 'func', type: 'snippet', insertText: 'function name($0) {\n    \n}', detail: 'function declaration' },
];

class JavaScriptAnalyzer {
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
            if (this.isAlpha(char) || char === '_' || char === '$') this.tokens.push(this.scanIdentifier());
            else if (this.isDigit(char)) this.tokens.push(this.scanNumber());
            else if (char === '"' || char === "'" || char === '`') this.tokens.push(this.scanString());
            else if (char === '/' && this.peekNext() === '/') this.tokens.push(this.scanSingleLineComment());
            else if (char === '/' && this.peekNext() === '*') this.tokens.push(this.scanMultiLineComment());
            else if (char === '/' && this.isRegexStart()) this.tokens.push(this.scanRegex());
            else if (OPERATORS.has(char)) this.tokens.push(this.scanOperator());
            else if (PUNCTUATION.has(char)) this.tokens.push(this.scanPunctuation());
            else this.advance();
        }
    }
    createToken(type, start, end) { return { startIndex: start, endIndex: end, type }; }
    scanIdentifier() { const start = this.offset; while (!this.isAtEnd() && (this.isAlphaNumeric(this.peek()) || this.peek() === '_' || this.peek() === '$')) this.advance(); const text = this.text.substring(start, this.offset); let type = 'variable'; if (KEYWORDS.has(text)) type = 'keyword'; else if (BOOLEANS.has(text)) type = 'boolean'; else { const tempOffset = this.skipWhitespace(this.offset); if(this.text[tempOffset] === '(') type = 'function'; } return this.createToken(type, start, this.offset); }
    scanNumber() { const start = this.offset; while (!this.isAtEnd() && this.isDigit(this.peek())) this.advance(); if (this.peek() === '.' && this.isDigit(this.peekNext())) { this.advance(); while (!this.isAtEnd() && this.isDigit(this.peek())) this.advance(); } return this.createToken('number', start, this.offset); }
    scanString() { const start = this.offset; const quote = this.advance(); while (!this.isAtEnd() && this.peek() !== quote) { if (this.peek() === '\\' && !this.isAtEnd()) this.advance(); this.advance(); } if (!this.isAtEnd()) this.advance(); return this.createToken('string', start, this.offset); }
    scanSingleLineComment() { const start = this.offset; while(!this.isAtEnd() && this.peek() !== '\n') this.advance(); return this.createToken('comment', start, this.offset); }
    scanMultiLineComment() { const start = this.offset; this.advance(); this.advance(); while(!this.isAtEnd() && (this.peek() !== '*' || this.peekNext() !== '/')) this.advance(); if(!this.isAtEnd()) { this.advance(); this.advance(); } return this.createToken('comment', start, this.offset); }
    scanRegex() { const start = this.offset; this.advance(); while(!this.isAtEnd() && this.peek() !== '/') { if (this.peek() === '\\') this.advance(); this.advance(); } if(!this.isAtEnd()) this.advance(); while(!this.isAtEnd() && this.isAlpha(this.peek())) this.advance(); return this.createToken('regex', start, this.offset); }
    scanOperator() { const start = this.offset; while (!this.isAtEnd() && OPERATORS.has(this.peek())) this.advance(); if (this.text[start] === '.' && this.offset > start + 1) { this.offset = start + 1; return this.createToken('operator', start, this.offset); } return this.createToken('operator', start, this.offset); }
    scanPunctuation() { const start = this.offset; this.advance(); return this.createToken('punctuation', start, this.offset); }

    parse() {
        this.declarations.clear(); for (let i = 0; i < this.tokens.length; i++) { const token = this.tokens[i]; const tokenText = this.text.substring(token.startIndex, token.endIndex); if (token.type === 'keyword' && ['const', 'let', 'var', 'function', 'class'].includes(tokenText)) { const nextToken = this.findNextMeaningfulToken(i); if (nextToken && (nextToken.type === 'variable' || nextToken.type === 'function')) { const name = this.text.substring(nextToken.startIndex, nextToken.endIndex); if (!this.declarations.has(name)) this.declarations.set(name, { index: nextToken.startIndex, type: tokenText }); } } }
    }
    findDiagnostics() { this.diagnostics = []; const regex = /console\.log/g; let match; while((match = regex.exec(this.text))) { this.diagnostics.push({ startIndex: match.index, endIndex: match.index + 11, message: 'デバッグ用のconsole.logが残っています。', severity: 'warning' }); } }

    getLineAndCol(index) { const lines = this.text.substring(0, index).split('\n'); const line = lines.length - 1; const col = lines[line].length; return { line, col }; }
    
    computeFoldingRanges() {
        this.foldingRanges = [];
        const stack = [];
        const openBrackets = new Set(['{', '[']);
        const closeBrackets = new Set(['}', ']']);
    
        for (const token of this.tokens) {
            if (token.type === 'punctuation') {
                const char = this.text.substring(token.startIndex, token.endIndex);
                if (openBrackets.has(char)) {
                    stack.push(token.startIndex);
                } else if (closeBrackets.has(char) && stack.length > 0) {
                    const startIndex = stack.pop();
                    const { line: startLine } = this.getLineAndCol(startIndex);
                    const { line: endLine } = this.getLineAndCol(token.startIndex);
    
                    if (startLine < endLine) {
                        this.foldingRanges.push({ startLine, endLine, placeholder: '...' });
                    }
                }
            }
        }
    }

    getCharType(char) { if (/\s/.test(char)) return 'space'; if (/[\w$]/.test(char)) return 'word'; return 'symbol'; }
    
    computeWordBoundaries() { this.wordBoundaries = [0]; if (this.text.length === 0) return; let lastType = this.getCharType(this.text[0]); for (let i = 1; i < this.text.length; i++) { const currentType = this.getCharType(this.text[i]); if (currentType !== lastType) { this.wordBoundaries.push(i); lastType = currentType; } } }

    getNextWordBoundary(index, direction) { if (direction === 'right') { for (const boundary of this.wordBoundaries) { if (boundary > index) return boundary; } return this.text.length; } else { for (let i = this.wordBoundaries.length - 1; i >= 0; i--) { const boundary = this.wordBoundaries[i]; if (boundary < index) return boundary; } return 0; } }
    
    getHoverInfoAt(index) { const wordInfo = this.findWordAt(index); if (!wordInfo) return null; if (wordInfo.word === 'console') return { content: 'Console API へのアクセスを提供します。' }; if (wordInfo.word === 'greet' && this.declarations.has('greet')) return { content: 'function greet(name: string): string\n\n指定された名前で挨拶を返します。' }; return null; }
    getDefinitionLocationAt(index) { const wordInfo = this.findWordAt(index); if (!wordInfo) return null; const declaration = this.declarations.get(wordInfo.word); if (declaration) return { targetIndex: declaration.index }; return null; }
    getOccurrencesAt(index) { const wordInfo = this.findWordAt(index); if (!wordInfo || KEYWORDS.has(wordInfo.word)) return []; const occurrences = []; const wordRegex = new RegExp(`\\b${wordInfo.word}\\b`, 'g'); let match; while ((match = wordRegex.exec(this.text))) { occurrences.push({ startIndex: match.index, endIndex: match.index + match[0].length }); } return occurrences; }
    
    getCompletions(index) { let suggestions = []; const prefixInfo = this.getPrefixAt(index); const prefix = prefixInfo ? prefixInfo.prefix.toLowerCase() : ''; KEYWORDS.forEach(kw => suggestions.push({ label: kw, type: 'keyword' })); this.declarations.forEach((val, key) => suggestions.push({ label: key, type: val.type, detail: val.type })); SNIPPETS.forEach(snip => suggestions.push(snip)); if (!prefix) return suggestions; return suggestions.filter(s => { const label = s.label.toLowerCase(); if (s.type === 'snippet') { const text = (s.insertText || '').toLowerCase(); return label.startsWith(prefix) || text.startsWith(prefix); } return label.startsWith(prefix); }); }
    
    getIndentationAt(index) {
        const lineStart = this.text.lastIndexOf('\n', index - 1) + 1;
        const line = this.text.substring(lineStart, index);
        const currentIndent = line.match(/^\s*/)[0];
        const trimmedLine = line.trim();
        const charBefore = this.text[index - 1] || '\n';
        const charAfter = this.text[index] || '\n';
        if ((charBefore === '{' && charAfter === '}') || (charBefore === '(' && charAfter === ')')) {
            const newIndent = currentIndent + '    ';
            return { textToInsert: `\n${newIndent}\n${currentIndent}`, cursorOffset: newIndent.length + 1 };
        }
        if (trimmedLine.endsWith('{') || trimmedLine.endsWith('(') || trimmedLine.endsWith('[')) {
            return { textToInsert: '\n' + currentIndent + '    ', cursorOffset: currentIndent.length + 5 };
        }
        return { textToInsert: '\n' + currentIndent, cursorOffset: currentIndent.length + 1 };
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

        if (startLineIndex === undefined) return { newText: this.text, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd };
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

    toggleCommentAt(selectionStart, selectionEnd) { const lineStartIndex = this.text.lastIndexOf('\n', selectionStart - 1) + 1; let lineEndIndex = this.text.indexOf('\n', selectionEnd); if (lineEndIndex === -1) lineEndIndex = this.text.length; const selectedLinesText = this.text.substring(lineStartIndex, lineEndIndex); const lines = selectedLinesText.split('\n'); const isAllCommented = lines.filter(line => line.trim() !== '').every(line => line.trim().startsWith('//')); let newLines, selectionDelta = 0; if (isAllCommented) { newLines = lines.map(line => { const match = line.match(/^(\s*)\/\/\s?(.*)/); if (match) { selectionDelta -= 3; return match[1] + match[2]; } return line; }); } else { const minIndent = Math.min(...lines.filter(l => l.trim()).map(l => l.match(/^\s*/)[0].length)); const indent = ' '.repeat(minIndent); newLines = lines.map(line => { if (line.trim() === '') return line; selectionDelta += 2; return indent + '//' + line.substring(minIndent); }); } const newText = this.text.substring(0, lineStartIndex) + newLines.join('\n') + this.text.substring(lineEndIndex); return { newText, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd + selectionDelta }; }
    
    findMatchingBracket(index) {
        const bracketPairs = { '(': ')', '[': ']', '{': '}', ')': '(', ']': '[', '}': '{' };
        const char = this.text[index];
        if (!bracketPairs[char]) return null;
    
        const nonCodeTokens = this.tokens.filter(t => t.type === 'string' || t.type === 'comment');
        const isInNonCode = (idx) => nonCodeTokens.some(t => idx >= t.startIndex && idx < t.endIndex);
    
        if (isInNonCode(index)) return null;
    
        const isOpening = ['(', '[', '{'].includes(char);
        const partner = bracketPairs[char];
        const direction = isOpening ? 1 : -1;
        let stack = 1;
        let currentIndex = index + direction;
    
        while (currentIndex >= 0 && currentIndex < this.text.length) {
            if (isInNonCode(currentIndex)) {
                currentIndex += direction;
                continue;
            }
    
            const currentChar = this.text[currentIndex];
            if (currentChar === char) {
                stack++;
            } else if (currentChar === partner) {
                stack--;
            }
    
            if (stack === 0) {
                return currentIndex;
            }
            currentIndex += direction;
        }
        return null;
    }

    findEnclosingBrackets(index) {
        const openBrackets = new Set(['(', '[', '{']);
        const bracketPairs = { '(': ')', '[': ']', '{': '}' }; // Opening -> closing
        const nonCodeTokens = this.tokens.filter(t => t.type === 'string' || t.type === 'comment');
        const isInNonCode = (idx) => nonCodeTokens.some(t => idx >= t.startIndex && idx < t.endIndex);

        const stack = [];
        // Scan up to the cursor to find the last unmatched opening bracket
        for (let i = 0; i < index; i++) {
            if (isInNonCode(i)) continue;

            const char = this.text[i];
            if (openBrackets.has(char)) { // It's an opening bracket
                stack.push({ char: char, index: i });
            } else if (Object.values(bracketPairs).includes(char)) { // It's a closing bracket
                if (stack.length > 0 && bracketPairs[stack[stack.length - 1].char] === char) {
                    stack.pop();
                }
            }
        }

        // If the stack is not empty, the top is our candidate
        if (stack.length > 0) {
            const openingBracket = stack[stack.length - 1];
            const matchingBracketIndex = this.findMatchingBracket(openingBracket.index);

            // The match must be valid and must be *after* the cursor
            if (matchingBracketIndex !== null && matchingBracketIndex >= index) {
                return [
                    { startIndex: openingBracket.index, endIndex: openingBracket.index + 1 },
                    { startIndex: matchingBracketIndex, endIndex: matchingBracketIndex + 1 }
                ];
            }
        }

        return null;
    }
    
    getBracketMatchAt(index) {
        let adjacentMatch = null;
        // Check for a bracket immediately to the left of the cursor
        if (index > 0) {
            const checkIndex = index - 1;
            const matchingBracketIndex = this.findMatchingBracket(checkIndex);
            if (matchingBracketIndex !== null) {
                adjacentMatch = [
                    { startIndex: checkIndex, endIndex: checkIndex + 1 },
                    { startIndex: matchingBracketIndex, endIndex: matchingBracketIndex + 1 }
                ];
            }
        }
        
        // If no match on the left, check for a bracket immediately at/to the right of the cursor
        if (!adjacentMatch && index < this.text.length) {
            const checkIndex = index;
            const matchingBracketIndex = this.findMatchingBracket(checkIndex);
            if (matchingBracketIndex !== null) {
                adjacentMatch = [
                    { startIndex: checkIndex, endIndex: checkIndex + 1 },
                    { startIndex: matchingBracketIndex, endIndex: matchingBracketIndex + 1 }
                ];
            }
        }

        if (adjacentMatch) {
            return adjacentMatch;
        }

        // If no adjacent bracket, find the enclosing pair
        return this.findEnclosingBrackets(index);
    }
    
    isAtEnd(offset = this.offset) { return offset >= this.text.length; }
    peek(offset = this.offset) { return this.isAtEnd(offset) ? '\0' : this.text.charAt(offset); }
    peekNext(offset = this.offset) { return this.isAtEnd(offset + 1) ? '\0' : this.text.charAt(offset + 1); }
    advance() { this.offset++; return this.text.charAt(this.offset - 1); }
    isAlpha(char) { return (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z'); }
    isDigit(char) { return char >= '0' && char <= '9'; }
    isAlphaNumeric(char) { return this.isAlpha(char) || this.isDigit(char); }
    skipWhitespace(offset = this.offset) { while (offset < this.text.length && /\s/.test(this.peek(offset))) offset++; return offset; }
    findNextMeaningfulToken(currentIndex) { for (let i = currentIndex + 1; i < this.tokens.length; i++) if (this.tokens[i].type !== 'comment') return this.tokens[i]; return null; }
    findWordAt(index) { const wordRegex = /[\w$]+/g; let match; while ((match = wordRegex.exec(this.text))) { if (index >= match.index && index <= match.index + match[0].length) return { word: match[0], startIndex: match.index, endIndex: match.index + match[0].length }; } return null; }
    isRegexStart() { let prevToken = this.tokens.length > 0 ? this.tokens[this.tokens.length - 1] : null; if (!prevToken) return true; const prevText = this.text.substring(prevToken.startIndex, prevToken.endIndex); return prevToken.type === 'operator' || (prevToken.type === 'punctuation' && !['++', '--', ')', ']'].includes(prevText)); }
    getPrefixAt(index) { let start = index; while (start > 0 && /[\w$]/.test(this.text[start - 1])) start--; if (start === index) return null; return { prefix: this.text.substring(start, index), startIndex: start, endIndex: index }; }
}