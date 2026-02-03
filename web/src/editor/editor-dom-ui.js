/**
 * Canvas外のDOM要素（ポップアップ、補完リスト、問題パネル）の管理を担当します。
 * エディタの状態に基づいてUIを更新し、UIからのインタラクションをエディタ本体に伝達します。
 */
class EditorDOMUI {
    /**
     * @param {CanvasEditor} editor - 親となるCanvasEditorのインスタンス
     * @param {object} elements - { popup, problemsPanel, completionList } を含むDOM要素のオブジェクト
     */
    constructor(editor, elements) {
        this.editor = editor;
        this.popup = elements.popup;
        this.problemsPanel = elements.problemsPanel;
        this.completionList = elements.completionList;

        this.isCompletionVisible = false;
        this.completionSuggestions = [];
        this.selectedSuggestionIndex = 0;
    }

    showPopup(content, x, y) {
        this.popup.style.display = 'block';
        this.popup.textContent = content;
        this.popup.style.left = `${x + 10}px`;
        this.popup.style.top = `${y + 10}px`;
    }

    hidePopup() {
        this.popup.style.display = 'none';
    }

    updateProblemsPanel() {
        if (!this.problemsPanel) return;
        this.problemsPanel.innerHTML = '';
        this.editor.diagnostics.forEach(diag => {
            const { row, col } = this.editor.utils.getPosFromIndex(diag.startIndex, this.editor.lines);
            const li = document.createElement('li');
            li.className = `severity-${diag.severity}`;
            li.textContent = diag.message;
            const locationSpan = document.createElement('span');
            locationSpan.className = 'problem-location';
            locationSpan.textContent = `[${row + 1}, ${col + 1}]`;
            li.appendChild(locationSpan);
            li.addEventListener('click', () => {
                this.editor.setCursor(diag.startIndex);
                this.editor.focus();
            });
            this.problemsPanel.appendChild(li);
        });
    }

    getCompletionTypeAbbreviation(type) {
        switch (type) {
            case 'keyword': return 'K';
            case 'snippet': return 'S';
            case 'function': return 'f';
            case 'class': return 'C';
            case 'variable': case 'const': case 'let': case 'var': return 'v';
            default: return '·';
        }
    }

    showCompletion(suggestions) {
        this.completionSuggestions = suggestions;
        this.selectedSuggestionIndex = 0;
        this.isCompletionVisible = true;
        this.completionList.innerHTML = '';
        suggestions.forEach((item, index) => {
            const li = document.createElement('li');
            li.dataset.index = String(index);
            const leftDiv = document.createElement('div');
            leftDiv.className = 'completion-item-left';
            const typeSpan = document.createElement('span');
            typeSpan.className = 'completion-type';
            typeSpan.textContent = this.getCompletionTypeAbbreviation(item.type);
            leftDiv.appendChild(typeSpan);
            const labelSpan = document.createElement('span');
            labelSpan.textContent = item.label;
            leftDiv.appendChild(labelSpan);
            li.appendChild(leftDiv);
            if (item.detail) {
                const detailSpan = document.createElement('span');
                detailSpan.className = 'completion-detail';
                detailSpan.textContent = item.detail;
                li.appendChild(detailSpan);
            }
            if (index === this.selectedSuggestionIndex) {
                li.classList.add('selected');
            }
            li.addEventListener('mousedown', (e) => {
                e.preventDefault();
                this.selectedSuggestionIndex = index;
                this.editor.acceptCompletion();
            });
            this.completionList.appendChild(li);
        });
        this.completionList.style.display = 'block';
        this.editor.updateTextareaPosition();
    }

    hideCompletion() {
        if (!this.isCompletionVisible) return;
        this.isCompletionVisible = false;
        this.completionSuggestions = [];
        this.completionList.style.display = 'none';
    }

    updateCompletionSelection(direction) {
        if (!this.isCompletionVisible) return;
        const newIndex = this.selectedSuggestionIndex + direction;
        const total = this.completionSuggestions.length;
        if (newIndex < 0 || newIndex >= total) return;

        const items = this.completionList.children;
        items[this.selectedSuggestionIndex].classList.remove('selected');
        this.selectedSuggestionIndex = newIndex;
        items[this.selectedSuggestionIndex].classList.add('selected');
        items[this.selectedSuggestionIndex].scrollIntoView({ block: 'nearest' });
    }
}