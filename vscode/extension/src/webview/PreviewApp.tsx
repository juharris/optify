import React, { useState, useEffect, useCallback } from 'react';
import { MultiValue } from 'react-select';
import { JsonViewer } from '@textea/json-viewer';
import { ErrorBoundary } from './ErrorBoundary';
import { FeatureOption, FeatureSelect } from './FeatureSelect';
import { FileLink } from './FileLink';
import { ImportGraph } from './ImportGraph';
import { LoadingIndicator } from './LoadingIndicator';
import { multilineStringValueType } from './multilineStringValueType';
import { OptionOriginPanel } from './OptionOriginPanel';
import { PreviewData, FeatureGraphData, MessageFromExtension } from './types';
import { useOptionAnalysis } from './useOptionAnalysis';
import { vscode } from './vscode-api';

const VALUE_TYPES = [multilineStringValueType];

export const PreviewApp: React.FC = () => {
	const [previewData, setPreviewData] = useState<PreviewData | undefined>(undefined);
	const [graphData, setGraphData] = useState<FeatureGraphData | undefined>(undefined);
	const [isGraphLoading, setIsGraphLoading] = useState(true);
	const [theme, setTheme] = useState<'light' | 'dark'>('dark');
	const [showGraph, setShowGraph] = useState(false);
	const [expandAll, setExpandAll] = useState<boolean | undefined>(true);
	const [selectedFeatures, setSelectedFeatures] = useState<FeatureOption[]>([]);
	const optionAnalysis = useOptionAnalysis(previewData);

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
				setSelectedFeatures(message.data.features.map(f => ({ value: f, label: f })));
			} else if (message.type === 'updateGraph') {
				setGraphData(message.data.graphData);
				setIsGraphLoading(false);
			}
		};

		window.addEventListener('message', handleMessage);

		vscode.postMessage({ command: 'ready' });

		return () => window.removeEventListener('message', handleMessage);
	}, []);

	const handleOpenFile = useCallback((path: string, keyPath?: string) => {
		vscode.postMessage({ command: 'openFile', path, keyPath });
	}, []);

	const handleToggleConfigurableValues = useCallback(() => {
		if (!previewData) { return; }
		const newValue = !previewData.areConfigurableValuesEnabled;
		vscode.postMessage({ command: 'setConfigurableValues', enabled: newValue });
	}, [previewData]);

	const handleFeaturesChange = useCallback((newValue: MultiValue<FeatureOption>) => {
		const features = Array.from(newValue);
		setSelectedFeatures(features);
		vscode.postMessage({ command: 'setFeatures', features: features.map(f => f.value) });
	}, []);

	const configurableValuesLabel = previewData?.areConfigurableValuesEnabled
		? '✓ Configurable Values Enabled'
		: '✗ Configurable Values Disabled';

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

			{/* Features input box */}
			{previewData && (
				<div style={{ marginBottom: '1rem' }}>
					<strong>Features:</strong>
					<FeatureSelect
						onChange={handleFeaturesChange}
						onOpenFile={handleOpenFile}
						previewData={previewData}
						selectedFeatures={selectedFeatures}
					/>
				</div>
			)}

			{!previewData && <LoadingIndicator label="Loading configuration..." />}

			{/* Dependents */}
			{previewData?.dependents && previewData.dependents.length > 0 && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-foreground)' }}>Dependents</h3>
					<p>Features that import this one:</p>
					<ul>
						{previewData.dependents.map((dep) => (
							<li key={dep.name}>
								<FileLink onOpenFile={handleOpenFile} path={dep.path}>
									{dep.name}
								</FileLink>
							</li>
						))}
					</ul>
				</div>
			)}

			{/* Import graph */}
			{previewData && (
				<div style={{ marginBottom: '1rem' }}>
					<div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.5rem' }}>
						<h3 style={{ color: 'var(--vscode-foreground)', margin: 0 }}>Import Graph</h3>
						{graphData && (
							<button
								onClick={() => setShowGraph(prev => !prev)}
								style={{
									padding: '2px 8px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
									backgroundColor: 'var(--vscode-button-secondaryBackground)',
									color: 'var(--vscode-button-secondaryForeground)',
									border: '1px solid var(--vscode-button-border, transparent)',
								}}
							>
								{showGraph ? 'Hide Import Graph' : 'Show Import Graph'}
							</button>
						)}
					</div>
					{isGraphLoading && <LoadingIndicator label="Loading import graph..." />}
					{!isGraphLoading && graphData && showGraph && (
						<ErrorBoundary fallback={(error) => (
							<div style={{
								padding: '1rem', borderRadius: '4px', whiteSpace: 'pre-wrap' as const,
								backgroundColor: 'var(--vscode-inputValidation-errorBackground)',
								border: '1px solid var(--vscode-inputValidation-errorBorder)',
								color: 'var(--vscode-inputValidation-errorForeground)',
							}}>
								Failed to render graph: {error.message}
							</div>
						)}>
							<ImportGraph
								graphData={graphData}
								theme={theme}
								onOpenFile={handleOpenFile}
							/>
						</ErrorBoundary>
					)}
				</div>
			)}

			{previewData?.error && (
				<div style={{ marginBottom: '1rem' }}>
					<h3 style={{ color: 'var(--vscode-errorForeground)' }}>Error</h3>
					<div
						style={{
							padding: '1rem',
							backgroundColor: 'var(--vscode-inputValidation-errorBackground)',
							border: '1px solid var(--vscode-inputValidation-errorBorder)',
							borderRadius: '4px',
							color: 'var(--vscode-inputValidation-errorForeground)',
							whiteSpace: 'pre-wrap' as const,
						}}
					>
						{previewData.error}
					</div>
					{/* Show configurable values toggle as disabled during errors so users can see its state */}
					{previewData.areConfigurableValuesEnabledDefault && (
						<button
							disabled
							style={{
								marginTop: '0.5rem',
								padding: '2px 8px', fontSize: '0.8rem', cursor: 'not-allowed', borderRadius: '3px',
								opacity: 0.5,
								backgroundColor: previewData.areConfigurableValuesEnabled ? 'var(--vscode-button-background)' : 'var(--vscode-button-secondaryBackground)',
								color: previewData.areConfigurableValuesEnabled ? 'var(--vscode-button-foreground)' : 'var(--vscode-button-secondaryForeground)',
								border: '1px solid var(--vscode-button-border, transparent)',
							}}
						>
							{configurableValuesLabel}
						</button>
					)}
				</div>
			)}

			{previewData && !previewData.error && (
				<div style={{ marginTop: '0.5rem', marginBottom: '0.75rem' }}>
					<div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', flexWrap: 'wrap' }}>
						<h3 style={{ color: 'var(--vscode-foreground)', margin: 0 }}>Configuration</h3>
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
						{/* Only show configurable values toggle when the config default enables it; otherwise they won't work */}
						{previewData.areConfigurableValuesEnabledDefault && (
							<button
								onClick={handleToggleConfigurableValues}
								style={{
									padding: '2px 8px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
									backgroundColor: previewData.areConfigurableValuesEnabled ? 'var(--vscode-button-background)' : 'var(--vscode-button-secondaryBackground)',
									color: previewData.areConfigurableValuesEnabled ? 'var(--vscode-button-foreground)' : 'var(--vscode-button-secondaryForeground)',
									border: '1px solid var(--vscode-button-border, transparent)',
								}}
							>
								{configurableValuesLabel}
							</button>
						)}
					</div>
					{previewData.configurableValuesMessage && (
						<div
							style={{
								marginTop: '0.5rem',
								maxWidth: '70rem',
							}}
						>
							<p
								style={{
									marginTop: 0,
									marginBottom: previewData.configurableValuesError ? '0.5rem' : 0,
									color: 'var(--vscode-descriptionForeground)',
								}}
							>
								{previewData.configurableValuesMessage}
							</p>
							{previewData.configurableValuesError && (
								<div
									style={{
										padding: '0.75rem',
										backgroundColor: 'var(--vscode-inputValidation-warningBackground)',
										border: '1px solid var(--vscode-inputValidation-warningBorder)',
										borderRadius: '4px',
										color: 'var(--vscode-inputValidation-warningForeground)',
										whiteSpace: 'pre-wrap' as const,
									}}
								>
									{previewData.configurableValuesError}
								</div>
							)}
						</div>
					)}
				</div>
			)}

			{previewData?.isUnsaved && (
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

			{previewData && !previewData.error && optionAnalysis.keyPath && (
				<OptionOriginPanel
					analysisData={optionAnalysis.analysisData}
					featurePaths={previewData.featurePaths}
					keyPath={optionAnalysis.keyPath}
					onClose={optionAnalysis.close}
					onOpenFile={handleOpenFile}
				/>
			)}

			{previewData && !previewData.error && (
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
						key={`json-${String(expandAll)}`}
						value={previewData.config}
						theme={theme}
						collapseStringsAfterLength={120}
						defaultInspectControl={expandAll !== undefined
							? expandAll
								? () => true
								// Show top-level keys but collapse their values
								: (path: (string | number)[]) => path.length < 1
							: (path, value) => {
								if (path.length < 2) {
									// Keep top-level nodes expanded
									return true;
								}
								if (value) {
									// Collapse large objects/arrays by default.
									if (Array.isArray(value)) {
										return value.length < 8;
									}
									if (typeof value === 'object') {
										return Object.keys(value).length < 8;
									}
								}
								// Show primitives
								return true;
							}
						}
						defaultInspectDepth={7}
						displayDataTypes={false}
						displaySize={false}
						highlightUpdates={true}
						keyRenderer={optionAnalysis.keyRenderer}
						maxDisplayLength={100}
						rootName={false}
						valueTypes={VALUE_TYPES}
					/>
				</div>
			)}
		</div>
	);
};
