// md-worker.js

importScripts('../javascript/js-analyzer.js');

class MarkdownAnalyzer {
    constructor(text) {
        this.text = text;
        this.tokens = [];
        this.foldingRanges = [];
    }

    analyze() {
        this.tokenize();
        this.computeFoldingRanges();
    }

    createToken(type, start, end) {
        return { startIndex: start, endIndex: end, type };
    }

    /**
     * テキスト全体をトークン化します。
     * 最初にコードブロックを特定し、その中身を言語に応じて処理します。
     * 残りの部分をMarkdownとして解析します。
     */
    tokenize() {
        this.tokens = [];
        const codeBlockRegex = /^```(\w*)\n([\s\S]*?)\n^```/gm;
        let match;
        let lastIndex = 0;

        while ((match = codeBlockRegex.exec(this.text)) !== null) {
            const lang = match[1].toLowerCase();
            const content = match[2];
            const blockStartIndex = match.index;
            const blockEndIndex = codeBlockRegex.lastIndex;

            // このコードブロックより前のテキストをMarkdownとして解析
            const textBefore = this.text.substring(lastIndex, blockStartIndex);
            this.tokenizeMarkdownFragment(textBefore, lastIndex);

            // コードブロック自体を処理
            const openingFenceEnd = blockStartIndex + match[0].indexOf('\n') + 1;
            this.tokens.push(this.createToken('code-block', blockStartIndex, openingFenceEnd - 1));
            
            // 閉じフェンスの開始位置を見つける
            let closingFenceStart = match[0].lastIndexOf('\n```') + blockStartIndex;
            if (closingFenceStart < openingFenceEnd) closingFenceStart = blockEndIndex - 3;
            
            this.tokens.push(this.createToken('code-block', closingFenceStart, blockEndIndex));

            // JavaScriptコードブロックなら、JavaScriptとしてハイライト
            if (lang === 'javascript' || lang === 'js') {
                const jsAnalyzer = new JavaScriptAnalyzer(content);
                jsAnalyzer.tokenize();
                for (const token of jsAnalyzer.tokens) {
                    this.tokens.push(this.createToken(
                        token.type,
                        openingFenceEnd + token.startIndex,
                        openingFenceEnd + token.endIndex
                    ));
                }
            } else {
                // 他の言語は一括で文字列としてハイライト
                this.tokens.push(this.createToken('string', openingFenceEnd, closingFenceStart));
            }

            lastIndex = blockEndIndex;
        }

        // 最後のコードブロック以降の残りのテキストを解析
        const remainingText = this.text.substring(lastIndex);
        this.tokenizeMarkdownFragment(remainingText, lastIndex);
    }
    
    /**
     * テキストの断片をMarkdownとして解析し、トークンを生成します。
     * @param {string} fragment - 解析対象のテキスト断片
     * @param {number} offset - 元のテキストにおける断片の開始インデックス
     */
    tokenizeMarkdownFragment(fragment, offset) {
        const lines = fragment.split('\n');
        let currentOffset = offset;

        for (const line of lines) {
            const lineLength = line.length;
            
            // 見出し
            let match = line.match(/^(#+) /);
            if (match) {
                this.tokens.push(this.createToken('heading', currentOffset, currentOffset + lineLength));
            }
             // リスト
            match = line.match(/^(\s*)([-*+] |[0-9]+\.) /);
            if (match) {
                this.tokens.push(this.createToken('list', currentOffset + match.length, currentOffset + match.length - 1));
            }
            
            // インライン要素
            const inlinePatterns = [
                { type: 'inline-code', regex: /`([^`]+?)`/g },
                { type: 'bold', regex: /(\*\*|__)(.+?)\1/g },
                { type: 'italic', regex: /(\*|_)(.+?)\1/g },
                { type: 'link', regex: /\[(.+?)\]\((.+?)\)/g },
            ];

            for (const pattern of inlinePatterns) {
                let inlineMatch;
                while((inlineMatch = pattern.regex.exec(line))) {
                    const start = currentOffset + inlineMatch.index;
                    const end = start + inlineMatch[0].length;
                    const isOverlapping = this.tokens.some(t => t.startIndex < end && t.endIndex > start);
                    if (!isOverlapping) {
                       this.tokens.push(this.createToken(pattern.type, start, end));
                    }
                }
            }
            currentOffset += lineLength + 1; // +1 for the newline
        }
    }

    computeFoldingRanges() {
        this.foldingRanges = [];
        const lines = this.text.split('\n');
        const headingStack = [];

        for(let i = 0; i < lines.length; i++) {
            const match = lines[i].match(/^(#+) /);
            if (match) {
                const level = match[1].length;
                while(headingStack.length > 0 && headingStack[headingStack.length - 1].level >= level) {
                    const lastHeading = headingStack.pop();
                    if (i - 1 > lastHeading.line) {
                       this.foldingRanges.push({ startLine: lastHeading.line, endLine: i - 1, placeholder: '...' });
                    }
                }
                headingStack.push({ line: i, level: level });
            }
        }
        while(headingStack.length > 0) {
            const lastHeading = headingStack.pop();
            if (lines.length - 1 > lastHeading.line) {
                this.foldingRanges.push({ startLine: lastHeading.line, endLine: lines.length - 1, placeholder: '...' });
            }
        }
        this.foldingRanges.sort((a,b) => a.startLine - b.startLine);
    }
}

let analyzer;

self.onmessage = (event) => {
    const { type, payload, requestId } = event.data;
    switch (type) {
        case 'updateText':
            analyzer = new MarkdownAnalyzer(payload);
            analyzer.analyze();
            self.postMessage({
                type: 'update',
                payload: {
                    tokens: analyzer.tokens,
                    diagnostics: [],
                    foldingRanges: analyzer.foldingRanges,
                    config: { highlightWhitespace: false, highlightIndent: false }
                }
            });
            break;
        case 'getHoverInfo': 
        case 'getDefinitionLocation':
        case 'getBracketMatch':
            if (analyzer) self.postMessage({ type, payload: null, requestId });
            break;
        case 'getOccurrences':
        case 'getCompletions':
            if (analyzer) self.postMessage({ type, payload: [], requestId });
            break;
        case 'getIndentation':
        case 'toggleComment':
        case 'adjustIndentation':
        case 'getNextWordBoundary':
             if (analyzer) self.postMessage({ type, payload: null, requestId });
             break;
    }
};