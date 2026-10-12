import * as assert from 'assert';
import { parseJsonPointer, toJsonPointer } from '../json-pointer';

suite('JSON Pointer Test Suite', () => {
	test('toJsonPointer joins keys', () => {
		assert.strictEqual(toJsonPointer(['myConfig', 'myKey']), '/myConfig/myKey');
		assert.strictEqual(toJsonPointer(['list', 0, 'name']), '/list/0/name');
		assert.strictEqual(toJsonPointer([]), '');
	});

	test('toJsonPointer escapes "~" and "/"', () => {
		assert.strictEqual(toJsonPointer(['a/b', 'c~d']), '/a~1b/c~0d');
	});

	test('parseJsonPointer unescapes keys', () => {
		assert.deepStrictEqual(parseJsonPointer('/a~1b/c~0d'), ['a/b', 'c~d']);
		assert.deepStrictEqual(parseJsonPointer('/myConfig/myKey'), ['myConfig', 'myKey']);
	});

	test('parseJsonPointer round trips keys with "~1" in them', () => {
		const keys = ['~1', '/~'];
		assert.deepStrictEqual(parseJsonPointer(toJsonPointer(keys)), keys);
	});

	test('parseJsonPointer returns no keys for invalid pointers', () => {
		assert.deepStrictEqual(parseJsonPointer(''), []);
		assert.deepStrictEqual(parseJsonPointer('myConfig'), []);
	});
});
