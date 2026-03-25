import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Card } from "primereact/card";
import { InputText } from "primereact/inputtext";
import { InputTextarea } from "primereact/inputtextarea";
import { Button } from "primereact/button";
import { Message } from "primereact/message";
import { FloatLabel } from "primereact/floatlabel";
import { ProgressSpinner } from "primereact/progressspinner";
import { UserMenu } from "../components/UserMenu";
import {
  catSessionProfileHandler,
  catUpdateProfileHandler,
} from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import type { Cat, Interest, CatPhoto } from "../api/types.gen";

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

  // Read-only data kept for the PUT payload
  const [interests, setInterests] = useState<Interest[]>([]);
  const [photos, setPhotos] = useState<CatPhoto[]>([]);

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
        // birthDate comes as an ISO string; slice to yyyy-MM-dd for the date input
        setBirthDate(cat.birthDate ? cat.birthDate.slice(0, 10) : "");
        setInterests(cat.interests ?? []);
        setPhotos(cat.photos ?? []);
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

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault();
    setSaveError(null);
    setSaveSuccess(false);
    setSaving(true);

    try {
      const { data, error: apiError } = await catUpdateProfileHandler({
        body: {
          biography,
          avatarFilename,
          birthDate,
          interests,
          photos,
        },
      });

      if (apiError || !data) {
        setSaveError("Could not save your profile. Please try again.");
        return;
      }

      // Re-fetch the updated profile and sync it into the auth store
      const { data: refreshed } = await catSessionProfileHandler();
      const updatedCat = (refreshed as unknown as { results?: Cat } | undefined)
        ?.results;
      if (updatedCat) {
        setCat(updatedCat);
      }

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
          <div className="mb-6">
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
            <form onSubmit={handleSubmit} className="flex flex-col gap-6">
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
