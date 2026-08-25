// @ts-nocheck

// Generated from the shared-note attachment commands in src/lib.rs.

export type Result<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

export const commands = {
  async beginSharedUploadOperation(
    operationId: string,
  ): Promise<Result<null, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|begin_shared_upload_operation",
          { operationId },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async cancelSharedUploadOperation(
    operationId: string,
  ): Promise<Result<boolean, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|cancel_shared_upload_operation",
          { operationId },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async prepareSharedUpload(
    operationId: string,
    attachmentId: string,
    expected: SharedUploadVersion,
  ): Promise<Result<PreparedSharedUpload, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|prepare_shared_upload",
          { operationId, attachmentId, expected },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async readSharedUploadRange(
    attachmentId: string,
    cacheId: string,
    expected: SharedUploadVersion,
    start: number,
    end: number,
  ): Promise<Result<number[], string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|read_shared_upload_range",
          { attachmentId, cacheId, expected, start, end },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async validateSharedUpload(
    operationId: string,
    attachmentId: string,
    cacheId: string,
    expected: SharedUploadVersion,
  ): Promise<Result<boolean, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|validate_shared_upload",
          { operationId, attachmentId, cacheId, expected },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async cleanupSharedUpload(cacheId: string): Promise<Result<boolean, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|cleanup_shared_upload",
          { cacheId },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async downloadSharedAttachment(
    operationId: string,
    scopeId: string,
    attachmentId: string,
    signedUrl: string,
    expectedSha256: string,
    expectedSizeBytes: number,
  ): Promise<Result<SharedAttachmentCacheResult, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|download_shared_attachment",
          {
            operationId,
            scopeId,
            attachmentId,
            signedUrl,
            expectedSha256,
            expectedSizeBytes,
          },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async sharedAttachmentPath(
    scopeId: string,
    attachmentId: string,
  ): Promise<Result<string | null, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|shared_attachment_path",
          { scopeId, attachmentId },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async removeSharedAttachment(
    scopeId: string,
    attachmentId: string,
  ): Promise<Result<boolean, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|remove_shared_attachment",
          { scopeId, attachmentId },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async clearSharedAttachmentScope(
    scopeId: string,
  ): Promise<Result<number, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|clear_shared_attachment_scope",
          { scopeId },
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
  async clearSharedAttachmentPreviewScopes(): Promise<Result<boolean, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE(
          "plugin:attachment-sync|clear_shared_attachment_preview_scopes",
        ),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      return { status: "error", error: e as any };
    }
  },
};

export type PreparedSharedUpload = {
  cacheId: string;
  sha256: string;
  sizeBytes: number;
};
export type SharedAttachmentCacheResult = {
  cacheId: string;
  localPath: string;
  sizeBytes: number;
  sha256: string;
};
export type SharedUploadVersion = {
  sha256: string;
  sizeBytes: number;
  filename: string;
  contentType: string;
  remoteObjectKey: string;
};

import { invoke as TAURI_INVOKE } from "@tauri-apps/api/core";
