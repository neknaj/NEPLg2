"use strict";

// Responsibility budgets measure executable Rust surface, not documentation volume.
// Project policy requires detailed comments, so comment-only and blank lines are ignored.
function implementationLineCount(text) {
    let inBlockComment = false;
    return text.split(/\r?\n/).filter((line) => {
        let hasImplementation = false;
        for (let index = 0; index < line.length;) {
            if (inBlockComment) {
                const end = line.indexOf("*/", index);
                if (end === -1) {
                    return false;
                }
                inBlockComment = false;
                index = end + 2;
                continue;
            }
            if (line.startsWith("/*", index)) {
                inBlockComment = true;
                index += 2;
                continue;
            }
            if (line.startsWith("//", index)) {
                break;
            }
            if (/\s/.test(line[index])) {
                index += 1;
                continue;
            }
            hasImplementation = true;
            break;
        }
        return hasImplementation;
    }).length;
}

module.exports = {
    implementationLineCount,
};
