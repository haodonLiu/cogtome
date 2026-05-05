import { useState, useEffect } from 'react';
import { listTraces, getTracesForSkill, type TraceInfo, type TraceDetail, type TraceNode } from '../api/client';

interface TraceStats {
  avgMs: number;
  totalCalls: number;
  successRate: number;
  slowestNode: { id: string; avgMs: number } | null;
}

function computeStats(traces: TraceDetail[]): TraceStats {
  if (traces.length === 0) {
    return { avgMs: 0, totalCalls: 0, successRate: 0, slowestNode: null };
  }
  const totalMs = traces.reduce((sum, t) => sum + (t.duration_ms || 0), 0);
  const successCount = traces.filter(t => t.status === 'success').length;
  const nodeTimes: Record<string, number[]> = {};
  traces.forEach(t => {
    t.nodes.forEach(n => {
      if (n.ms !== null) {
        if (!nodeTimes[n.id]) nodeTimes[n.id] = [];
        nodeTimes[n.id].push(n.ms);
      }
    });
  });
  let slowestNode: { id: string; avgMs: number } | null = null;
  for (const [id, times] of Object.entries(nodeTimes)) {
    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    if (!slowestNode || avg > slowestNode.avgMs) slowestNode = { id, avgMs: Math.round(avg) };
  }
  return {
    avgMs: Math.round(totalMs / traces.length),
    totalCalls: traces.length,
    successRate: Math.round((successCount / traces.length) * 100),
    slowestNode,
  };
}

function NodeBar({ node, maxMs }: { node: TraceNode; maxMs: number }) {
  const width = node.ms ? Math.max(2, (node.ms / maxMs) * 100) : 0;
  const bgColor = node.ok ? 'var(--success)' : 'var(--danger)';
  return (
    <div style={nodeBarStyle}>
      <span style={nodeLabelStyle}>{node.id}</span>
      <div style={nodeTrackStyle}>
        <div style={{ ...nodeFillStyle, width: `${width}%`, backgroundColor: bgColor }} />
      </div>
      <span style={nodeValueStyle}>{node.ms !== null ? `${node.ms}ms` : '-'}</span>
    </div>
  );
}

export function TraceDashboard() {
  const [traces, setTraces] = useState<TraceInfo[]>([]);
  const [selectedSkill, setSelectedSkill] = useState<string | null>(null);
  const [skillTraces, setSkillTraces] = useState<TraceDetail[]>([]);
  const [stats, setStats] = useState<TraceStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    listTraces().then(setTraces).catch(console.error).finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!selectedSkill) return;
    getTracesForSkill(selectedSkill)
      .then(data => { setSkillTraces(data); setStats(computeStats(data)); })
      .catch(console.error);
  }, [selectedSkill]);

  if (loading) return <div style={loadingStyle}>Loading traces...</div>;

  return (
    <div className="trace-shell">
      <div className="trace-sidebar">
        <h3 className="trace-sidebar-title">Trace History</h3>
        <div style={sidebarListStyle}>
          {traces.map(t => (
            <button
              key={t.skill}
              className={`trace-sidebar-item${selectedSkill === t.skill ? ' active' : ''}`}
              onClick={() => setSelectedSkill(t.skill)}
            >
              <div style={sidebarNameStyle}>{t.skill}</div>
              <div style={sidebarMetaStyle}>{t.trace_count} traces</div>
            </button>
          ))}
          {traces.length === 0 && <div style={emptyStyle}>No traces recorded</div>}
        </div>
      </div>

      <div className="trace-main">
        {selectedSkill && stats ? (
          <>
            <h2 style={sectionTitleStyle}>{selectedSkill}</h2>

            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-value">{stats.avgMs}ms</div>
                <div className="stat-label">Avg Duration</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">{stats.totalCalls}</div>
                <div className="stat-label">Total Calls</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">{stats.successRate}%</div>
                <div className="stat-label">Success Rate</div>
              </div>
              <div className="stat-card">
                <div className="stat-value">{stats.slowestNode ? `${stats.slowestNode.avgMs}ms` : '-'}</div>
                <div className="stat-label">Slowest: {stats.slowestNode?.id || '-'}</div>
              </div>
            </div>

            <h3 style={subTitleStyle}>Recent Executions</h3>
            <div style={traceListStyle}>
              {skillTraces.slice(0, 5).map(trace => {
                const maxMs = Math.max(...trace.nodes.map(n => n.ms || 0), 1);
                return (
                  <div key={trace.trace_id} style={traceCardStyle}>
                    <div style={traceHeaderStyle}>
                      <span style={traceTimeStyle}>{trace.started_at ? new Date(trace.started_at).toLocaleString() : '-'}</span>
                      <span style={{
                        ...traceStatusStyle,
                        background: trace.status === 'success' ? 'var(--success-bg)' : 'var(--danger-bg)',
                        color: trace.status === 'success' ? 'var(--success)' : 'var(--danger)',
                      }}>{trace.status}</span>
                    </div>
                    <div style={traceMetaStyle}>Duration: {trace.duration_ms}ms | Trace ID: {trace.trace_id.slice(0, 8)}...</div>
                    <div style={nodeListStyle}>
                      {trace.nodes.map(node => <NodeBar key={node.id} node={node} maxMs={maxMs} />)}
                    </div>
                  </div>
                );
              })}
            </div>
          </>
        ) : (
          <div style={emptyMainStyle}>Select a skill to view trace history</div>
        )}
      </div>
    </div>
  );
}

