import { TextDocument } from "vscode";

const LINE_TEXT_CANDIDATES = new Set([
    '{',
    '',
    'metadata:'
]);

const STOP_BEFORE = new Set([
    '}',
    'imports:',
    '  "imports":',
    '    "imports":',
    '\t"imports":',
    'options:',
    '  "options":',
    '    "options":',
    '\t"options":',
]);

export const getDecorationLineNumber = (document: TextDocument): number => {
    let result = 0;
    while (result < document.lineCount) {
        const lineText = document.lineAt(result).text;
        if (LINE_TEXT_CANDIDATES.has(lineText)) {
            return result;
        }
        if (STOP_BEFORE.has(lineText)) {
            return 0;
        }
        ++result;
    }
    return 0;
};
