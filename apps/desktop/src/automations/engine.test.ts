import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  exportMeetingMarkdown: vi.fn(),
  getStoredSettingValues: vi.fn(),
  setSettingValue: vi.fn(),
}));

vi.mock("@anlg/plugin-local-api", () => ({
  commands: { exportMeetingMarkdown: mocks.exportMeetingMarkdown },
}));

vi.mock("~/settings/queries", () => ({
  getStoredSettingValues: mocks.getStoredSettingValues,
  setSettingValue: mocks.setSettingValue,
}));

import { runMeetingCompletedAutomations } from "./engine";

function storedSettings(values: Record<string, unknown>) {
  mocks.getStoredSettingValues.mockResolvedValue({
    values,
    hasValues: new Set(Object.keys(values)),
  });
}

function recordedRun() {
  const calls = mocks.setSettingValue.mock.calls.filter(
    (entry) => entry[0] === "automation_markdown_export_last_run",
  );
  const call = calls[calls.length - 1];
  return call ? JSON.parse(call[1] as string) : null;
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.setSettingValue.mockResolvedValue(undefined);
});

describe("runMeetingCompletedAutomations", () => {
  it("does nothing while Markdown export is disabled", async () => {
    storedSettings({
      automation_markdown_export_enabled: false,
      automation_markdown_export_directory: "/exports",
    });

    await runMeetingCompletedAutomations("session-1");

    expect(mocks.exportMeetingMarkdown).not.toHaveBeenCalled();
    expect(mocks.setSettingValue).not.toHaveBeenCalled();
  });

  it("exports the meeting and records a successful run", async () => {
    storedSettings({
      automation_markdown_export_enabled: true,
      automation_markdown_export_directory: "/exports",
    });
    mocks.exportMeetingMarkdown.mockResolvedValue({
      status: "ok",
      data: "/exports/2026-08-07 Standup [abc123].md",
    });

    await runMeetingCompletedAutomations("session-1");

    expect(mocks.exportMeetingMarkdown).toHaveBeenCalledWith(
      "session-1",
      "/exports",
    );
    expect(recordedRun()).toMatchObject({
      status: "success",
      detail: "/exports/2026-08-07 Standup [abc123].md",
    });
  });

  it("records a failed run when the local export command errors", async () => {
    storedSettings({
      automation_markdown_export_enabled: true,
      automation_markdown_export_directory: "/exports",
    });
    mocks.exportMeetingMarkdown.mockResolvedValue({
      status: "error",
      error: "could not write markdown export: denied",
    });

    await runMeetingCompletedAutomations("session-1");

    expect(recordedRun()).toMatchObject({
      status: "error",
      detail: "could not write markdown export: denied",
    });
  });

  it("skips an enabled export without a directory", async () => {
    storedSettings({
      automation_markdown_export_enabled: true,
      automation_markdown_export_directory: "  ",
    });

    await runMeetingCompletedAutomations("session-1");

    expect(mocks.exportMeetingMarkdown).not.toHaveBeenCalled();
    expect(mocks.setSettingValue).not.toHaveBeenCalled();
  });
});
