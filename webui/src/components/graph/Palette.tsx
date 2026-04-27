import type { BlockType } from '../../types';

interface PaletteProps {
  motifs?: { name: string }[];
  onDragStart: (type: BlockType, name?: string) => void;
}

export function BlockPalette({ motifs = [], onDragStart }: PaletteProps) {
  return (
    <div style={{
      width: 220,
      background: '#0f0f1a',
      borderRight: '1px solid #3b3b5c',
      padding: 16,
      overflowY: 'auto',
      fontFamily: 'monospace',
    }}>
      <div style={{ color: '#64748b', fontSize: 11, marginBottom: 12, textTransform: 'uppercase', letterSpacing: 1 }}>
        Blocks
      </div>

      {/* Control flow */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Control</div>
        <BlockItem type="if" label="if" color="#f59e0b" onDragStart={onDragStart} />
        <BlockItem type="foreach" label="foreach" color="#06b6d4" onDragStart={onDragStart} />
        <BlockItem type="match" label="match" color="#ec4899" onDragStart={onDragStart} />
      </div>

      {/* Parallel */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Parallel</div>
        <BlockItem type="fork" label="fork" color="#8b5cf6" onDragStart={onDragStart} />
        <BlockItem type="join" label="join" color="#8b5cf6" onDragStart={onDragStart} />
      </div>

      {/* Motifs */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Motifs</div>
        {motifs.map((m) => (
          <BlockItem
            key={m.name}
            type="motif"
            label={m.name}
            color="#a855f7"
            name={m.name}
            onDragStart={onDragStart}
          />
        ))}
      </div>

      {/* Output */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Output</div>
        <BlockItem type="return" label="return" color="#22c55e" onDragStart={onDragStart} />
      </div>
    </div>
  );
}

function BlockItem({ type, label, color, name, onDragStart }: {
  type: BlockType;
  label: string;
  color: string;
  name?: string;
  onDragStart: (type: BlockType, name?: string) => void;
}) {
  return (
    <div
      draggable
      onDragStart={() => onDragStart(type, name)}
      style={{
        background: '#1a1a2e',
        border: `1px solid ${color}40`,
        borderRadius: 4,
        padding: '6px 8px',
        marginBottom: 4,
        cursor: 'grab',
        color: '#e2e8f0',
        fontSize: 12,
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        transition: 'border-color 0.15s',
      }}
      onMouseEnter={(e) => e.currentTarget.style.borderColor = color}
      onMouseLeave={(e) => e.currentTarget.style.borderColor = `${color}40`}
    >
      <div style={{
        width: 16,
        height: 16,
        borderRadius: 3,
        background: color,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: 9,
        fontWeight: 700,
        color: '#000',
      }}>
        {type[0].toUpperCase()}
      </div>
      {label}
    </div>
  );
}