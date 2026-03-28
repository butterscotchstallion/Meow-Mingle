import { useRef, useState } from "react";
import { Chip } from "primereact/chip";
import type { Cat, CatPhoto, Interest } from "../api/types.gen";

// ─── Helpers ──────────────────────────────────────────────────────────────────

const ONLINE_THRESHOLD_MS = 15 * 60 * 1000;

export function isOnline(lastSeen?: string | null): boolean {
  if (!lastSeen) return false;
  return Date.now() - new Date(lastSeen).getTime() <= ONLINE_THRESHOLD_MS;
}

export function photoUrl(photo: CatPhoto): string {
  return `/images/cats/${photo.filename}`;
}

// ─── Types ────────────────────────────────────────────────────────────────────

const SWIPE_THRESHOLD = 100;
const TILT_FACTOR = 0.12;

type SwipeDirection = "left" | "right" | null;

// ─── Shared card shell ────────────────────────────────────────────────────────
//
// Renders the photo carousel, info strip, and interests.
// Used by both the swipeable and static variants.

interface CatCardShellProps {
  cat: Cat;
  /** Extra overlays rendered on top of the photo area (e.g. LIKE/NOPE stamps) */
  photoOverlay?: React.ReactNode;
  /** Extra content rendered below the info strip */
  footer?: React.ReactNode;
  /** Whether the biography should be clamped (card) or fully expanded (profile) */
  expandBio?: boolean;
}

function CatCardShell({
  cat,
  photoOverlay,
  footer,
  expandBio = false,
}: CatCardShellProps) {
  const [photoIndex, setPhotoIndex] = useState(0);
  const photos = cat.photos ?? [];
  const hasPhotos = photos.length > 0;
  const currentPhoto = hasPhotos ? photos[photoIndex] : null;
  const url = currentPhoto ? photoUrl(currentPhoto) : null;

  function prevPhoto(e: React.MouseEvent) {
    e.stopPropagation();
    setPhotoIndex((i) => Math.max(0, i - 1));
  }

  function nextPhoto(e: React.MouseEvent) {
    e.stopPropagation();
    setPhotoIndex((i) => Math.min(photos.length - 1, i + 1));
  }

  return (
    <div className="w-full h-full pb-5 rounded-3xl overflow-hidden shadow-2xl flex flex-col bg-purple-950 border border-purple-900">
      {/* Photo area */}
      <div className="relative flex-1 bg-[#12071f] overflow-hidden">
        {url ? (
          <img
            src={url}
            alt={currentPhoto?.altText ?? cat.name}
            width={currentPhoto?.width ?? undefined}
            height={currentPhoto?.height ?? undefined}
            draggable={false}
            className="w-full h-full object-cover pointer-events-none"
          />
        ) : (
          <div className="w-full h-full flex flex-col items-center justify-center gap-3 text-purple-800">
            <i className="pi pi-image text-6xl" />
            <span className="text-sm">No photos yet</span>
          </div>
        )}

        {/* Pagination dots */}
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

        {/* Tap zones for photo nav */}
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

        {/* Caller-supplied overlays (e.g. LIKE/NOPE stamps) */}
        {photoOverlay}
      </div>

      {/* Info strip */}
      <div className="p-5 flex flex-col gap-3">
        <div className="flex items-baseline gap-3">
          <h2 className="text-2xl font-bold text-purple-100">{cat.name}</h2>
          {cat.age != null && (
            <span className="text-xl text-purple-300">{cat.age}</span>
          )}
          {isOnline(cat.lastSeen) && (
            <span className="ml-auto flex items-center gap-1.5 text-xs text-emerald-400">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse inline-block" />
              Online now
            </span>
          )}
        </div>

        {cat.breedName && (
          <div className="flex items-center gap-2 text-sm text-purple-400">
            <i className="pi pi-tag text-xs" />
            <span>{cat.breedName}</span>
          </div>
        )}

        {cat.biography && (
          <p
            className={`text-sm text-purple-300 leading-relaxed ${
              expandBio ? "" : "line-clamp-3"
            }`}
          >
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

        {footer}
      </div>
    </div>
  );
}

// ─── Swipeable card ───────────────────────────────────────────────────────────

export interface SwipeCatCardProps {
  cat: Cat;
  onSwipe: (dir: "left" | "right") => void;
  isTop: boolean;
}

export function SwipeCatCard({ cat, onSwipe, isTop }: SwipeCatCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const startX = useRef(0);
  const currentX = useRef(0);
  const dragging = useRef(false);
  const [dragX, setDragX] = useState(0);
  const [committed, setCommitted] = useState<SwipeDirection>(null);

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
      <CatCardShell
        cat={cat}
        photoOverlay={
          <>
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
          </>
        }
      />
    </div>
  );
}

// ─── Static card ──────────────────────────────────────────────────────────────
//
// Non-interactive version used on profile view pages.

export interface StaticCatCardProps {
  cat: Cat;
  /** Optional action buttons rendered at the bottom of the info strip */
  footer?: React.ReactNode;
}

export function StaticCatCard({ cat, footer }: StaticCatCardProps) {
  return <CatCardShell cat={cat} expandBio footer={footer} />;
}
