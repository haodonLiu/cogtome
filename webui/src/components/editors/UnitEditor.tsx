import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { getUnit, saveUnit } from '../../api/client';
import { ChatAssistant } from './ChatAssistant';

export function UnitEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [config, setConfig] = useState({ timeout: 30, concurrency: 1, description: '' });
  const [testInput, setTestInput] = useState('{}');
  const [testOutput, setTestOutput] = useState<string | null>(null);
  const [testStatus, setTestStatus] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (name) {
      getUnit(name).then((unit) => {
        setConfig({
          timeout: unit.timeout ?? 30,
          concurrency: unit.concurrency ?? 1,
          description: unit.description ?? '',
        });
      }).catch(() => {});
    }
  }, [name]);

  const handleTest = async () => {
    if (!name) return;
    try {
      const res = await fetch('/api/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type: 'unit', name, input: JSON.parse(testInput) }),
      });
      const data = await res.json();
      if (data.result) {
        setTestOutput(JSON.stringify(data.result, null, 2));
        setTestStatus('success');
      } else if (data.error) {
        setTestOutput(JSON.stringify(data.error, null, 2));
        setTestStatus('error');
      }
    } catch (e) {
      setTestOutput(String(e));
      setTestStatus('error');
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', padding: 24 }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, marginBottom: 24 }}>
        <button onClick={() => navigate(-1)} style={buttonStyle}>← Back</button>
        <h2 style={{ margin: 0, flex: 1 }}>Unit: {name}</h2>
        <button onClick={() => setSaving(true)} disabled={saving} style={buttonStyle}>
          {saving ? 'Saving...' : 'Save'}
        </button>
      </div>

      {/* Main content */}
      <div style={{ display: 'flex', flex: 1, gap: 24 }}>
        {/* Config panel */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div style={cardStyle}>
            <h3 style={{ marginTop: 0 }}>Configuration</h3>
            <label style={labelStyle}>Name</label>
            <input value={name || ''} disabled style={inputStyle} />

            <label style={labelStyle}>Timeout (seconds)</label>
            <input
              type="range"
              min={1}
              max={300}
              value={config.timeout}
              onChange={(e) => setConfig({ ...config, timeout: Number(e.target.value) })}
              style={{ width: '100%' }}
            />
            <span>{config.timeout}s</span>

            <label style={labelStyle}>Concurrency</label>
            <input
              type="number"
              min={1}
              max={100}
              value={config.concurrency}
              onChange={(e) => setConfig({ ...config, concurrency: Number(e.target.value) })}
              style={inputStyle}
            />

            <label style={labelStyle}>Description</label>
            <textarea
              value={config.description}
              onChange={(e) => setConfig({ ...config, description: e.target.value })}
              style={{ ...inputStyle, height: 100 }}
            />
          </div>
        </div>

        {/* Test panel */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div style={cardStyle}>
            <h3 style={{ marginTop: 0 }}>Test</h3>
            <label style={labelStyle}>Input JSON</label>
            <textarea
              value={testInput}
              onChange={(e) => setTestInput(e.target.value)}
              style={{ ...inputStyle, height: 120, fontFamily: 'monospace' }}
            />
            <button onClick={handleTest} style={{ ...buttonStyle, marginTop: 8 }}>
              ▶ Run Test
            </button>
          </div>

          <div style={cardStyle}>
            <h3 style={{ marginTop: 0 }}>Output</h3>
            {testStatus && (
              <div style={{
                padding: '4px 8px',
                borderRadius: 4,
                background: testStatus === 'success' ? '#22c55e20' : '#ef444420',
                color: testStatus === 'success' ? '#22c55e' : '#ef4444',
                marginBottom: 8,
                fontSize: 13,
              }}>
                {testStatus === 'success' ? '✓ Success' : '✗ Error'}
              </div>
            )}
            {testOutput && (
              <pre style={{ background: '#0f0f1a', padding: 12, borderRadius: 4, overflow: 'auto', fontSize: 12 }}>
                {testOutput}
              </pre>
            )}
          </div>
        </div>
      </div>

      <ChatAssistant context={{ type: 'unit', name: name || '' }} />
    </div>
  );
}

const cardStyle: React.CSSProperties = {
  background: '#1a1a2e',
  border: '1px solid #3b3b5c',
  borderRadius: 8,
  padding: 16,
};

const buttonStyle: React.CSSProperties = {
  background: '#7c3aed',
  color: '#fff',
  border: 'none',
  padding: '8px 16px',
  borderRadius: 4,
  cursor: 'pointer',
};

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: '#0f0f1a',
  border: '1px solid #3b3b5c',
  borderRadius: 4,
  padding: '8px',
  color: '#e2e8f0',
  marginBottom: 8,
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: '#94a3b8',
  fontSize: 13,
  marginBottom: 4,
  marginTop: 8,
};