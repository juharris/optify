import * as vscode from 'vscode';

/**
 * Gets the HTML for the preview webview, which loads the React app.
 */
export function getPreviewHtmlShell(webview: vscode.Webview, extensionUri: vscode.Uri): string {
	const scriptUri = webview.asWebviewUri(
		vscode.Uri.joinPath(extensionUri, 'out', 'webview.js')
	);

	return `<!DOCTYPE html>
<html>
<head>
	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src ${webview.cspSource};">
	<title>Configuration Preview</title>
	<style>
		body {
			margin: 0;
			padding: 0;
			background-color: var(--vscode-editor-background);
			color: var(--vscode-editor-foreground);
		}
		@keyframes optify-spin {
			to { transform: rotate(360deg); }
		}
		.optify-origin-button {
			margin-right: 0.25em;
			padding: 0 0.15em;
			background: none;
			border: none;
			color: var(--vscode-textLink-foreground);
			cursor: pointer;
			font-size: 0.9em;
			/* Dim the button until hovering over the key, but keep enough contrast to see it in dark themes. */
			opacity: 0.8;
		}
		.data-key:hover > .data-key-key > .optify-origin-button,
		.optify-origin-button:focus-visible {
			opacity: 1;
		}
		.optify-spinner {
			display: inline-block;
			width: 16px;
			height: 16px;
			border: 2px solid var(--vscode-progressBar-background, #0078d4);
			border-top-color: transparent;
			border-radius: 50%;
			animation: optify-spin 1s linear infinite;
		}
	</style>
</head>
<body>
	<div id="root"></div>
	<script src="${scriptUri}"></script>
</body>
</html>`;
}
