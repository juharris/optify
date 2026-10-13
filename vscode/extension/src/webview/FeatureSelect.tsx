import React, { useMemo } from 'react';
import Select, { GroupBase, MultiValue, MultiValueGenericProps, StylesConfig, components as SelectComponents } from 'react-select';
import { PreviewData } from './types';

export interface FeatureOption {
	value: string;
	label: string;
	path?: string;
}

const selectStyles: StylesConfig<FeatureOption, true> = {
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
};

export interface FeatureSelectProps {
	onChange: (features: MultiValue<FeatureOption>) => void;
	onOpenFile: (path: string) => void;
	previewData: PreviewData;
	selectedFeatures: FeatureOption[];
}

/**
 * Selects the features to build the preview for.
 * Clicking a selected feature opens its file.
 */
export const FeatureSelect: React.FC<FeatureSelectProps> = ({ onChange, onOpenFile, previewData, selectedFeatures }) => {
	const selectOptions = useMemo(() => {
		const opts: FeatureOption[] = [];
		for (const name of previewData.allFeatureNames) {
			opts.push({ value: name, label: name, path: previewData.featurePaths[name] });
			const aliases = previewData.featureAliases[name];
			if (aliases) {
				for (const alias of aliases) {
					opts.push({ value: name, label: `${name} (alias ${alias})`, path: previewData.featurePaths[name] });
				}
			}
		}
		return opts;
	}, [previewData]);

	const selectComponents = useMemo(() => ({
		MultiValueLabel: (props: MultiValueGenericProps<FeatureOption, true, GroupBase<FeatureOption>>) => {
			const path = (props.data as { path?: string }).path;
			return (
				<div
					onClick={() => path && onOpenFile(path)}
					style={{ cursor: path ? 'pointer' : 'default', display: 'flex' }}
				>
					<SelectComponents.MultiValueLabel {...props} />
				</div>
			);
		},
	}), [onOpenFile]);

	return (
		<Select
			isMulti
			options={selectOptions}
			value={selectedFeatures}
			onChange={onChange}
			placeholder="Search and select features..."
			styles={selectStyles}
			components={selectComponents}
			menuPortalTarget={document.body}
			menuPosition="fixed"
		/>
	);
};
