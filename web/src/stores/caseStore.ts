import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface CaseStore {
  caseId: string | null;
  caseName: string | null;
  setCaseId: (id: string, name: string) => void;
  clear: () => void;
}

export const useCaseStore = create<CaseStore>()(
  persist(
    (set) => ({
      caseId: null,
      caseName: null,
      setCaseId: (id, name) => set({ caseId: id, caseName: name }),
      clear: () => set({ caseId: null, caseName: null }),
    }),
    { name: 'nexus-case' }
  )
);
