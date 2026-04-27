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
    backgroundColor: 'var(--bg-card)',
    borderRadius: 'var(--radius-lg)',
    padding: '20px',
    border: '1px solid var(--border)',
    boxShadow: 'var(--shadow-sm)',
  },
  title: {
    margin: '0 0 16px 0',
    fontSize: '13px',
    color: 'var(--text-secondary)',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    fontWeight: 600,
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    maxHeight: '400px',
    overflowY: 'auto',
  },
  item: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: 'var(--bg-page)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-md)',
    padding: '10px 14px',
    cursor: 'pointer',
    color: 'inherit',
    textAlign: 'left',
    transition: 'var(--transition)',
    fontFamily: 'inherit',
    fontSize: '14px',
  },
  name: {
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  meta: {
    fontSize: '12px',
    color: 'var(--text-tertiary)',
  },
  loading: {
    color: 'var(--text-tertiary)',
    fontSize: '14px',
  },
  empty: {
    color: 'var(--text-tertiary)',
    fontSize: '14px',
    textAlign: 'center',
    padding: '16px 0',
  },
};
