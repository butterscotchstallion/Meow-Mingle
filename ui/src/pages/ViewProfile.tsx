import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "primereact/button";
import { ProgressSpinner } from "primereact/progressspinner";
import { Message } from "primereact/message";
import { AppLayout } from "../components/AppLayout";
import { StaticCatCard } from "../components/CatCard";
import { catDetailHandler } from "../api/sdk.gen";
import type { Cat, CatDetailResponse } from "../api/types.gen";

export function ViewProfile() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [cat, setCat] = useState<Cat | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) {
      setError("No cat ID provided.");
      setLoading(false);
      return;
    }

    async function fetchCat() {
      setLoading(true);
      setError(null);
      try {
        const { data, error: apiError } = await catDetailHandler({
          path: { id: id! },
        });

        if (apiError || !data) {
          setError("Could not load this profile. Please try again.");
          return;
        }

        const response = data as unknown as CatDetailResponse;
        if (!response.results) {
          setError("Cat not found.");
          return;
        }

        setCat(response.results);
      } catch {
        setError("An unexpected error occurred.");
      } finally {
        setLoading(false);
      }
    }

    fetchCat();
  }, [id]);

  return (
    <AppLayout mainClassName="flex-1 flex flex-col items-center py-10 px-4">
      {loading && (
        <div className="flex flex-col items-center gap-4 mt-16">
          <ProgressSpinner style={{ width: 56, height: 56 }} />
          <p className="text-purple-400 text-sm">Loading profile…</p>
        </div>
      )}

      {!loading && error && (
        <div className="flex flex-col items-center gap-4 w-full max-w-sm">
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

      {!loading && !error && cat && (
        <div className="w-full max-w-sm" style={{ height: 620 }}>
          <StaticCatCard cat={cat} />
        </div>
      )}
    </AppLayout>
  );
}
