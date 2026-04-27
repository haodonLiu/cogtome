import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useStructureStore } from '../store/structureStore';
import { Card, Spinner, EmptyState, ErrorBanner } from './ui';

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
          <Spinner />
          <span>Loading motifs...</span>
        </div>
      )}

      {listError && (
        <ErrorBanner message={listError} />
      )}

      {!isLoadingLists && !listError && (
        <div style={styles.grid}>
          {motifs.map((m) => (
            <Link key={m.name} to={`/motifs/${encodeURIComponent(m.name)}`} className="motif-card" style={styles.cardLink}>
              <Card hoverable padding="lg" style={styles.cardInner}>
                <h3 style={styles.cardTitle}>{m.name}</h3>
                <p style={styles.cardMeta}>{m.step_count} {m.step_count === 1 ? 'step' : 'steps'}</p>
                <div style={styles.cardArrow}>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="9 18 15 12 9 6"/>
                  </svg>
                </div>
              </Card>
            </Link>
          ))}
          {motifs.length === 0 && (
            <div style={styles.emptyWrapper}>
              <EmptyState
                title="No motifs found"
                description="Check your skills directory for motif YAML files"
              />
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
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
    gap: '16px',
  },
  cardLink: {
    textDecoration: 'none',
    color: 'inherit',
  },
  cardInner: {
    position: 'relative',
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
  emptyWrapper: {
    gridColumn: '1 / -1',
  },
};