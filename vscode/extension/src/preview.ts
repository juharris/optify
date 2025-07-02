export class PreviewBuilder {
	getPreviewHtml(features: string[], config: any): string {
		const configJson = JSON.stringify(config, null, 2);
		const featuresString = JSON.stringify(features);
		return `<!DOCTYPE html>
<html>
<head>
	<title>Configuration Preview</title>
	<style>
		body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; }
		h2 { border-bottom: 2px solid #007acc; padding-bottom: 10px; }
		pre { padding: 1rem; overflow-x: auto; background-color: #383838; color: #d8d8d8; border-radius: 4px; }
		code { background-color: transparent; font-family: 'Courier New', Courier, monospace; }
	</style>
</head>
<body>
	<h2>Configuration Preview</h2>
	<div>Features: <code>${featuresString}</code></div>
	<pre><code>${configJson}</code></pre>
</body>
</html>`;
	}

	getErrorPreviewHtml(features: string[], message: string): string {
		const featuresString = JSON.stringify(features);
		return `<!DOCTYPE html>
<html>
<head>
	<title>Error Building Preview</title>
	<style>
		body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; }
		h2 { color: red; }
		code { background-color: transparent; font-family: 'Courier New', Courier, monospace; }
		pre { padding: 1rem; overflow-x: auto; background-color: #f8d7da; color: #721c24; border-radius: 4px; white-space: pre-wrap; word-wrap: break-word; }
	</style>
</head>
<body>
	<h2>Error Building Preview</h2>
	<div>Features: <code>${featuresString}</code></div>
	<pre>${message}</pre>
</body>
</html>`;
	}
}