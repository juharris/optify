import * as assert from 'assert';
import path from 'path';
import * as vscode from 'vscode';
import { PreviewPanels } from '../preview/preview-panels';

const configsRoot = path.join(__dirname, '../../src/test/configs');

function getPreviewTabs(): vscode.Tab[] {
	return vscode.window.tabGroups.all
		.flatMap(group => group.tabs)
		.filter(tab => tab.label === 'Optify Preview: feature');
}

/**
 * Waits for the tabs to update because VS Code updates them asynchronously.
 */
async function waitForPreviewTabCount(expectedCount: number): Promise<void> {
	const timeoutMs = 5000;
	const start = Date.now();
	while (getPreviewTabs().length !== expectedCount && Date.now() - start < timeoutMs) {
		await new Promise(resolve => setTimeout(resolve, 20));
	}
	assert.equal(getPreviewTabs().length, expectedCount);
}

suite('Preview Panels Test Suite', () => {
	test('opens one preview per file and closes it', async () => {
		const extension = vscode.extensions.getExtension('optify-config.optify');
		assert.ok(extension);
		const subscriptions: vscode.Disposable[] = [];
		const context = { extensionUri: extension.extensionUri, subscriptions } as unknown as vscode.ExtensionContext;
		const previewPanels = new PreviewPanels(context);
		const filePath = path.join(configsRoot, 'feature.yaml');

		try {
			await previewPanels.openOrReveal(filePath, configsRoot, 'feature');
			await waitForPreviewTabCount(1);

			// Opening the same file again reveals the existing preview.
			await previewPanels.openOrReveal(filePath, configsRoot, 'feature');
			await waitForPreviewTabCount(1);

			previewPanels.updateAll();
		} finally {
			previewPanels.dispose();
			await vscode.window.tabGroups.close(getPreviewTabs());
			subscriptions.forEach(subscription => subscription.dispose());
		}
		await waitForPreviewTabCount(0);
	});
});
