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
      <div className="page-header">
        <div>
          <h2 className="page-title">Motifs</h2>
          <p className="page-subtitle">Browse available motif definitions</p>
        </div>
      </div>

      {isLoadingLists && (
        <div style={loadingStyle}>
          <Spinner />
          <span>Loading motifs...</span>
        </div>
      )}

      {listError && <ErrorBanner message={listError} />}

      {!isLoadingLists && !listError && (
        <div className="card-grid card-grid--compact">
          {motifs.map((m) => (
            <Link key={m.name} to={`/motifs/${encodeURIComponent(m.name)}`} className="motif-card" style={linkStyle}>
              <Card hoverable padding="lg" style={cardInnerStyle}>
                <h3 style={titleStyle}>{m.name}</h3>
                <p style={metaStyle}>{m.step_count} {m.step_count === 1 ? 'step' : 'steps'}</p>
                <div style={arrowStyle}>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="9 18 15 12 9 6"/>
                  </svg>
                </div>
              </Card>
            </Link>
          ))}
          {motifs.length === 0 && (
            <div style={{ gridColumn: '1 / -1' }}>
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

const loadingStyle: React.CSSProperties = {
  display: 'flex', alignItems: 'center', gap: '12px',
  color: 'var(--text-tertiary)', padding: '40px 0', fontSize: '14px',
};
const linkStyle: React.CSSProperties = { textDecoration: 'none', color: 'inherit' };
const cardInnerStyle: React.CSSProperties = { position: 'relative' };
const titleStyle: React.CSSProperties = { margin: '0 0 4px', fontSize: '16px', fontWeight: 600, color: 'var(--text-primary)' };
const metaStyle: React.CSSProperties = { margin: 0, color: 'var(--text-tertiary)', fontSize: '13px' };
const arrowStyle: React.CSSProperties = { position: 'absolute', right: '20px', bottom: '20px', color: 'var(--text-tertiary)' };
