import { useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  AutoComplete,
  AutoCompleteCompleteEvent,
} from "primereact/autocomplete";
import { useAuthStore } from "../store/authStore";
import type { Cat } from "../api/types.gen";

export function CatAutocomplete() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);
  const roles = useAuthStore((s) => s.roles);
  const [value, setValue] = useState("");
  const [suggestions, setSuggestions] = useState<Cat[]>([]);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const isAdmin = cat !== null && roles.some((r) => r.slug === "cat-admin");
  if (!isAdmin) return null;

  function search(event: AutoCompleteCompleteEvent) {
    const q = event.query.trim();
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!q) {
      setSuggestions([]);
      return;
    }
    debounceRef.current = setTimeout(async () => {
      try {
        const res = await fetch(
          `/api/v1/cats/autocomplete?q=${encodeURIComponent(q)}`,
          { credentials: "include" },
        );
        if (!res.ok) return;
        const data = await res.json();
        setSuggestions(data.results ?? []);
      } catch {
        setSuggestions([]);
      }
    }, 300);
  }

  return (
    <AutoComplete
      value={value}
      suggestions={suggestions}
      completeMethod={search}
      field="name"
      placeholder="Search cats…"
      onChange={(e) =>
        setValue(typeof e.value === "string" ? e.value : (e.value?.name ?? ""))
      }
      onSelect={(e) => {
        const selected = e.value as Cat;
        setValue("");
        setSuggestions([]);
        navigate(`/cats/${selected.id}`);
      }}
      inputClassName="!bg-purple-950 !border-purple-800 !text-purple-100 placeholder:!text-purple-600 focus:!border-purple-500 !text-sm !py-1.5 !px-3"
      panelClassName="!bg-purple-950 !border-purple-800"
      pt={{
        item: { className: "!text-purple-100 hover:!bg-purple-900" },
      }}
    />
  );
}
