import * as assert from 'assert';
import { ConfigParser } from '../config-parser';

suite('ConfigParser File References Test Suite', () => {
	suite('JSON file references', () => {
		test('should find file reference in simple JSON', () => {
			const text = `{
	"options": {
		"template": {
			"file": "templates/header.liquid"
		}
	}
}`;
			const refs = ConfigParser.findFileReferences(text, 'json');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should handle file references with various spacing in JSON', () => {
			const text = `{
	"options": {
		"template": {
			"file":"templates/no-space.liquid"
		},
		"another": {
			"file" : "templates/space-before.liquid"
		}
	}
}`;
			const refs = ConfigParser.findFileReferences(text, 'json');
			assert.strictEqual(refs.length, 2);
			assert.strictEqual(refs[0].filePath, 'templates/no-space.liquid');
			assert.strictEqual(refs[1].filePath, 'templates/space-before.liquid');
		});

		test('should return empty array when no options in JSON', () => {
			const text = `{
	"imports": ["feature1"]
}`;
			const refs = ConfigParser.findFileReferences(text, 'json');
			assert.strictEqual(refs.length, 0);
		});
	});

	suite('YAML file references', () => {
		test('should find file reference in simple YAML', () => {
			const text = `options:
  template:
    file: templates/header.liquid`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should find file reference with double quotes in YAML', () => {
			const text = `options:
  template:
    file: "templates/header.liquid"`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should find file reference with single quotes in YAML', () => {
			const text = `options:
  template:
    file: 'templates/header.liquid'`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should find file reference in inline object form in YAML', () => {
			const text = `options:
  template: { file: "templates/header.liquid" }`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should find file reference in inline object without quotes in YAML', () => {
			const text = `options:
  template: { file: templates/header.liquid }`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should find multiple file references in YAML', () => {
			const text = `options:
  header:
    file: templates/header.liquid
  footer:
    file: "templates/footer.liquid"
  sidebar: { file: 'templates/sidebar.liquid' }`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 3);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
			assert.strictEqual(refs[1].filePath, 'templates/footer.liquid');
			assert.strictEqual(refs[2].filePath, 'templates/sidebar.liquid');
		});

		test('should stop at next top-level key in YAML', () => {
			const text = `options:
  template:
    file: templates/header.liquid
conditions:
  jsonPointer: /test`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should ignore comments in YAML', () => {
			const text = `options:
  # This is a comment
  template:
    file: templates/header.liquid
  # file: templates/commented.liquid`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);
			assert.strictEqual(refs[0].filePath, 'templates/header.liquid');
		});

		test('should return empty array when no options in YAML', () => {
			const text = `imports:
  - feature1`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 0);
		});
	});

	suite('File reference ranges', () => {
		test('should return correct range for file path in JSON', () => {
			const text = `{
	"options": {
		"template": {
			"file": "templates/header.liquid"
		}
	}
}`;
			const refs = ConfigParser.findFileReferences(text, 'json');
			assert.strictEqual(refs.length, 1);

			// The file path "templates/header.liquid" should be on line 3
			// Starting after "file": "
			const expectedLine = 3;
			assert.strictEqual(refs[0].range.start.line, expectedLine);
			assert.strictEqual(refs[0].range.end.line, expectedLine);
		});

		test('should return correct range for file path in YAML', () => {
			const text = `options:
  template:
    file: templates/header.liquid`;
			const refs = ConfigParser.findFileReferences(text, 'yaml');
			assert.strictEqual(refs.length, 1);

			// The file path should be on line 2
			const expectedLine = 2;
			assert.strictEqual(refs[0].range.start.line, expectedLine);
			assert.strictEqual(refs[0].range.end.line, expectedLine);

			// Verify the range extracts the correct text
			const lines = text.split('\n');
			const line = lines[expectedLine];
			const extractedPath = line.substring(refs[0].range.start.character, refs[0].range.end.character);
			assert.strictEqual(extractedPath, 'templates/header.liquid');
		});
	});
});
