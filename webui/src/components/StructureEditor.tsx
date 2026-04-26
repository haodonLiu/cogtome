import { useEffect, useState } from 'react';
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
import { sortableKeyboardCoordinates } from '@dnd-kit/sortable';
import { useStructureStore } from '../store/structureStore';
import { MotifPalette } from './MotifPalette';
import { SelectedMotifList } from './SelectedMotifList';

export function StructureEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [showWarning, setShowWarning] = useState(false);

  const {
    currentStructure,
    isDirty,
    isSaving,
    saveError,
    createNewStructure,
    selectStructure,
    updateStructure,
    saveStructure,
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

  function handleSave() {
    if (!isNew && isDirty) {
      setShowWarning(true);
    } else {
      saveStructure();
    }
  }

  function confirmSave() {
    setShowWarning(false);
    saveStructure();
  }

  if (!currentStructure) {
    return <p>Loading...</p>;
  }

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <button style={styles.backBtn} onClick={() => navigate('/structures')}>
          &larr; Back
        </button>
        <h2 style={styles.title}>{isNew ? 'New Structure' : currentStructure.name}</h2>
        <button
          style={styles.saveBtn}
          onClick={handleSave}
          disabled={isSaving || !currentStructure.name}
        >
          {isSaving ? 'Saving...' : 'Save'}
        </button>
      </div>

      {saveError && <p style={styles.error}>Error: {saveError}</p>}

      {showWarning && (
        <div style={styles.warning}>
          <p>Warning: This will overwrite the existing YAML file and remove any comments.</p>
          <div style={styles.warningButtons}>
            <button style={styles.cancelBtn} onClick={() => setShowWarning(false)}>
              Cancel
            </button>
            <button style={styles.confirmBtn} onClick={confirmSave}>
              Overwrite
            </button>
          </div>
        </div>
      )}

      <div style={styles.form}>
        <div style={styles.field}>
          <label style={styles.label}>Name</label>
          <input
            style={styles.input}
            value={currentStructure.name}
            onChange={(e) => updateStructure({ name: e.target.value })}
            placeholder="structure-name"
            disabled={!isNew}
          />
        </div>
      </div>

      <div style={styles.editor}>
        <div style={styles.palette}>
          <MotifPalette onSelect={addMotif} />
        </div>
        <div style={styles.selected}>
          <h3 style={styles.sectionTitle}>Selected Motifs (drag to reorder)</h3>
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
            <SelectedMotifList
              motifs={currentStructure.motifs}
              onRemove={removeMotif}
              onReorder={reorderMotifs}
            />
          </DndContext>
        </div>
      </div>

      {isDirty && (
        <p style={styles.dirtyHint}>
          You have unsaved changes
        </p>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    maxWidth: '1000px',
    margin: '0 auto',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    marginBottom: '1.5rem',
  },
  backBtn: {
    background: 'none',
    border: '1px solid #0f3460',
    color: '#aaa',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  title: {
    flex: 1,
    margin: 0,
    fontSize: '1.5rem',
  },
  saveBtn: {
    backgroundColor: '#e94560',
    color: '#fff',
    border: 'none',
    padding: '0.75rem 1.5rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '1rem',
  },
  error: {
    color: '#e94560',
    marginBottom: '1rem',
  },
  warning: {
    backgroundColor: '#3d2a2a',
    border: '1px solid #e94560',
    borderRadius: '4px',
    padding: '1rem',
    marginBottom: '1rem',
  },
  warningButtons: {
    display: 'flex',
    gap: '0.5rem',
    marginTop: '0.5rem',
  },
  cancelBtn: {
    backgroundColor: '#333',
    color: '#fff',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  confirmBtn: {
    backgroundColor: '#e94560',
    color: '#fff',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  form: {
    marginBottom: '1.5rem',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  label: {
    fontSize: '0.875rem',
    color: '#888',
  },
  input: {
    backgroundColor: '#16213e',
    border: '1px solid #0f3460',
    borderRadius: '4px',
    padding: '0.75rem',
    color: '#fff',
    fontSize: '1rem',
    maxWidth: '300px',
  },
  editor: {
    display: 'grid',
    gridTemplateColumns: '250px 1fr',
    gap: '1.5rem',
  },
  palette: {},
  selected: {},
  sectionTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1rem',
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
  },
  dirtyHint: {
    marginTop: '1rem',
    color: '#888',
    fontStyle: 'italic',
  },
};
