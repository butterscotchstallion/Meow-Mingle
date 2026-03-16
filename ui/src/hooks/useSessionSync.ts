import { useEffect, useRef } from "react";
import { useLocation } from "react-router-dom";
import { sessionGetFromCookieHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";

/**
 * Fires once per route navigation (pathname change).
 * If a sessionId is present in the store, it hits /api/v1/session/{id} and
 * writes the refreshed Cat back into the store. If the session is gone or
 * invalid the existing clearAuth-on-401 interceptor in main.tsx handles the
 * redirect, so this hook does nothing extra on failure.
 */
export function useSessionSync() {
  const location = useLocation();
  const sessionId = useAuthStore((s) => s.sessionId);
  const setCat = useAuthStore((s) => s.setCat);
  const clearAuth = useAuthStore((s) => s.clearAuth);

  // Track the last pathname we already synced so a re-render of App that
  // doesn't change the route doesn't fire a second request.
  const lastSyncedPath = useRef<string | null>(null);

  useEffect(() => {
    if (lastSyncedPath.current === location.pathname) return;
    lastSyncedPath.current = location.pathname;

    if (!sessionId) return;

    sessionGetFromCookieHandler().then(({ data }) => {
      if (data?.status === "OK" && data.results) {
        setCat(data.results);
        console.info(`Updated cat from session: `, data.results);
      }
    });
    // Intentionally no error handling here — the 401 interceptor in main.tsx
    // already calls clearAuth() and redirects on HTTP 401 responses.
  }, [location.pathname, sessionId, setCat, clearAuth]);
}
