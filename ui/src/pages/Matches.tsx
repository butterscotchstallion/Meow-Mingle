import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { UserMenu } from "../components/UserMenu";
import { Button } from "primereact/button";
import { Chip } from "primereact/chip";
import { ProgressSpinner } from "primereact/progressspinner";
import { Message } from "primereact/message";
import { matchSuggestionsHandler } from "../api/sdk.gen";
import type { Cat, Interest } from "../api/types.gen";

interface CatPhoto {
  id: string;
  order?: number | null;
  createdAt?: string | null;
  filename: string;
  width?: number | null;
  height?: number | null;
  altText?: string | null;
}

interface CatWithPhotos extends Cat {
  photos: CatPhoto[];
  lastSeen?: string | null;
}

const ONLINE_THRESHOLD_MS = 15 * 60 * 1000;

function isOnline(lastSeen?: string | null): boolean {
  if (!lastSeen) return false;
  return Date.now() - new Date(lastSeen).getTime() <= ONLINE_THRESHOLD_MS;
}

// ─── Swipe card ──────────────────────────────────────────────────────────────

const SWIPE_THRESHOLD = 100; // px needed to commit a swipe
const TILT_FACTOR = 0.12; // deg per px of drag

type SwipeDirection = "left" | "right" | null;

interface SwipeCardProps {
  cat: CatWithPhotos;
  onSwipe: (dir: "left" | "right") => void;
  /** Whether this card is on top of the stack */
  isTop: boolean;
}

