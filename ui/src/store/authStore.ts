import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Cat, Role } from "../api/types.gen";

export interface AuthState {
  cat: Cat | null;
  sessionId: string | null;
  roles: Role[];
  setAuth: (cat: Cat, sessionId: string) => void;
  setCat: (cat: Cat) => void;
  setRoles: (roles: Role[]) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      cat: null,
      sessionId: null,
      roles: [],
      setAuth: (cat, sessionId) => set({ cat, sessionId }),
      setCat: (cat) => set({ cat }),
      setRoles: (roles) => set({ roles }),
      clearAuth: () => set({ cat: null, sessionId: null, roles: [] }),
    }),
    {
      name: "meow-mingle-auth",
    },
  ),
);
