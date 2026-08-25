import {
  commands as attachmentCommands,
  type SharedAttachmentCacheResult,
} from "@anlg/plugin-attachment-sync";

export type { SharedAttachmentCacheResult };

export const attachmentTransferNative = {
  prepareSharedUpload(
    attachmentId: string,
    expectedSha256: string,
    expectedSizeBytes: number,
    expectedFilename: string,
    expectedContentType: string,
    expectedRemoteObjectKey: string,
    signal?: AbortSignal,
  ) {
    return runCancellableNative(
      signal,
      "prepare shared attachment upload",
      {
        label: "shared attachment upload operation",
        begin: (operationId) =>
          attachmentCommands.beginSharedUploadOperation(operationId),
        cancel: (operationId) =>
          attachmentCommands.cancelSharedUploadOperation(operationId),
      },
      (operationId) =>
        attachmentCommands.prepareSharedUpload(operationId, attachmentId, {
          sha256: expectedSha256,
          sizeBytes: expectedSizeBytes,
          filename: expectedFilename,
          contentType: expectedContentType,
          remoteObjectKey: expectedRemoteObjectKey,
        }),
    );
  },
  async readSharedUploadRange(
    attachmentId: string,
    cacheId: string,
    expectedSha256: string,
    expectedSizeBytes: number,
    expectedFilename: string,
    expectedContentType: string,
    expectedRemoteObjectKey: string,
    start: number,
    end: number,
  ) {
    const bytes = await unwrapNative(
      attachmentCommands.readSharedUploadRange(
        attachmentId,
        cacheId,
        {
          sha256: expectedSha256,
          sizeBytes: expectedSizeBytes,
          filename: expectedFilename,
          contentType: expectedContentType,
          remoteObjectKey: expectedRemoteObjectKey,
        },
        start,
        end,
      ),
      "read shared attachment upload snapshot",
    );
    return Uint8Array.from(bytes);
  },
  validateSharedUpload(
    attachmentId: string,
    cacheId: string,
    expectedSha256: string,
    expectedSizeBytes: number,
    expectedFilename: string,
    expectedContentType: string,
    expectedRemoteObjectKey: string,
    signal?: AbortSignal,
  ) {
    return runCancellableNative(
      signal,
      "validate shared attachment upload",
      {
        label: "shared attachment upload operation",
        begin: (operationId) =>
          attachmentCommands.beginSharedUploadOperation(operationId),
        cancel: (operationId) =>
          attachmentCommands.cancelSharedUploadOperation(operationId),
      },
      (operationId) =>
        attachmentCommands.validateSharedUpload(
          operationId,
          attachmentId,
          cacheId,
          {
            sha256: expectedSha256,
            sizeBytes: expectedSizeBytes,
            filename: expectedFilename,
            contentType: expectedContentType,
            remoteObjectKey: expectedRemoteObjectKey,
          },
        ),
    );
  },
  cleanupSharedUpload(cacheId: string) {
    return unwrapNative(
      attachmentCommands.cleanupSharedUpload(cacheId),
      "clean shared attachment upload snapshot",
    );
  },
  downloadSharedAttachment(
    input: {
      scopeId: string;
      attachmentId: string;
      signedUrl: string;
      expectedSha256: string;
      expectedSizeBytes: number;
    },
    signal?: AbortSignal,
  ) {
    return runCancellableDownload<SharedAttachmentCacheResult>(
      signal,
      "download shared attachment",
      (operationId) =>
        attachmentCommands.downloadSharedAttachment(
          operationId,
          input.scopeId,
          input.attachmentId,
          input.signedUrl,
          input.expectedSha256,
          input.expectedSizeBytes,
        ),
    );
  },
  sharedAttachmentPath(scopeId: string, attachmentId: string) {
    return unwrapNative(
      attachmentCommands.sharedAttachmentPath(scopeId, attachmentId),
      "resolve shared attachment cache",
    );
  },
  removeSharedAttachment(scopeId: string, attachmentId: string) {
    return unwrapNative(
      attachmentCommands.removeSharedAttachment(scopeId, attachmentId),
      "remove shared attachment cache",
    );
  },
  clearSharedAttachmentScope(scopeId: string) {
    return unwrapNative(
      attachmentCommands.clearSharedAttachmentScope(scopeId),
      "clear shared attachment cache",
    );
  },
};

async function runCancellableDownload<T>(
  signal: AbortSignal | undefined,
  label: string,
  operation: (
    operationId: string,
  ) => Promise<{ status: "ok"; data: T } | { status: "error"; error: string }>,
) {
  return runCancellableNative(
    signal,
    label,
    {
      label: "shared attachment download",
      begin: (operationId) =>
        attachmentCommands.beginSharedUploadOperation(operationId),
      cancel: (operationId) =>
        attachmentCommands.cancelSharedUploadOperation(operationId),
    },
    operation,
  );
}

async function runCancellableNative<T>(
  signal: AbortSignal | undefined,
  label: string,
  control: {
    label: string;
    begin: (
      operationId: string,
    ) => Promise<
      { status: "ok"; data: unknown } | { status: "error"; error: string }
    >;
    cancel: (
      operationId: string,
    ) => Promise<
      { status: "ok"; data: boolean } | { status: "error"; error: string }
    >;
  },
  operation: (
    operationId: string,
  ) => Promise<{ status: "ok"; data: T } | { status: "error"; error: string }>,
) {
  throwIfAborted(signal);
  const operationId = crypto.randomUUID();
  let begun = false;
  let abortRequested = false;
  let cancellation: Promise<boolean> | undefined;
  const cancel = () => {
    cancellation ??= unwrapNative(
      control.cancel(operationId),
      `cancel ${control.label}`,
    ).catch(() => false);
    return cancellation;
  };
  const abort = () => {
    abortRequested = true;
    if (begun) void cancel();
  };
  signal?.addEventListener("abort", abort, { once: true });

  try {
    await unwrapNative(control.begin(operationId), `begin ${control.label}`);
    begun = true;
    if (abortRequested || signal?.aborted) {
      await cancel();
      throwAbort(signal);
    }
    return await unwrapNative(operation(operationId), label);
  } finally {
    signal?.removeEventListener("abort", abort);
    if (begun) await cancel();
  }
}

function throwIfAborted(signal?: AbortSignal) {
  if (signal?.aborted) throwAbort(signal);
}

function throwAbort(signal?: AbortSignal): never {
  if (signal?.reason) throw signal.reason;
  const error = new Error("Shared attachment operation aborted");
  error.name = "AbortError";
  throw error;
}

async function unwrapNative<T>(
  operation: Promise<
    { status: "ok"; data: T } | { status: "error"; error: string }
  >,
  label: string,
): Promise<T> {
  const result = await operation;
  if (result.status === "error") {
    throw new NativeAttachmentTransferError(label, result.error);
  }
  return result.data;
}

export class NativeAttachmentTransferError extends Error {
  constructor(
    label: string,
    readonly nativeMessage: string,
  ) {
    super(`${label} failed: ${nativeMessage}`);
    this.name = "NativeAttachmentTransferError";
  }
}
