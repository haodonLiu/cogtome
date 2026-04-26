import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useStructureStore } from '../store/structureStore';

export function MotifList() {
  const { motifs, isLoadingLists, listError, loadMotifs } = useStructureStore();

  useEffect(() => {
    loadMotifs();
  }, [loadMotifs]);

  return (
    <div>
      <div style={styles.header}>
        <h2 style={styles.title}>Motifs</h2>
      </div>

      {isLoadingLists && <p>Loading...</p>}
      {listError && <p style={styles.error}>Error: {listError}</p>}

      {!isLoadingLists && !listError && (
        <div style={styles.grid}>
          {motifs.map((m) => (
            <Link key={m.name} to={`/motifs/${encodeURIComponent(m.name)}`} style={styles.card}>
              <h3 style={styles.cardTitle}>{m.name}</h3>
              <p style={styles.cardMeta}>{m.step_count} steps</p>
            </Link>
          ))}
          {motifs.length === 0 && <p style={styles.empty}>No motifs found</p>}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    marginBottom: '1.5rem',
  },
  title: {
    margin: 0,
    fontSize: '1.5rem',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
    gap: '1rem',
  },
  card: {
    backgroundColor: '#16213e',
    padding: '1.5rem',
    borderRadius: '8px',
    textDecoration: 'none',
    color: 'inherit',
    border: '1px solid #0f3460',
    transition: 'border-color 0.2s',
  },
  cardTitle: {
    margin: '0 0 0.5rem 0',
    fontSize: '1.25rem',
    color: '#fff',
  },
  cardMeta: {
    margin: 0,
    color: '#888',
    fontSize: '0.875rem',
  },
  error: {
    color: '#e94560',
  },
  empty: {
    color: '#888',
    gridColumn: '1 / -1',
  },
};
