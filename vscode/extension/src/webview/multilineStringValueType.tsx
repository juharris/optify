import React from 'react';
import { DataType } from '@textea/json-viewer';

/**
 * Displays multi-line strings in the JSON viewer.
 * Without this, "\n"s are shown as spaces.
 */
export const multilineStringValueType: DataType<string> = {
	is: (value: unknown) => typeof value === 'string' && value.includes('\n'),
	Component: (props) => {
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
};
