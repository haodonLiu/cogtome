import { useState } from 'react';
import { BlockNode, BlockEdge } from '../../types';

interface PropertyPanelProps {
  selectedNode: BlockNode | null;
  selectedEdge: BlockEdge | null;
  onUpdateNode: (id: string, data: Partial<BlockNode['data']>) => void;
  onDeleteNode: (id: string) => void;
  onDeleteEdge: (id: string) => void;
}

function EmptyState() {
  return (
    <div className="panel-empty">
      <div className="panel-empty-icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="8" y="8" width="32" height="32" rx="4" stroke="currentColor" strokeWidth="2" strokeDasharray="4 2" />
          <path d="M24 18v12M18 24h12" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </svg>
      </div>
      <p className="panel-empty-title">No Selection</p>
      <p className="panel-empty-hint">Select a node or edge to view and edit its properties</p>
    </div>
  );
}

function SectionHeader({ title }: { title: string }) {
  return (
    <div className="panel-section-header">
      <span>{title}</span>
    </div>
  );
}

function KeyValueEditor({
  value,
  onChange,
  keyPlaceholder = 'key',
  valPlaceholder = 'value',
}: {
  value: Record<string, string>;
  onChange: (v: Record<string, string>) => void;
  keyPlaceholder?: string;
  valPlaceholder?: string;
}) {
  const [entries, setEntries] = useState<Array<{ k: string; v: string }>>(
    Object.entries(value).map(([k, v]) => ({ k, v }))
  );

  const sync = (next: Array<{ k: string; v: string }>) => {
    setEntries(next);
    const obj: Record<string, string> = {};
    next.forEach(({ k, v }) => {
      if (k.trim()) obj[k.trim()] = v;
    });
    onChange(obj);
  };

  const add = () => sync([...entries, { k: '', v: '' }]);

  const remove = (i: number) => sync(entries.filter((_, idx) => idx !== i));

  const update = (i: number, field: 'k' | 'v', val: string) => {
    const next = entries.map((e, idx) => (idx === i ? { ...e, [field]: val } : e));
    sync(next);
  };

  return (
    <div className="kv-editor">
      {entries.map((entry, i) => (
        <div key={i} className="kv-row">
          <input
            className="kv-key"
            placeholder={keyPlaceholder}
            value={entry.k}
            onChange={(e) => update(i, 'k', e.target.value)}
          />
          <input
            className="kv-val"
            placeholder={valPlaceholder}
            value={entry.v}
            onChange={(e) => update(i, 'v', e.target.value)}
          />
          <button className="kv-delete" onClick={() => remove(i)} type="button">
            ×
          </button>
        </div>
      ))}
      <button className="kv-add" onClick={add} type="button">
        + Add Field
      </button>
    </div>
  );
}

export function PropertyPanel({
  selectedNode,
  selectedEdge,
  onUpdateNode,
  onDeleteNode,
  onDeleteEdge,
}: PropertyPanelProps) {
  if (selectedEdge) {
    return (
      <div className="property-panel">
        <SectionHeader title="Edge" />
        <div className="panel-content">
          <div className="panel-edge-info">
            <span className="edge-source">{selectedEdge.source}</span>
            <span className="edge-arrow">→</span>
            <span className="edge-target">{selectedEdge.target}</span>
          </div>
        </div>
        <div className="panel-footer">
          <button
            className="btn-danger"
            onClick={() => onDeleteEdge(selectedEdge.id)}
          >
            Delete Edge
          </button>
        </div>
      </div>
    );
  }

  if (!selectedNode) {
    return (
      <div className="property-panel">
        <EmptyState />
      </div>
    );
  }

  const { type, data, id } = selectedNode;

  return (
    <div className="property-panel">
      <SectionHeader title="Identity" />
      <div className="panel-content">
        <div className="panel-field">
          <label className="field-label">Type</label>
          <div className="field-value type-badge">{type}</div>
        </div>

        {(type === 'unit' || type === 'motif') && (
          <>
            <div className="panel-field">
              <label className="field-label">Name</label>
              <input
                className="field-input"
                value={data.name || ''}
                onChange={(e) => onUpdateNode(id, { name: e.target.value })}
                disabled={type === 'unit'}
              />
            </div>
          </>
        )}
      </div>

      {(type === 'if' || type === 'match') && (
        <>
          <SectionHeader title="Condition" />
          <div className="panel-content">
            <div className="panel-field">
              <textarea
                className="field-textarea mono"
                value={data.condition || ''}
                onChange={(e) => onUpdateNode(id, { condition: e.target.value })}
                placeholder="e.g., ${steps.check.value} == 'ok'"
                rows={3}
              />
            </div>
          </div>
        </>
      )}

      {type === 'foreach' && (
        <>
          <SectionHeader title="Iteration" />
          <div className="panel-content">
            <div className="panel-field">
              <label className="field-label">Over (expression)</label>
              <input
                className="field-input mono"
                value={data.over || ''}
                onChange={(e) => onUpdateNode(id, { over: e.target.value })}
                placeholder="${items}"
              />
            </div>
            <div className="panel-field">
              <label className="field-label">Max Iterations</label>
              <input
                type="number"
                className="field-input"
                value={data.maxIterations || 50}
                onChange={(e) => onUpdateNode(id, { maxIterations: Number(e.target.value) })}
              />
            </div>
          </div>
        </>
      )}

      {type === 'return' && (
        <>
          <SectionHeader title="Return Mappings" />
          <div className="panel-content">
            <KeyValueEditor
              value={data.mappings || {}}
              onChange={(v) => onUpdateNode(id, { mappings: v })}
              keyPlaceholder="output key"
              valPlaceholder="${steps...}"
            />
          </div>
        </>
      )}

      {type === 'unit' && data.inputs && (
        <>
          <SectionHeader title="Input Mapping" />
          <div className="panel-content">
            <KeyValueEditor
              value={data.inputs}
              onChange={(v) => onUpdateNode(id, { inputs: v })}
              keyPlaceholder="param"
              valPlaceholder="${...}"
            />
          </div>
        </>
      )}

      {type === 'motif' && data.inputs && (
        <>
          <SectionHeader title="Motif Inputs" />
          <div className="panel-content">
            <KeyValueEditor
              value={data.inputs}
              onChange={(v) => onUpdateNode(id, { inputs: v })}
              keyPlaceholder="input"
              valPlaceholder="${...}"
            />
          </div>
        </>
      )}

      <div className="panel-footer">
        <button
          className="btn-danger"
          onClick={() => onDeleteNode(id)}
        >
          Delete Node
        </button>
      </div>
    </div>
  );
}
