import { useEffect, useRef, useState, useCallback } from "react";
import { Link } from "react-router-dom";
import { Card } from "primereact/card";
import { InputText } from "primereact/inputtext";
import { InputTextarea } from "primereact/inputtextarea";
import { Button } from "primereact/button";
import { Message } from "primereact/message";
import { FloatLabel } from "primereact/floatlabel";
import { ProgressSpinner } from "primereact/progressspinner";
import { Dialog } from "primereact/dialog";
import { UserMenu } from "../components/UserMenu";
import { catSessionProfileHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import type { Cat, CatPhoto } from "../api/types.gen";

const MAX_PHOTOS = 6;

interface PhotoPreview {
  file: File;
  previewUrl: string;
}

// A lightbox entry can be either a persisted photo or a staged new one
interface LightboxItem {
  src: string;
  alt: string;
}

export function EditProfile() {
  const setCat = useAuthStore((s) => s.setCat);
  const cat = useAuthStore((s) => s.cat);

  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Form fields
  const [biography, setBiography] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [existingPhotos, setExistingPhotos] = useState<CatPhoto[]>([]);
  const [newPhotos, setNewPhotos] = useState<PhotoPreview[]>([]);

  // Avatar picker
  const [avatarFilename, setAvatarFilename] = useState("");
  const [avatarPreviewUrl, setAvatarPreviewUrl] = useState<string | null>(null);
  const [avatarFile, setAvatarFile] = useState<File | null>(null);
  const avatarInputRef = useRef<HTMLInputElement>(null);

  // Photo uploads
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Lightbox
  const [lightbox, setLightbox] = useState<LightboxItem | null>(null);

  // Drag-and-drop reorder state
  const dragIndexRef = useRef<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  useEffect(() => {
    async function fetchProfile() {
      setLoading(true);
      setLoadError(null);
      try {
        const { data, error: apiError } = await catSessionProfileHandler();
        if (apiError || !data) {
          setLoadError("Could not load your profile. Please try again.");
          return;
        }
        const cat = (data as unknown as { results?: Cat }).results;
        if (!cat) {
          setLoadError("No profile found for this session.");
          return;
        }
        setBiography(cat.biography ?? "");
        setAvatarFilename(cat.avatarFilename ?? "");
        setBirthDate(cat.birthDate ? cat.birthDate.slice(0, 10) : "");
        // Sort by the order column so the grid reflects the saved order
        const sorted = [...(cat.photos ?? [])].sort(
          (a, b) => (a.order ?? 0) - (b.order ?? 0),
        );
        setExistingPhotos(sorted);
      } catch {
        setLoadError(
          "An unexpected error occurred while loading your profile.",
        );
      } finally {
        setLoading(false);
      }
    }

    fetchProfile();
  }, []);

  // Revoke blob URLs on unmount
  useEffect(() => {
    return () => {
      newPhotos.forEach((p) => URL.revokeObjectURL(p.previewUrl));
      if (avatarPreviewUrl) URL.revokeObjectURL(avatarPreviewUrl);
    };
  }, [newPhotos, avatarPreviewUrl]);

  // ── Avatar picker ──────────────────────────────────────────────────────────

  function handleAvatarChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    if (avatarPreviewUrl) URL.revokeObjectURL(avatarPreviewUrl);
    setAvatarFile(file);
    setAvatarPreviewUrl(URL.createObjectURL(file));
    setAvatarFilename(file.name);
    e.target.value = "";
  }

  function clearAvatar() {
    if (avatarPreviewUrl) URL.revokeObjectURL(avatarPreviewUrl);
    setAvatarFile(null);
    setAvatarPreviewUrl(null);
    setAvatarFilename("");
  }

  // ── Photo grid ─────────────────────────────────────────────────────────────

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(e.target.files ?? []);
    if (!files.length) return;
    const remaining = MAX_PHOTOS - existingPhotos.length - newPhotos.length;
    const accepted = files.slice(0, remaining);
    const previews: PhotoPreview[] = accepted.map((file) => ({
      file,
      previewUrl: URL.createObjectURL(file),
    }));
    setNewPhotos((prev) => [...prev, ...previews]);
    e.target.value = "";
  }

  function removeExistingPhoto(index: number) {
    setExistingPhotos((prev) => prev.filter((_, i) => i !== index));
  }

  // ── Drag-and-drop reorder ──────────────────────────────────────────────────

  const handleDragStart = useCallback((index: number) => {
    dragIndexRef.current = index;
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, index: number) => {
    e.preventDefault();
    if (dragIndexRef.current !== null && dragIndexRef.current !== index) {
      setDragOverIndex(index);
    }
  }, []);

  const handleDrop = useCallback((index: number) => {
    const from = dragIndexRef.current;
    if (from === null || from === index) {
      dragIndexRef.current = null;
      setDragOverIndex(null);
      return;
    }
    setExistingPhotos((prev) => {
      const next = [...prev];
      const [moved] = next.splice(from, 1);
      next.splice(index, 0, moved);
      return next;
    });
    dragIndexRef.current = null;
    setDragOverIndex(null);
  }, []);

  const handleDragEnd = useCallback(() => {
    dragIndexRef.current = null;
    setDragOverIndex(null);
  }, []);

  function removeNewPhoto(index: number) {
    setNewPhotos((prev) => {
      URL.revokeObjectURL(prev[index].previewUrl);
      return prev.filter((_, i) => i !== index);
    });
  }

  // ── Submit ─────────────────────────────────────────────────────────────────

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaveError(null);
    setSaveSuccess(false);
    setSaving(true);

    try {
      const form = new FormData();
      form.append("biography", biography);

      // If the user picked a new avatar file, upload it as the avatar field;
      // otherwise send back the existing filename string.
      if (avatarFile) {
        form.append("avatar", avatarFile, avatarFile.name);
      } else {
        form.append("avatar_filename", avatarFilename);
      }

      if (birthDate) {
        form.append("birth_date", new Date(birthDate).toISOString());
      }

      for (const photo of newPhotos) {
        form.append("photo", photo.file, photo.file.name);
      }

      // Send the current display order of existing photos so the backend
      // can persist it to the photos.order column
      const photoOrder = existingPhotos.map((p, i) => ({ id: p.id, order: i }));
      form.append("photo_order", JSON.stringify(photoOrder));

      const res = await fetch("/api/v1/profile", {
        method: "PUT",
        body: form,
        credentials: "include",
      });

      if (!res.ok) {
        setSaveError(
          `Could not save your profile (${res.status}). Please try again.`,
        );
        return;
      }

      // Re-fetch and sync auth store
      const { data: refreshed } = await catSessionProfileHandler();
      const updatedCat = (refreshed as unknown as { results?: Cat } | undefined)
        ?.results;
      if (updatedCat) {
        setCat(updatedCat);
        setExistingPhotos(updatedCat.photos ?? []);
        setAvatarFilename(updatedCat.avatarFilename ?? "");
      }

      // Clear staged state
      if (avatarPreviewUrl) URL.revokeObjectURL(avatarPreviewUrl);
      setAvatarFile(null);
      setAvatarPreviewUrl(null);
      setNewPhotos([]);
      setSaveSuccess(true);
    } catch {
      setSaveError("An unexpected error occurred while saving.");
    } finally {
      setSaving(false);
    }
  }

  // ── Helpers ────────────────────────────────────────────────────────────────

  const totalPhotos = existingPhotos.length + newPhotos.length;
  const atLimit = totalPhotos >= MAX_PHOTOS;

  const existingAvatarUrl = avatarFilename
    ? `/images/cats/${avatarFilename}`
    : null;

  // ── Render ─────────────────────────────────────────────────────────────────

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
        <UserMenu />
      </header>

      {/* Lightbox */}
      <Dialog
        visible={lightbox !== null}
        onHide={() => setLightbox(null)}
        dismissableMask
        closable
        header={null}
        pt={{
          root: {
            className:
              "!bg-transparent !shadow-none !border-0 !p-0 !m-0 !max-w-[90vw]",
          },
          content: { className: "!bg-transparent !p-0" },
          mask: { className: "!bg-black/80" },
        }}
        style={{ background: "transparent" }}
      >
        {lightbox && (
          <img
            src={lightbox.src}
            alt={lightbox.alt}
            className="max-h-[80vh] max-w-[85vw] object-contain rounded-xl shadow-2xl"
          />
        )}
      </Dialog>

      {/* Main */}
      <main className="flex-1 flex flex-col items-center py-10 px-4">
        <Card className="w-full max-w-lg shadow-lg">
          {/* Heading */}
          <div className="mb-8">
            <div className="flex items-center justify-between gap-4">
              <h1 className="text-2xl font-bold text-purple-100">
                Edit Profile
              </h1>
              {cat && (
                <Link to={`/cats/${cat.id}`} className="no-underline">
                  <Button
                    label="View Profile"
                    icon="pi pi-eye"
                    outlined
                    size="small"
                  />
                </Link>
              )}
            </div>
            <p className="mt-1 text-purple-400 text-sm">
              Update your public profile information.
            </p>
          </div>

          {loading && (
            <div className="flex flex-col items-center gap-4 py-10">
              <ProgressSpinner style={{ width: 48, height: 48 }} />
              <p className="text-purple-400 text-sm">Loading your profile…</p>
            </div>
          )}

          {!loading && loadError && (
            <Message severity="error" text={loadError} className="w-full" />
          )}

          {!loading && !loadError && (
            <form onSubmit={handleSubmit} className="flex flex-col gap-8">
              {/* ── Avatar picker ── */}
              <div className="flex flex-col gap-2">
                <span className="text-sm font-medium text-purple-200">
                  Avatar
                </span>
                <div className="flex items-center gap-4">
                  {/* Preview circle */}
                  <button
                    type="button"
                    onClick={() => {
                      const src = avatarPreviewUrl ?? existingAvatarUrl;
                      if (src) setLightbox({ src, alt: "Avatar" });
                    }}
                    className="shrink-0 w-20 h-20 rounded-full overflow-hidden border-2 border-purple-700 bg-purple-950 flex items-center justify-center cursor-pointer hover:border-purple-400 transition-colors focus:outline-none"
                    aria-label="Preview avatar"
                    disabled={!avatarPreviewUrl && !existingAvatarUrl}
                    style={{
                      cursor:
                        avatarPreviewUrl || existingAvatarUrl
                          ? "pointer"
                          : "default",
                    }}
                  >
                    {avatarPreviewUrl || existingAvatarUrl ? (
                      <img
                        src={avatarPreviewUrl ?? existingAvatarUrl!}
                        alt="Avatar preview"
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <i className="pi pi-user text-2xl text-purple-600" />
                    )}
                  </button>

                  <div className="flex flex-col gap-2 flex-1">
                    <input
                      ref={avatarInputRef}
                      type="file"
                      accept="image/*"
                      className="hidden"
                      onChange={handleAvatarChange}
                    />
                    <Button
                      type="button"
                      label={avatarFilename ? "Change avatar" : "Upload avatar"}
                      icon="pi pi-upload"
                      outlined
                      size="small"
                      className="w-full"
                      onClick={() => avatarInputRef.current?.click()}
                    />
                    {(avatarPreviewUrl || avatarFilename) && (
                      <Button
                        type="button"
                        label="Remove avatar"
                        icon="pi pi-trash"
                        outlined
                        severity="danger"
                        size="small"
                        className="w-full"
                        onClick={clearAvatar}
                      />
                    )}
                  </div>
                </div>
              </div>

              {/* ── Birth date ── */}
              <FloatLabel className="w-full">
                <InputText
                  id="birthDate"
                  type="date"
                  value={birthDate}
                  onChange={(e) => setBirthDate(e.target.value)}
                  className="w-full"
                />
                <label htmlFor="birthDate">Date of birth</label>
              </FloatLabel>

              {/* ── Biography ── */}
              <FloatLabel className="w-full">
                <InputTextarea
                  id="biography"
                  value={biography}
                  onChange={(e) => setBiography(e.target.value)}
                  className="w-full"
                  rows={4}
                  maxLength={500}
                  autoResize
                />
                <label htmlFor="biography">Biography</label>
              </FloatLabel>

              {/* ── Photos ── */}
              <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-purple-200">
                    Photos
                  </span>
                  <span className="text-xs text-purple-500">
                    {totalPhotos} / {MAX_PHOTOS}
                    {existingPhotos.length > 1 && (
                      <span className="ml-2 text-purple-600">
                        · drag to reorder
                      </span>
                    )}
                  </span>
                </div>

                {/* Combined grid */}
                {(existingPhotos.length > 0 || newPhotos.length > 0) && (
                  <div className="grid grid-cols-3 gap-2">
                    {/* Persisted photos — draggable to reorder */}
                    {existingPhotos.map((photo, i) => {
                      const src = `/images/cats/${photo.filename}`;
                      const alt = photo.altText ?? `Photo ${i + 1}`;
                      const isDragOver = dragOverIndex === i;
                      return (
                        <div
                          key={photo.id}
                          className={`relative group aspect-square transition-transform ${
                            isDragOver
                              ? "scale-105 ring-2 ring-purple-400 rounded-lg"
                              : ""
                          }`}
                          draggable
                          onDragStart={() => handleDragStart(i)}
                          onDragOver={(e) => handleDragOver(e, i)}
                          onDrop={() => handleDrop(i)}
                          onDragEnd={handleDragEnd}
                        >
                          <img
                            src={src}
                            alt={alt}
                            onClick={() => setLightbox({ src, alt })}
                            className="w-full h-full object-cover rounded-lg border border-purple-800 cursor-grab active:cursor-grabbing hover:brightness-90 transition-[filter]"
                            draggable={false}
                          />
                          {/* Drag handle hint */}
                          <div className="absolute bottom-1 right-1 opacity-0 group-hover:opacity-70 transition-opacity pointer-events-none">
                            <i className="pi pi-bars text-white text-xs drop-shadow" />
                          </div>
                          {/* Position badge */}
                          <span className="absolute bottom-1 left-1 text-[10px] font-semibold bg-black/50 text-white px-1.5 py-0.5 rounded pointer-events-none">
                            {i + 1}
                          </span>
                          {/* Remove button */}
                          <button
                            type="button"
                            onClick={() => removeExistingPhoto(i)}
                            className="absolute top-1 right-1 w-6 h-6 rounded-full bg-black/60 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-rose-600 cursor-pointer"
                            aria-label={`Remove photo ${i + 1}`}
                          >
                            <i className="pi pi-times text-xs" />
                          </button>
                        </div>
                      );
                    })}

                    {/* Staged new photos */}
                    {newPhotos.map((photo, i) => {
                      const alt = `New photo ${i + 1}`;
                      return (
                        <div key={i} className="relative group aspect-square">
                          <img
                            src={photo.previewUrl}
                            alt={alt}
                            onClick={() =>
                              setLightbox({ src: photo.previewUrl, alt })
                            }
                            className="w-full h-full object-cover rounded-lg border border-dashed border-purple-700 cursor-pointer hover:brightness-90 transition-[filter]"
                          />
                          {/* Remove button */}
                          <button
                            type="button"
                            onClick={() => removeNewPhoto(i)}
                            className="absolute top-1 right-1 w-6 h-6 rounded-full bg-black/60 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-rose-600 cursor-pointer"
                            aria-label={`Remove new photo ${i + 1}`}
                          >
                            <i className="pi pi-times text-xs" />
                          </button>
                          {/* New badge */}
                          <span className="absolute bottom-1 left-1 text-[10px] font-semibold bg-purple-700 text-purple-100 px-1.5 py-0.5 rounded pointer-events-none">
                            New
                          </span>
                        </div>
                      );
                    })}
                  </div>
                )}

                {/* Add photos */}
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  multiple
                  className="hidden"
                  onChange={handleFileChange}
                />
                <Button
                  type="button"
                  label="Add photos"
                  icon="pi pi-image"
                  outlined
                  disabled={atLimit}
                  className="w-full"
                  onClick={() => fileInputRef.current?.click()}
                />
              </div>

              {saveError && (
                <Message severity="error" text={saveError} className="w-full" />
              )}

              {saveSuccess && (
                <Message
                  severity="success"
                  text="Profile saved successfully!"
                  className="w-full"
                />
              )}

              <Button
                type="submit"
                label="Save Profile"
                icon="pi pi-check"
                loading={saving}
                className="w-full"
              />
            </form>
          )}
        </Card>
      </main>
    </div>
  );
}
