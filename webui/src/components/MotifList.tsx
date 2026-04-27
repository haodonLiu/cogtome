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
        <div>
          <h2 style={styles.title}>Motifs</h2>
          <p style={styles.subtitle}>Browse available motif definitions</p>
        </div>
      </div>

      {isLoadingLists && (
        <div style={styles.loading}>
          <div style={styles.spinner} />
          <span>Loading motifs...</span>
        </div>
      )}

      {listError && (
        <div style={styles.errorBanner}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#ef4444" strokeWidth="2" strokeLinecap="round">
            <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          {listError}
        </div>
      )}

      {!isLoadingLists && !listError && (
        <div style={styles.grid}>
          {motifs.map((m) => (
            <Link key={m.name} to={`/motifs/${encodeURIComponent(m.name)}`} className="motif-card" style={styles.card}>
              <h3 style={styles.cardTitle}>{m.name}</h3>
              <p style={styles.cardMeta}>{m.step_count} {m.step_count === 1 ? 'step' : 'steps'}</p>
              <div style={styles.cardArrow}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </div>
            </Link>
          ))}
          {motifs.length === 0 && (
            <div style={styles.empty}>
              <p style={styles.emptyTitle}>No motifs found</p>
              <p style={styles.emptyText}>Check your skills directory for motif YAML files</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-end',
    marginBottom: '28px',
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: 700,
    color: 'var(--text-primary)',
    letterSpacing: '-0.3px',
  },
  subtitle: {
    margin: '4px 0 0 0',
    fontSize: '14px',
    color: 'var(--text-tertiary)',
  },
  loading: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    color: 'var(--text-tertiary)',
    padding: '40px 0',
    fontSize: '14px',
  },
  spinner: {
    width: '18px',
    height: '18px',
    border: '2px solid var(--border)',
    borderTopColor: 'var(--accent)',
    borderRadius: '50%',
    animation: 'spin 0.8s linear infinite',
  },
  errorBanner: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    backgroundColor: 'var(--danger-bg)',
    color: 'var(--danger)',
    padding: '12px 16px',
    borderRadius: 'var(--radius-md)',
    fontSize: '14px',
    marginBottom: '20px',
    border: '1px solid #fecaca',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
    gap: '16px',
  },
  card: {
    backgroundColor: 'var(--bg-card)',
    padding: '24px',
    borderRadius: 'var(--radius-lg)',
    textDecoration: 'none',
    color: 'inherit',
    border: '1px solid var(--border)',
    transition: 'var(--transition)',
    boxShadow: 'var(--shadow-sm)',
    position: 'relative',
    overflow: 'hidden',
  },
  cardTitle: {
    margin: '0 0 4px 0',
    fontSize: '16px',
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  cardMeta: {
    margin: 0,
    color: 'var(--text-tertiary)',
    fontSize: '13px',
  },
  cardArrow: {
    position: 'absolute',
    right: '20px',
    bottom: '20px',
    color: 'var(--text-tertiary)',
    transition: 'var(--transition)',
  },
  empty: {
    gridColumn: '1 / -1',
    textAlign: 'center',
    padding: '64px 24px',
    backgroundColor: 'var(--bg-card)',
    borderRadius: 'var(--radius-lg)',
    border: '1px dashed var(--border)',
  },
  emptyTitle: {
    margin: '0 0 4px 0',
    fontSize: '16px',
    fontWeight: 600,
    color: 'var(--text-secondary)',
  },
  emptyText: {
    margin: 0,
    fontSize: '14px',
    color: 'var(--text-tertiary)',
  },
};
