import { OptionsWatcher } from "@optify/config";

export interface PreviewWhileEditingOptions {
	features?: string[]
	overrides?: string;
}

export class PreviewBuilder {
	private highlightJson(json: string): string {
		return json
			.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, (match) => {
				let cls = 'number';
				if (match.startsWith('"')) {
					if (match.endsWith(':')) {
						cls = 'key';
					} else {
						cls = 'string';
					}
				} else if (/true|false/.test(match)) {
					cls = 'boolean';
				} else if (/null/.test(match)) {
					cls = 'null';
				}
				return `<span class="${cls}">${match}</span>`;
			});
	}

	getPreviewHtml(
		features: string[],
		config: any,
		dependents: string[] | null,
		provider: OptionsWatcher
	): string {
		const configJson = JSON.stringify(config, null, 2);
		const highlightedConfig = this.highlightJson(configJson);
		const featuresString = JSON.stringify(features);
		const highlightedFeatures = this.highlightJson(featuresString);

		const renderDependents = () => {
			if (!dependents || dependents.length === 0) {
				return '';
			}
			const featuresWithMetadata = provider.featuresWithMetadata();
			const dependentLinks = dependents.map(dep => {
				const path = featuresWithMetadata[dep]?.path();
				return `<li><a href="#" class="dependent-link" data-path="${path}">${dep}</a></li>`;
			});

			return `
			<div><h3>Dependents</h3>Features that depend on this one:\n<ul>${dependentLinks.join('\n')}</ul></div>`;
		};

		return `<!DOCTYPE html>
<html>
<head>
	<title>Configuration Preview</title>
	<style>
		body { 
			font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
			padding: 1rem; 
			background-color: var(--vscode-editor-background);
			color: var(--vscode-editor-foreground);
		}
		h2 { 
			border-bottom: 2px solid var(--vscode-focusBorder); 
			padding-bottom: 10px;
			color: var(--vscode-foreground);
		}
		h3 {
			color: var(--vscode-foreground);
		}
		pre { 
			padding: 1rem; 
			overflow-x: auto; 
			background-color: var(--vscode-textCodeBlock-background); 
			border: 1px solid var(--vscode-widget-border);
			border-radius: 4px; 
			white-space: pre-wrap; 
			word-wrap: break-word; 
		}
		code { 
			background-color: transparent; 
			font-family: var(--vscode-editor-font-family), 'Courier New', Courier, monospace;
			font-size: var(--vscode-editor-font-size);
			line-height: 1.5;
		}
		
		/* JSON Syntax Highlighting */
		.string { color: var(--vscode-debugTokenExpression-string); }
		.number { color: var(--vscode-debugTokenExpression-number); }
		.boolean { color: var(--vscode-debugTokenExpression-boolean); }
		.null { color: var(--vscode-debugIcon-breakpointDisabledForeground); }
		.key { color: var(--vscode-debugTokenExpression-name); }
		
		/* Fallback colors for when VS Code variables are not available */
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .string { color: #ce9178; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .number { color: #b5cea8; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .boolean { color: #569cd6; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .null { color: #569cd6; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .key { color: #9cdcfe; }
		
		/* Light theme overrides */
		body.vscode-light .string { color: #a31515; }
		body.vscode-light .number { color: #098658; }
		body.vscode-light .boolean { color: #0000ff; }
		body.vscode-light .null { color: #0000ff; }
		body.vscode-light .key { color: #001080; }
		
		/* Dependent links */
		.dependent-link {
			color: var(--vscode-textLink-foreground);
			text-decoration: none;
			cursor: pointer;
		}
		.dependent-link:hover {
			text-decoration: underline;
			color: var(--vscode-textLink-activeForeground);
		}
	</style>
	<script>
		const vscode = acquireVsCodeApi();
		
		document.addEventListener('DOMContentLoaded', () => {
			document.querySelectorAll('.dependent-link').forEach(link => {
				link.addEventListener('click', (e) => {
					e.preventDefault();
					vscode.postMessage({
						command: 'openFile',
						path: e.target.dataset.path,
					});
				});
			});
		});
	</script>
</head>
<body>
	<h2>Configuration Preview</h2>
	<div>Features:<pre><code>${highlightedFeatures}</code></pre></div>
	${renderDependents()}
	<h3>Configuration:</h3>
	<pre><code>${highlightedConfig}</code></pre>
</body>
</html>`;
	}

	getErrorPreviewHtml(
		features: string[],
		message: string): string {
		const featuresString = JSON.stringify(features);
		const highlightedFeatures = this.highlightJson(featuresString);
		return `<!DOCTYPE html>
<html>
<head>
	<title>Error Building Preview</title>
	<style>
		body { 
			font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
			padding: 1rem;
			background-color: var(--vscode-editor-background);
			color: var(--vscode-editor-foreground);
		}
		h2 { 
			color: var(--vscode-errorForeground);
		}
		pre { 
			padding: 1rem; 
			overflow-x: auto; 
			background-color: var(--vscode-inputValidation-errorBackground); 
			border: 1px solid var(--vscode-inputValidation-errorBorder);
			border-radius: 4px; 
			white-space: pre-wrap; 
			word-wrap: break-word; 
		}
		code { 
			background-color: transparent; 
			font-family: var(--vscode-editor-font-family), 'Courier New', Courier, monospace;
			font-size: var(--vscode-editor-font-size);
			line-height: 1.5;
		}
		
		/* JSON Syntax Highlighting */
		.string { color: var(--vscode-debugTokenExpression-string); }
		.number { color: var(--vscode-debugTokenExpression-number); }
		.boolean { color: var(--vscode-debugTokenExpression-boolean); }
		.null { color: var(--vscode-debugIcon-breakpointDisabledForeground); }
		.key { color: var(--vscode-debugTokenExpression-name); }
		
		/* Fallback colors for when VS Code variables are not available */
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) h2 { color: #d73a49; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) pre { background-color: #f8d7da; border-color: #f5c6cb; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .string { color: #ce9178; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .number { color: #b5cea8; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .boolean { color: #569cd6; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .null { color: #569cd6; }
		body:not(.vscode-dark):not(.vscode-light):not(.vscode-high-contrast) .key { color: #9cdcfe; }
		
		/* Light theme overrides */
		body.vscode-light .string { color: #a31515; }
		body.vscode-light .number { color: #098658; }
		body.vscode-light .boolean { color: #0000ff; }
		body.vscode-light .null { color: #0000ff; }
		body.vscode-light .key { color: #001080; }
	</style>
</head>
<body>
	<h2>Error Building Preview</h2>
	<div>Features: <pre><code>${highlightedFeatures}</code></pre></div>
	<pre>${message}</pre>
</body>
</html>`;
	}
}