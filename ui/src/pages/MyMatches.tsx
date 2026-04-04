import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ProgressSpinner } from "primereact/progressspinner";
import { Message } from "primereact/message";
import { Button } from "primereact/button";
import { AppLayout } from "../components/AppLayout";
import {
  matchesListHandler,
  catDetailHandler,
  matchMarkSeenHandler,
} from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import { photoUrl } from "../components/CatCard";
import type { Match, Cat, CatDetailResponse } from "../api/types.gen";

async function markMatchSeen(matchId: string): Promise<void> {
  await matchMarkSeenHandler({ path: { match_id: matchId } });
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/** Returns a human-readable relative time string for an ISO 8601 date. */
function relativeTime(iso: string): string {
  const now = Date.now();
  const then = new Date(iso).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) return "just now";

  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin} minute${diffMin === 1 ? "" : "s"} ago`;

  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr} hour${diffHr === 1 ? "" : "s"} ago`;

  const diffDay = Math.floor(diffHr / 24);
  if (diffDay < 7) return `${diffDay} day${diffDay === 1 ? "" : "s"} ago`;

  const diffWeek = Math.floor(diffDay / 7);
  if (diffWeek < 5) return `${diffWeek} week${diffWeek === 1 ? "" : "s"} ago`;

  const diffMonth = Math.floor(diffDay / 30);
  if (diffMonth < 12)
    return `${diffMonth} month${diffMonth === 1 ? "" : "s"} ago`;

  const diffYear = Math.floor(diffDay / 365);
  return `${diffYear} year${diffYear === 1 ? "" : "s"} ago`;
}

/**
 * Returns a full human-readable date string suitable for a tooltip,
 * e.g. "Thursday, 3 April 2025 at 14:32".
 */
