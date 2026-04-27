import React, { useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useStructureStore } from '../store/structureStore';

export function StructureList() {
  const { structures, isLoadingLists, listError, loadStructures, createNewStructure, deleteStructure } =
    useStructureStore();
  const navigate = useNavigate();

  useEffect(() => {
    loadStructures();
  }, [loadStructures]);

  const handleDelete = async (e: React.MouseEvent, name: string) => {
    e.preventDefault();
    e.stopPropagation();
    if (!confirm(`Delete structure "${name}"? This cannot be undone.`)) return;
    await deleteStructure(name);
  };

  const handleNew = () => {
    createNewStructure();
    navigate('/structures/new');
  };

  return (
    <div>
      {/* Page Header */}
      <div style={styles.header}>
        <div>
          <h2 style={styles.title}>Structures</h2>
          <p style={styles.subtitle}>Build and manage execution pipelines</p>
        </div>
        <button style={styles.newBtn} onClick={handleNew}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          New Structure
        </button>
      </div>

      {/* Error */}
      {listError && (
        <div style={styles.errorBanner}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#ef4444" strokeWidth="2" strokeLinecap="round">
            <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          {listError}
        </div>
      )}

      {/* Loading */}
      {isLoadingLists && (
        <div style={styles.loading}>
          <div style={styles.spinner} />
          <span>Loading structures...</span>
        </div>
      )}

      {/* Grid */}
      {!isLoadingLists && !listError && (
        <div style={styles.grid}>
          {structures.map((s) => (
            <Link key={s.name} to={`/structures/${encodeURIComponent(s.name)}`} className="structure-card" style={styles.card}>
              <div style={styles.cardTop}>
                <div style={styles.cardIcon}>
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4f46e5" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/>
                  </svg>
                </div>
                <button
                  className="delete-btn"
                  style={styles.deleteBtn}
                  onClick={(e) => handleDelete(e, s.name)}
                  title="Delete"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                  </svg>
                </button>
              </div>
              <h3 style={styles.cardTitle}>{s.name}</h3>
              <p style={styles.cardMeta}>{s.motif_count} {s.motif_count === 1 ? 'motif' : 'motifs'}</p>
              <div style={styles.cardArrow}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </div>
            </Link>
          ))}

          {structures.length === 0 && (
            <div style={styles.empty}>
              <div style={styles.emptyIcon}>
                <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/>
                </svg>
              </div>
              <p style={styles.emptyTitle}>No structures yet</p>
              <p style={styles.emptyText}>Create your first structure to start building pipelines</p>
              <button style={styles.emptyBtn} onClick={handleNew}>
                Create Structure
              </button>
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
  newBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    backgroundColor: 'var(--accent)',
    color: '#fff',
    border: 'none',
    padding: '10px 20px',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 600,
    fontFamily: 'inherit',
    transition: 'var(--transition)',
    boxShadow: 'var(--shadow-sm)',
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
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
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
  cardTop: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  cardIcon: {
    width: '40px',
    height: '40px',
    borderRadius: '10px',
    backgroundColor: 'var(--accent-light)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  deleteBtn: {
    background: 'none',
    border: 'none',
    color: 'var(--text-tertiary)',
    cursor: 'pointer',
    padding: '6px',
    borderRadius: 'var(--radius-sm)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'var(--transition)',
    opacity: 0,
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
  emptyIcon: {
    marginBottom: '16px',
  },
  emptyTitle: {
    margin: '0 0 4px 0',
    fontSize: '16px',
    fontWeight: 600,
    color: 'var(--text-secondary)',
  },
  emptyText: {
    margin: '0 0 20px 0',
    fontSize: '14px',
    color: 'var(--text-tertiary)',
  },
  emptyBtn: {
    backgroundColor: 'var(--accent)',
    color: '#fff',
    border: 'none',
    padding: '10px 24px',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 600,
    fontFamily: 'inherit',
    transition: 'var(--transition)',
  },
};
