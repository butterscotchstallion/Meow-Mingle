import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Cat } from "../api/types.gen";

export interface AuthState {
  cat: Cat | null;
  sessionId: string | null;
  setAuth: (cat: Cat, sessionId: string) => void;
  setCat: (cat: Cat) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      cat: null,
      sessionId: null,
      setAuth: (cat, sessionId) => set({ cat, sessionId }),
      setCat: (cat) => set({ cat }),
      clearAuth: () => set({ cat: null, sessionId: null }),
    }),
    {
      name: "meow-mingle-auth",
    },
  ),
);
