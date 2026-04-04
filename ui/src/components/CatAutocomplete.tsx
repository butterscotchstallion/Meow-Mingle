import { useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  AutoComplete,
  AutoCompleteCompleteEvent,
} from "primereact/autocomplete";
import { catAutocompleteHandler } from "../api/sdk.gen";
import { useAuthStore } from "../store/authStore";
import type { Cat } from "../api/types.gen";

// ─── Shared search logic ──────────────────────────────────────────────────────

async function fetchSuggestions(q: string): Promise<Cat[]> {
  if (!q.trim()) return [];
  try {
    const { data } = await catAutocompleteHandler({ query: { q: q.trim() } });
    return data?.results ?? [];
  } catch {
    return [];
  }
}

// ─── CatAutocompleteField ─────────────────────────────────────────────────────
//
// Controlled cat-picker for use inside forms. The caller owns the selected Cat
// state; this component calls onSelect / onClear to report changes.

export interface CatAutocompleteFieldProps {
  inputId?: string;
  placeholder?: string;
  value: Cat | null;
  onSelect: (cat: Cat) => void;
  onClear: () => void;
  className?: string;
}

export function CatAutocompleteField({
  inputId,
  placeholder = "Search cats…",
  value,
  onSelect,
  onClear,
  className,
}: CatAutocompleteFieldProps) {
  const [inputText, setInputText] = useState("");
  const [suggestions, setSuggestions] = useState<Cat[]>([]);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  function search(event: AutoCompleteCompleteEvent) {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(async () => {
      setSuggestions(await fetchSuggestions(event.query));
    }, 300);
  }

  return (
    <AutoComplete
      inputId={inputId}
      // When a cat is selected show its name; otherwise show the raw typed text
      value={value ? value.name : inputText}
      suggestions={suggestions}
      completeMethod={search}
      field="name"
      placeholder={placeholder}
      forceSelection
      onChange={(e) => {
        if (typeof e.value === "string") {
          setInputText(e.value);
          // User cleared the field by deleting text
          if (e.value === "") onClear();
        }
      }}
      onSelect={(e) => {
        const selected = e.value as Cat;
        setInputText(selected.name);
        onSelect(selected);
      }}
      onClear={() => {
        setInputText("");
        onClear();
      }}
      className={className}
    />
  );
}

// ─── CatAutocomplete ──────────────────────────────────────────────────────────
//
// Header search widget. Admin-only, navigates to the selected cat's profile.

export function CatAutocomplete() {
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);
  const roles = useAuthStore((s) => s.roles);
  const [selected, setSelected] = useState<Cat | null>(null);

  const isAdmin = cat !== null && roles.some((r) => r.slug === "cat-admin");
  if (!isAdmin) return null;

  return (
    <CatAutocompleteField
      placeholder="Search cats…"
      value={selected}
      onSelect={(c) => {
        setSelected(null);
        navigate(`/cats/${c.id}`);
      }}
      onClear={() => setSelected(null)}
      className="[&_input]:text-sm! [&_input]:py-1.5! [&_input]:px-3!"
    />
  );
}
