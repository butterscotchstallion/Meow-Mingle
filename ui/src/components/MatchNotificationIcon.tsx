import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { matchesListHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";

export function MatchNotificationIcon() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);
  const [hasNewMatches, setHasNewMatches] = useState(false);

  useEffect(() => {
    if (!cat) return;

    matchesListHandler()
      .then(({ data }) => {
        setHasNewMatches((data?.results?.length ?? 0) > 0);
      })
      .catch(() => {});
  }, [cat]);

  if (!cat) return null;

  return (
    <button
      title={hasNewMatches ? "You have new matches" : "No new matches"}
      type="button"
      aria-label={hasNewMatches ? "You have new matches" : "No new matches"}
      onClick={() => navigate("/my-matches")}
      className="flex items-center justify-center w-10 h-10 rounded-full bg-transparent border-0 cursor-pointer transition-colors hover:bg-purple-950 focus:outline-none focus:ring-2 focus:ring-purple-500"
    >
      <i
        className="pi pi-heart-fill text-xl transition-colors"
        style={{ color: hasNewMatches ? "#d8b4fe" : "#4a1d96" }}
      />
    </button>
  );
}
