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
      <div style={styles.header}>
        <h2 style={styles.title}>Units</h2>
        <div style={styles.headerRight}>
          <Button variant="primary">+ New Unit</Button>
          <input
            type="text"
            placeholder="Search units..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={styles.searchInput}
          />
          <div style={styles.viewToggle}>
            <button
              onClick={() => setViewMode('block')}
              style={{...styles.viewBtn, ...(viewMode === 'block' ? styles.viewBtnActive : {})}}
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
              style={{...styles.viewBtn, ...(viewMode === 'list' ? styles.viewBtnActive : {})}}
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
        <div style={styles.grid}>
          {filteredUnits.map((unit) => (
            <Link
              key={unit.name}
              to={`/units/${encodeURIComponent(unit.name)}`}
            >
              <Card hoverable>
                <div style={styles.cardHeader}>
                  <div style={styles.icon}>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <rect x="4" y="4" width="16" height="16" rx="2"/>
                      <rect x="9" y="9" width="6" height="6"/>
                    </svg>
                  </div>
                  <h3 style={styles.cardTitle}>{unit.name}</h3>
                </div>
                <div style={styles.cardMeta}>
                  <span style={styles.metaItem}>
                    Timeout: {unit.timeout ?? 30}s
                  </span>
                  <span style={styles.metaItem}>
                    Concurrency: {unit.concurrency ?? 1}
                  </span>
                </div>
                <div style={styles.divider} />
                <p style={styles.description}>
                  {unit.description || 'No description available'}
                </p>
              </Card>
            </Link>
          ))}
        </div>
      ) : (
        <div style={styles.list}>
          <div style={styles.listHeader}>
            <span style={{...styles.listCell, ...styles.listCellName}}>Name</span>
            <span style={styles.listCell}>Timeout</span>
            <span style={styles.listCell}>Concurrency</span>
            <span style={{...styles.listCell, ...styles.listCellDesc}}>Description</span>
          </div>
          {filteredUnits.map((unit) => (
            <Link
              key={unit.name}
              to={`/units/${encodeURIComponent(unit.name)}`}
              style={styles.listItem}
            >
              <span style={{...styles.listCell, ...styles.listCellName}}>
                {unit.name}
              </span>
              <span style={styles.listCell}>{unit.timeout ?? 30}s</span>
              <span style={styles.listCell}>{unit.concurrency ?? 1}</span>
              <span style={{...styles.listCell, ...styles.listCellDesc}}>
                {unit.description || '-'}
              </span>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 24,
    gap: 16,
  },
  headerRight: {
    display: 'flex',
    alignItems: 'center',
    gap: 12,
  },
  title: {
    margin: 0,
    fontSize: 24,
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  searchInput: {
    padding: '8px 16px',
    borderRadius: 8,
    border: '1px solid var(--border)',
    background: 'var(--bg-card)',
    color: 'var(--text-primary)',
    fontSize: 14,
    width: 240,
  },
  viewToggle: {
    display: 'flex',
    border: '1px solid var(--border)',
    borderRadius: 8,
    overflow: 'hidden',
  },
  viewBtn: {
    padding: '8px 12px',
    background: 'var(--bg-card)',
    border: 'none',
    cursor: 'pointer',
    fontSize: 14,
    color: 'var(--text-secondary)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  viewBtnActive: {
    background: 'var(--accent)',
    color: '#fff',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: 16,
  },
  cardHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: 12,
    marginBottom: 12,
  },
  icon: {
    width: 40,
    height: 40,
    borderRadius: 8,
    background: 'var(--accent-light)',
    color: 'var(--accent)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  cardTitle: {
    margin: 0,
    fontSize: 20,
    fontWeight: 600,
    color: 'var(--text-primary)',
    fontFamily: 'var(--font-sans)',
  },
  cardMeta: {
    display: 'flex',
    gap: 16,
    marginBottom: 8,
  },
  metaItem: {
    fontSize: 14,
    color: 'var(--text-secondary)',
    fontFamily: 'var(--font-sans)',
  },
  divider: {
    height: 1,
    backgroundColor: 'var(--border)',
    marginBottom: 12,
  },
  description: {
    margin: 0,
    fontSize: 14,
    color: 'var(--text-secondary)',
    lineHeight: 1.5,
    display: '-webkit-box',
    WebkitLineClamp: 2,
    WebkitBoxOrient: 'vertical',
    overflow: 'hidden',
    fontFamily: 'var(--font-sans)',
  },
  // List view styles
  list: {
    display: 'flex',
    flexDirection: 'column',
    background: 'var(--bg-card)',
    border: '1px solid var(--border)',
    borderRadius: 12,
    overflow: 'hidden',
  },
  listHeader: {
    display: 'flex',
    padding: '12px 16px',
    background: 'var(--bg-page)',
    borderBottom: '1px solid var(--border)',
    fontWeight: 600,
    fontSize: 13,
    color: 'var(--text-secondary)',
  },
  listItem: {
    display: 'flex',
    padding: '14px 16px',
    borderBottom: '1px solid var(--border)',
    textDecoration: 'none',
    color: 'inherit',
    transition: 'var(--transition)',
    cursor: 'pointer',
  },
  listCell: {
    fontSize: 14,
    color: 'var(--text-secondary)',
    fontFamily: 'var(--font-sans)',
    flexGrow: 0,
    flexShrink: 1,
    flexBasis: 100,
    minWidth: 100,
  },
  listCellName: {
    flexGrow: 1,
    flexShrink: 1,
    flexBasis: 200,
    fontWeight: 500,
    color: 'var(--text-primary)',
    fontFamily: 'var(--font-mono)',
    minWidth: 200,
  },
  listCellDesc: {
    flexGrow: 2,
    flexShrink: 1,
    flexBasis: 200,
    minWidth: 200,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
};
