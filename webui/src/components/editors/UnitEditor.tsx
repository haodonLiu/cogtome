import { useState, useEffect, useCallback } from 'react';
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
      const res = await fetch('/run', {
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
    <div className="editor-shell">
      {/* Template Selection Modal */}
      {showTemplateModal && (
        <div className="modal-overlay" onClick={() => setShowTemplateModal(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h3 className="modal-title">Select Template</h3>
              <button onClick={() => setShowTemplateModal(false)} className="modal-close">×</button>
            </div>
            <div className="template-grid">
              {allTemplates.map((template) => (
                <button
                  key={template.id}
                  onClick={() => handleApplyTemplate(template)}
                  className="template-card"
                >
                  <span className="template-icon">{template.icon}</span>
                  <span className="template-name">{template.name}</span>
                  <span className="template-desc">{template.description}</span>
                  {template.category === 'custom' && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteTemplate(template.id);
                      }}
                      className="delete-template-btn"
                    >
                      Delete
                    </button>
                  )}
                </button>
              ))}
            </div>
            <div className="modal-footer">
              <button onClick={() => setShowTemplateModal(false)} className="btn-secondary">Cancel</button>
            </div>
          </div>
        </div>
      )}

      {/* Toolbar */}
      <div className="editor-toolbar">
        <button onClick={() => navigate('/units')} className="btn-secondary">← Back</button>
        <h2 className="editor-title">Unit: {name}</h2>
        <div className="editor-toolbar-spacer" />
        <button onClick={() => setShowTemplateModal(true)} className="btn-secondary">Templates</button>
        <button onClick={() => handleSaveTemplate({})} className="btn-secondary">Save Template</button>
        <button onClick={handleSave} disabled={saving} className="btn-primary">
          {saving ? 'Saving...' : 'Save'}
        </button>
      </div>

      {/* Main content */}
      <div className="editor-body">
        {/* Config panel */}
        <div className="editor-panel editor-panel--left">
          <div className="card">
            <h3 className="section-title">Configuration</h3>
            <label className="label">Name</label>
            <input value={name || ''} disabled className="input" />

            <label className="label">Command</label>
            <input
              value={config.command}
              onChange={(e) => setConfig({ ...config, command: e.target.value })}
              className="input"
              placeholder="e.g., curl, jq, cat"
            />

            <label className="label">Args (one per line)</label>
            <textarea
              value={config.args.join('\n')}
              onChange={(e) => setConfig({ ...config, args: e.target.value.split('\n').filter(Boolean) })}
              className="input textarea"
              placeholder="--flag&#10;${variable}"
            />

            <label className="label">Timeout (seconds)</label>
            <div className="slider-container">
              <input
                type="range"
                min={1}
                max={300}
                value={config.timeout}
                onChange={(e) => setConfig({ ...config, timeout: Number(e.target.value) })}
                className="slider"
              />
              <span className="slider-value">{config.timeout}s</span>
            </div>

            <label className="label">Concurrency</label>
            <input
              type="number"
              min={1}
              max={100}
              value={config.concurrency}
              onChange={(e) => setConfig({ ...config, concurrency: Number(e.target.value) })}
              className="input"
            />

            <label className="label">Description</label>
            <textarea
              value={config.description}
              onChange={(e) => setConfig({ ...config, description: e.target.value })}
              className="input textarea"
            />

            {selectedTemplate && (
              <div className="template-badge">
                Using: {selectedTemplate.icon} {selectedTemplate.name}
              </div>
            )}
          </div>
        </div>

        {/* Test panel */}
        <div className="editor-panel editor-panel--right">
          <div className="card">
            <h3 className="section-title">Test</h3>
            <label className="label">Input JSON</label>
            <textarea
              value={testInput}
              onChange={(e) => setTestInput(e.target.value)}
              className="input textarea"
            />
            <button onClick={handleTest} className="btn-primary btn-run">
              ▶ Run Test
            </button>
          </div>

          <div className="card">
            <h3 className="section-title">Output</h3>
            {testStatus && (
              <div className={`status-badge status-badge--${testStatus}`}>
                {testStatus === 'success' ? '✓ Success' : '✗ Error'}
              </div>
            )}
            {testOutput && (
              <pre className="output">{testOutput}</pre>
            )}
          </div>
        </div>
      </div>

      <ChatAssistant context={{ type: 'unit', name: name || '' }} />
    </div>
  );
}
