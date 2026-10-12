/**
 * Builds a JSON pointer (RFC 6901), such as "/myConfig/myKey", from the keys to a value.
 */
export function toJsonPointer(keys: readonly (string | number)[]): string {
	return keys
		.map(key => '/' + String(key).replace(/~/g, '~0').replace(/\//g, '~1'))
		.join('');
}

/**
 * Parses a JSON pointer (RFC 6901), such as "/myConfig/myKey", into the unescaped keys to a value.
 */
export function parseJsonPointer(pointer: string): string[] {
	if (!pointer.startsWith('/')) {
		return [];
	}

	return pointer
		.substring(1)
		.split('/')
		.map(key => key.replace(/~1/g, '/').replace(/~0/g, '~'));
}
