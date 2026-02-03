export class NeplLanguageProvider {
    constructor() {
        this.listeners = [];
        this.keywords = new Set([
            'fn', 'let', 'struct', 'enum', 'impl', 'mod', 'use', 'pub', 'mut',
            'if', 'else', 'while', 'for', 'in', 'return', 'break', 'continue',
            'match', 'type', 'const', 'static', 'trait', 'where', 'as', 'super', 'self', 'Self',
            'true', 'false', 'null'
        ]);
        this.types = new Set([
            'i32', 'i64', 'f32', 'f64', 'bool', 'char', 'str', 'String', 'Vec', 'Option', 'Result'
        ]);
    }

    onUpdate(callback) {
        this.listeners.push(callback);
    }

    updateText(text) {
        const tokens = this.tokenize(text);
        const diagnostics = []; // TODO: Integrate compiler diagnostics later

        for (const listener of this.listeners) {
            listener({ tokens, diagnostics });
        }
    }

    tokenize(text) {
        const tokens = [];
        let current = 0;

        // Simple regex-based tokenizer (not perfect, but fast for playground)
        // Matches: Comments, Strings, Numbers, Keywords/Identifiers, Operators, Whitespace
        const regex = /(\/\/.*)|(\/\*[\s\S]*?\*\/)|("(\\.|[^"\\])*")|(-?\d+(\.\d+)?)|([a-zA-Z_]\w*)|([^\s\w]+)|(\s+)/g;

        let match;
        while ((match = regex.exec(text)) !== null) {
            const val = match[0];
            const start = match.index;
            const end = start + val.length;

            let type = 'default';

            if (match[1] || match[2]) { // Comment
                type = 'comment';
            } else if (match[3]) { // String
                type = 'string';
            } else if (match[5]) { // Number
                type = 'number';
            } else if (match[7]) { // Identifier
                if (this.keywords.has(val)) type = 'keyword';
                else if (this.types.has(val)) type = 'type'; // Use type color if available, or fall back
                else type = 'variable';
            } else if (match[8]) { // Operator/Punctuation
                type = 'operator';
            }

            if (type !== 'default' && !match[9]) { // Skip whitespace
                tokens.push({ startIndex: start, endIndex: end, type });
            }
        }

        return tokens;
    }

    // Stub methods for other provider features
    async getOccurrences(index) { return []; }
    async getBracketMatch(index) { return []; }
    async getCompletions(index) { return []; }
    async getHoverInfo(index) { return null; }
}
