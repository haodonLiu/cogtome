import React, { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { getUnit, saveUnit } from '../../api/client';
import { ChatAssistant } from './ChatAssistant';

export interface UnitTemplate {
  id: string;
  name: string;
  description: string;
  category: 'http' | 'file' | 'transform' | 'ai' | 'custom';
  icon: string;
  defaultConfig: {
    timeout?: number;
    concurrency?: number;
    description?: string;
    command?: string;
    args?: string[];
  };
}

const PRESET_TEMPLATES: UnitTemplate[] = [
  {
    id: 'http-request',
    name: 'HTTP Request',
    description: 'Make HTTP requests to external APIs',
    category: 'http',
    icon: '🌐',
    defaultConfig: {
      timeout: 30,
      concurrency: 5,
      description: 'Executes an HTTP request',
      command: 'curl',
      args: ['-s', '-X', '${method}', '${url}', '-H', '${headers}', '-d', '${body}'],
    },
  },
  {
    id: 'file-read',
    name: 'File Read',
    description: 'Read content from files',
    category: 'file',
    icon: '📄',
    defaultConfig: {
      timeout: 10,
      concurrency: 3,
      description: 'Reads a file and outputs its content',
      command: 'cat',
      args: ['${file_path}'],
    },
  },
  {
    id: 'file-write',
    name: 'File Write',
    description: 'Write content to files',
    category: 'file',
    icon: '💾',
    defaultConfig: {
      timeout: 10,
      concurrency: 1,
      description: 'Writes content to a file',
      command: 'tee',
      args: ['${file_path}'],
    },
  },
  {
    id: 'json-transform',
    name: 'JSON Transform',
    description: 'Transform JSON data using jq',
    category: 'transform',
    icon: '🔄',
    defaultConfig: {
      timeout: 15,
      concurrency: 10,
      description: 'Transforms JSON using jq expressions',
      command: 'jq',
      args: ['${expression}', '-n', '${input}'],
    },
  },
  {
    id: 'data-filter',
    name: 'Data Filter',
    description: 'Filter and aggregate data arrays',
    category: 'transform',
    icon: '🔍',
    defaultConfig: {
      timeout: 20,
      concurrency: 8,
      description: 'Filters data based on conditions',
      command: 'jq',
      args: ['${filter}', '-s', '.'],
    },
  },
  {
    id: 'ai-call',
    name: 'AI Call',
    description: 'Call AI models for processing',
    category: 'ai',
    icon: '🤖',
    defaultConfig: {
      timeout: 60,
      concurrency: 2,
      description: 'Calls an AI model with input',
      command: '${AI_COMMAND}',
      args: ['--model', '${model}', '--prompt', '${prompt}'],
    },
  },
];

const STORAGE_KEY = 'cogtome_unit_templates';

function loadUserTemplates(): UnitTemplate[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function saveUserTemplates(templates: UnitTemplate[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
}

export function UnitEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [config, setConfig] = useState({ timeout: 30, concurrency: 1, description: '', command: '', args: [] as string[] });
  const [testInput, setTestInput] = useState('{}');
  const [testOutput, setTestOutput] = useState<string | null>(null);
  const [testStatus, setTestStatus] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [userTemplates, setUserTemplates] = useState<UnitTemplate[]>([]);
  const [showTemplateModal, setShowTemplateModal] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<UnitTemplate | null>(null);

  useEffect(() => {
    setUserTemplates(loadUserTemplates());

    if (name) {
      getUnit(name).then((unit) => {
        setConfig({
          timeout: unit.timeout ?? 30,
          concurrency: unit.concurrency ?? 1,
          description: unit.description ?? '',
          command: unit.command ?? '',
          args: unit.args ?? [],
        });
      }).catch(() => {});
    }
  }, [name]);

  const allTemplates = [...PRESET_TEMPLATES, ...userTemplates];

  const handleApplyTemplate = useCallback((template: UnitTemplate) => {
    setConfig(prev => ({
      ...prev,
      timeout: template.defaultConfig.timeout ?? prev.timeout,
      concurrency: template.defaultConfig.concurrency ?? prev.concurrency,
      description: template.defaultConfig.description ?? prev.description,
      command: template.defaultConfig.command ?? prev.command,
      args: template.defaultConfig.args ?? prev.args,
    }));
    setSelectedTemplate(template);
    setShowTemplateModal(false);
  }, []);

  const handleSaveTemplate = useCallback((template: Partial<UnitTemplate>) => {
    const newTemplate: UnitTemplate = {
      id: `custom-${Date.now()}`,
      name: template.name || 'Custom Template',
      description: template.description || '',
      category: 'custom',
      icon: template.icon || '⚡',
      defaultConfig: {
        timeout: config.timeout,
        concurrency: config.concurrency,
        description: config.description,
        command: config.command,
        args: config.args,
      },
    };
    const updated = [...userTemplates, newTemplate];
    setUserTemplates(updated);
    saveUserTemplates(updated);
  }, [config, userTemplates]);

  const handleDeleteTemplate = useCallback((templateId: string) => {
    const updated = userTemplates.filter(t => t.id !== templateId);
    setUserTemplates(updated);
    saveUserTemplates(updated);
  }, [userTemplates]);

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

  const handleSave = async () => {
    if (!name) return;
    setSaving(true);
    try {
      await saveUnit(name, {
        timeout: config.timeout,
        concurrency: config.concurrency,
        description: config.description,
      });
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div style={styles.container}>
      {/* Template Selection Modal */}
      {showTemplateModal && (
        <div style={styles.modalOverlay} onClick={() => setShowTemplateModal(false)}>
          <div style={styles.modal} onClick={e => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3 style={styles.modalTitle}>Select Template</h3>
              <button onClick={() => setShowTemplateModal(false)} style={styles.closeButton}>×</button>
            </div>
            <div style={styles.templateGrid}>
              {allTemplates.map((template) => (
                <button
                  key={template.id}
                  onClick={() => handleApplyTemplate(template)}
                  style={styles.templateCard}
                >
                  <span style={styles.templateIcon}>{template.icon}</span>
                  <span style={styles.templateName}>{template.name}</span>
                  <span style={styles.templateDesc}>{template.description}</span>
                  {template.category === 'custom' && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteTemplate(template.id);
                      }}
                      style={styles.deleteTemplateBtn}
                    >
                      Delete
                    </button>
                  )}
                </button>
              ))}
            </div>
            <div style={styles.modalFooter}>
              <button onClick={() => setShowTemplateModal(false)} style={styles.cancelBtn}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Header */}
      <div style={styles.header}>
        <button onClick={() => navigate('/units')} style={buttonStyle}>← Back</button>
        <h2 style={styles.title}>Unit: {name}</h2>
        <button onClick={() => setShowTemplateModal(true)} style={templateButtonStyle}>
          Templates
        </button>
        <button onClick={() => handleSaveTemplate({})} style={templateButtonStyle}>
          Save as Template
        </button>
        <button onClick={handleSave} disabled={saving} style={buttonStyle}>
          {saving ? 'Saving...' : 'Save'}
        </button>
      </div>

      {/* Main content */}
      <div style={styles.main}>
        {/* Config panel */}
        <div style={styles.leftPanel}>
          <div style={cardStyle}>
            <h3 style={styles.sectionTitle}>Configuration</h3>
            <label style={labelStyle}>Name</label>
            <input value={name || ''} disabled style={inputStyle} />

            <label style={labelStyle}>Command</label>
            <input
              value={config.command}
              onChange={(e) => setConfig({ ...config, command: e.target.value })}
              style={inputStyle}
              placeholder="e.g., curl, jq, cat"
            />

            <label style={labelStyle}>Args (one per line)</label>
            <textarea
              value={config.args.join('\n')}
              onChange={(e) => setConfig({ ...config, args: e.target.value.split('\n').filter(Boolean) })}
              style={{ ...inputStyle, height: 80, fontFamily: 'monospace' }}
              placeholder="--flag&#10;${variable}"
            />

            <label style={labelStyle}>Timeout (seconds)</label>
            <div style={sliderContainer}>
              <input
                type="range"
                min={1}
                max={300}
                value={config.timeout}
                onChange={(e) => setConfig({ ...config, timeout: Number(e.target.value) })}
                style={sliderStyle}
              />
              <span style={sliderValue}>{config.timeout}s</span>
            </div>

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
              style={{ ...inputStyle, height: 80 }}
            />

            {selectedTemplate && (
              <div style={styles.templateBadge}>
                Using: {selectedTemplate.icon} {selectedTemplate.name}
              </div>
            )}
          </div>
        </div>

        {/* Test panel */}
        <div style={styles.rightPanel}>
          <div style={cardStyle}>
            <h3 style={styles.sectionTitle}>Test</h3>
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
            <h3 style={styles.sectionTitle}>Output</h3>
            {testStatus && (
              <div style={{
                ...statusStyle,
                background: testStatus === 'success' ? '#22c55e20' : '#ef444420',
                color: testStatus === 'success' ? '#22c55e' : '#ef4444',
              }}>
                {testStatus === 'success' ? '✓ Success' : '✗ Error'}
              </div>
            )}
            {testOutput && (
              <pre style={outputStyle}>
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

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    padding: 24,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: 12,
    marginBottom: 24,
  },
  title: {
    margin: 0,
    flex: 1,
    fontSize: 20,
    fontWeight: 600,
  },
  main: {
    display: 'flex',
    flex: 1,
    gap: 24,
    overflow: 'hidden',
  },
  leftPanel: {
    flex: 1,
    overflowY: 'auto',
  },
  rightPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    gap: 16,
    overflowY: 'auto',
  },
  sectionTitle: {
    marginTop: 0,
    marginBottom: 16,
    fontSize: 16,
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  templateBadge: {
    marginTop: 16,
    padding: '8px 12px',
    background: 'var(--accent-light)',
    color: 'var(--accent)',
    borderRadius: 6,
    fontSize: 13,
  },
  modalOverlay: {
    position: 'fixed',
    inset: 0,
    background: 'rgba(0,0,0,0.7)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    background: 'var(--bg-card)',
    borderRadius: 12,
    padding: 24,
    maxWidth: 600,
    maxHeight: '80vh',
    overflow: 'auto',
    border: '1px solid var(--border)',
  },
  modalHeader: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 20,
  },
  modalTitle: {
    margin: 0,
    fontSize: 18,
    fontWeight: 600,
  },
  closeButton: {
    background: 'none',
    border: 'none',
    color: 'var(--text-secondary)',
    fontSize: 24,
    cursor: 'pointer',
  },
  templateGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: 12,
    marginBottom: 20,
  },
  templateCard: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-start',
    padding: 16,
    background: 'var(--bg-page)',
    border: '1px solid var(--border)',
    borderRadius: 8,
    cursor: 'pointer',
    textAlign: 'left',
    transition: 'var(--transition)',
    position: 'relative',
  },
  templateIcon: {
    fontSize: 24,
    marginBottom: 8,
  },
  templateName: {
    fontWeight: 600,
    fontSize: 14,
    marginBottom: 4,
  },
  templateDesc: {
    fontSize: 12,
    color: 'var(--text-secondary)',
  },
  deleteTemplateBtn: {
    position: 'absolute',
    top: 8,
    right: 8,
    background: 'none',
    border: 'none',
    color: '#ef4444',
    fontSize: 12,
    cursor: 'pointer',
    opacity: 0.7,
  },
  modalFooter: {
    display: 'flex',
    justifyContent: 'flex-end',
  },
};

const cardStyle: React.CSSProperties = {
  background: 'var(--bg-card)',
  border: '1px solid var(--border)',
  borderRadius: 8,
  padding: 16,
};

const buttonStyle: React.CSSProperties = {
  background: 'var(--accent)',
  color: '#fff',
  border: 'none',
  padding: '8px 16px',
  borderRadius: 6,
  cursor: 'pointer',
  fontSize: 14,
  fontWeight: 500,
};

const templateButtonStyle: React.CSSProperties = {
  background: 'var(--bg-page)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  padding: '8px 16px',
  borderRadius: 6,
  cursor: 'pointer',
  fontSize: 14,
  fontWeight: 500,
};

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-page)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  padding: '8px 12px',
  color: 'var(--text-primary)',
  marginBottom: 12,
  fontSize: 14,
  boxSizing: 'border-box',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: 'var(--text-secondary)',
  fontSize: 13,
  marginBottom: 6,
  marginTop: 8,
  fontWeight: 500,
};

const sliderContainer: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 12,
};

const sliderStyle: React.CSSProperties = {
  flex: 1,
};

const sliderValue: React.CSSProperties = {
  fontFamily: 'monospace',
  fontSize: 14,
  color: 'var(--text-primary)',
  minWidth: 40,
};

const statusStyle: React.CSSProperties = {
  padding: '4px 8px',
  borderRadius: 4,
  marginBottom: 8,
  fontSize: 13,
  fontWeight: 500,
};

const outputStyle: React.CSSProperties = {
  background: 'var(--bg-page)',
  padding: 12,
  borderRadius: 6,
  overflow: 'auto',
  fontSize: 12,
  fontFamily: 'monospace',
  maxHeight: 300,
};
