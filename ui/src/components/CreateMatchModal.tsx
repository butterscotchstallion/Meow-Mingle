import { useState } from "react";
import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { Dropdown } from "primereact/dropdown";
import { Message } from "primereact/message";
import { matchAddUpdateHandler } from "../api/sdk.gen";
import { MatchStatus } from "../api/types.gen";
import type { Cat } from "../api/types.gen";
import { CatAutocompleteField } from "./CatAutocomplete";

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

export function CreateMatchModal({ visible, onHide }: Props) {
  const [initiator, setInitiator] = useState<Cat | null>(null);
  const [target, setTarget] = useState<Cat | null>(null);
  const [status, setStatus] = useState<MatchStatus>(MatchStatus.PENDING);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  function resetForm() {
    setInitiator(null);
    setTarget(null);
    setStatus(MatchStatus.PENDING);
    setError(null);
    setSuccess(false);
  }

  function handleHide() {
    resetForm();
    onHide();
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
          <label htmlFor="match-initiator" className="text-sm font-medium">
            Initiator <span className="text-red-400">*</span>
          </label>
          <CatAutocompleteField
            inputId="match-initiator"
            placeholder="Search cats…"
            value={initiator}
            onSelect={setInitiator}
            onClear={() => setInitiator(null)}
            className="w-full"
          />
        </div>

        {/* Target */}
        <div className="flex flex-col gap-1.5">
          <label htmlFor="match-target" className="text-sm font-medium">
            Target <span className="text-red-400">*</span>
          </label>
          <CatAutocompleteField
            inputId="match-target"
            placeholder="Search cats…"
            value={target}
            onSelect={setTarget}
            onClear={() => setTarget(null)}
            className="w-full"
          />
        </div>

        {/* Status */}
        <div className="flex flex-col gap-1.5">
          <label htmlFor="match-status" className="text-sm font-medium">
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