function SwipeCard({ cat, onSwipe, isTop }: SwipeCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const startX = useRef(0);
  const currentX = useRef(0);
  const dragging = useRef(false);
  const [dragX, setDragX] = useState(0);
  const [committed, setCommitted] = useState<SwipeDirection>(null);
  const [photoIndex, setPhotoIndex] = useState(0);

  const photos: CatPhoto[] = cat.photos ?? [];
  const hasPhotos = photos.length > 0;

  // Pointer-based drag (works for both mouse and touch)
  function onPointerDown(e: React.PointerEvent) {
    if (!isTop) return;
    dragging.current = true;
    startX.current = e.clientX;
    currentX.current = e.clientX;
    cardRef.current?.setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: React.PointerEvent) {
    if (!dragging.current) return;
    currentX.current = e.clientX;
    setDragX(currentX.current - startX.current);
  }

  function onPointerUp() {
    if (!dragging.current) return;
    dragging.current = false;
    const delta = currentX.current - startX.current;
    if (Math.abs(delta) >= SWIPE_THRESHOLD) {
      const dir = delta > 0 ? "right" : "left";
      setCommitted(dir);
      setTimeout(() => onSwipe(dir), 350);
    } else {
      setDragX(0);
    }
  }

  const rotation = committed
    ? committed === "right"
      ? 30
      : -30
    : dragX * TILT_FACTOR;

  const translateX = committed
    ? committed === "right"
      ? "120vw"
      : "-120vw"
    : `${dragX}px`;

  const overlayOpacity = Math.min(Math.abs(dragX) / SWIPE_THRESHOLD, 1);
  const showLike = dragX > 20 || committed === "right";
  const showNope = dragX < -20 || committed === "left";

  function prevPhoto(e: React.MouseEvent) {
    e.stopPropagation();
    setPhotoIndex((i) => Math.max(0, i - 1));
  }
  function nextPhoto(e: React.MouseEvent) {
    e.stopPropagation();
    setPhotoIndex((i) => Math.min(photos.length - 1, i + 1));
  }

  const photoUrl = hasPhotos
    ? `http://localhost:3000/photos/${photos[photoIndex].filename}`
    : null;

  return (
    <div
      ref={cardRef}
      className="absolute inset-0 select-none cursor-grab active:cursor-grabbing"
      style={{
        transform: `translateX(${translateX}) rotate(${rotation}deg)`,
        transition: dragging.current ? "none" : "transform 0.35s ease",
        touchAction: "none",
      }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
    >
      {/* Card shell */}
      <div className="w-full h-full rounded-3xl overflow-hidden shadow-2xl flex flex-col bg-slate-800 border border-slate-700">
        {/* Photo area */}
        <div className="relative flex-1 bg-slate-900 overflow-hidden">
          {photoUrl ? (
            <img
              src={photoUrl}
              alt={photos[photoIndex].altText ?? cat.name}
              width={photos[photoIndex].width ?? undefined}
              height={photos[photoIndex].height ?? undefined}
              draggable={false}
              className="w-full h-full object-cover pointer-events-none"
            />
          ) : (
            <div className="w-full h-full flex flex-col items-center justify-center gap-3 text-slate-600">
              <i className="pi pi-image text-6xl" />
              <span className="text-sm">No photos yet</span>
            </div>
          )}

          {/* Photo pagination dots */}
          {photos.length > 1 && (
            <div className="absolute top-3 inset-x-3 flex gap-1">
              {photos.map((_, i) => (
                <div
                  key={i}
                  className="flex-1 h-1 rounded-full"
                  style={{
                    background:
                      i === photoIndex
                        ? "rgba(255,255,255,0.95)"
                        : "rgba(255,255,255,0.35)",
                  }}
                />
              ))}
            </div>
          )}

          {/* Photo nav tap zones */}
          {photos.length > 1 && (
            <>
              <button
                className="absolute left-0 top-0 h-full w-1/3 opacity-0"
                onClick={prevPhoto}
                aria-label="Previous photo"
              />
              <button
                className="absolute right-0 top-0 h-full w-1/3 opacity-0"
                onClick={nextPhoto}
                aria-label="Next photo"
              />
            </>
          )}

          {/* LIKE / NOPE overlays */}
          <div
            className="absolute top-8 left-6 border-4 border-emerald-400 rounded-lg px-3 py-1 rotate-[-20deg]"
            style={{
              opacity: showLike ? overlayOpacity : 0,
              transition: "opacity 0.1s",
            }}
          >
            <span className="text-emerald-400 font-black text-3xl tracking-widest">
              LIKE
            </span>
          </div>
          <div
            className="absolute top-8 right-6 border-4 border-rose-400 rounded-lg px-3 py-1 rotate-20"
            style={{
              opacity: showNope ? overlayOpacity : 0,
              transition: "opacity 0.1s",
            }}
          >
            <span className="text-rose-400 font-black text-3xl tracking-widest">
              NOPE
            </span>
          </div>
        </div>

        {/* Info area */}
        <div className="p-5 flex flex-col gap-3">
          <div className="flex items-baseline gap-3">
            <h2 className="text-2xl font-bold text-slate-100">{cat.name}</h2>
            {cat.age != null && (
              <span className="text-xl text-slate-300">{cat.age}</span>
            )}
            {isOnline((cat as CatWithPhotos).lastSeen) && (
              <span className="ml-auto flex items-center gap-1.5 text-xs text-emerald-400">
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse inline-block" />
                Online now
              </span>
            )}
          </div>

          {cat.breedName && (
            <div className="flex items-center gap-2 text-sm text-slate-400">
              <i className="pi pi-tag text-xs" />
              <span>{cat.breedName}</span>
            </div>
          )}

          {cat.biography && (
            <p className="text-sm text-slate-300 line-clamp-3 leading-relaxed">
              {cat.biography}
            </p>
          )}

          {cat.interests.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {cat.interests.map((interest: Interest) => (
                <Chip
                  key={interest.id}
                  label={interest.name}
                  className="text-xs"
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export function Matches() {
  const [suggestions, setSuggestions] = useState<CatWithPhotos[]>([]);
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
      setSuggestions(data.results as CatWithPhotos[]);
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

  function handleButton(dir: "left" | "right") {
    setIndex((i) => i + 1);
    // visual-only: the button press just advances without animation
    // a real implementation could trigger the card animation imperatively
    void dir;
  }

  const remaining = suggestions.slice(index);
  const hasCurrent = remaining.length > 0;

  return (
    <div className="flex flex-col min-h-screen bg-slate-900">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-slate-800">
        <Link
          to="/matches"
          className="text-xl font-bold text-slate-100 hover:text-purple-400 transition-colors no-underline"
        >
          🐱 Meow Mingle
        </Link>
        <UserMenu />
      </header>

      {/* Main */}
      <main className="flex-1 flex flex-col items-center justify-center px-4 py-6 gap-6">
        {loading && (
          <div className="flex flex-col items-center gap-4">
            <ProgressSpinner style={{ width: 56, height: 56 }} />
            <p className="text-slate-400 text-sm">Finding matches…</p>
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
            <h2 className="text-2xl font-bold text-slate-100">No more cats!</h2>
            <p className="text-slate-400 text-sm max-w-xs">
              You've seen everyone for now. Check back later for new matches.
            </p>
          </div>
        )}

        {!loading && !error && hasCurrent && (
          <>
            {/* Card stack — render top 3 for depth effect */}
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
                    <SwipeCard cat={cat} onSwipe={handleSwipe} isTop={isTop} />
                  </div>
                );
              })}
            </div>

            {/* Action buttons
            <div className="flex items-center gap-8">
              <Button
                rounded
                aria-label="Pass"
                onClick={() => handleButton("left")}
                className="!w-16 !h-16 !p-0 !bg-slate-800 !border-slate-600 hover:!bg-rose-950 hover:!border-rose-500 transition-colors"
                icon={
                  <i
                    className="pi pi-times"
                    style={{ fontSize: "1.5rem", color: "#f87171" }}
                  />
                }
              />
              <Button
                rounded
                aria-label="Like"
                onClick={() => handleButton("right")}
                className="!w-16 !h-16 !p-0 !bg-slate-800 !border-slate-600 hover:!bg-emerald-950 hover:!border-emerald-500 transition-colors"
                icon={
                  <i
                    className="pi pi-heart-fill"
                    style={{ fontSize: "1.5rem", color: "#34d399" }}
                  />
                }
              />
            </div>
            */}
          </>
        )}
      </main>
    </div>
  );
}
