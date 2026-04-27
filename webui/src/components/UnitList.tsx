import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { listUnits } from '../api/client';
import { UnitInfo } from '../types';

export function UnitList() {
  const [units, setUnits] = useState<UnitInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

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
        <input
          type="text"
          placeholder="Search units..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={styles.searchInput}
        />
      </div>

      {loading ? (
        <div style={styles.loading}>Loading units...</div>
      ) : filteredUnits.length === 0 ? (
        <div style={styles.empty}>
          {search ? 'No units match your search' : 'No units found'}
        </div>
      ) : (
        <div style={styles.grid}>
          {filteredUnits.map((unit) => (
            <Link
              key={unit.name}
              to={`/units/${encodeURIComponent(unit.name)}`}
              style={styles.card}
            >
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
              {unit.description && (
                <p style={styles.description}>{unit.description}</p>
              )}
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
  loading: {
    textAlign: 'center',
    padding: 48,
    color: 'var(--text-secondary)',
  },
  empty: {
    textAlign: 'center',
    padding: 48,
    color: 'var(--text-secondary)',
    background: 'var(--bg-card)',
    borderRadius: 12,
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: 16,
  },
  card: {
    display: 'flex',
    flexDirection: 'column',
    padding: 16,
    background: 'var(--bg-card)',
    border: '1px solid var(--border)',
    borderRadius: 12,
    textDecoration: 'none',
    color: 'inherit',
    transition: 'var(--transition)',
    cursor: 'pointer',
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
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--text-primary)',
    fontFamily: 'monospace',
  },
  cardMeta: {
    display: 'flex',
    gap: 16,
    marginBottom: 8,
  },
  metaItem: {
    fontSize: 12,
    color: 'var(--text-secondary)',
    fontFamily: 'monospace',
  },
  description: {
    margin: 0,
    fontSize: 13,
    color: 'var(--text-secondary)',
    lineHeight: 1.5,
    display: '-webkit-box',
    WebkitLineClamp: 2,
    WebkitBoxOrient: 'vertical',
    overflow: 'hidden',
  },
};
