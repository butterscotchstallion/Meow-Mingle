import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ProgressSpinner } from "primereact/progressspinner";
import { Message } from "primereact/message";
import { Button } from "primereact/button";
import { Chip } from "primereact/chip";
import { AppLayout } from "../components/AppLayout";
import { matchesListHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import type { Match } from "../api/types.gen";
import { MatchStatus } from "../api/types.gen";

function statusLabel(status: MatchStatus | null | undefined): string {
  switch (status) {
    case MatchStatus.ACCEPTED:
      return "Accepted";
    case MatchStatus.PENDING:
      return "Pending";
    case MatchStatus.DECLINED:
      return "Declined";
    default:
      return "Unknown";
  }
}

function statusSeverity(
  status: MatchStatus | null | undefined,
): "success" | "warn" | "danger" | "secondary" {
  switch (status) {
    case MatchStatus.ACCEPTED:
      return "success";
    case MatchStatus.PENDING:
      return "warn";
    case MatchStatus.DECLINED:
      return "danger";
    default:
      return "secondary";
  }
}

export function MyMatches() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);

  const [matches, setMatches] = useState<Match[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Auth guard — redirect to sign in if not signed in
  useEffect(() => {
    if (!cat) {
      navigate("/signin", { replace: true });
    }
  }, [cat, navigate]);

  useEffect(() => {
    if (!cat) return;

    const controller = new AbortController();

    async function fetchMatches() {
      setLoading(true);
      setError(null);
      try {
        const { data, error: apiError } = await matchesListHandler({
          signal: controller.signal,
        });
        if (controller.signal.aborted) return;
        if (apiError || !data) {
          setError("Could not load your matches. Please try again.");
          return;
        }
        setMatches(data.results);
      } catch {
        if (controller.signal.aborted) return;
        setError("An unexpected error occurred.");
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    }

    fetchMatches();
    return () => controller.abort();
  }, [cat]);

  if (!cat) return null;

  return (
    <AppLayout mainClassName="flex-1 flex flex-col items-center py-10 px-4">
      <div className="w-full max-w-lg flex flex-col gap-6">
        <h1 className="text-2xl font-bold text-purple-100">My Matches</h1>

        {loading && (
          <div className="flex flex-col items-center gap-4 py-16">
            <ProgressSpinner style={{ width: 48, height: 48 }} />
            <p className="text-purple-400 text-sm">Loading your matches…</p>
          </div>
        )}

        {!loading && error && (
          <div className="flex flex-col items-center gap-4">
            <Message severity="error" text={error} className="w-full" />
            <Button
              label="Try again"
              onClick={() => {
                setError(null);
                setLoading(true);
              }}
            />
          </div>
        )}

        {!loading && !error && matches.length === 0 && (
          <div className="flex flex-col items-center gap-4 py-16 text-center">
            <span className="text-7xl">💔</span>
            <h2 className="text-xl font-bold text-purple-100">No matches yet</h2>
            <p className="text-purple-400 text-sm max-w-xs">
              Start swiping to find your purrfect match!
            </p>
            <Button
              label="Find Matches"
              icon="pi pi-heart"
              onClick={() => navigate("/matches")}
            />
          </div>
        )}

        {!loading && !error && matches.length > 0 && (
          <ul className="flex flex-col gap-3">
            {matches.map((match) => {
              const isInitiator = match.initiator_id === cat.id;
              const otherCatId = isInitiator
                ? match.target_id
                : match.initiator_id;
              const role = isInitiator ? "You liked them" : "They liked you";

              return (
                <li
                  key={match.id}
                  className="flex items-center justify-between gap-4 rounded-xl border border-purple-900 bg-purple-950/50 px-5 py-4 hover:border-purple-700 transition-colors"
                >
                  <div className="flex flex-col gap-1 min-w-0">
                    <button
                      type="button"
                      onClick={() => navigate(`/cats/${otherCatId}`)}
                      className="text-left text-purple-100 font-medium hover:text-purple-300 transition-colors truncate bg-transparent border-0 cursor-pointer p-0"
                    >
                      {otherCatId}
                    </button>
                    <span className="text-xs text-purple-500">{role}</span>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    <Chip
                      label={statusLabel(match.status)}
                      className={`text-xs p-chip-${statusSeverity(match.status)}`}
                    />
                    {match.seen === false && (
                      <span className="w-2 h-2 rounded-full bg-purple-300 shrink-0" />
                    )}
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </AppLayout>
  );
}
