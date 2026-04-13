import React, { useState, useEffect, useCallback, useMemo } from 'react';
import Select, { MultiValue, StylesConfig, components as SelectComponents } from 'react-select';
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
	const [selectedFeatures, setSelectedFeatures] = useState<Array<{ value: string; label: string }>>([]);
	const [featuresInputDirty, setFeaturesInputDirty] = useState(false);

	useEffect(() => {
		// Detect VS Code theme (dark / light / high-contrast) once on mount.
		const body = document.body;
		const isDark = body.classList.contains('vscode-dark') || body.classList.contains('vscode-high-contrast');
		setTheme(isDark ? 'dark' : 'light');
	}, []);

	useEffect(() => {
		const handleMessage = (event: MessageEvent<MessageFromExtension>) => {
			const message = event.data;
			if (message.type === 'updateConfig') {
				setPreviewData(message.data);
				if (!featuresInputDirty) {
					setSelectedFeatures(message.data.features.map(f => ({ value: f, label: f })));
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
		if (!previewData) {return;}
		const newValue = !previewData.areConfigurableStringsEnabled;
		vscode.postMessage({ command: 'setConfigurableStrings', enabled: newValue });
	}, [previewData]);

	const selectOptions = useMemo(() => {
		if (!previewData) {return [];}
		const opts: Array<{ value: string; label: string; path?: string }> = [];
		for (const name of previewData.allFeatureNames) {
			opts.push({ value: name, label: name, path: previewData.featurePaths[name] });
			const aliases = previewData.featureAliases[name] ?? [];
			for (const alias of aliases) {
				opts.push({ value: alias, label: `${alias} [${name}]`, path: previewData.featurePaths[name] });
			}
		}
		return opts;
	}, [previewData]);

	const handleFeaturesChange = useCallback((newValue: MultiValue<{ value: string; label: string; path?: string }>) => {
		setSelectedFeatures(Array.from(newValue));
		setFeaturesInputDirty(true);
	}, []);

	const handleFeaturesSubmit = useCallback(() => {
		vscode.postMessage({ command: 'setFeatures', features: selectedFeatures.map(f => f.value) });
		setFeaturesInputDirty(false);
	}, [selectedFeatures]);

	const selectStyles = useMemo((): StylesConfig<{ value: string; label: string; path?: string }, true> => ({
		control: (base) => ({
			...base,
			backgroundColor: 'var(--vscode-input-background)',
			borderColor: 'var(--vscode-input-border)',
			color: 'var(--vscode-input-foreground)',
			boxShadow: 'none',
			'&:hover': { borderColor: 'var(--vscode-focusBorder)' },
		}),
		input: (base) => ({ ...base, color: 'var(--vscode-input-foreground)' }),
		menu: (base) => ({
			...base,
			backgroundColor: 'var(--vscode-dropdown-background)',
			border: '1px solid var(--vscode-dropdown-border)',
			zIndex: 100,
		}),
		option: (base, state) => ({
			...base,
			backgroundColor: state.isFocused
				? 'var(--vscode-list-hoverBackground)'
				: state.isSelected
					? 'var(--vscode-list-activeSelectionBackground)'
					: 'transparent',
			color: state.isSelected
				? 'var(--vscode-list-activeSelectionForeground)'
				: 'var(--vscode-dropdown-foreground)',
			cursor: 'pointer',
		}),
		multiValue: (base) => ({
			...base,
			backgroundColor: 'var(--vscode-badge-background)',
		}),
		multiValueLabel: (base, { data }) => ({
			...base,
			color: 'var(--vscode-badge-foreground)',
			cursor: data.path ? 'pointer' : 'default',
		}),
		multiValueRemove: (base) => ({
			...base,
			color: 'var(--vscode-badge-foreground)',
			'&:hover': {
				backgroundColor: 'var(--vscode-inputValidation-errorBackground)',
				color: 'var(--vscode-errorForeground)',
			},
		}),
		placeholder: (base) => ({ ...base, color: 'var(--vscode-input-placeholderForeground)' }),
		indicatorSeparator: (base) => ({ ...base, backgroundColor: 'var(--vscode-input-border)' }),
		dropdownIndicator: (base) => ({ ...base, color: 'var(--vscode-input-foreground)' }),
		clearIndicator: (base) => ({ ...base, color: 'var(--vscode-input-foreground)' }),
	}), []);

	const selectComponents = useMemo(() => ({
		MultiValueLabel: (props: React.ComponentProps<typeof SelectComponents.MultiValueLabel>) => {
			const path = (props.data as { path?: string }).path;
			return (
				<div
					onClick={() => path && handleOpenFile(path)}
					style={{ cursor: path ? 'pointer' : 'default', display: 'flex' }}
				>
					<SelectComponents.MultiValueLabel {...props} />
				</div>
			);
		},
	}), [handleOpenFile]);

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
					{/* Only show configurable strings toggle when the config default enables it; otherwise they won't work */}
					{previewData.areConfigurableStringsEnabledDefault && (
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
					)}

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
				<div style={{ marginBottom: '1rem' }}>
					<strong>Features:</strong>
					<div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem', alignItems: 'flex-start' }}>
						<div style={{ flex: 1 }}>
							<Select
								isMulti
								options={selectOptions}
								value={selectedFeatures}
								onChange={handleFeaturesChange}
								placeholder="Search and select features..."
								styles={selectStyles}
								components={selectComponents}
								menuPortalTarget={document.body}
								menuPosition="fixed"
							/>
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

			{previewData && !previewData.error && (
				<div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginTop: '0.5rem' }}>
					<h3 style={{ color: 'var(--vscode-foreground)', margin: 0 }}>Configuration:</h3>
					<button
						onClick={() => setExpandAll(prev => prev === true ? false : true)}
						style={{
							padding: '2px 8px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
							backgroundColor: 'var(--vscode-button-secondaryBackground)',
							color: 'var(--vscode-button-secondaryForeground)',
							border: '1px solid var(--vscode-button-border, transparent)',
						}}
					>
						{expandAll === true ? '⊟ Collapse All' : '⊞ Expand All'}
					</button>
				</div>
			)}

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
						key={`json-${String(expandAll)}`}
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
