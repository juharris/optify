import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { JsonViewerKeyRenderer } from '@textea/json-viewer';
import { createOptionKeyRenderer } from './OptionKeyRenderer';
import { MessageFromExtension, OptionAnalysisData, PreviewData } from './types';
import { vscode } from './vscode-api';

export interface OptionAnalysisState {
	/** The analysis for `keyPath` or `undefined` while it is loading. */
	analysisData?: OptionAnalysisData;
	close: () => void;
	/** Renders keys in the JSON viewer with a button to analyze the option. */
	keyRenderer: JsonViewerKeyRenderer;
	/** A JSON pointer to the option being analyzed or `undefined` if no option is being analyzed. */
	keyPath?: string;
}

/**
 * Manages analyzing where an option in the preview comes from.
 * The analysis is requested again when the preview changes so that it stays in sync.
 */
export function useOptionAnalysis(previewData: PreviewData | undefined): OptionAnalysisState {
	const [analysisData, setAnalysisData] = useState<OptionAnalysisData | undefined>(undefined);
	const [keyPath, setKeyPath] = useState<string | undefined>(undefined);
	// The message handler is only registered once, so it reads the current key path from a ref.
	const keyPathRef = useRef<string | undefined>(undefined);

	useEffect(() => {
		const handleMessage = (event: MessageEvent<MessageFromExtension>) => {
			const message = event.data;
			// Ignore results for options that are no longer selected.
			if (message.type === 'optionAnalysis' && message.data.keyPath === keyPathRef.current) {
				setAnalysisData(message.data);
			}
		};

		window.addEventListener('message', handleMessage);
		return () => window.removeEventListener('message', handleMessage);
	}, []);

	useEffect(() => {
		// Keep the analysis in sync with the preview, such as when features change or files are edited.
		const currentKeyPath = keyPathRef.current;
		if (previewData && currentKeyPath) {
			vscode.postMessage({ command: 'analyzeOption', keyPath: currentKeyPath });
		}
	}, [previewData]);

	const analyze = useCallback((newKeyPath: string) => {
		keyPathRef.current = newKeyPath;
		setKeyPath(newKeyPath);
		setAnalysisData(undefined);
		vscode.postMessage({ command: 'analyzeOption', keyPath: newKeyPath });
	}, []);

	const close = useCallback(() => {
		keyPathRef.current = undefined;
		setKeyPath(undefined);
		setAnalysisData(undefined);
	}, []);

	const keyRenderer = useMemo(() => createOptionKeyRenderer(analyze), [analyze]);

	return { analysisData, close, keyRenderer, keyPath };
}
