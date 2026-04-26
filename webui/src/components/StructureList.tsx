import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useStructureStore } from '../store/structureStore';

export function StructureList() {
  const { structures, isLoadingLists, listError, loadStructures, createNewStructure } =
    useStructureStore();

  useEffect(() => {
    loadStructures();
  }, [loadStructures]);

  return (
    <div>
      <div style={styles.header}>
        <h2 style={styles.title}>Structures</h2>
        <button style={styles.button} onClick={createNewStructure}>
          + New Structure
        </button>
      </div>

      {isLoadingLists && <p>Loading...</p>}
      {listError && <p style={styles.error}>Error: {listError}</p>}

      {!isLoadingLists && !listError && (
        <div style={styles.grid}>
          {structures.map((s) => (
            <Link key={s.name} to={`/structures/${encodeURIComponent(s.name)}`} style={styles.card}>
              <h3 style={styles.cardTitle}>{s.name}</h3>
              <p style={styles.cardMeta}>{s.motif_count} motifs</p>
            </Link>
          ))}
          {structures.length === 0 && <p style={styles.empty}>No structures found</p>}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1.5rem',
  },
  title: {
    margin: 0,
    fontSize: '1.5rem',
  },
  button: {
    backgroundColor: '#e94560',
    color: '#fff',
    border: 'none',
    padding: '0.75rem 1.5rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '1rem',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))',
    gap: '1rem',
  },
  card: {
    backgroundColor: '#16213e',
    padding: '1.5rem',
    borderRadius: '8px',
    textDecoration: 'none',
    color: 'inherit',
    border: '1px solid #0f3460',
    transition: 'transform 0.2s, border-color 0.2s',
  },
  cardTitle: {
    margin: '0 0 0.5rem 0',
    fontSize: '1.25rem',
    color: '#e94560',
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
