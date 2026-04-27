import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy, sortableKeyboardCoordinates } from '@dnd-kit/sortable';
import { useStructureStore } from '../store/structureStore';
import { MotifPalette } from './MotifPalette';
import { SelectedMotifList } from './SelectedMotifList';
import * as api from '../api/client';

export function StructureEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [showWarning, setShowWarning] = useState(false);
  const [validateResult, setValidateResult] = useState<{ valid: boolean; message: string } | null>(null);

  const {
    currentStructure,
    isDirty,
    isSaving,
    saveError,
    createNewStructure,
    selectStructure,
    updateStructure,
    saveStructure,
    deleteStructure,
    clearCurrent,
    addMotif,
    removeMotif,
    reorderMotifs,
  } = useStructureStore();

  const isNew = !name;

  useEffect(() => {
    if (name) {
      selectStructure(name);
    } else {
      createNewStructure();
    }
    return () => clearCurrent();
  }, [name, selectStructure, createNewStructure, clearCurrent]);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    const items = currentStructure?.motifs || [];
    const oldIndex = items.findIndex((_, i) => `${items[i].name}-${i}` === active.id);
    const newIndex = items.findIndex((_, i) => `${items[i].name}-${i}` === over.id);
    if (oldIndex !== -1 && newIndex !== -1) {
      reorderMotifs(oldIndex, newIndex);
    }
  }

  async function handleSave() {
    if (!isNew && isDirty) {
      setShowWarning(true);
    } else {
      await saveStructure();
      const { saveError: err, currentStructure: cs } = useStructureStore.getState();
      if (!err && isNew && cs?.name) {
        navigate(`/structures/${encodeURIComponent(cs.name)}`);
      }
    }
  }

  async function confirmSave() {
    setShowWarning(false);
    await saveStructure();
    const { saveError: err, currentStructure: cs } = useStructureStore.getState();
    if (!err && isNew && cs?.name) {
      navigate(`/structures/${encodeURIComponent(cs.name)}`);
    }
  }

  function handleDelete() {
    if (!name || !currentStructure) return;
    if (!confirm(`Delete structure "${name}"? This cannot be undone.`)) return;
    deleteStructure(name).then(() => {
      navigate('/structures');
    });
  }

  async function handleValidate() {
    if (!currentStructure?.name) return;
    if (isDirty) {
      await saveStructure();
    }
    try {
      const result = await api.validateStructure(currentStructure.name);
      setValidateResult({ valid: result.valid, message: result.errors.map(e => e.message).join('; ') || 'Structure is valid' });
    } catch (e) {
      setValidateResult({ valid: false, message: (e as Error).message });
    }
  }

  if (!currentStructure) {
    return (
      <div style={styles.loading}>
        <div style={styles.spinner} />
        <span>Loading structure...</span>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      {/* Breadcrumb + Title */}
      <div style={styles.breadcrumb}>
        <button style={styles.backLink} onClick={() => navigate('/structures')}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
          Structures
        </button>
        <span style={styles.breadcrumbSep}>/</span>
        <span style={styles.breadcrumbCurrent}>{isNew ? 'New' : currentStructure.name}</span>
      </div>

      {/* Header */}
      <div style={styles.header}>
        <h2 style={styles.title}>{isNew ? 'New Structure' : currentStructure.name}</h2>
        <div style={styles.actions}>
          {!isNew && (
            <button style={styles.ghostBtn} onClick={handleValidate}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
              Validate
            </button>
          )}
          {!isNew && (
            <button style={styles.dangerGhostBtn} onClick={handleDelete}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
              Delete
            </button>
          )}
          <button
            style={isSaving ? styles.savingBtn : styles.primaryBtn}
            onClick={handleSave}
            disabled={isSaving || !currentStructure.name}
          >
            {isSaving ? (
              <>
                <div style={styles.btnSpinner} />
                Saving...
              </>
            ) : (
              <>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>
                </svg>
                Save
              </>
            )}
          </button>
        </div>
      </div>

      {/* Alerts */}
      {validateResult && (
        <div style={validateResult.valid ? styles.successAlert : styles.errorAlert}>
          <div style={styles.alertIcon}>
            {validateResult.valid ? (
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#22c55e" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
            ) : (
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#ef4444" strokeWidth="2" strokeLinecap="round">
                <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
            )}
          </div>
          <span style={styles.alertText}>{validateResult.message}</span>
          <button style={styles.alertClose} onClick={() => setValidateResult(null)}>×</button>
        </div>
      )}

      {saveError && (
        <div style={styles.errorAlert}>
          <div style={styles.alertIcon}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#ef4444" strokeWidth="2" strokeLinecap="round">
              <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
          </div>
          <span style={styles.alertText}>{saveError}</span>
        </div>
      )}

      {showWarning && (
        <div style={styles.warningAlert}>
          <div style={styles.warningIcon}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" strokeWidth="2" strokeLinecap="round">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
          </div>
          <div style={styles.warningContent}>
            <p style={styles.warningTitle}>Overwrite existing file?</p>
            <p style={styles.warningText}>Saving will replace the current YAML file. Any comments or formatting will be lost.</p>
            <div style={styles.warningActions}>
              <button style={styles.cancelBtn} onClick={() => setShowWarning(false)}>Cancel</button>
              <button style={styles.confirmBtn} onClick={confirmSave}>Overwrite</button>
            </div>
          </div>
        </div>
      )}

      {/* Name Field */}
      <div style={styles.card}>
        <div style={styles.field}>
          <label style={styles.label}>Structure Name</label>
          <input
            style={styles.input}
            value={currentStructure.name}
            onChange={(e) => updateStructure({ name: e.target.value })}
            placeholder="my-structure"
            disabled={!isNew}
          />
          {!isNew && <span style={styles.hint}>Name cannot be changed after creation</span>}
        </div>
      </div>

      {/* Editor */}
      <div style={styles.editor}>
        <div style={styles.palettePanel}>
          <MotifPalette onSelect={addMotif} />
        </div>
        <div style={styles.selectedPanel}>
          <div style={styles.panelHeader}>
            <span style={styles.panelTitle}>Selected Motifs</span>
            <span style={styles.panelCount}>{currentStructure.motifs.length}</span>
          </div>
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
            <SortableContext
              items={currentStructure.motifs.map((m, i) => `${m.name}-${i}`)}
              strategy={verticalListSortingStrategy}
            >
              <SelectedMotifList
                motifs={currentStructure.motifs}
                onRemove={removeMotif}
                onReorder={reorderMotifs}
              />
            </SortableContext>
          </DndContext>
        </div>
      </div>

      {/* Dirty indicator */}
      {isDirty && (
        <div style={styles.dirtyBar}>
          <span style={styles.dirtyDot} />
          Unsaved changes
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    maxWidth: '1000px',
    margin: '0 auto',
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
  breadcrumb: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '20px',
    fontSize: '13px',
    color: 'var(--text-tertiary)',
  },
  backLink: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    background: 'none',
    border: 'none',
    color: 'var(--text-secondary)',
    cursor: 'pointer',
    fontSize: '13px',
    fontFamily: 'inherit',
    padding: '2px 0',
    transition: 'var(--transition)',
  },
  breadcrumbSep: {
    color: 'var(--text-tertiary)',
  },
  breadcrumbCurrent: {
    color: 'var(--text-primary)',
    fontWeight: 500,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    marginBottom: '24px',
  },
  title: {
    flex: 1,
    margin: 0,
    fontSize: '22px',
    fontWeight: 700,
    letterSpacing: '-0.3px',
  },
  actions: {
    display: 'flex',
    gap: '8px',
  },
  primaryBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
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
  savingBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    backgroundColor: 'var(--accent-hover)',
    color: '#fff',
    border: 'none',
    padding: '10px 20px',
    borderRadius: 'var(--radius-md)',
    cursor: 'not-allowed',
    fontSize: '14px',
    fontWeight: 600,
    fontFamily: 'inherit',
    opacity: 0.8,
  },
  btnSpinner: {
    width: '14px',
    height: '14px',
    border: '2px solid rgba(255,255,255,0.3)',
    borderTopColor: '#fff',
    borderRadius: '50%',
    animation: 'spin 0.8s linear infinite',
  },
  ghostBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    backgroundColor: 'transparent',
    color: 'var(--text-secondary)',
    border: '1px solid var(--border)',
    padding: '10px 16px',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 500,
    fontFamily: 'inherit',
    transition: 'var(--transition)',
  },
  dangerGhostBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    backgroundColor: 'transparent',
    color: 'var(--danger)',
    border: '1px solid #fecaca',
    padding: '10px 16px',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 500,
    fontFamily: 'inherit',
    transition: 'var(--transition)',
  },
  card: {
    backgroundColor: 'var(--bg-card)',
    borderRadius: 'var(--radius-lg)',
    padding: '24px',
    marginBottom: '20px',
    border: '1px solid var(--border)',
    boxShadow: 'var(--shadow-sm)',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  label: {
    fontSize: '13px',
    fontWeight: 600,
    color: 'var(--text-secondary)',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  input: {
    backgroundColor: 'var(--bg-page)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-md)',
    padding: '10px 14px',
    color: 'var(--text-primary)',
    fontSize: '14px',
    fontFamily: 'inherit',
    maxWidth: '320px',
    outline: 'none',
    transition: 'var(--transition)',
  },
  hint: {
    fontSize: '12px',
    color: 'var(--text-tertiary)',
  },
  editor: {
    display: 'grid',
    gridTemplateColumns: '280px 1fr',
    gap: '20px',
  },
  palettePanel: {},
  selectedPanel: {
    backgroundColor: 'var(--bg-card)',
    borderRadius: 'var(--radius-lg)',
    border: '1px solid var(--border)',
    boxShadow: 'var(--shadow-sm)',
    overflow: 'hidden',
  },
  panelHeader: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '16px 20px',
    borderBottom: '1px solid var(--border)',
    backgroundColor: 'var(--bg-page)',
  },
  panelTitle: {
    fontSize: '13px',
    fontWeight: 600,
    color: 'var(--text-secondary)',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  panelCount: {
    fontSize: '12px',
    fontWeight: 600,
    color: 'var(--text-tertiary)',
    backgroundColor: 'var(--bg-active)',
    padding: '2px 8px',
    borderRadius: '10px',
  },
  successAlert: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    backgroundColor: 'var(--success-bg)',
    border: '1px solid #bbf7d0',
    borderRadius: 'var(--radius-md)',
    padding: '12px 16px',
    marginBottom: '16px',
  },
  errorAlert: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    backgroundColor: 'var(--danger-bg)',
    border: '1px solid #fecaca',
    borderRadius: 'var(--radius-md)',
    padding: '12px 16px',
    marginBottom: '16px',
  },
  warningAlert: {
    display: 'flex',
    alignItems: 'flex-start',
    gap: '12px',
    backgroundColor: 'var(--warning-bg)',
    border: '1px solid #fde68a',
    borderRadius: 'var(--radius-md)',
    padding: '16px',
    marginBottom: '16px',
  },
  alertIcon: {
    display: 'flex',
    alignItems: 'center',
    flexShrink: 0,
  },
  alertText: {
    fontSize: '14px',
    color: 'var(--text-secondary)',
    flex: 1,
  },
  alertClose: {
    background: 'none',
    border: 'none',
    color: 'var(--text-tertiary)',
    cursor: 'pointer',
    fontSize: '18px',
    padding: '0 4px',
    lineHeight: 1,
  },
  warningIcon: {
    display: 'flex',
    alignItems: 'center',
    flexShrink: 0,
    marginTop: '2px',
  },
  warningContent: {
    flex: 1,
  },
  warningTitle: {
    margin: '0 0 4px 0',
    fontSize: '14px',
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  warningText: {
    margin: '0 0 12px 0',
    fontSize: '13px',
    color: 'var(--text-secondary)',
  },
  warningActions: {
    display: 'flex',
    gap: '8px',
  },
  cancelBtn: {
    backgroundColor: '#fff',
    color: 'var(--text-secondary)',
    border: '1px solid var(--border)',
    padding: '8px 16px',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 500,
    fontFamily: 'inherit',
    transition: 'var(--transition)',
  },
  confirmBtn: {
    backgroundColor: 'var(--warning)',
    color: '#fff',
    border: 'none',
    padding: '8px 16px',
    borderRadius: 'var(--radius-md)',
    cursor: 'pointer',
    fontSize: '13px',
    fontWeight: 600,
    fontFamily: 'inherit',
    transition: 'var(--transition)',
  },
  dirtyBar: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginTop: '20px',
    padding: '10px 16px',
    backgroundColor: 'var(--accent-light)',
    borderRadius: 'var(--radius-md)',
    fontSize: '13px',
    color: 'var(--accent)',
    fontWeight: 500,
  },
  dirtyDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    backgroundColor: 'var(--accent)',
    animation: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
  },
};
