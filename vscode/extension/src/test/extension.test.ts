import * as assert from 'assert';

import * as vscode from 'vscode';

suite('Extension Test Suite', () => {
	vscode.window.showInformationMessage('Start all tests.');

	test('extension activates', () => {
		const ext = vscode.extensions.getExtension('optify-config.optify');
		assert.ok(ext, 'Extension should be present');
	});
});
