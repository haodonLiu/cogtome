import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import type { MotifRef } from '../types';

interface SelectedMotifListProps {
  motifs: MotifRef[];
  onRemove: (index: number) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
}

export function SelectedMotifList({ motifs, onRemove }: SelectedMotifListProps) {
  if (motifs.length === 0) {
    return (
      <div style={styles.empty}>
        <p style={styles.emptyTitle}>No motifs selected</p>
        <p style={styles.hint}>Click a motif from the left panel to add it</p>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      {motifs.map((motif, index) => (
        <SortableMotifItem
          key={`${motif.name}-${index}`}
          motif={motif}
          index={index}
          onRemove={() => onRemove(index)}
        />
      ))}
    </div>
  );
}

interface SortableMotifItemProps {
  motif: MotifRef;
  index: number;
  onRemove: () => void;
}

function SortableMotifItem({ motif, index, onRemove }: SortableMotifItemProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: `${motif.name}-${index}`,
  });

  const style: React.CSSProperties = {
    ...styles.item,
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes}>
      <button style={styles.dragHandle} {...listeners}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="9" cy="12" r="1"/><circle cx="9" cy="5" r="1"/><circle cx="9" cy="19" r="1"/>
          <circle cx="15" cy="12" r="1"/><circle cx="15" cy="5" r="1"/><circle cx="15" cy="19" r="1"/>
        </svg>
      </button>
      <div style={styles.content}>
        <span style={styles.index}>{index + 1}.</span>
        <span style={styles.name}>{motif.name}</span>
      </div>
      <button style={styles.removeBtn} onClick={onRemove} title="Remove">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    padding: '16px 20px',
  },
  empty: {
    textAlign: 'center',
    padding: '48px 24px',
    color: 'var(--text-tertiary)',
    backgroundColor: 'var(--bg-page)',
    borderRadius: 'var(--radius-md)',
    border: '1px dashed var(--border)',
    margin: '16px 20px',
  },
  emptyTitle: {
    margin: '0 0 4px 0',
    fontSize: '14px',
    fontWeight: 500,
    color: 'var(--text-secondary)',
  },
  hint: {
    fontSize: '13px',
    margin: 0,
  },
  item: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    backgroundColor: 'var(--bg-page)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-md)',
    padding: '10px 12px',
  },
  dragHandle: {
    background: 'none',
    border: 'none',
    color: 'var(--text-tertiary)',
    cursor: 'grab',
    padding: '4px',
    display: 'flex',
    alignItems: 'center',
    borderRadius: 'var(--radius-sm)',
  },
  content: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  index: {
    color: 'var(--text-tertiary)',
    fontSize: '13px',
    fontWeight: 500,
    minWidth: '24px',
  },
  name: {
    fontWeight: 600,
    color: 'var(--text-primary)',
    fontSize: '14px',
  },
  removeBtn: {
    background: 'none',
    border: 'none',
    color: 'var(--danger)',
    cursor: 'pointer',
    padding: '4px',
    display: 'flex',
    alignItems: 'center',
    borderRadius: 'var(--radius-sm)',
    transition: 'var(--transition)',
  },
};
