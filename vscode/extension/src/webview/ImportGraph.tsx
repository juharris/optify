import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { FeatureGraphData } from './types';

interface NodePos {
	x: number;
	y: number;
	vx: number;
	vy: number;
}

type LayoutType = 'force' | 'circular' | 'hierarchical';

interface ImportGraphProps {
	graphData: FeatureGraphData;
	theme: 'light' | 'dark';
	onOpenFile?: (path: string) => void;
}

const GRAPH_WIDTH = 900;
const GRAPH_HEIGHT = 600;
const NODE_RADIUS = 10;
const MAX_HOVER_LABEL_LENGTH = 35;
const HOVER_LABEL_TRUNCATE_LENGTH = 33;
const MAX_LABEL_LENGTH = 20;
const LABEL_TRUNCATE_LENGTH = 18;

function buildHierarchicalLayout(
	nodeIds: string[],
	edges: Array<{ source: string; target: string }>
): Record<string, { x: number; y: number }> {
	const inDegree = new Map<string, number>();
	const outEdges = new Map<string, string[]>();

	for (const n of nodeIds) {
		inDegree.set(n, 0);
		outEdges.set(n, []);
	}
	for (const e of edges) {
		inDegree.set(e.target, (inDegree.get(e.target) ?? 0) + 1);
		const existing = outEdges.get(e.source) ?? [];
		outEdges.set(e.source, [...existing, e.target]);
	}

	const levels = new Map<string, number>();
	const queue: string[] = nodeIds.filter(n => (inDegree.get(n) ?? 0) === 0);
	let maxLevel = 0;

	// BFS to assign levels
	const visited = new Set<string>();
	while (queue.length > 0) {
		const cur = queue.shift()!;
		if (visited.has(cur)) continue;
		visited.add(cur);
		const level = levels.get(cur) ?? 0;
		if (level > maxLevel) maxLevel = level;
		for (const next of (outEdges.get(cur) ?? [])) {
			const nextLevel = Math.max(levels.get(next) ?? 0, level + 1);
			levels.set(next, nextLevel);
			inDegree.set(next, (inDegree.get(next) ?? 1) - 1);
			if ((inDegree.get(next) ?? 1) <= 0) {
				queue.push(next);
			}
		}
	}

	// Handle cycles: assign remaining nodes to level 0
	for (const n of nodeIds) {
		if (!levels.has(n)) levels.set(n, 0);
	}

	const byLevel = new Map<number, string[]>();
	for (const n of nodeIds) {
		const l = levels.get(n) ?? 0;
		byLevel.set(l, [...(byLevel.get(l) ?? []), n]);
	}

	const numLevels = maxLevel + 1;
	const pos: Record<string, { x: number; y: number }> = {};
	for (let l = 0; l <= maxLevel; l++) {
		const nodesAtLevel = byLevel.get(l) ?? [];
		const y = GRAPH_HEIGHT * 0.1 + (l / Math.max(numLevels - 1, 1)) * (GRAPH_HEIGHT * 0.8);
		nodesAtLevel.forEach((n, i) => {
			const x = nodesAtLevel.length === 1
				? GRAPH_WIDTH / 2
				: GRAPH_WIDTH * 0.1 + (i / Math.max(nodesAtLevel.length - 1, 1)) * (GRAPH_WIDTH * 0.8);
			pos[n] = { x, y };
		});
	}

	return pos;
}

