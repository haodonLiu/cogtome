import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useStructureStore } from '../store/structureStore';
import { Card, Spinner, EmptyState, ErrorBanner, Button } from './ui';

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
      <div className="page-header">
        <div>
          <h2 className="page-title">Structures</h2>
          <p className="page-subtitle">Build and manage execution pipelines</p>
        </div>
        <Button onClick={handleNew}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          New Structure
        </Button>
      </div>

      {listError && <ErrorBanner message={listError} />}

      {isLoadingLists && (
        <div style={loadingStyle}>
          <Spinner />
          <span>Loading structures...</span>
        </div>
      )}

      {!isLoadingLists && !listError && (
        <div className="card-grid">
          {structures.map((s) => (
            <Card key={s.name} hoverable onClick={() => navigate(`/structures/${encodeURIComponent(s.name)}`)} className="structure-card" style={cardStyle}>
              <div style={cardTopStyle}>
                <div style={iconStyle}>
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4f46e5" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/>
                  </svg>
                </div>
                <button
                  className="delete-btn"
                  style={deleteBtnStyle}
                  onClick={(e) => handleDelete(e, s.name)}
                  title="Delete"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                  </svg>
                </button>
              </div>
              <h3 style={cardTitleStyle}>{s.name}</h3>
              <p style={cardMetaStyle}>{s.motif_count} {s.motif_count === 1 ? 'motif' : 'motifs'}</p>
              <div style={arrowStyle}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </div>
            </Card>
          ))}

          {structures.length === 0 && (
            <EmptyState
              title="No structures yet"
              description="Create your first structure to start building pipelines"
              action={<Button onClick={handleNew}>Create Structure</Button>}
            />
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
const cardStyle: React.CSSProperties = { padding: '24px', position: 'relative', overflow: 'hidden' };
const cardTopStyle: React.CSSProperties = { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' };
const iconStyle: React.CSSProperties = { width: '40px', height: '40px', borderRadius: '10px', backgroundColor: 'var(--accent-light)', display: 'flex', alignItems: 'center', justifyContent: 'center' };
const deleteBtnStyle: React.CSSProperties = { background: 'none', border: 'none', color: 'var(--text-tertiary)', cursor: 'pointer', padding: '6px', borderRadius: 'var(--radius-sm)', display: 'flex', alignItems: 'center', justifyContent: 'center', opacity: 0 };
const cardTitleStyle: React.CSSProperties = { margin: '0 0 4px', fontSize: '16px', fontWeight: 600, color: 'var(--text-primary)' };
const cardMetaStyle: React.CSSProperties = { margin: 0, color: 'var(--text-tertiary)', fontSize: '13px' };
const arrowStyle: React.CSSProperties = { position: 'absolute', right: '20px', bottom: '20px', color: 'var(--text-tertiary)' };
