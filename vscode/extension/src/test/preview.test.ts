import * as assert from 'assert';

import { PreviewBuilder } from '../preview';

suite('Preview Builder Test Suite', () => {
	test('preview error', () => {
		const previewBuilder = new PreviewBuilder();
		const features = ['feature1', 'feature2'];
		const message = 'This is an error message';
		const html = previewBuilder.getErrorPreviewHtml(features, message);
		assert.ok(html.startsWith('<!DOCTYPE html>'), html);
		assert.ok(html.includes('<h2>Error Building Preview</h2>'), html);
	});
});
