import { t } from "@lingui/core/macro";
import { Trans } from "@lingui/react/macro";
import { CircleNotch, Globe, LockKey } from "@phosphor-icons/react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from "@anlg/ui/components/ui/select";

export type GeneralAccessTarget = "restricted" | "link";
export type GeneralAccessValue = GeneralAccessTarget | "public";

export function GeneralAccessSelector({
  value,
  disabled,
  canExpand,
  pending,
  onValueChange,
}: {
  value: GeneralAccessValue;
  disabled: boolean;
  canExpand: boolean;
  pending: boolean;
  onValueChange: (value: GeneralAccessTarget) => void;
}) {
  const AccessIcon = value === "restricted" ? LockKey : Globe;

  return (
    <div className="flex items-center gap-2 rounded-lg px-1.5 py-1">
      <span className="bg-muted flex size-7 shrink-0 items-center justify-center rounded-md">
        {pending ? (
          <CircleNotch className="size-3.5 animate-spin" aria-hidden="true" />
        ) : (
          <AccessIcon className="size-3.5" aria-hidden="true" />
        )}
      </span>
      <Select
        value={value}
        disabled={disabled || pending}
        onValueChange={(nextValue) => {
          const target = resolveGeneralAccessTarget(nextValue);
          if (target) onValueChange(target);
        }}
      >
        <SelectTrigger
          aria-label={t`General access`}
          className="h-8 min-w-0 flex-1 rounded-md border-0 bg-transparent px-2 text-xs shadow-none"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent align="end">
          <SelectItem value="restricted">
            <Trans>Only people invited</Trans>
          </SelectItem>
          <SelectItem value="link" disabled={!canExpand}>
            <Trans>Anyone with the link</Trans>
          </SelectItem>
          {value === "public" ? (
            <>
              <SelectSeparator />
              <SelectItem value="public">
                <Trans>Public on the web</Trans>
              </SelectItem>
            </>
          ) : null}
        </SelectContent>
      </Select>
    </div>
  );
}

export function resolveGeneralAccessTarget(
  value: string,
): GeneralAccessTarget | null {
  return value === "restricted" || value === "link" ? value : null;
}
