import React, { useMemo, useState } from 'react';
import { GraphCanvas, GraphEdge as ReagraphEdge, GraphNode as ReagraphNode, InternalGraphNode, darkTheme, lightTheme } from 'reagraph';
import { FeatureGraphData } from './types';

type LayoutType = 'forceDirected3d' | 'forceDirected2d' | 'circular2d' | 'treeTd2d' | 'hierarchicalTd';

interface ImportGraphProps {
graphData: FeatureGraphData;
theme: 'light' | 'dark';
onOpenFile?: (path: string) => void;
}

const LAYOUT_LABELS: Record<LayoutType, string> = {
forceDirected3d: '3D Force',
forceDirected2d: '2D Force',
circular2d: 'Circular',
treeTd2d: 'Tree',
hierarchicalTd: 'Hierarchical',
};

export const ImportGraph: React.FC<ImportGraphProps> = ({ graphData, theme, onOpenFile }) => {
const { nodes, edges } = graphData;
const [layout, setLayout] = useState<LayoutType>('forceDirected3d');
const [filter, setFilter] = useState('');

const isDark = theme === 'dark';

// Material Design colors — distinct enough to tell apart at a glance.
const enabledColor = isDark ? '#66bb6a' : '#388e3c';    // Material Green 400 / 700
const hasImportsColor = isDark ? '#42a5f5' : '#1565c0'; // Material Blue 400 / 800
const leafColor = isDark ? '#ffa726' : '#e65100';       // Material Orange 400 / 900

const graphNodes: ReagraphNode[] = useMemo(() => {
const f = filter.toLowerCase();
return nodes
.filter(n => !f || n.id.toLowerCase().includes(f))
.map(n => ({
id: n.id,
label: n.id,
fill: n.isEnabled ? enabledColor : n.hasImports ? hasImportsColor : leafColor,
data: { path: n.path },
}));
}, [nodes, filter, enabledColor, hasImportsColor, leafColor]);

const visibleNodeIds = useMemo(() => new Set(graphNodes.map(n => n.id)), [graphNodes]);

const graphEdges: ReagraphEdge[] = useMemo(() =>
edges
.filter(e => visibleNodeIds.has(e.source) && visibleNodeIds.has(e.target))
.map((e, i) => ({ id: `e-${i}`, source: e.source, target: e.target })),
[edges, visibleNodeIds],
);

const graphTheme = useMemo(() => ({
...(isDark ? darkTheme : lightTheme),
canvas: { background: isDark ? '#1e1e1e' : '#ffffff' },
}), [isDark]);

const handleNodeClick = (node: InternalGraphNode) => {
const path = (node.data as { path?: string })?.path;
if (path && onOpenFile) {
onOpenFile(path);
}
};

const textColor = isDark ? '#d4d4d4' : '#1e1e1e';
const mutedColor = isDark ? '#9a9a9a' : '#555555';
const btnBase: React.CSSProperties = {
padding: '2px 8px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
border: '1px solid var(--vscode-button-border, transparent)',
};

return (
<div>
<div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', marginBottom: '0.5rem', flexWrap: 'wrap' }}>
<span style={{ color: textColor, fontSize: '0.85rem' }}>Layout:</span>
{(Object.keys(LAYOUT_LABELS) as LayoutType[]).map(lt => (
<button
key={lt}
onClick={() => setLayout(lt)}
style={{
...btnBase,
backgroundColor: layout === lt
? 'var(--vscode-button-background)'
: 'var(--vscode-button-secondaryBackground)',
color: layout === lt
? 'var(--vscode-button-foreground)'
: 'var(--vscode-button-secondaryForeground)',
}}
>
{LAYOUT_LABELS[lt]}
</button>
))}
<input
type="text"
placeholder="Filter nodes..."
value={filter}
onChange={e => setFilter(e.target.value)}
style={{
padding: '2px 6px', fontSize: '0.8rem',
backgroundColor: 'var(--vscode-input-background)',
color: 'var(--vscode-input-foreground)',
border: '1px solid var(--vscode-input-border)',
borderRadius: '3px', width: '140px',
}}
/>
</div>
<div style={{ display: 'flex', gap: '1rem', fontSize: '0.75rem', color: mutedColor, marginBottom: '0.5rem' }}>
<span><span style={{ color: enabledColor }}>&#9679;</span> Enabled</span>
<span><span style={{ color: hasImportsColor }}>&#9679;</span> Has imports</span>
<span><span style={{ color: leafColor }}>&#9679;</span> No imports</span>
</div>
<div style={{ height: '600px', border: '1px solid var(--vscode-widget-border)', borderRadius: '4px', overflow: 'hidden' }}>
{graphNodes.length > 0
? <GraphCanvas
nodes={graphNodes}
edges={graphEdges}
layoutType={layout}
theme={graphTheme}
draggable
onNodeClick={handleNodeClick}
/>
: <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: mutedColor }}>
No nodes match the filter.
</div>
}
</div>
<div style={{ fontSize: '0.75rem', color: mutedColor, marginTop: '0.25rem' }}>
{graphNodes.length} nodes &middot; {graphEdges.length} edges &middot; Click a node to open its file &middot; Scroll to zoom &middot; Drag to pan
</div>
</div>
);
};
