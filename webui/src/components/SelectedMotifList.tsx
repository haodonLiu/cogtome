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
        <p>No motifs selected</p>
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
        &#9776;
      </button>
      <div style={styles.content}>
        <span style={styles.index}>{index + 1}.</span>
        <span style={styles.name}>{motif.name}</span>
      </div>
      <button style={styles.removeBtn} onClick={onRemove}>
        &times;
      </button>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  empty: {
    textAlign: 'center',
    padding: '2rem',
    color: '#888',
    backgroundColor: '#16213e',
    borderRadius: '8px',
    border: '1px dashed #0f3460',
  },
  hint: {
    fontSize: '0.875rem',
    marginTop: '0.5rem',
  },
  item: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    backgroundColor: '#16213e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    padding: '0.75rem',
  },
  dragHandle: {
    background: 'none',
    border: 'none',
    color: '#888',
    cursor: 'grab',
    fontSize: '1rem',
    padding: '0.25rem',
  },
  content: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  },
  index: {
    color: '#888',
    fontSize: '0.875rem',
  },
  name: {
    fontWeight: 'bold',
    color: '#fff',
  },
  removeBtn: {
    background: 'none',
    border: 'none',
    color: '#e94560',
    cursor: 'pointer',
    fontSize: '1.5rem',
    padding: '0 0.5rem',
  },
};
