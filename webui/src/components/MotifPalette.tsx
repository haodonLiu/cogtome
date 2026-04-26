import { useEffect } from 'react';
import { useStructureStore } from '../store/structureStore';

interface MotifPaletteProps {
  onSelect: (motifName: string) => void;
}

export function MotifPalette({ onSelect }: MotifPaletteProps) {
  const { motifs, isLoadingLists, loadMotifs } = useStructureStore();

  useEffect(() => {
    loadMotifs();
  }, [loadMotifs]);

  return (
    <div style={styles.container}>
      <h3 style={styles.title}>Available Motifs</h3>
      {isLoadingLists && <p style={styles.loading}>Loading...</p>}
      {!isLoadingLists && (
        <div style={styles.list}>
          {motifs.map((m) => (
            <button
              key={m.name}
              style={styles.item}
              onClick={() => onSelect(m.name)}
              title={`${m.step_count} steps`}
            >
              <span style={styles.name}>{m.name}</span>
              <span style={styles.meta}>{m.step_count} steps</span>
            </button>
          ))}
          {motifs.length === 0 && <p style={styles.empty}>No motifs found</p>}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: '#16213e',
    borderRadius: '8px',
    padding: '1rem',
    border: '1px solid #0f3460',
  },
  title: {
    margin: '0 0 1rem 0',
    fontSize: '1rem',
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
    maxHeight: '400px',
    overflowY: 'auto',
  },
  item: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    padding: '0.75rem 1rem',
    cursor: 'pointer',
    color: 'inherit',
    textAlign: 'left',
    transition: 'border-color 0.2s',
  },
  name: {
    fontWeight: 'bold',
    color: '#fff',
  },
  meta: {
    fontSize: '0.75rem',
    color: '#888',
  },
  loading: {
    color: '#888',
  },
  empty: {
    color: '#888',
    fontSize: '0.875rem',
  },
};
