import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import ForceGraph2D, { ForceGraphMethods, ForceGraphProps, NodeObject } from 'react-force-graph-2d';
import { FeatureGraphData } from './types';

interface ImportGraphProps {
	graphData: FeatureGraphData;
	theme: 'light' | 'dark';
	onOpenFile?: (path: string) => void;
}

interface GraphNode extends NodeObject {
	id: string;
	path: string;
	isEnabled: boolean;
	hasImports: boolean;
}

interface GraphLink {
	source: string;
	target: string;
}

export const ImportGraph: React.FC<ImportGraphProps> = ({ graphData, theme, onOpenFile }) => {
	const { nodes, edges } = graphData;
	const [filter, setFilter] = useState('');
	type DagMode = NonNullable<ForceGraphProps['dagMode']>;
	const [dagMode, setDagMode] = useState<DagMode | undefined>(undefined);
	const containerRef = useRef<HTMLDivElement>(null);
	const graphRef = useRef<ForceGraphMethods<GraphNode, GraphLink> | undefined>(undefined);
	const [dimensions, setDimensions] = useState({ width: 600, height: 400 });
	const [hoverNode, setHoverNode] = useState<GraphNode | null>(null);
	// Track label bounding boxes per frame to skip overlapping labels.
	const drawnLabelsRef = useRef<Array<{ x1: number; y1: number; x2: number; y2: number }>>([]);

	const isDark = theme === 'dark';

	// Material Design colors — distinct enough to tell apart at a glance.
	const enabledColor = isDark ? '#66bb6a' : '#388e3c';
	const hasImportsColor = isDark ? '#42a5f5' : '#1565c0';
	const leafColor = isDark ? '#ffa726' : '#e65100';
	const textColor = isDark ? '#d4d4d4' : '#1e1e1e';
	const mutedColor = isDark ? '#9a9a9a' : '#555555';
	const backgroundColor = isDark ? '#1e1e1e' : '#ffffff';
	const linkColor = isDark ? 'rgba(255,255,255,0.2)' : 'rgba(0,0,0,0.2)';
	const dimmedColor = isDark ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.08)';

	useEffect(() => {
		if (!containerRef.current) { return; }
		const observer = new ResizeObserver(entries => {
			for (const entry of entries) {
				setDimensions({
					width: entry.contentRect.width,
					height: entry.contentRect.height,
				});
			}
		});
		observer.observe(containerRef.current);
		return () => observer.disconnect();
	}, []);

	// Build a neighbor map for hover highlighting and filter expansion.
	const neighborMap = useMemo(() => {
		const map = new Map<string, Set<string>>();
		for (const e of edges) {
			if (!map.has(e.source)) { map.set(e.source, new Set()); }
			if (!map.has(e.target)) { map.set(e.target, new Set()); }
			map.get(e.source)!.add(e.target);
			map.get(e.target)!.add(e.source);
		}
		return map;
	}, [edges]);

	const directMatchIds = useMemo(() => {
		const f = filter.toLowerCase();
		return new Set(
			nodes
				.filter(n => !f || n.id.toLowerCase().includes(f))
				.map(n => n.id)
		);
	}, [nodes, filter]);

	// Include 1-degree neighbors of direct filter matches so the graph stays connected.
	const filteredNodeIds = useMemo(() => {
		const f = filter.toLowerCase();
		if (!f) { return directMatchIds; }
		const expanded = new Set(directMatchIds);
		for (const id of directMatchIds) {
			const neighbors = neighborMap.get(id);
			if (neighbors) {
				for (const n of neighbors) { expanded.add(n); }
			}
		}
		return expanded;
	}, [directMatchIds, neighborMap, filter]);

	const isHighlighted = useCallback((nodeId: string) => {
		if (!hoverNode) { return true; }
		if (nodeId === hoverNode.id) { return true; }
		return neighborMap.get(hoverNode.id)?.has(nodeId) ?? false;
	}, [hoverNode, neighborMap]);

	const graphData2D = useMemo(() => ({
		nodes: nodes
			.filter(n => filteredNodeIds.has(n.id))
			.map(n => ({ ...n } as GraphNode)),
		links: edges
			.filter(e => filteredNodeIds.has(e.source) && filteredNodeIds.has(e.target))
			.map(e => ({ source: e.source, target: e.target })),
	}), [nodes, edges, filteredNodeIds]);

	const getNodeColor = useCallback((node: GraphNode) => {
		if (node.isEnabled) { return enabledColor; }
		if (node.hasImports) { return hasImportsColor; }
		return leafColor;
	}, [enabledColor, hasImportsColor, leafColor]);

	const handleNodeClick = useCallback((node: GraphNode) => {
		if (node.path && onOpenFile) {
			onOpenFile(node.path);
		}
	}, [onOpenFile]);

	const handleNodeHover = useCallback((node: GraphNode | null) => {
		setHoverNode(node);
	}, []);

	const nodeCanvasObject = useCallback((node: GraphNode, ctx: CanvasRenderingContext2D, globalScale: number) => {
		const highlighted = isHighlighted(node.id);
		const isFilterNeighbor = filter && !directMatchIds.has(node.id);
		const label = node.id;
		const fontSize = Math.max(12 / globalScale, 2);
		ctx.font = `${fontSize}px sans-serif`;
		ctx.globalAlpha = !highlighted ? 0.15 : isFilterNeighbor ? 0.35 : 1;

		const nodeRadius = 5;
		if (node.isEnabled) {
			// Star shape distinguishes enabled (selected) features from other nodes.
			const spikes = 5;
			const outerRadius = nodeRadius * 4;
			const innerRadius = nodeRadius * 2;
			ctx.beginPath();
			for (let i = 0; i < spikes * 2; i++) {
				const r = i % 2 === 0 ? outerRadius : innerRadius;
				const angle = (Math.PI * i) / spikes - Math.PI / 2;
				ctx.lineTo(node.x! + r * Math.cos(angle), node.y! + r * Math.sin(angle));
			}
			ctx.closePath();
		} else {
			ctx.beginPath();
			ctx.arc(node.x!, node.y!, nodeRadius, 0, 2 * Math.PI);
		}
		ctx.fillStyle = getNodeColor(node);
		ctx.fill();

		// Only draw labels when zoomed in enough.
		if (globalScale > 0.7) {
			const textWidth = ctx.measureText(label).width;
			const labelX1 = node.x! - textWidth / 2;
			const labelY1 = node.y! + nodeRadius + 1;
			const labelX2 = labelX1 + textWidth;
			const labelY2 = labelY1 + fontSize;
			const overlaps = drawnLabelsRef.current.some(
				r => labelX1 < r.x2 && labelX2 > r.x1 && labelY1 < r.y2 && labelY2 > r.y1
			);
			if (!overlaps) {
				drawnLabelsRef.current.push({ x1: labelX1, y1: labelY1, x2: labelX2, y2: labelY2 });
				ctx.textAlign = 'center';
				ctx.textBaseline = 'top';
				ctx.fillStyle = textColor;
				ctx.fillText(label, node.x!, labelY1);
			}
		}
		ctx.globalAlpha = 1;
	}, [getNodeColor, textColor, isHighlighted, filter, directMatchIds]);

	const btnBase: React.CSSProperties = {
		padding: '2px 8px', fontSize: '0.8rem', cursor: 'pointer', borderRadius: '3px',
		border: '1px solid var(--vscode-button-border, transparent)',
	};

	const handleZoomToFit = useCallback(() => {
		graphRef.current?.zoomToFit(400, 40);
	}, []);

	// Auto-fit after initial render.
	useEffect(() => {
		const timer = setTimeout(() => graphRef.current?.zoomToFit(400, 40), 500);
		return () => clearTimeout(timer);
	}, [graphData2D]);

	return (
		<div>
			<div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', marginBottom: '0.5rem', flexWrap: 'wrap' }}>
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
				<button
					onClick={handleZoomToFit}
					style={{
						...btnBase,
						backgroundColor: 'var(--vscode-button-secondaryBackground)',
						color: 'var(--vscode-button-secondaryForeground)',
					}}
				>
					Fit to view
				</button>
				<select
					value={dagMode || ''}
					onChange={e => setDagMode((e.target.value as DagMode) || undefined)}
					style={{
						padding: '2px 6px', fontSize: '0.8rem', borderRadius: '3px',
						backgroundColor: 'var(--vscode-dropdown-background)',
						color: 'var(--vscode-dropdown-foreground)',
						border: '1px solid var(--vscode-dropdown-border)',
						cursor: 'pointer',
					}}
				>
					<option value="">Force-directed</option>
					<option value="td">Top → Down</option>
					<option value="bu">Bottom → Up</option>
					<option value="lr">Left → Right</option>
					<option value="rl">Right → Left</option>
					<option value="radialout">Radial out</option>
					<option value="radialin">Radial in</option>
				</select>
			</div>
			<div style={{ display: 'flex', gap: '1rem', fontSize: '0.75rem', color: mutedColor, marginBottom: '0.5rem' }}>
				<span><span style={{ color: enabledColor }}>★</span> Enabled</span>
				<span><span style={{ color: hasImportsColor }}>&#9679;</span> Has imports</span>
				<span><span style={{ color: leafColor }}>&#9679;</span> No imports</span>
			</div>
			<div
				ref={containerRef}
				style={{ height: '400px', border: '1px solid var(--vscode-widget-border)', borderRadius: '4px', overflow: 'hidden' }}
			>
				{graphData2D.nodes.length > 0
					? <ForceGraph2D
						ref={graphRef}
						graphData={graphData2D}
						width={dimensions.width}
						height={dimensions.height}
						backgroundColor={backgroundColor}
						onRenderFramePre={() => { drawnLabelsRef.current = []; }}
						nodeCanvasObject={nodeCanvasObject}
						nodePointerAreaPaint={(node: GraphNode, color, ctx) => {
							ctx.beginPath();
							ctx.arc(node.x!, node.y!, 6, 0, 2 * Math.PI);
							ctx.fillStyle = color;
							ctx.fill();
						}}
						autoPauseRedraw={false}
						linkColor={(link: any) => {
							if (!hoverNode) { return linkColor; }
							const sourceId = typeof link.source === 'object' ? link.source.id : link.source;
							const targetId = typeof link.target === 'object' ? link.target.id : link.target;
							if (sourceId === hoverNode.id || targetId === hoverNode.id) { return linkColor; }
							return dimmedColor;
						}}
						linkWidth={4}
						linkDirectionalArrowLength={6}
						linkDirectionalArrowRelPos={0.75}
						onNodeHover={handleNodeHover}
						onNodeClick={handleNodeClick}
						cooldownTicks={100}
						d3VelocityDecay={0.5}
						dagMode={dagMode}
						enableZoomInteraction={true}
						enablePanInteraction={true}
					/>
					: <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: mutedColor }}>
						No nodes match the filter.
					</div>
				}
			</div>
			<div style={{ fontSize: '0.75rem', color: mutedColor, marginTop: '0.25rem' }}>
				{graphData2D.nodes.length} nodes &middot; {graphData2D.links.length} edges &middot; Click a node to open its file &middot; Scroll to zoom &middot; Drag to pan
			</div>
		</div>
	);
};