const loadingStyle: React.CSSProperties = { padding: '40px', color: 'var(--text-tertiary)', fontSize: '14px' };
const sidebarListStyle: React.CSSProperties = { display: 'flex', flexDirection: 'column', gap: '2px' };
const sidebarNameStyle: React.CSSProperties = { fontSize: '13px', fontWeight: 600 };
const sidebarMetaStyle: React.CSSProperties = { fontSize: '11px', color: 'var(--text-tertiary)', marginTop: '2px' };
const emptyStyle: React.CSSProperties = { fontSize: '13px', color: 'var(--text-tertiary)', padding: '8px 0' };
const sectionTitleStyle: React.CSSProperties = { margin: '0 0 20px', fontSize: '20px', fontWeight: 700, color: 'var(--text-primary)' };
const subTitleStyle: React.CSSProperties = { margin: '24px 0 12px', fontSize: '12px', fontWeight: 600, color: 'var(--text-tertiary)', textTransform: 'uppercase', letterSpacing: '0.5px' };
const traceListStyle: React.CSSProperties = { display: 'flex', flexDirection: 'column', gap: '12px' };
const traceCardStyle: React.CSSProperties = { background: 'var(--bg-card)', border: '1px solid var(--border)', borderRadius: 'var(--radius-lg)', padding: '16px 20px' };
const traceHeaderStyle: React.CSSProperties = { display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' };
const traceTimeStyle: React.CSSProperties = { fontSize: '13px', fontWeight: 600, color: 'var(--text-primary)' };
const traceStatusStyle: React.CSSProperties = { fontSize: '11px', fontWeight: 600, padding: '3px 10px', borderRadius: '99px' };
const traceMetaStyle: React.CSSProperties = { fontSize: '12px', color: 'var(--text-tertiary)', marginBottom: '12px', fontFamily: 'var(--font-mono)' };
const nodeListStyle: React.CSSProperties = { display: 'flex', flexDirection: 'column', gap: '4px' };
const nodeBarStyle: React.CSSProperties = { display: 'flex', alignItems: 'center', gap: '8px', padding: '4px 0' };
const nodeLabelStyle: React.CSSProperties = { fontSize: '12px', color: 'var(--text-secondary)', width: '120px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontFamily: 'var(--font-mono)', flexShrink: 0 };
const nodeTrackStyle: React.CSSProperties = { flex: 1, height: '6px', background: 'var(--bg-page)', borderRadius: '3px', overflow: 'hidden' };
const nodeFillStyle: React.CSSProperties = { height: '100%', borderRadius: '3px', transition: 'width 0.3s ease' };
const nodeValueStyle: React.CSSProperties = { fontSize: '12px', color: 'var(--text-secondary)', width: '56px', textAlign: 'right', fontFamily: 'var(--font-mono)', flexShrink: 0 };
const emptyMainStyle: React.CSSProperties = { textAlign: 'center', color: 'var(--text-tertiary)', marginTop: '80px', fontSize: '14px' };
