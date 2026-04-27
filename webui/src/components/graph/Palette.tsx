import { useState, useMemo } from 'react';
import type { BlockType } from '../../types';

interface PaletteProps {
  motifs?: { name: string }[];
  onDragStart: (type: BlockType, name?: string) => void;
}

interface BlockGroup {
  label: string;
  items: { type: BlockType; label: string; colorVar: string; name?: string }[];
}

const GROUPS: BlockGroup[] = [
  {
    label: 'Structure',
    items: [
      { type: 'unit', label: 'unit', colorVar: '--node-unit' },
      { type: 'motif', label: 'motif', colorVar: '--node-motif' },
    ],
  },
  {
    label: 'Control',
    items: [
      { type: 'if', label: 'if', colorVar: '--node-if' },
      { type: 'foreach', label: 'foreach', colorVar: '--node-foreach' },
      { type: 'match', label: 'match', colorVar: '--node-match' },
    ],
  },
  {
    label: 'Parallel',
    items: [
      { type: 'fork', label: 'fork', colorVar: '--node-fork' },
      { type: 'join', label: 'join', colorVar: '--node-join' },
    ],
  },
  {
    label: 'Output',
    items: [{ type: 'return', label: 'return', colorVar: '--node-return' }],
  },
];

export function BlockPalette({ motifs = [], onDragStart }: PaletteProps) {
  const [search, setSearch] = useState('');
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [draggingType, setDraggingType] = useState<string | null>(null);

  const toggleCollapse = (label: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(label)) {
        next.delete(label);
      } else {
        next.add(label);
      }
      return next;
    });
  };

  const filteredGroups = useMemo(() => {
    if (!search.trim()) return GROUPS;
    const q = search.toLowerCase();
    return GROUPS.map((g) => ({
      ...g,
      items: g.items.filter((i) => i.label.toLowerCase().includes(q)),
    })).filter((g) => g.items.length > 0);
  }, [search]);

  const motifGroup = useMemo(() => {
    if (!search.trim()) {
      return motifs.map((m) => ({ type: 'motif' as BlockType, label: m.name, colorVar: '--node-motif', name: m.name }));
    }
    const q = search.toLowerCase();
    return motifs
      .filter((m) => m.name.toLowerCase().includes(q))
      .map((m) => ({ type: 'motif' as BlockType, label: m.name, colorVar: '--node-motif', name: m.name }));
  }, [motifs, search]);

  const motifCollapsed = collapsed.has('Motifs');

  return (
    <div className="palette">
      <div style={{ color: 'var(--text-secondary)', fontSize: 11, marginBottom: 12, textTransform: 'uppercase', letterSpacing: 1 }}>
        Blocks
      </div>

      <input
        className="palette-search"
        type="text"
        placeholder="Search blocks..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />

      {filteredGroups.map((group) => (
        <div key={group.label} className="palette-group">
          <button
            className="palette-group-header"
            onClick={() => toggleCollapse(group.label)}
          >
            <span className={`palette-chevron ${collapsed.has(group.label) ? 'collapsed' : ''}`}>▶</span>
            <span>{group.label}</span>
          </button>
          {!collapsed.has(group.label) && group.items.map((item) => (
            <BlockItem
              key={item.type}
              type={item.type}
              label={item.label}
              colorVar={item.colorVar}
              name={item.name}
              onDragStart={onDragStart}
              isDragging={draggingType === item.type}
              onDragStateChange={setDraggingType}
            />
          ))}
        </div>
      ))}

      {motifGroup.length > 0 && (
        <div className="palette-group">
          <button
            className="palette-group-header"
            onClick={() => toggleCollapse('Motifs')}
          >
            <span className={`palette-chevron ${motifCollapsed ? 'collapsed' : ''}`}>▶</span>
            <span>Motifs</span>
            <span className="palette-count">{motifGroup.length}</span>
          </button>
          {!motifCollapsed && motifGroup.map((item) => (
            <BlockItem
              key={item.name || item.type}
              type={item.type}
              label={item.label}
              colorVar={item.colorVar}
              name={item.name}
              onDragStart={onDragStart}
              isDragging={draggingType === item.name}
              onDragStateChange={setDraggingType}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function BlockItem({ type, label, colorVar, name, onDragStart, isDragging, onDragStateChange }: {
  type: BlockType;
  label: string;
  colorVar: string;
  name?: string;
  onDragStart: (type: BlockType, name?: string) => void;
  isDragging?: boolean;
  onDragStateChange?: (id: string | null) => void;
}) {
  const id = name || type;
  const color = `var(${colorVar})`;

  return (
    <div
      draggable
      onDragStart={(e) => {
        e.dataTransfer.setData('application/reactflow', type);
        e.dataTransfer.setData('application/blockname', name || '');
        e.dataTransfer.effectAllowed = 'move';
        onDragStart(type, name);
        onDragStateChange?.(id);
      }}
      onDragEnd={() => onDragStateChange?.(null)}
      className={`block-item ${isDragging ? 'dragging' : ''}`}
      style={{ '--item-color': color } as React.CSSProperties}
    >
      <div className="block-item-icon">
        {type[0].toUpperCase()}
      </div>
      <span className="block-item-label">{label}</span>
    </div>
  );
}