export const ImportGraph: React.FC<ImportGraphProps> = ({ graphData, theme, onOpenFile }) => {
	const { nodes, edges } = graphData;
	const [layout, setLayout] = useState<LayoutType>('force');
	const [positions, setPositions] = useState<Record<string, NodePos>>({});
	const [hoveredNode, setHoveredNode] = useState<string | null>(null);
	const [dragging, setDragging] = useState<string | null>(null);
	const [filter, setFilter] = useState('');
	const animFrameRef = useRef<number | null>(null);
	const posRef = useRef<Record<string, NodePos>>({});
	const svgRef = useRef<SVGSVGElement>(null);

	const isDark = theme === 'dark';

	const colors = useMemo(() => ({
		background: isDark ? '#1e1e1e' : '#ffffff',
		nodeEnabled: isDark ? '#4ec9b0' : '#0098a6',
		nodeNoImports: isDark ? '#569cd6' : '#0070c1',
		nodeDefault: isDark ? '#9cdcfe' : '#001080',
		nodeStroke: isDark ? 'rgba(255,255,255,0.6)' : 'rgba(0,0,0,0.6)',
		edge: isDark ? 'rgba(200,200,200,0.35)' : 'rgba(0,0,0,0.25)',
		edgeHighlight: isDark ? '#f5a623' : '#e87b00',
		text: isDark ? '#d4d4d4' : '#1e1e1e',
		textSmall: isDark ? '#9a9a9a' : '#555555',
	}), [isDark]);

	const nodeIds = useMemo(() => nodes.map(n => n.id), [nodes]);

	const initPositions = useCallback((lt: LayoutType) => {
		const pos: Record<string, NodePos> = {};
		const n = nodeIds.length;

		if (lt === 'circular') {
			nodeIds.forEach((id, i) => {
				const angle = (i / n) * 2 * Math.PI - Math.PI / 2;
				const r = Math.min(GRAPH_WIDTH, GRAPH_HEIGHT) * 0.4;
				pos[id] = {
					x: GRAPH_WIDTH / 2 + r * Math.cos(angle),
					y: GRAPH_HEIGHT / 2 + r * Math.sin(angle),
					vx: 0, vy: 0,
				};
			});
		} else if (lt === 'hierarchical') {
			const hpos = buildHierarchicalLayout(nodeIds, edges);
			nodeIds.forEach(id => {
				const p = hpos[id] ?? { x: GRAPH_WIDTH / 2, y: GRAPH_HEIGHT / 2 };
				pos[id] = { ...p, vx: 0, vy: 0 };
			});
		} else {
			// Force: random initial positions with some jitter
			nodeIds.forEach((id, i) => {
				const angle = (i / nodeIds.length) * 2 * Math.PI;
				const r = 80 + Math.random() * 150;
				pos[id] = {
					x: GRAPH_WIDTH / 2 + r * Math.cos(angle),
					y: GRAPH_HEIGHT / 2 + r * Math.sin(angle),
					vx: 0, vy: 0,
				};
			});
		}
		posRef.current = pos;
		setPositions({ ...pos });
		return pos;
	}, [nodeIds, edges]);

	// Force simulation
	useEffect(() => {
		if (animFrameRef.current) {
			cancelAnimationFrame(animFrameRef.current);
		}

		if (layout !== 'force') {
			initPositions(layout);
			return;
		}

		initPositions('force');

		let alpha = 1.0;
		const alphaDecay = 0.018;
		let frameCount = 0;

		const simulate = () => {
			if (alpha < 0.001) return;
			alpha *= (1 - alphaDecay);
			frameCount++;

			const p = posRef.current;
			const ids = nodeIds;
			const count = ids.length;

			// Repulsion between all pairs
			const repulsion = 1800;
			for (let i = 0; i < count; i++) {
				for (let j = i + 1; j < count; j++) {
					const pi = p[ids[i]];
					const pj = p[ids[j]];
					if (!pi || !pj) continue;
					const dx = pj.x - pi.x;
					const dy = pj.y - pi.y;
					const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
					const force = (repulsion / (dist * dist)) * alpha;
					const fx = (dx / dist) * force;
					const fy = (dy / dist) * force;
					pi.vx -= fx; pi.vy -= fy;
					pj.vx += fx; pj.vy += fy;
				}
			}

			// Spring attraction for edges
			const springLength = 120;
			const springStrength = 0.3;
			for (const e of edges) {
				const ps = p[e.source];
				const pt = p[e.target];
				if (!ps || !pt) continue;
				const dx = pt.x - ps.x;
				const dy = pt.y - ps.y;
				const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
				const force = (dist - springLength) * springStrength * alpha;
				const fx = (dx / dist) * force;
				const fy = (dy / dist) * force;
				ps.vx += fx; ps.vy += fy;
				pt.vx -= fx; pt.vy -= fy;
			}

			// Center gravity
			const centerStrength = 0.05;
			for (const id of ids) {
				const pi = p[id];
				if (!pi) continue;
				pi.vx += (GRAPH_WIDTH / 2 - pi.x) * centerStrength * alpha;
				pi.vy += (GRAPH_HEIGHT / 2 - pi.y) * centerStrength * alpha;
			}

			// Apply velocity + damping + bounds
			const damping = 0.85;
			const padding = 30;
			for (const id of ids) {
				const pi = p[id];
				if (!pi || id === dragging) continue;
				pi.vx *= damping;
				pi.vy *= damping;
				pi.x = Math.max(padding, Math.min(GRAPH_WIDTH - padding, pi.x + pi.vx));
				pi.y = Math.max(padding, Math.min(GRAPH_HEIGHT - padding, pi.y + pi.vy));
			}

			posRef.current = { ...p };
			// Only update React state every 3 frames to reduce re-renders
			if (frameCount % 3 === 0) {
				setPositions({ ...p });
			}

			animFrameRef.current = requestAnimationFrame(simulate);
		};

		animFrameRef.current = requestAnimationFrame(simulate);

		return () => {
			if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current);
		};
	}, [layout, nodeIds, edges, initPositions, dragging]);

	const filteredNodes = useMemo(() => {
		if (!filter) return null;
		const f = filter.toLowerCase();
		return new Set(nodeIds.filter(id => id.toLowerCase().includes(f)));
	}, [nodeIds, filter]);

	const getNodeColor = useCallback((nodeId: string) => {
		const node = nodes.find(n => n.id === nodeId);
		if (!node) return colors.nodeDefault;
		if (node.isEnabled) return colors.nodeEnabled;
		if (!node.hasImports) return colors.nodeNoImports;
		return colors.nodeDefault;
	}, [nodes, colors]);

	const handleMouseDown = useCallback((id: string, e: React.MouseEvent<SVGElement>) => {
		e.preventDefault();
		setDragging(id);
	}, []);

	const handleMouseMove = useCallback((e: React.MouseEvent<SVGSVGElement>) => {
		if (!dragging || !svgRef.current) return;
		const rect = svgRef.current.getBoundingClientRect();
		const scaleX = GRAPH_WIDTH / rect.width;
		const scaleY = GRAPH_HEIGHT / rect.height;
		const x = (e.clientX - rect.left) * scaleX;
		const y = (e.clientY - rect.top) * scaleY;
		const prev = posRef.current[dragging];
		posRef.current = {
			...posRef.current,
			[dragging]: { ...prev, x, y, vx: 0, vy: 0 },
		};
		setPositions({ ...posRef.current });
	}, [dragging]);

	const handleMouseUp = useCallback(() => {
		setDragging(null);
	}, []);

	const visibleEdges = useMemo(() => {
		if (!filteredNodes) return edges;
		return edges.filter(e => filteredNodes.has(e.source) || filteredNodes.has(e.target));
	}, [edges, filteredNodes]);

	return (
		<div style={{ position: 'relative' }}>
			<div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', marginBottom: '0.5rem', flexWrap: 'wrap' }}>
				<span style={{ color: colors.text, fontSize: '0.85rem' }}>Layout:</span>
				{(['force', 'circular', 'hierarchical'] as LayoutType[]).map(lt => (
					<button
						key={lt}
						onClick={() => setLayout(lt)}
						style={{
							padding: '2px 8px',
							fontSize: '0.8rem',
							cursor: 'pointer',
							backgroundColor: layout === lt
								? 'var(--vscode-button-background)'
								: 'var(--vscode-button-secondaryBackground)',
							color: layout === lt
								? 'var(--vscode-button-foreground)'
								: 'var(--vscode-button-secondaryForeground)',
							border: '1px solid var(--vscode-button-border, transparent)',
							borderRadius: '3px',
						}}
					>
						{lt.charAt(0).toUpperCase() + lt.slice(1)}
					</button>
				))}
				<input
					type="text"
					placeholder="Filter nodes..."
					value={filter}
					onChange={e => setFilter(e.target.value)}
					style={{
						padding: '2px 6px',
						fontSize: '0.8rem',
						backgroundColor: 'var(--vscode-input-background)',
						color: 'var(--vscode-input-foreground)',
						border: '1px solid var(--vscode-input-border)',
						borderRadius: '3px',
						width: '140px',
					}}
				/>
			</div>
			<div style={{ display: 'flex', gap: '1rem', fontSize: '0.75rem', color: colors.textSmall, marginBottom: '0.5rem' }}>
				<span><span style={{ color: colors.nodeEnabled }}>●</span> Enabled</span>
				<span><span style={{ color: colors.nodeNoImports }}>●</span> No imports</span>
				<span><span style={{ color: colors.nodeDefault }}>●</span> Has imports</span>
			</div>
			<svg
				ref={svgRef}
				width="100%"
				viewBox={`0 0 ${GRAPH_WIDTH} ${GRAPH_HEIGHT}`}
				style={{
					backgroundColor: colors.background,
					border: '1px solid var(--vscode-widget-border)',
					borderRadius: '4px',
					cursor: dragging ? 'grabbing' : 'default',
					display: 'block',
				}}
				onMouseMove={handleMouseMove}
				onMouseUp={handleMouseUp}
				onMouseLeave={handleMouseUp}
			>
				<defs>
					<marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
						<polygon points="0 0, 8 3, 0 6" fill={colors.edge} />
					</marker>
					<marker id="arrowhead-highlight" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
						<polygon points="0 0, 8 3, 0 6" fill={colors.edgeHighlight} />
					</marker>
				</defs>

				{visibleEdges.map((e, i) => {
					const sp = positions[e.source];
					const tp = positions[e.target];
					if (!sp || !tp) return null;

					const dx = tp.x - sp.x;
					const dy = tp.y - sp.y;
					const dist = Math.sqrt(dx * dx + dy * dy) + 0.1;
					// Shorten end point to not overlap with node circle
					const tx = tp.x - (dx / dist) * (NODE_RADIUS + 8);
					const ty = tp.y - (dy / dist) * (NODE_RADIUS + 8);

					const isHighlighted = hoveredNode === e.source || hoveredNode === e.target;
					const isFiltered = filteredNodes !== null && !filteredNodes.has(e.source) && !filteredNodes.has(e.target);

					return (
						<line
							key={i}
							x1={sp.x} y1={sp.y}
							x2={tx} y2={ty}
							stroke={isHighlighted ? colors.edgeHighlight : colors.edge}
							strokeWidth={isHighlighted ? 2 : 1}
							markerEnd={isHighlighted ? 'url(#arrowhead-highlight)' : 'url(#arrowhead)'}
							opacity={isFiltered ? 0.05 : 1}
						/>
					);
				})}

				{nodeIds.map(id => {
					const pos = positions[id];
					if (!pos) return null;
					const nodeData = nodes.find(n => n.id === id);
					const isHovered = hoveredNode === id;
					const isFiltered = filteredNodes !== null && !filteredNodes.has(id);
					const color = getNodeColor(id);
					const r = isHovered ? NODE_RADIUS * 1.4 : NODE_RADIUS;
					const showLabel = nodeIds.length <= 50;

					return (
						<g
							key={id}
							transform={`translate(${pos.x},${pos.y})`}
							style={{ cursor: nodeData?.path ? 'pointer' : 'default', userSelect: 'none' }}
							onMouseEnter={() => setHoveredNode(id)}
							onMouseLeave={() => setHoveredNode(null)}
							onMouseDown={(e) => handleMouseDown(id, e)}
							onClick={() => {
								if (nodeData?.path && onOpenFile) {
									onOpenFile(nodeData.path);
								}
							}}
							opacity={isFiltered ? 0.15 : 1}
						>
							<circle
								r={r}
								fill={color}
								stroke={colors.nodeStroke}
								strokeWidth={isHovered ? 2 : 1}
								opacity={0.9}
							/>
							{isHovered && (
								<text
									dy="-16"
									textAnchor="middle"
									fontSize="11"
									fill={colors.text}
									style={{ pointerEvents: 'none' }}
								>
									{id.length > MAX_HOVER_LABEL_LENGTH ? id.slice(0, HOVER_LABEL_TRUNCATE_LENGTH) + '…' : id}
								</text>
							)}
							{showLabel && !isHovered && (
								<text
									dy={NODE_RADIUS + 11}
									textAnchor="middle"
									fontSize="8"
									fill={colors.textSmall}
									style={{ pointerEvents: 'none' }}
								>
									{id.length > MAX_LABEL_LENGTH ? id.slice(0, LABEL_TRUNCATE_LENGTH) + '…' : id}
								</text>
							)}
						</g>
					);
				})}
			</svg>
			<div style={{ fontSize: '0.75rem', color: colors.textSmall, marginTop: '0.25rem' }}>
				{nodes.length} nodes · {edges.length} edges · Click a node to open its file · Drag nodes to rearrange
			</div>
		</div>
	);
};
