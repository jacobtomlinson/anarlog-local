import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  beginSharedUploadOperation: vi.fn(),
  cancelSharedUploadOperation: vi.fn(),
  prepareSharedUpload: vi.fn(),
  downloadSharedAttachment: vi.fn(),
}));

vi.mock("@anlg/plugin-attachment-sync", () => ({
  commands: mocks,
}));

import { attachmentTransferNative } from "./native";

describe("native shared attachment operations", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.beginSharedUploadOperation.mockResolvedValue({
      status: "ok",
      data: null,
    });
    mocks.cancelSharedUploadOperation.mockResolvedValue({
      status: "ok",
      data: true,
    });
  });

  it("cancels a shared upload when its signal aborts", async () => {
    const controller = new AbortController();
    let finish:
      | ((value: { status: "error"; error: string }) => void)
      | undefined;
    mocks.prepareSharedUpload.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );

    const upload = attachmentTransferNative.prepareSharedUpload(
      "attachment-1",
      "a".repeat(64),
      42,
      "diagram.png",
      "image/png",
      "owner/share/object.sna1",
      controller.signal,
    );
    await vi.waitFor(() =>
      expect(mocks.prepareSharedUpload).toHaveBeenCalled(),
    );

    const operationId = mocks.beginSharedUploadOperation.mock.calls[0]?.[0];
    expect(mocks.prepareSharedUpload).toHaveBeenCalledWith(
      operationId,
      "attachment-1",
      {
        sha256: "a".repeat(64),
        sizeBytes: 42,
        filename: "diagram.png",
        contentType: "image/png",
        remoteObjectKey: "owner/share/object.sna1",
      },
    );

    controller.abort();
    await vi.waitFor(() =>
      expect(mocks.cancelSharedUploadOperation).toHaveBeenCalledWith(
        operationId,
      ),
    );
    finish?.({ status: "error", error: "shared attachment was cancelled" });
    await expect(upload).rejects.toThrow("cancelled");
  });

  it("registers shared downloads with their cache scope", async () => {
    mocks.downloadSharedAttachment.mockResolvedValueOnce({
      status: "ok",
      data: {
        cacheId: "cache-1",
        localPath: "/cache/file.bin",
        sizeBytes: 42,
        sha256: "b".repeat(64),
      },
    });

    await expect(
      attachmentTransferNative.downloadSharedAttachment({
        scopeId: "viewer-1",
        attachmentId: "22222222-2222-4222-8222-222222222222",
        signedUrl: "https://project.supabase.co/shared?token=one",
        expectedSha256: "b".repeat(64),
        expectedSizeBytes: 42,
      }),
    ).resolves.toMatchObject({ cacheId: "cache-1" });

    const operationId = mocks.beginSharedUploadOperation.mock.calls[0]?.[0];
    expect(mocks.beginSharedUploadOperation).toHaveBeenCalledWith(operationId);
    expect(mocks.downloadSharedAttachment).toHaveBeenCalledWith(
      operationId,
      "viewer-1",
      "22222222-2222-4222-8222-222222222222",
      "https://project.supabase.co/shared?token=one",
      "b".repeat(64),
      42,
    );
    expect(mocks.cancelSharedUploadOperation).toHaveBeenCalledWith(operationId);
  });
});
