import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { UserMenu } from "../components/UserMenu";
import { DebugMenu } from "../components/DebugMenu";
import { Button } from "primereact/button";
import { ProgressSpinner } from "primereact/progressspinner";
import { Message } from "primereact/message";
import { matchSuggestionsHandler } from "../api/sdk.gen";
import { SwipeCatCard } from "../components/CatCard";
import type { Cat } from "../api/types.gen";

export function Matches() {
  const [suggestions, setSuggestions] = useState<Cat[]>([]);
  const [index, setIndex] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const { data, error: apiError } = await matchSuggestionsHandler();
      if (apiError || !data) {
        setError("Could not load match suggestions. Please try again.");
        return;
      }
      setSuggestions(data.results as Cat[]);
      setIndex(0);
    } catch {
      setError("An unexpected error occurred.");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  function handleSwipe(_dir: "left" | "right") {
    setIndex((i) => i + 1);
  }

  const remaining = suggestions.slice(index);
  const hasCurrent = remaining.length > 0;

  return (
    <div className="flex flex-col min-h-screen bg-[#12071f]">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-purple-950">
        <Link
          to="/matches"
          className="text-xl font-bold text-purple-100 hover:text-purple-500 transition-colors no-underline"
        >
          🐱 Meow Mingle
        </Link>
        <div className="flex items-center gap-2">
          <DebugMenu />
          <UserMenu />
        </div>
      </header>

      {/* Main */}
      <main className="flex-1 flex flex-col items-center justify-center px-4 py-6 gap-6">
        {loading && (
          <div className="flex flex-col items-center gap-4">
            <ProgressSpinner style={{ width: 56, height: 56 }} />
            <p className="text-purple-400 text-sm">Finding matches…</p>
          </div>
        )}

        {!loading && error && (
          <>
            <Message
              severity="error"
              text={error}
              className="w-full max-w-sm"
            />
            <p>
              <Button label="Try again" onClick={() => load()} />
            </p>
          </>
        )}

        {!loading && !error && !hasCurrent && (
          <div className="flex flex-col items-center gap-4 text-center">
            <span className="text-7xl">😿</span>
            <h2 className="text-2xl font-bold text-purple-100">
              No more cats!
            </h2>
            <p className="text-purple-400 text-sm max-w-xs">
              You've seen everyone for now. Check back later for new matches.
            </p>
          </div>
        )}

        {!loading && !error && hasCurrent && (
          <div className="relative w-full max-w-sm" style={{ height: 560 }}>
            {remaining.slice(0, 3).map((cat, stackIndex) => {
              const isTop = stackIndex === 0;
              const scale = 1 - stackIndex * 0.04;
              const translateY = stackIndex * 12;
              return (
                <div
                  key={cat.id}
                  className="absolute inset-0"
                  style={{
                    transform: isTop
                      ? undefined
                      : `scale(${scale}) translateY(${translateY}px)`,
                    zIndex: 10 - stackIndex,
                    transformOrigin: "bottom center",
                  }}
                >
                  <SwipeCatCard cat={cat} onSwipe={handleSwipe} isTop={isTop} />
                </div>
              );
            })}
          </div>
        )}
      </main>
    </div>
  );
}
