import { create } from 'zustand';

export interface NexusSSEEvent {
  type: string;
  case_id: string;
  payload: Record<string, unknown>;
  source_worker: string;
  timestamp: string;
}

interface EventStore {
  events: NexusSSEEvent[];
  connected: boolean;
  addEvent: (event: NexusSSEEvent) => void;
  setConnected: (v: boolean) => void;
  clearEvents: () => void;
}

export const useEventStore = create<EventStore>((set) => ({
  events: [],
  connected: false,
  addEvent: (event) =>
    set((state) => ({
      events: [event, ...state.events].slice(0, 200),
    })),
  setConnected: (v) => set({ connected: v }),
  clearEvents: () => set({ events: [] }),
}));
