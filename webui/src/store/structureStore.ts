import { create } from 'zustand';
import type { StructureInfo, MotifInfo, StructureManifest } from '../types';
import * as api from '../api/client';

interface StructureStore {
  // Lists
  structures: StructureInfo[];
  motifs: MotifInfo[];
  isLoadingLists: boolean;
  listError: string | null;

  // Current structure being edited
  currentStructure: StructureManifest | null;
  isDirty: boolean;
  isSaving: boolean;
  saveError: string | null;

  // Actions - list loading
  loadStructures: () => Promise<void>;
  loadMotifs: () => Promise<void>;

  // Actions - editing
  createNewStructure: () => void;
  selectStructure: (name: string) => Promise<void>;
  updateStructure: (data: Partial<StructureManifest>) => void;
  saveStructure: () => Promise<void>;
  clearCurrent: () => void;

  // Motif management
  addMotif: (motifName: string) => void;
  removeMotif: (index: number) => void;
  reorderMotifs: (fromIndex: number, toIndex: number) => void;
}

export const useStructureStore = create<StructureStore>((set, get) => ({
  // Initial state
  structures: [],
  motifs: [],
  isLoadingLists: false,
  listError: null,
  currentStructure: null,
  isDirty: false,
  isSaving: false,
  saveError: null,

  loadStructures: async () => {
    set({ isLoadingLists: true, listError: null });
    try {
      const structures = await api.listStructures();
      set({ structures, isLoadingLists: false });
    } catch (e) {
      set({ listError: (e as Error).message, isLoadingLists: false });
    }
  },

  loadMotifs: async () => {
    set({ isLoadingLists: true, listError: null });
    try {
      const motifs = await api.listMotifs();
      set({ motifs, isLoadingLists: false });
    } catch (e) {
      set({ listError: (e as Error).message, isLoadingLists: false });
    }
  },

  createNewStructure: () => {
    set({
      currentStructure: {
        name: '',
        motifs: [],
      },
      isDirty: false,
      saveError: null,
    });
  },

  selectStructure: async (name: string) => {
    try {
      const manifest = await api.getStructure(name);
      set({ currentStructure: manifest, isDirty: false, saveError: null });
    } catch (e) {
      set({ saveError: (e as Error).message });
    }
  },

  updateStructure: (data: Partial<StructureManifest>) => {
    const { currentStructure } = get();
    if (!currentStructure) return;
    set({
      currentStructure: { ...currentStructure, ...data },
      isDirty: true,
    });
  },

  saveStructure: async () => {
    const { currentStructure } = get();
    if (!currentStructure || !currentStructure.name) return;

    set({ isSaving: true, saveError: null });
    try {
      await api.saveStructure(currentStructure.name, currentStructure);
      set({ isSaving: false, isDirty: false });
      // Reload structures list
      await get().loadStructures();
    } catch (e) {
      set({ saveError: (e as Error).message, isSaving: false });
    }
  },

  clearCurrent: () => {
    set({ currentStructure: null, isDirty: false, saveError: null });
  },

  addMotif: (motifName: string) => {
    const { currentStructure } = get();
    if (!currentStructure) return;
    const newMotifs = [...currentStructure.motifs, { name: motifName }];
    set({
      currentStructure: { ...currentStructure, motifs: newMotifs },
      isDirty: true,
    });
  },

  removeMotif: (index: number) => {
    const { currentStructure } = get();
    if (!currentStructure) return;
    const newMotifs = currentStructure.motifs.filter((_, i) => i !== index);
    set({
      currentStructure: { ...currentStructure, motifs: newMotifs },
      isDirty: true,
    });
  },

  reorderMotifs: (fromIndex: number, toIndex: number) => {
    const { currentStructure } = get();
    if (!currentStructure) return;
    const newMotifs = [...currentStructure.motifs];
    const [removed] = newMotifs.splice(fromIndex, 1);
    newMotifs.splice(toIndex, 0, removed);
    set({
      currentStructure: { ...currentStructure, motifs: newMotifs },
      isDirty: true,
    });
  },
}));
