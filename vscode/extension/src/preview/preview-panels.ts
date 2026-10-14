import * as vscode from 'vscode';
import { ConfigParser } from '../config-parser';
import type { MessageToExtension } from '../webview/types';
import { readConfigurableValuesDefault } from './configurable-values-default';
import { buildOptifyGraphData } from './graph-data';
import { getPreviewHtmlShell } from './html';
import { openFile } from './open-file';
import { buildOptifyOptionAnalysis } from './option-analysis';
import { buildOptifyPreviewData } from './preview-data';
import { PreviewWhileEditingOptions } from './types';
import { sendGraphUpdate, sendPreviewUpdate } from './webview-messages';

const EDIT_DEBOUNCE_MILLISECONDS = 250;

interface ActivePreview {
	panel: vscode.WebviewPanel
	documentChangeListener: vscode.Disposable
	documentSaveListener: vscode.Disposable
	debounceTimer?: NodeJS.Timeout
	updatePreview: () => void
	areConfigurableValuesEnabled: boolean
	customFeatures?: string[]
	/** The unsaved changes that the preview shows, if any. */
	editingOptions?: PreviewWhileEditingOptions
}

/**
 * Manages the preview panels for feature files.
 */
export class PreviewPanels implements vscode.Disposable {
	/** The open previews by the path of the previewed file. */
	private readonly activePreviews = new Map<string, ActivePreview>();

	constructor(private readonly context: vscode.ExtensionContext) { }

	/**
	 * Closes all previews.
	 */
	dispose(): void {
		for (const filePath of this.activePreviews.keys()) {
			this.close(filePath);
		}
		// It should already be empty, but we'll clear it just in case.
		this.activePreviews.clear();
	}

	/**
	 * Opens a preview for a feature file or reveals the existing preview for it.
	 */
	async openOrReveal(filePath: string, optifyRoot: string, canonicalName: string): Promise<void> {
		const existingPreview = this.activePreviews.get(filePath);
		if (existingPreview) {
			existingPreview.panel.reveal();
			return;
		}

		const configurableValuesDefault = readConfigurableValuesDefault(optifyRoot);

		const panel = vscode.window.createWebviewPanel(
			'optifyPreview',
			`Optify Preview: ${canonicalName}`,
			vscode.ViewColumn.Beside,
			{
				enableScripts: true,
				enableFindWidget: true,
				retainContextWhenHidden: true,
				localResourceRoots: [vscode.Uri.joinPath(this.context.extensionUri, 'out')]
			}
		);

		panel.webview.html = getPreviewHtmlShell(panel.webview, this.context.extensionUri);

		/**
		 * Builds and sends the preview.
		 * Remembers the unsaved changes that the preview shows so that option analysis matches the preview.
		 */
		const sendPreview = (editingOptions?: PreviewWhileEditingOptions) => {
			const preview = this.activePreviews.get(filePath);
			if (preview) {
				preview.editingOptions = editingOptions;
			}
			const data = buildOptifyPreviewData(
				preview?.customFeatures ?? [canonicalName],
				optifyRoot,
				editingOptions,
				preview?.areConfigurableValuesEnabled ?? configurableValuesDefault,
				configurableValuesDefault,
			);
			sendPreviewUpdate(panel, data);
		};

		const sendGraph = () => {
			const preview = this.activePreviews.get(filePath);
			const graphData = buildOptifyGraphData(preview?.customFeatures ?? [canonicalName], optifyRoot);
			sendGraphUpdate(panel, graphData);
		};

		const updatePreview = () => {
			sendPreview();
			sendGraph();
		};

		// Handle messages from the webview
		panel.webview.onDidReceiveMessage(
			async (message: MessageToExtension) => {
				if (message.command === 'ready') {
					updatePreview();
				} else if (message.command === 'analyzeOption') {
					const preview = this.activePreviews.get(filePath);
					const data = buildOptifyOptionAnalysis(
						preview?.customFeatures ?? [canonicalName],
						optifyRoot,
						message.keyPath,
						preview?.editingOptions,
					);
					panel.webview.postMessage({ type: 'optionAnalysis', data });
				} else if (message.command === 'openFile') {
					if (message.path) {
						await openFile(message.path, message.keyPath);
					}
				} else if (message.command === 'setConfigurableValues') {
					const preview = this.activePreviews.get(filePath);
					if (preview) {
						preview.areConfigurableValuesEnabled = message.enabled === true;
						sendPreview();
					}
				} else if (message.command === 'setFeatures') {
					const preview = this.activePreviews.get(filePath);
					if (preview) {
						const features: string[] = message.features ?? [];
						preview.customFeatures = features.length > 0 ? features : undefined;
						updatePreview();
					}
				}
			},
			undefined,
			this.context.subscriptions
		);

		const documentChangeListener = vscode.workspace.onDidChangeTextDocument((event) => {
			if (event.document.uri.fsPath === filePath) {
				const preview = this.activePreviews.get(filePath);
				if (!preview) {
					return;
				}

				if (preview.debounceTimer) {
					clearTimeout(preview.debounceTimer);
				}

				preview.debounceTimer = setTimeout(() => {
					const documentText = event.document.getText();
					const config = ConfigParser.parse(documentText, event.document.languageId);
					const editingFeatures = config.imports ?? [];
					sendPreview({
						features: editingFeatures,
						overrides: config.options ? JSON.stringify(config.options) : undefined
					});
				}, EDIT_DEBOUNCE_MILLISECONDS);
			}
		});

		const documentSaveListener = vscode.workspace.onDidSaveTextDocument((document) => {
			if (document.uri.fsPath !== filePath) {
				return;
			}

			const preview = this.activePreviews.get(filePath);
			if (!preview) {
				return;
			}

			if (preview.debounceTimer) {
				clearTimeout(preview.debounceTimer);
				preview.debounceTimer = undefined;
			}

			preview.updatePreview();
		});

		this.context.subscriptions.push(documentChangeListener);
		this.context.subscriptions.push(documentSaveListener);

		this.activePreviews.set(filePath, {
			panel,
			documentChangeListener,
			documentSaveListener,
			updatePreview,
			areConfigurableValuesEnabled: configurableValuesDefault,
		});

		// Clean up when panel is closed
		panel.onDidDispose(() => {
			this.close(filePath);
		});
	}

	/**
	 * Updates all open previews, such as when feature files change.
	 */
	updateAll(): void {
		for (const preview of this.activePreviews.values()) {
			preview.updatePreview();
		}
	}

	private close(filePath: string): void {
		const preview = this.activePreviews.get(filePath);
		if (preview) {
			preview.documentChangeListener.dispose();
			preview.documentSaveListener.dispose();
			// Clear any pending debounce timer
			if (preview.debounceTimer) {
				clearTimeout(preview.debounceTimer);
			}
			this.activePreviews.delete(filePath);
		}
	}
}
