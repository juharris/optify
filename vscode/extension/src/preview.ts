export interface PreviewWhileEditingOptions {
	features?: string[]
	overrides?: string;
}

export class PreviewBuilder {
	getPreviewHtml(
		features: string[],
		config: any): string {
		const configJson = JSON.stringify(config, null, 2);
		const featuresString = JSON.stringify(features);
		return `<!DOCTYPE html>
<html>
<head>
	<title>Configuration Preview</title>
	<style>
		body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 1rem; font-size: 1rem; }
		h2 { border-bottom: 2px solid #007acc; padding-bottom: 10px; }
		pre { padding: 1rem; overflow-x: auto; background-color: #383838; color: #d8d8d8; border-radius: 4px; white-space: pre-wrap; word-wrap: break-word; }
		code { background-color: transparent; font-family: 'Courier New', Courier, monospace; }
	</style>
</head>
<body>
	<h2>Configuration Preview</h2>
	<div>Features:<pre><code>${featuresString}</code></pre></div>
	<h3>Configuration:</h3>
	<pre><code>${configJson}</code></pre>
</body>
</html>`;
	}

	getErrorPreviewHtml(
		features: string[],
		message: string): string {
		const featuresString = JSON.stringify(features);
		return `<!DOCTYPE html>
<html>
<head>
	<title>Error Building Preview</title>
	<style>
		body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 1rem; font-size: 1rem; }
		h2 { color: red; }
		pre { padding: 1rem; overflow-x: auto; background-color: #f8d7da; color: #721c24; border-radius: 4px; white-space: pre-wrap; word-wrap: break-word; }
		code { background-color: transparent; font-family: 'Courier New', Courier, monospace; }
	</style>
</head>
<body>
	<h2>Error Building Preview</h2>
	<div>Features: <pre>${featuresString}</pre></div>
	<pre>${message}</pre>
</body>
</html>`;
	}
}