import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { JsonViewer } from '@textea/json-viewer';
import { PreviewData, MessageFromExtension, MessageToExtension } from './types';

declare const acquireVsCodeApi: () => {
	postMessage(message: MessageToExtension): void;
	getState(): any;
	setState(state: any): void;
};

const vscode = acquireVsCodeApi();

export const PreviewApp: React.FC = () => {
	const [previewData, setPreviewData] = useState<PreviewData>({
		features: [],
		config: {},
		dependents: null,
		isUnsaved: false,
	});
	const [theme, setTheme] = useState<'light' | 'dark'>('dark');

	useEffect(() => {
		const updateTheme = () => {
			const body = document.body;
			const isDark = body.classList.contains('vscode-dark') || body.classList.contains('vscode-high-contrast');
			setTheme(isDark ? 'dark' : 'light');
		};

		updateTheme();
	}, []);

	useEffect(() => {
		const handleMessage = (event: MessageEvent<MessageFromExtension>) => {
			const message = event.data;
			if (message.type === 'updateConfig') {
				setPreviewData(message.data);
			}
		};

		window.addEventListener('message', handleMessage);

		vscode.postMessage({ command: 'ready' });

		return () => window.removeEventListener('message', handleMessage);
	}, []);

	const handleOpenFile = useCallback((path: string) => {
		vscode.postMessage({
			command: 'openFile',
			path,
		});
	}, []);

	// Ensure that multi-line strings are displayed correctly.
	// Without this, "\n"s are shown as spaces.
	const valueTypes = useMemo(() => [
		{
			is: (value: any) => typeof value === 'string' && value.includes('\n'),
			Component: (props: any) => {
				const [isExpanded, setIsExpanded] = React.useState(false);
				const maxLength = 80;
				const shouldCollapse = props.value.length > maxLength;

				const displayValue = shouldCollapse && !isExpanded
					? props.value.substring(0, maxLength)
					: props.value;

				const lines = displayValue.split('\n');

				return (
					<span
						style={{
							whiteSpace: 'pre-wrap',
							color: 'var(--vscode-debugTokenExpression-string)',
							cursor: shouldCollapse ? 'pointer' : 'default',
						}}
						onClick={() => shouldCollapse && setIsExpanded(!isExpanded)}
					>
						"{lines.map((line: string, i: number) => (
							<React.Fragment key={i}>
								{line}
								{i < lines.length - 1 && '\n'}
							</React.Fragment>
						))}
						{shouldCollapse && !isExpanded && '…'}
						"
					</span>
				);
			},
		},
	], []);

	return (
		<div style={{ padding: '1rem', fontFamily: 'var(--vscode-font-family)' }}>
			<h2
				style={{
					borderBottom: '2px solid var(--vscode-focusBorder)',
					paddingBottom: '10px',
					color: 'var(--vscode-foreground)',
				}}
			>
				Configuration Preview
			</h2>

			<div style={{ marginBottom: '1rem' }}>
				<strong>Features:</strong>
				<div
					style={{
						marginTop: '0.5rem',
						padding: '0.5rem',
						backgroundColor: 'var(--vscode-textCodeBlock-background)',
						border: '1px solid var(--vscode-widget-border)',
						borderRadius: '4px',
					}}
				>
					<JsonViewer
						value={previewData.features}
						theme={theme}
						rootName={false}
						displayDataTypes={false}
						displaySize={false}
					/>
				</div>
			</div>

			{previewData.dependents && previewData.dependents.length > 0 && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-foreground)' }}>Dependents</h3>
					<p>Features that import this one:</p>
					<ul>
						{previewData.dependents.map((dep) => (
							<li key={dep.name}>
								<a
									href="#"
									onClick={(e) => {
										e.preventDefault();
										handleOpenFile(dep.path);
									}}
									style={{
										color: 'var(--vscode-textLink-foreground)',
										textDecoration: 'none',
										cursor: 'pointer',
									}}
									onMouseEnter={(e) => {
										e.currentTarget.style.textDecoration = 'underline';
									}}
									onMouseLeave={(e) => {
										e.currentTarget.style.textDecoration = 'none';
									}}
								>
									{dep.name}
								</a>
							</li>
						))}
					</ul>
				</div>
			)}

			{previewData.error && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-errorForeground)' }}>Error</h3>
					<div
						style={{
							padding: '1rem',
							backgroundColor: 'var(--vscode-inputValidation-errorBackground)',
							border: '1px solid var(--vscode-inputValidation-errorBorder)',
							borderRadius: '4px',
							color: 'var(--vscode-inputValidation-errorForeground)',
							whiteSpace: 'pre-wrap',
						}}
					>
						{previewData.error}
					</div>
				</div>
			)}

			{!previewData.error && <h3 style={{ color: 'var(--vscode-foreground)' }}>Configuration:</h3>}

			{previewData.isUnsaved && (
				<div
					style={{
						padding: '0.5rem',
						marginBottom: '1rem',
						backgroundColor: 'var(--vscode-inputValidation-warningBackground)',
						border: '1px solid var(--vscode-inputValidation-warningBorder)',
						borderRadius: '4px',
						color: 'var(--vscode-inputValidation-warningForeground)',
					}}
				>
					⚠️ Preview for unsaved changes
				</div>
			)}

			{!previewData.error && (
				<div
					style={{
						padding: '1rem',
						backgroundColor: 'var(--vscode-textCodeBlock-background)',
						border: '1px solid var(--vscode-widget-border)',
						borderRadius: '4px',
						overflow: 'auto',
					}}
				>
					<JsonViewer
						value={previewData.config}
						theme={theme}
						collapseStringsAfterLength={80}
						defaultInspectDepth={6}
						displayDataTypes={false}
						displaySize={false}
						highlightUpdates={true}
						maxDisplayLength={10}
						rootName={false}
						valueTypes={valueTypes}
					/>
				</div>
			)}
		</div>
	);
};
