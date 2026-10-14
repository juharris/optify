import * as assert from 'assert';
import { findOptionKeySpan } from '../option-location';

/** Gets the text for the span of the key for the option. */
function findKey(text: string, keyPath: string): string | undefined {
	const span = findOptionKeySpan(text, keyPath);
	return span && text.substring(span.start, span.end);
}

/** Gets the line number of the key for the option. */
function findLine(text: string, keyPath: string): number | undefined {
	const span = findOptionKeySpan(text, keyPath);
	return span && text.substring(0, span.start).split('\n').length - 1;
}

suite('Option Location Test Suite', () => {
	const json = `{
	"imports": ["base"],
	"options": {
		"myConfig": {
			"myKey": 1,
			"key": {
				"a/b": true
			}
		}
	}
}`;

	const yaml = `imports:
  - base
options:
  myConfig:
    myKey: 1
    key:
      'a/b': true
`;

	test('finds keys in JSON', () => {
		assert.strictEqual(findKey(json, '/myConfig/myKey'), 'myKey');
		assert.strictEqual(findLine(json, '/myConfig/myKey'), 4);
	});

	test('finds keys in YAML', () => {
		assert.strictEqual(findKey(yaml, '/myConfig/myKey'), 'myKey');
		assert.strictEqual(findLine(yaml, '/myConfig/myKey'), 4);
	});

	test('finds escaped keys with special characters', () => {
		assert.strictEqual(findLine(json, '/myConfig/key/a~1b'), 6);
		assert.strictEqual(findLine(yaml, '/myConfig/key/a~1b'), 6);
	});

	test('does not match the end of a longer key', () => {
		assert.strictEqual(findLine(json, '/myConfig/key'), 5);
		assert.strictEqual(findLine(yaml, '/myConfig/key'), 5);
	});

	test('finds unquoted keys in JSON5', () => {
		const json5 = `{
	options: {
		myConfig: {
			myKey: 1,
		},
	},
}`;
		assert.strictEqual(findLine(json5, '/myConfig/myKey'), 3);
	});

	test('falls back to the deepest key found', () => {
		assert.strictEqual(findLine(yaml, '/myConfig/missing'), 3);
		assert.strictEqual(findKey(yaml, '/missing'), 'options');
	});

	test('returns undefined without options', () => {
		assert.strictEqual(findOptionKeySpan('imports:\n  - base\n', '/myConfig'), undefined);
	});
});
