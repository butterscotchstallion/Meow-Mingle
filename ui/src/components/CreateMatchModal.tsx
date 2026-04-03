import { useRef, useState } from "react";
import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { Dropdown } from "primereact/dropdown";
import {
  AutoComplete,
  AutoCompleteCompleteEvent,
} from "primereact/autocomplete";
import { Message } from "primereact/message";
import { matchAddUpdateHandler } from "../api/sdk.gen";
import { MatchStatus } from "../api/types.gen";
import type { Cat } from "../api/types.gen";

interface Props {
  visible: boolean;
  onHide: () => void;
}

interface StatusOption {
  label: string;
  value: MatchStatus;
}

const STATUS_OPTIONS: StatusOption[] = [
  { label: "Pending", value: MatchStatus.PENDING },
  { label: "Accepted", value: MatchStatus.ACCEPTED },
  { label: "Declined", value: MatchStatus.DECLINED },
];

const AUTOCOMPLETE_INPUT_CLASS =
  "!bg-purple-950 !border-purple-800 !text-purple-100 placeholder:!text-purple-600 focus:!border-purple-500 !text-sm !py-1.5 !px-3 w-full";

const AUTOCOMPLETE_PT = {
  item: { className: "!text-purple-100 hover:!bg-purple-900" },
};

async function searchCats(q: string): Promise<Cat[]> {
  if (!q.trim()) return [];
  try {
    const res = await fetch(
      `/api/v1/cats/autocomplete?q=${encodeURIComponent(q.trim())}`,
      { credentials: "include" },
    );
    if (!res.ok) return [];
    const data = await res.json();
    return data.results ?? [];
  } catch {
    return [];
  }
}

export function CreateMatchModal({ visible, onHide }: Props) {
  const [initiator, setInitiator] = useState<Cat | null>(null);
  const [initiatorInput, setInitiatorInput] = useState("");
  const [initiatorSuggestions, setInitiatorSuggestions] = useState<Cat[]>([]);

  const [target, setTarget] = useState<Cat | null>(null);
  const [targetInput, setTargetInput] = useState("");
  const [targetSuggestions, setTargetSuggestions] = useState<Cat[]>([]);

  const [status, setStatus] = useState<MatchStatus>(MatchStatus.PENDING);

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const initiatorDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  const targetDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  function resetForm() {
    setInitiator(null);
    setInitiatorInput("");
    setInitiatorSuggestions([]);
    setTarget(null);
    setTargetInput("");
    setTargetSuggestions([]);
    setStatus(MatchStatus.PENDING);
    setError(null);
    setSuccess(false);
  }

  function handleHide() {
    resetForm();
    onHide();
  }

  function searchInitiator(e: AutoCompleteCompleteEvent) {
    if (initiatorDebounce.current) clearTimeout(initiatorDebounce.current);
    initiatorDebounce.current = setTimeout(async () => {
      setInitiatorSuggestions(await searchCats(e.query));
    }, 300);
  }

  function searchTarget(e: AutoCompleteCompleteEvent) {
    if (targetDebounce.current) clearTimeout(targetDebounce.current);
    targetDebounce.current = setTimeout(async () => {
      setTargetSuggestions(await searchCats(e.query));
    }, 300);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);

    if (!initiator) {
      setError("Please select an initiator.");
      return;
    }
    if (!target) {
      setError("Please select a target.");
      return;
    }
    if (initiator.id === target.id) {
      setError("Initiator and target must be different cats.");
      return;
    }

    setSubmitting(true);
    try {
      const { data, error: apiError } = await matchAddUpdateHandler({
        body: {
          initiator_id: initiator.id,
          target_id: target.id,
          status,
        },
      });

      if (apiError || !data) {
        setError("Failed to create match. Please try again.");
        return;
      }

      setSuccess(true);
      setTimeout(() => handleHide(), 1200);
    } catch {
      setError("An unexpected error occurred.");
    } finally {
      setSubmitting(false);
    }
  }

  const footer = (
    <div className="flex justify-end gap-2 pt-2">
      <Button
        label="Cancel"
        icon="pi pi-times"
        severity="secondary"
        text
        onClick={handleHide}
        disabled={submitting}
      />
      <Button
        label={submitting ? "Creating…" : "Create Match"}
        icon={submitting ? "pi pi-spin pi-spinner" : "pi pi-heart"}
        onClick={handleSubmit}
        disabled={submitting || !initiator || !target}
      />
    </div>
  );

  return (
    <Dialog
      header="Create Match"
      visible={visible}
      onHide={handleHide}
      footer={footer}
      style={{ width: "min(480px, 95vw)" }}
      modal
      draggable={false}
      resizable={false}
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-5 pt-2">
        {/* Initiator */}
        <div className="flex flex-col gap-1.5">
          <label
            htmlFor="match-initiator"
            className="text-sm font-medium text-purple-200"
          >
            Initiator <span className="text-red-400">*</span>
          </label>
          <AutoComplete
            inputId="match-initiator"
            value={initiator ?? initiatorInput}
            suggestions={initiatorSuggestions}
            completeMethod={searchInitiator}
            field="name"
            placeholder="Search cats…"
            forceSelection
            onChange={(e) => {
              if (typeof e.value === "string") {
                setInitiatorInput(e.value);
                if (e.value === "") setInitiator(null);
              } else {
                setInitiatorInput(e.value?.name ?? "");
              }
            }}
            onSelect={(e) => {
              setInitiator(e.value as Cat);
              setInitiatorInput((e.value as Cat).name);
            }}
            onClear={() => {
              setInitiator(null);
              setInitiatorInput("");
            }}
            inputClassName={AUTOCOMPLETE_INPUT_CLASS}
            panelClassName="!bg-purple-950 !border-purple-800"
            pt={AUTOCOMPLETE_PT}
            className="w-full"
          />
        </div>

        {/* Target */}
        <div className="flex flex-col gap-1.5">
          <label
            htmlFor="match-target"
            className="text-sm font-medium text-purple-200"
          >
            Target <span className="text-red-400">*</span>
          </label>
          <AutoComplete
            inputId="match-target"
            value={target ?? targetInput}
            suggestions={targetSuggestions}
            completeMethod={searchTarget}
            field="name"
            placeholder="Search cats…"
            forceSelection
            onChange={(e) => {
              if (typeof e.value === "string") {
                setTargetInput(e.value);
                if (e.value === "") setTarget(null);
              } else {
                setTargetInput(e.value?.name ?? "");
              }
            }}
            onSelect={(e) => {
              setTarget(e.value as Cat);
              setTargetInput((e.value as Cat).name);
            }}
            onClear={() => {
              setTarget(null);
              setTargetInput("");
            }}
            inputClassName={AUTOCOMPLETE_INPUT_CLASS}
            panelClassName="!bg-purple-950 !border-purple-800"
            pt={AUTOCOMPLETE_PT}
            className="w-full"
          />
        </div>

        {/* Status */}
        <div className="flex flex-col gap-1.5">
          <label
            htmlFor="match-status"
            className="text-sm font-medium text-purple-200"
          >
            Status
          </label>
          <Dropdown
            inputId="match-status"
            value={status}
            options={STATUS_OPTIONS}
            onChange={(e) => setStatus(e.value as MatchStatus)}
            optionLabel="label"
            className="w-full"
          />
        </div>

        {error && <Message severity="error" text={error} className="w-full" />}
        {success && (
          <Message
            severity="success"
            text="Match created!"
            className="w-full"
          />
        )}
      </form>
    </Dialog>
  );
}
