import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { Card } from "primereact/card";
import { InputText } from "primereact/inputtext";
import { InputTextarea } from "primereact/inputtextarea";
import { Button } from "primereact/button";
import { Message } from "primereact/message";
import { FloatLabel } from "primereact/floatlabel";
import { ProgressSpinner } from "primereact/progressspinner";
import { UserMenu } from "../components/UserMenu";
import { catSessionProfileHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import type { Cat } from "../api/types.gen";

const MAX_PHOTOS = 6;

interface PhotoPreview {
  file: File;
  previewUrl: string;
}

export function EditProfile() {
  const setCat = useAuthStore((s) => s.setCat);

  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Form fields
  const [biography, setBiography] = useState("");
  const [avatarFilename, setAvatarFilename] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [newPhotos, setNewPhotos] = useState<PhotoPreview[]>([]);

  const fileInputRef = useRef<HTMLInputElement>(null);

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

  // Revoke object URLs on unmount to avoid memory leaks
  useEffect(() => {
    return () => {
      newPhotos.forEach((p) => URL.revokeObjectURL(p.previewUrl));
    };
  }, [newPhotos]);

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(e.target.files ?? []);
    if (!files.length) return;

    const remaining = MAX_PHOTOS - newPhotos.length;
    const accepted = files.slice(0, remaining);

    const previews: PhotoPreview[] = accepted.map((file) => ({
      file,
      previewUrl: URL.createObjectURL(file),
    }));

    setNewPhotos((prev) => [...prev, ...previews]);
    // Reset the input so the same file can be re-added after removal
    e.target.value = "";
  }

  function removePhoto(index: number) {
    setNewPhotos((prev) => {
      URL.revokeObjectURL(prev[index].previewUrl);
      return prev.filter((_, i) => i !== index);
    });
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaveError(null);
    setSaveSuccess(false);
    setSaving(true);

    try {
      const form = new FormData();
      form.append("biography", biography);
      form.append("avatar_filename", avatarFilename);
      // Convert yyyy-MM-dd to a full RFC 3339 timestamp the server can parse
      if (birthDate) {
        form.append("birth_date", new Date(birthDate).toISOString());
      }
      for (const photo of newPhotos) {
        form.append("photo", photo.file, photo.file.name);
      }

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

      // Re-fetch the updated profile and sync it into the auth store
      const { data: refreshed } = await catSessionProfileHandler();
      const updatedCat = (refreshed as unknown as { results?: Cat } | undefined)
        ?.results;
      if (updatedCat) {
        setCat(updatedCat);
      }

      // Clear staged photos after a successful save
      setNewPhotos([]);
      setSaveSuccess(true);
    } catch {
      setSaveError("An unexpected error occurred while saving.");
    } finally {
      setSaving(false);
    }
  }

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

      {/* Main */}
      <main className="flex-1 flex flex-col items-center py-10 px-4">
        <Card className="w-full max-w-lg shadow-lg">
          {/* Heading */}
          <div className="mb-8">
            <h1 className="text-2xl font-bold text-purple-100">Edit Profile</h1>
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
              {/* Avatar filename */}
              <FloatLabel className="w-full">
                <InputText
                  id="avatarFilename"
                  value={avatarFilename}
                  onChange={(e) => setAvatarFilename(e.target.value)}
                  className="w-full"
                  maxLength={255}
                />
                <label htmlFor="avatarFilename">Avatar filename</label>
              </FloatLabel>

              {/* Birth date */}
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

              {/* Biography */}
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

              {/* Photo uploads */}
              <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-purple-200">
                    Photos
                  </span>
                  <span className="text-xs text-purple-500">
                    {newPhotos.length} / {MAX_PHOTOS}
                  </span>
                </div>

                {/* Preview grid */}
                {newPhotos.length > 0 && (
                  <div className="grid grid-cols-3 gap-2">
                    {newPhotos.map((photo, i) => (
                      <div key={i} className="relative group aspect-square">
                        <img
                          src={photo.previewUrl}
                          alt={`Photo ${i + 1}`}
                          className="w-full h-full object-cover rounded-lg border border-purple-800"
                        />
                        <button
                          type="button"
                          onClick={() => removePhoto(i)}
                          className="absolute top-1 right-1 w-6 h-6 rounded-full bg-black/60 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-rose-600 cursor-pointer"
                          aria-label={`Remove photo ${i + 1}`}
                        >
                          <i className="pi pi-times text-xs" />
                        </button>
                      </div>
                    ))}
                  </div>
                )}

                {/* Add photos button */}
                {newPhotos.length < MAX_PHOTOS && (
                  <>
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
                      className="w-full"
                      onClick={() => fileInputRef.current?.click()}
                    />
                  </>
                )}
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
