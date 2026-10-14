import { GetOptionsPreferences } from "@optify/config";
import { PreviewWhileEditingOptions } from "./types";

/**
 * Creates preferences that use the same features and overrides as the preview.
 */
export function createPreferences(editingOptions?: PreviewWhileEditingOptions): GetOptionsPreferences {
	const preferences = new GetOptionsPreferences();
	preferences.setSkipFeatureNameConversion(true);
	if (editingOptions?.overrides) {
		preferences.setOverridesJson(editingOptions.overrides);
	}
	return preferences;
}
