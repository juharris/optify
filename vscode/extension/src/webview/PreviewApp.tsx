import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { JsonViewer } from '@textea/json-viewer';
import { ImportGraph } from './ImportGraph';
import { PreviewData, MessageFromExtension, MessageToExtension } from './types';

declare const acquireVsCodeApi: () => {
	postMessage(message: MessageToExtension): void;
	getState(): any;
	setState(state: any): void;
};

const vscode = acquireVsCodeApi();

export const PreviewApp: React.FC = () => {
	const [previewData, setPreviewData] = useState<PreviewData | undefined>(undefined);
	const [theme, setTheme] = useState<'light' | 'dark'>('dark');
	const [showGraph, setShowGraph] = useState(false);
	const [expandAll, setExpandAll] = useState<boolean | undefined>(undefined);
	const [featuresInput, setFeaturesInput] = useState<string>('');
	const [featuresInputDirty, setFeaturesInputDirty] = useState(false);
	const [suggestions, setSuggestions] = useState<string[]>([]);
	const [showSuggestions, setShowSuggestions] = useState(false);
	const inputRef = useRef<HTMLInputElement>(null);
	const blurTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	useEffect(() => {
		const updateTheme = () => {
			const body = document.body;
			const isDark = body.classList.contains('vscode-dark') || body.classList.contains('vscode-high-contrast');
			setTheme(isDark ? 'dark' : 'light');
		};
		updateTheme();
		// Clear blur timeout on unmount
		return () => {
			if (blurTimeoutRef.current !== null) {
				clearTimeout(blurTimeoutRef.current);
			}
		};
	}, []);

	useEffect(() => {
		const handleMessage = (event: MessageEvent<MessageFromExtension>) => {
			const message = event.data;
			if (message.type === 'updateConfig') {
				setPreviewData(message.data);
				if (!featuresInputDirty) {
					setFeaturesInput(message.data.features.join(', '));
				}
			} else if (message.type === 'openGraph') {
				setShowGraph(true);
			}
		};

		window.addEventListener('message', handleMessage);
		vscode.postMessage({ command: 'ready' });

		return () => window.removeEventListener('message', handleMessage);
	}, [featuresInputDirty]);

	const handleOpenFile = useCallback((path: string) => {
		vscode.postMessage({ command: 'openFile', path });
	}, []);

	const handleToggleConfigurableStrings = useCallback(() => {
		if (!previewData) return;
		const newValue = !previewData.areConfigurableStringsEnabled;
		vscode.postMessage({ command: 'setConfigurableStrings', enabled: newValue });
	}, [previewData]);

	const allSuggestionOptions = useMemo(() => {
		if (!previewData) return [];
		const options: Array<{ display: string; canonical: string; isAlias: boolean }> = [];
		for (const name of previewData.allFeatureNames) {
			options.push({ display: name, canonical: name, isAlias: false });
			const aliases = previewData.featureAliases[name] ?? [];
			for (const alias of aliases) {
				options.push({ display: `${alias} [${name}]`, canonical: name, isAlias: true });
			}
		}
		return options;
	}, [previewData]);

	const parseFeatures = useCallback((input: string): string[] => {
		return input.split(/[,\n]/).map(s => s.trim()).filter(Boolean);
	}, []);

	const handleFeaturesInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
		const value = e.target.value;
		setFeaturesInput(value);
		setFeaturesInputDirty(true);

		const parts = value.split(/[,]/);
		const lastPart = parts[parts.length - 1].trim().toLowerCase();
		if (lastPart.length > 0) {
			const filtered = allSuggestionOptions
				.filter(o => o.display.toLowerCase().includes(lastPart))
				.slice(0, 15);
			setSuggestions(filtered.map(o => o.display));
			setShowSuggestions(filtered.length > 0);
		} else {
			setShowSuggestions(false);
		}
	}, [allSuggestionOptions]);

	const handleSuggestionClick = useCallback((suggestion: string) => {
		const parts = featuresInput.split(/[,]/);
		parts[parts.length - 1] = ' ' + suggestion;
		setFeaturesInput(parts.join(','));
		setFeaturesInputDirty(true);
		setShowSuggestions(false);
		inputRef.current?.focus();
	}, [featuresInput]);

	const handleFeaturesSubmit = useCallback(() => {
		const features = parseFeatures(featuresInput);
		vscode.postMessage({ command: 'setFeatures', features });
		setFeaturesInputDirty(false);
		setShowSuggestions(false);
	}, [featuresInput, parseFeatures]);

	const handleFeaturesKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
		if (e.key === 'Enter') {
			e.preventDefault();
			handleFeaturesSubmit();
		} else if (e.key === 'Escape') {
			setShowSuggestions(false);
		}
	}, [handleFeaturesSubmit]);

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

	const displayFeatures = useMemo(() => {
		if (!previewData) return [];
		const inputFeatures = parseFeatures(featuresInput);
		return inputFeatures.map(f => {
			const isCanonical = previewData.allFeatureNames.includes(f);
			const path = isCanonical ? (previewData.featurePaths[f] ?? null) : null;
			let aliasOf: string | null = null;
			if (!isCanonical) {
				for (const [name, aliases] of Object.entries(previewData.featureAliases)) {
					if (aliases.includes(f)) {
						aliasOf = name;
						break;
					}
				}
			}
			const resolvedPath = path ?? (aliasOf ? (previewData.featurePaths[aliasOf] ?? null) : null);
			return { feature: f, path: resolvedPath, aliasOf };
		});
	}, [previewData, featuresInput, parseFeatures]);

	const configurableStringsLabel = previewData?.areConfigurableStringsEnabled
		? '✓ Configurable Strings Enabled'
		: '✗ Configurable Strings Disabled';

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

			{/* Controls toolbar */}
			{previewData && (
				<div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap', marginBottom: '1rem', alignItems: 'center' }}>
					{/* Configurable strings toggle */}
					<button
						onClick={handleToggleConfigurableStrings}
						title={`Default from config: ${previewData.areConfigurableStringsEnabledDefault ? 'enabled' : 'disabled'}`}
						style={{
							padding: '4px 10px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
							backgroundColor: previewData.areConfigurableStringsEnabled ? 'var(--vscode-button-background)' : 'var(--vscode-button-secondaryBackground)',
							color: previewData.areConfigurableStringsEnabled ? 'var(--vscode-button-foreground)' : 'var(--vscode-button-secondaryForeground)',
							border: '1px solid var(--vscode-button-border, transparent)',
						}}
					>
						{configurableStringsLabel}
					</button>

					{/* Expand/Collapse all */}
					<button
						onClick={() => setExpandAll(prev => prev === true ? false : true)}
						style={{
							padding: '4px 10px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
							backgroundColor: 'var(--vscode-button-secondaryBackground)',
							color: 'var(--vscode-button-secondaryForeground)',
							border: '1px solid var(--vscode-button-border, transparent)',
						}}
					>
						{expandAll === true ? '⊟ Collapse All' : '⊞ Expand All'}
					</button>

					{/* Graph toggle */}
					<button
						onClick={() => setShowGraph(prev => !prev)}
						style={{
							padding: '4px 10px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
							backgroundColor: showGraph ? 'var(--vscode-button-background)' : 'var(--vscode-button-secondaryBackground)',
							color: showGraph ? 'var(--vscode-button-foreground)' : 'var(--vscode-button-secondaryForeground)',
							border: '1px solid var(--vscode-button-border, transparent)',
						}}
					>
						{showGraph ? '⧁ Hide Graph' : '⧁ Show Graph'}
					</button>
				</div>
			)}

			{/* Features input box */}
			{previewData && (
				<div style={{ marginBottom: '1rem', position: 'relative' }}>
					<strong>Features:</strong>
					<div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem', alignItems: 'flex-start' }}>
						<div style={{ position: 'relative', flex: 1 }}>
							<input
								ref={inputRef}
								type="text"
								value={featuresInput}
								onChange={handleFeaturesInputChange}
								onKeyDown={handleFeaturesKeyDown}
								onBlur={() => {
									blurTimeoutRef.current = setTimeout(() => setShowSuggestions(false), 150);
								}}
								placeholder="Enter feature names, comma-separated..."
								style={{
									width: '100%', padding: '6px 8px', fontSize: '0.9rem', borderRadius: '3px',
									backgroundColor: 'var(--vscode-input-background)',
									color: 'var(--vscode-input-foreground)',
									border: '1px solid var(--vscode-input-border)',
									boxSizing: 'border-box' as const,
								}}
							/>
							{showSuggestions && (
								<div style={{
									position: 'absolute', top: '100%', left: 0, right: 0, zIndex: 100,
									backgroundColor: 'var(--vscode-dropdown-background)',
									border: '1px solid var(--vscode-dropdown-border)',
									borderRadius: '3px', maxHeight: '200px', overflowY: 'auto' as const,
								}}
								>
									{suggestions.map((s, i) => (
										<div
											key={i}
											onMouseDown={() => handleSuggestionClick(s)}
											style={{ padding: '4px 8px', cursor: 'pointer', fontSize: '0.85rem', color: 'var(--vscode-dropdown-foreground)' }}
											onMouseEnter={e => (e.currentTarget.style.backgroundColor = 'var(--vscode-list-hoverBackground)')}
											onMouseLeave={e => (e.currentTarget.style.backgroundColor = '')}
										>
											{s}
										</div>
									))}
								</div>
							)}
						</div>
						<button
							onClick={handleFeaturesSubmit}
							style={{
								padding: '6px 12px', fontSize: '0.9rem', cursor: 'pointer', borderRadius: '3px',
								backgroundColor: 'var(--vscode-button-background)',
								color: 'var(--vscode-button-foreground)',
								border: 'none', whiteSpace: 'nowrap' as const,
							}}
						>
							Preview
						</button>
					</div>

					{/* Clickable feature pills showing exact input */}
					{displayFeatures.length > 0 && (
						<div style={{
							marginTop: '0.5rem', padding: '0.4rem 0.5rem', borderRadius: '4px',
							backgroundColor: 'var(--vscode-textCodeBlock-background)',
							border: '1px solid var(--vscode-widget-border)',
							display: 'flex', flexWrap: 'wrap' as const, gap: '0.3rem',
						}}
						>
							{displayFeatures.map((f, i) => (
								<span
									key={i}
									onClick={() => f.path && handleOpenFile(f.path)}
									title={f.aliasOf ? `Alias for: ${f.aliasOf}` : (f.path ?? '')}
									style={{
										padding: '2px 8px', borderRadius: '12px', fontSize: '0.8rem',
										backgroundColor: f.aliasOf ? 'var(--vscode-badge-background)' : 'var(--vscode-button-secondaryBackground)',
										color: f.aliasOf ? 'var(--vscode-badge-foreground)' : 'var(--vscode-button-secondaryForeground)',
										cursor: f.path ? 'pointer' : 'default',
										border: '1px solid var(--vscode-widget-border)',
									}}
								>
									{f.feature}{f.aliasOf ? ` [${f.aliasOf}]` : ''}
								</span>
							))}
						</div>
					)}
				</div>
			)}

			{!previewData && <>Loading...</>}

			{/* Dependents */}
			{previewData?.dependents && previewData.dependents.length > 0 && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-foreground)' }}>Dependents</h3>
					<p>Features that import this one:</p>
					<ul>
						{previewData.dependents.map((dep) => (
							<li key={dep.name}>
								<a
									href="#"
									onClick={(e) => { e.preventDefault(); handleOpenFile(dep.path); }}
									style={{ color: 'var(--vscode-textLink-foreground)', textDecoration: 'none', cursor: 'pointer' }}
									onMouseEnter={(e) => { e.currentTarget.style.textDecoration = 'underline'; }}
									onMouseLeave={(e) => { e.currentTarget.style.textDecoration = 'none'; }}
								>
									{dep.name}
								</a>
							</li>
						))}
					</ul>
				</div>
			)}

			{/* Import graph */}
			{showGraph && previewData?.graphData && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-foreground)' }}>Import Graph</h3>
					<ImportGraph
						graphData={previewData.graphData}
						theme={theme}
						onOpenFile={handleOpenFile}
					/>
				</div>
			)}

			{previewData?.error && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-errorForeground)' }}>Error</h3>
					<div
						style={{
							padding: '1rem', borderRadius: '4px', whiteSpace: 'pre-wrap' as const,
							backgroundColor: 'var(--vscode-inputValidation-errorBackground)',
							border: '1px solid var(--vscode-inputValidation-errorBorder)',
							color: 'var(--vscode-inputValidation-errorForeground)',
						}}
					>
						{previewData.error}
					</div>
				</div>
			)}

			{previewData && !previewData.error && <h3 style={{ color: 'var(--vscode-foreground)' }}>Configuration:</h3>}

			{previewData?.isUnsaved && (
				<div
					style={{
						padding: '0.5rem', marginBottom: '1rem', borderRadius: '4px',
						backgroundColor: 'var(--vscode-inputValidation-warningBackground)',
						border: '1px solid var(--vscode-inputValidation-warningBorder)',
						color: 'var(--vscode-inputValidation-warningForeground)',
					}}
				>
					⚠️ Preview for unsaved changes
				</div>
			)}

			{previewData && !previewData.error && (
				<div
					style={{
						padding: '1rem', borderRadius: '4px', overflow: 'auto',
						backgroundColor: 'var(--vscode-textCodeBlock-background)',
						border: '1px solid var(--vscode-widget-border)',
					}}
				>
					<JsonViewer
						value={previewData.config}
						theme={theme}
						collapseStringsAfterLength={120}
						defaultInspectControl={expandAll !== undefined
							? () => expandAll
							: (path, value) => {
								if (path.length < 2) { return true; }
								if (value) {
									if (Array.isArray(value)) { return value.length < 8; }
									if (typeof value === 'object') { return Object.keys(value).length < 8; }
								}
								return true;
							}
						}
						defaultInspectDepth={7}
						displayDataTypes={false}
						displaySize={false}
						highlightUpdates={true}
						maxDisplayLength={100}
						rootName={false}
						valueTypes={valueTypes}
					/>
				</div>
			)}
		</div>
	);
};
