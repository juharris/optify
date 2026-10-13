import React from 'react';
import { DataItemProps, JsonViewerKeyRenderer } from '@textea/json-viewer';
import { toJsonPointer } from '../json-pointer';

/**
 * Creates a key renderer for the JSON viewer that adds a button to each option to show where it comes from.
 *
 * @param onAnalyzeOption Called with a JSON pointer to the option, such as "/myConfig/myKey".
 */
export function createOptionKeyRenderer(onAnalyzeOption: (keyPath: string) => void): JsonViewerKeyRenderer {
	const OptionKeyRenderer = ({ path }: DataItemProps) => {
		const key = path[path.length - 1];
		const keyPath = toJsonPointer(path);
		return (
			<>
				<button
					className="optify-origin-button"
					onClick={(e) => {
						// Prevent the JSON viewer from toggling the value when the button is clicked.
						e.preventDefault();
						e.stopPropagation();
						onAnalyzeOption(keyPath);
					}}
					title={`Show where ${keyPath} comes from`}
				>
					ⓘ
				</button>
				"{key}"
			</>
		);
	};
	// Array indices use the default rendering because values in arrays come from the same feature as the array.
	OptionKeyRenderer.when = ({ path }: DataItemProps) => typeof path[path.length - 1] === 'string';
	return OptionKeyRenderer;
}
