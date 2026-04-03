import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ProgressSpinner } from "primereact/progressspinner";
import { Message } from "primereact/message";
import { Button } from "primereact/button";
import { AppLayout } from "../components/AppLayout";
import { matchesListHandler, catDetailHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import { photoUrl } from "../components/CatCard";
import type { Match, Cat, CatDetailResponse } from "../api/types.gen";

interface MatchWithCat {
  match: Match;
  otherCat: Cat;
  isInitiator: boolean;
}

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

export function MyMatches() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);

  const [matchesWithCats, setMatchesWithCats] = useState<MatchWithCat[]>([]);
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
        const { data, error: apiError } = await matchesListHandler({
          signal: controller.signal,
        });
        if (controller.signal.aborted) return;
        if (apiError || !data) {
          setError("Could not load your matches. Please try again.");
          return;
        }

        const matches = data.results;

        // Fetch all other-cat profiles in parallel
        const resolved = await Promise.all(
          matches.map(async (match) => {
            const isInitiator = match.initiator_id === cat!.id;
            const otherId = isInitiator ? match.target_id : match.initiator_id;
            const otherCat = await fetchCatById(otherId);
            if (!otherCat) return null;
            return { match, otherCat, isInitiator } satisfies MatchWithCat;
          }),
        );

        if (controller.signal.aborted) return;
        setMatchesWithCats(resolved.filter((r) => r !== null));
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

        {!loading && !error && matchesWithCats.length === 0 && (
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

        {!loading && !error && matchesWithCats.length > 0 && (
          <ul className="flex flex-col gap-3">
            {matchesWithCats.map(({ match, otherCat, isInitiator }) => {
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
                    onClick={() => navigate(`/cats/${otherCat.id}`)}
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
        )}
      </div>
    </AppLayout>
  );
}
