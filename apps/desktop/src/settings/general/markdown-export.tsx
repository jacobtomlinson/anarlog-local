import { Trans, useLingui } from "@lingui/react/macro";
import { FolderOpen } from "@phosphor-icons/react";
import { useMutation } from "@tanstack/react-query";
import { open as selectFolder } from "@tauri-apps/plugin-dialog";

import { Button } from "@anlg/ui/components/ui/button";
import { sonnerToast } from "@anlg/ui/components/ui/toast";
import { cn, formatDistanceToNow } from "@anlg/utils";

import { parseAutomationRunRecord } from "~/automations/types";
import { setSettingValue, useStoredSettingValue } from "~/settings/queries";
import { SettingRow, SettingSwitchRow } from "~/settings/setting-row";

export function MarkdownExportSettings() {
  const { t } = useLingui();
  const enabled =
    useStoredSettingValue("automation_markdown_export_enabled").value ?? false;
  const directory = (
    useStoredSettingValue("automation_markdown_export_directory").value ?? ""
  ).trim();
  const lastRun = parseAutomationRunRecord(
    useStoredSettingValue("automation_markdown_export_last_run").value,
  );

  const chooseFolderMutation = useMutation({
    mutationKey: ["markdown-export-folder"],
    mutationFn: async () => {
      const selected = await selectFolder({
        title: t`Choose export folder`,
        directory: true,
        multiple: false,
        defaultPath: directory || undefined,
      });
      if (typeof selected === "string" && selected) {
        await setSettingValue("automation_markdown_export_directory", selected);
      }
    },
    onError: () => sonnerToast.error(t`Could not update the export folder`),
  });

  const enabledMutation = useMutation({
    mutationKey: ["markdown-export-enabled"],
    mutationFn: (nextEnabled: boolean) =>
      setSettingValue("automation_markdown_export_enabled", nextEnabled),
    onError: () => sonnerToast.error(t`Could not update Markdown export`),
  });

  return (
    <section>
      <h2 className="mb-4 font-sans text-lg font-semibold">
        <Trans>Markdown export</Trans>
      </h2>
      <div className="flex flex-col gap-4">
        <SettingSwitchRow
          title={<Trans>Export meetings automatically</Trans>}
          description={
            <Trans>Save each completed meeting as a local Markdown file.</Trans>
          }
          checked={enabled}
          onChange={(nextEnabled) => enabledMutation.mutate(nextEnabled)}
          disabled={
            enabledMutation.isPending || (!enabled && directory.length === 0)
          }
        />
        <SettingRow
          title={<Trans>Export folder</Trans>}
          description={
            directory || (
              <Trans>Choose where completed meetings are saved.</Trans>
            )
          }
          controlWidth="content"
        >
          {(labelProps) => (
            <Button
              {...labelProps}
              type="button"
              size="sm"
              variant="outline"
              onClick={() => chooseFolderMutation.mutate()}
              disabled={chooseFolderMutation.isPending}
            >
              <FolderOpen size={14} />
              <Trans>Choose folder</Trans>
            </Button>
          )}
        </SettingRow>
        {lastRun ? (
          <p
            className={cn([
              "truncate text-xs",
              lastRun.status === "error"
                ? "text-destructive"
                : "text-muted-foreground",
            ])}
            title={lastRun.detail}
          >
            {lastRun.status === "success" ? (
              <Trans>
                Last exported{" "}
                {formatDistanceToNow(new Date(lastRun.at), { addSuffix: true })}
                : {lastRun.detail}
              </Trans>
            ) : (
              <Trans>
                Last export failed{" "}
                {formatDistanceToNow(new Date(lastRun.at), { addSuffix: true })}
                : {lastRun.detail}
              </Trans>
            )}
          </p>
        ) : null}
      </div>
    </section>
  );
}
