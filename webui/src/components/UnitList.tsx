import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { listUnits } from '../api/client';
import { UnitInfo } from '../types';
import { Card, Spinner, EmptyState, Button } from './ui';

type ViewMode = 'block' | 'list';

export function UnitList() {
  const [units, setUnits] = useState<UnitInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [viewMode, setViewMode] = useState<ViewMode>('block');

  useEffect(() => {
    listUnits()
      .then(setUnits)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const filteredUnits = units.filter(u =>
    u.name.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div>
      <div className="page-header">
        <h2 className="page-title">Units</h2>
        <div className="page-actions">
          <Button variant="primary">+ New Unit</Button>
          <input
            type="text"
            placeholder="Search units..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="search-input"
          />
          <div className="view-toggle">
            <button
              onClick={() => setViewMode('block')}
              className={`view-toggle-btn${viewMode === 'block' ? ' active' : ''}`}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="1.5" y="1.5" width="5" height="5" rx="1"/>
                <rect x="9.5" y="1.5" width="5" height="5" rx="1"/>
                <rect x="1.5" y="9.5" width="5" height="5" rx="1"/>
                <rect x="9.5" y="9.5" width="5" height="5" rx="1"/>
              </svg>
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`view-toggle-btn${viewMode === 'list' ? ' active' : ''}`}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="1.5" y="2" width="14" height="3" rx="1"/>
                <rect x="1.5" y="7" width="14" height="3" rx="1"/>
                <rect x="1.5" y="12" width="14" height="3" rx="1"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      {loading ? (
        <Spinner />
      ) : filteredUnits.length === 0 ? (
        <EmptyState
          title={search ? 'No units match your search' : 'No units found'}
          description={search ? 'Try a different search term' : 'Create your first unit to get started'}
        />
      ) : viewMode === 'block' ? (
        <div className="card-grid">
          {filteredUnits.map((unit) => (
            <Link key={unit.name} to={`/units/${encodeURIComponent(unit.name)}`} style={linkStyle}>
              <Card hoverable>
                <div style={cardHeaderStyle}>
                  <div style={iconStyle}>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <rect x="4" y="4" width="16" height="16" rx="2"/>
                      <rect x="9" y="9" width="6" height="6"/>
                    </svg>
                  </div>
                  <h3 style={cardTitleStyle}>{unit.name}</h3>
                </div>
                <div style={metaRowStyle}>
                  <span style={metaItemStyle}>Timeout: {unit.timeout ?? 30}s</span>
                  <span style={metaItemStyle}>Concurrency: {unit.concurrency ?? 1}</span>
                </div>
                <div style={dividerStyle} />
                <p style={descStyle}>{unit.description || 'No description available'}</p>
              </Card>
            </Link>
          ))}
        </div>
      ) : (
        <div style={listContainerStyle}>
          <div style={listHeaderStyle}>
            <span style={{ ...listCellStyle, ...listCellNameStyle }}>Name</span>
            <span style={listCellStyle}>Timeout</span>
            <span style={listCellStyle}>Concurrency</span>
            <span style={{ ...listCellStyle, ...listCellDescStyle }}>Description</span>
          </div>
          {filteredUnits.map((unit) => (
            <Link key={unit.name} to={`/units/${encodeURIComponent(unit.name)}`} style={listItemStyle}>
              <span style={{ ...listCellStyle, ...listCellNameStyle }}>{unit.name}</span>
              <span style={listCellStyle}>{unit.timeout ?? 30}s</span>
              <span style={listCellStyle}>{unit.concurrency ?? 1}</span>
              <span style={{ ...listCellStyle, ...listCellDescStyle }}>{unit.description || '-'}</span>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}

const linkStyle: React.CSSProperties = { textDecoration: 'none', color: 'inherit' };
const cardHeaderStyle: React.CSSProperties = { display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '12px' };
const iconStyle: React.CSSProperties = { width: 40, height: 40, borderRadius: 8, background: 'var(--accent-light)', color: 'var(--accent)', display: 'flex', alignItems: 'center', justifyContent: 'center' };
const cardTitleStyle: React.CSSProperties = { margin: 0, fontSize: 18, fontWeight: 600, color: 'var(--text-primary)' };
const metaRowStyle: React.CSSProperties = { display: 'flex', gap: 16, marginBottom: 8 };
const metaItemStyle: React.CSSProperties = { fontSize: 13, color: 'var(--text-secondary)' };
const dividerStyle: React.CSSProperties = { height: 1, backgroundColor: 'var(--border)', marginBottom: 12 };
const descStyle: React.CSSProperties = { margin: 0, fontSize: 13, color: 'var(--text-secondary)', lineHeight: 1.5, display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' };
const listContainerStyle: React.CSSProperties = { display: 'flex', flexDirection: 'column', background: 'var(--bg-card)', border: '1px solid var(--border)', borderRadius: 12, overflow: 'hidden' };
const listHeaderStyle: React.CSSProperties = { display: 'flex', padding: '12px 16px', background: 'var(--bg-page)', borderBottom: '1px solid var(--border)', fontWeight: 600, fontSize: 12, color: 'var(--text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.5px' };
const listItemStyle: React.CSSProperties = { display: 'flex', padding: '14px 16px', borderBottom: '1px solid var(--border)', textDecoration: 'none', color: 'inherit', transition: 'var(--transition)' };
const listCellStyle: React.CSSProperties = { fontSize: 13, color: 'var(--text-secondary)', flex: '0 0 100px', minWidth: 80 };
const listCellNameStyle: React.CSSProperties = { flex: '1 1 200px', fontWeight: 500, color: 'var(--text-primary)', fontFamily: 'var(--font-mono)' };
const listCellDescStyle: React.CSSProperties = { flex: '2 1 200px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' };
