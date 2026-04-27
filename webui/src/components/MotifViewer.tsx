import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { getMotif } from '../api/client';

export function MotifViewer() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [content, setContent] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!name) return;
    setContent(null);
    setError(null);
    getMotif(name)
      .then((manifest) => setContent(JSON.stringify(manifest, null, 2)))
      .catch((e) => setError((e as Error).message));
  }, [name]);

  if (error) {
    return (
      <div style={styles.container}>
        <div style={styles.breadcrumb}>
          <button style={styles.backLink} onClick={() => navigate('/motifs')}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="15 18 9 12 15 6"/>
            </svg>
            Motifs
          </button>
          <span style={styles.breadcrumbSep}>/</span>
          <span style={styles.breadcrumbCurrent}>{name}</span>
        </div>
        <div style={styles.errorAlert}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#ef4444" strokeWidth="2" strokeLinecap="round">
            <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span>{error}</span>
        </div>
      </div>
    );
  }

  if (!content) {
    return (
      <div style={styles.container}>
        <div style={styles.loading}>
          <div style={styles.spinner} />
          <span>Loading motif...</span>
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <div style={styles.breadcrumb}>
        <button style={styles.backLink} onClick={() => navigate('/motifs')}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
          Motifs
        </button>
        <span style={styles.breadcrumbSep}>/</span>
        <span style={styles.breadcrumbCurrent}>{name}</span>
      </div>

      <h2 style={styles.title}>{name}</h2>
      <pre style={styles.code}>{content}</pre>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    maxWidth: '900px',
    margin: '0 auto',
  },
  breadcrumb: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '20px',
    fontSize: '13px',
    color: 'var(--text-tertiary)',
  },
  backLink: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    background: 'none',
    border: 'none',
    color: 'var(--text-secondary)',
    cursor: 'pointer',
    fontSize: '13px',
    fontFamily: 'inherit',
    padding: '2px 0',
    transition: 'var(--transition)',
  },
  breadcrumbSep: {
    color: 'var(--text-tertiary)',
  },
  breadcrumbCurrent: {
    color: 'var(--text-primary)',
    fontWeight: 500,
  },
  title: {
    margin: '0 0 16px 0',
    fontSize: '22px',
    fontWeight: 700,
    letterSpacing: '-0.3px',
    color: 'var(--text-primary)',
  },
  code: {
    backgroundColor: 'var(--bg-page)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-lg)',
    padding: '24px',
    overflow: 'auto',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '13px',
    lineHeight: 1.6,
    color: 'var(--text-secondary)',
    maxHeight: '70vh',
    boxShadow: 'var(--shadow-sm)',
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
  errorAlert: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    backgroundColor: 'var(--danger-bg)',
    border: '1px solid #fecaca',
    borderRadius: 'var(--radius-md)',
    padding: '12px 16px',
    fontSize: '14px',
    color: 'var(--danger)',
  },
};
