import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { matchesListHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";

export function MatchNotificationIcon() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);
  const hasUnseenMatches = useAuthStore((s) => s.hasUnseenMatches);
  const setHasUnseenMatches = useAuthStore((s) => s.setHasUnseenMatches);

  useEffect(() => {
    if (!cat) return;

    matchesListHandler()
      .then(({ data }) => {
        setHasUnseenMatches((data?.results?.length ?? 0) > 0);
      })
      .catch(() => {});
  }, [cat, setHasUnseenMatches]);

  if (!cat) return null;

  return (
    <button
      title={hasUnseenMatches ? "You have new matches" : "No new matches"}
      type="button"
      aria-label={hasUnseenMatches ? "You have new matches" : "No new matches"}
      onClick={() => navigate("/my-matches")}
      className="flex items-center justify-center w-10 h-10 rounded-full bg-transparent border-0 cursor-pointer transition-colors hover:bg-purple-950 focus:outline-none focus:ring-2 focus:ring-purple-500"
    >
      <i
        className="pi pi-heart-fill text-xl transition-colors"
        style={{ color: hasUnseenMatches ? "#d8b4fe" : "#4a1d96" }}
      />
    </button>
  );
}
