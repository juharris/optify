import { parseJsonPointer } from './json-pointer';

export interface TextSpan {
	start: number;
	end: number;
}

/**
 * Finds the key for an option in the text of a feature file.
 *
 * This searches for each key in the JSON pointer in order after the `options` key instead of fully parsing the file
 * so that it works for JSON, JSON5, and YAML files and still works while a file has syntax errors.
 * It may find the wrong occurrence when an earlier sibling has a nested key with the same name.
 *
 * @param text The text of a feature file.
 * @param keyPath A JSON pointer to the option within `options`, such as "/myConfig/myKey".
 * @returns The span of the deepest key in `keyPath` that was found, without quotes, or `undefined` if there is no `options` key.
 * A file may set an ancestor of `keyPath` instead of `keyPath` itself, so the deepest key found is the most relevant location.
 */
export function findOptionKeySpan(text: string, keyPath: string): TextSpan | undefined {
	let span = findKeySpan(text, 'options', 0);
	if (!span) {
		return undefined;
	}

	for (const key of parseJsonPointer(keyPath)) {
		const keySpan = findKeySpan(text, key, span.end);
		if (!keySpan) {
			break;
		}
		span = keySpan;
	}
	return span;
}

function escapeRegExp(text: string): string {
	return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function findKeySpan(text: string, key: string, fromIndex: number): TextSpan | undefined {
	// Match a key with or without quotes followed by a colon.
	// The lookbehind avoids matching the end of a longer key, such as "myKey" when looking for "Key".
	const keyPattern = new RegExp(`(?<![\\w$"'-])(["']?)${escapeRegExp(key)}\\1\\s*:`, 'g');
	keyPattern.lastIndex = fromIndex;
	const match = keyPattern.exec(text);
	if (!match) {
		return undefined;
	}

	const start = match.index + match[1].length;
	return { start, end: start + key.length };
}