function humanReadableDate(iso: string): string {
  const d = new Date(iso);
  const datePart = d.toLocaleDateString(undefined, {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
  const timePart = d.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  return `${datePart} at ${timePart}`;
}

// ---------------------------------------------------------------------------
// Types & data helpers
// ---------------------------------------------------------------------------

interface MatchWithCat {
  match: Match;
  otherCat: Cat;
  isInitiator: boolean;
}

// ---------------------------------------------------------------------------
// MatchList sub-component
// ---------------------------------------------------------------------------

interface MatchListProps {
  matches: MatchWithCat[];
  onMatchSeen?: (matchId: string) => void;
  navigate: (path: string) => void;
}

function MatchList({ matches, onMatchSeen, navigate }: MatchListProps) {
  return (
    <ul className="flex flex-col gap-3">
      {matches.map(({ match, otherCat, isInitiator }) => {
        const sortedPhotos = [...(otherCat.photos ?? [])].sort(
          (a, b) => (a.order ?? 0) - (b.order ?? 0),
        );
        const firstPhoto = sortedPhotos[0] ?? null;
        const thumbUrl = firstPhoto ? photoUrl(firstPhoto) : null;

        const meta: string[] = [];
        if (otherCat.age != null) meta.push(`${otherCat.age} yrs`);
        if (otherCat.breedName) meta.push(otherCat.breedName);

        return (
          <li key={match.id}>
            <button
              type="button"
              onClick={async () => {
                if (match.seen === false && isInitiator && onMatchSeen) {
                  await markMatchSeen(match.id);
                  onMatchSeen(match.id);
                }
                navigate(`/cats/${otherCat.id}`);
              }}
              className="w-full flex items-center gap-4 rounded-xl border border-purple-900 bg-purple-950/50 px-5 py-4 hover:border-purple-700 transition-colors text-left cursor-pointer"
            >
              {/* Circular thumbnail */}
              <div className="shrink-0 w-14 h-14 rounded-full overflow-hidden border-2 border-purple-800 bg-purple-900 flex items-center justify-center">
                {thumbUrl ? (
                  <img
                    src={thumbUrl}
                    alt={otherCat.name}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <i className="pi pi-user text-purple-600 text-xl" />
                )}
              </div>

              {/* Details */}
              <div className="flex flex-col gap-0.5 min-w-0">
                <span className="text-purple-100 font-medium truncate">
                  {otherCat.name}
                </span>
                {meta.length > 0 && (
                  <span className="text-xs text-purple-400 truncate">
                    {meta.join(" · ")}
                  </span>
                )}
                {match.createdAt && (
                  <span
                    title={humanReadableDate(match.createdAt)}
                    className="text-xs text-purple-700 cursor-default"
                  >
                    {relativeTime(match.createdAt)}
                  </span>
                )}
              </div>

              {/* Unseen indicator */}
              {match.seen === false && (
                <span className="ml-auto shrink-0 w-2 h-2 rounded-full bg-purple-300" />
              )}
            </button>
          </li>
        );
      })}
    </ul>
  );
}

// ---------------------------------------------------------------------------
// Data helpers
// ---------------------------------------------------------------------------

async function fetchCatById(id: string): Promise<Cat | null> {
  try {
    const { data, error } = await catDetailHandler({ path: { id } });
    if (error || !data) return null;
    const response = data as unknown as CatDetailResponse;
    return response.results ?? null;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function MyMatches() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);
  const setHasUnseenMatches = useAuthStore((s) => s.setHasUnseenMatches);

  const [unseenMatches, setUnseenMatches] = useState<MatchWithCat[]>([]);
  const [seenMatches, setSeenMatches] = useState<MatchWithCat[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
        // Fetch unseen and seen matches in parallel
        const [unseenData, seenData] = await Promise.all([
          matchesListHandler({
            query: { seen: false },
            signal: controller.signal,
          }),
          matchesListHandler({
            query: { seen: true },
            signal: controller.signal,
          }),
        ]);

        if (controller.signal.aborted) return;

        if (
          unseenData.error ||
          !unseenData.data ||
          seenData.error ||
          !seenData.data
        ) {
          setError("Could not load your matches. Please try again.");
          return;
        }

        async function resolveMatches(
          matches: Match[],
        ): Promise<MatchWithCat[]> {
          const resolved = await Promise.all(
            matches.map(async (match) => {
              const isInitiator = match.initiator_id === cat!.id;
              const otherId = isInitiator
                ? match.target_id
                : match.initiator_id;
              const otherCat = await fetchCatById(otherId);
              if (!otherCat) return null;
              return { match, otherCat, isInitiator } satisfies MatchWithCat;
            }),
          );
          return resolved.filter((r) => r !== null);
        }

        const [resolvedUnseen, resolvedSeen] = await Promise.all([
          resolveMatches(unseenData.data.results),
          resolveMatches(seenData.data.results),
        ]);

        if (controller.signal.aborted) return;
        setUnseenMatches(resolvedUnseen);
        setSeenMatches(resolvedSeen);
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

        {!loading &&
          !error &&
          unseenMatches.length === 0 &&
          seenMatches.length === 0 && (
            <div className="flex flex-col items-center gap-4 py-16 text-center">
              <span className="text-7xl">💔</span>
              <h2 className="text-xl font-bold text-purple-100">
                No matches yet
              </h2>
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

        {!loading &&
          !error &&
          (unseenMatches.length > 0 || seenMatches.length > 0) && (
            <div className="flex flex-col gap-8">
              {unseenMatches.length > 0 && (
                <section className="flex flex-col gap-3">
                  <h2 className="text-sm font-semibold text-purple-400 uppercase tracking-wider">
                    New
                  </h2>
                  <MatchList
                    matches={unseenMatches}
                    onMatchSeen={(matchId) => {
                      setUnseenMatches((prev) => {
                        const moved = prev.find((m) => m.match.id === matchId);
                        const remaining = prev.filter(
                          (m) => m.match.id !== matchId,
                        );
                        if (moved) {
                          setSeenMatches((s) => [
                            { ...moved, match: { ...moved.match, seen: true } },
                            ...s,
                          ]);
                          setHasUnseenMatches(
                            remaining.some((m) => m.isInitiator),
                          );
                        }
                        return remaining;
                      });
                    }}
                    navigate={navigate}
                  />
                </section>
              )}

              {seenMatches.length > 0 && (
                <section className="flex flex-col gap-3">
                  <h2 className="text-sm font-semibold text-purple-400 uppercase tracking-wider">
                    Previous
                  </h2>
                  <MatchList matches={seenMatches} navigate={navigate} />
                </section>
              )}
            </div>
          )}
      </div>
    </AppLayout>
  );
}
