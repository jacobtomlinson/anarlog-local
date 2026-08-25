import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  findPreviousUploads: vi.fn().mockResolvedValue([]),
  instances: [] as Array<{
    options: Record<string, any>;
  }>,
}));

vi.mock("tus-js-client", () => ({
  Upload: class {
    options: Record<string, any>;
    resumeFromPreviousUpload = vi.fn();
    start = vi.fn(() => this.options.onSuccess?.({}));

    constructor(_file: unknown, options: Record<string, any>) {
      this.options = options;
      mocks.instances.push(this);
    }

    findPreviousUploads() {
      return mocks.findPreviousUploads();
    }
  },
}));

import { uploadSharedAttachment } from "./storage";

describe("shared attachment storage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.instances.length = 0;
    mocks.findPreviousUploads.mockResolvedValue([]);
  });

  it("uploads plaintext through the signed TUS endpoint", async () => {
    const objectKey = [
      "11111111-1111-4111-8111-111111111111",
      "22222222-2222-4222-8222-222222222222",
      "33333333-3333-4333-8333-333333333333.sna1",
    ].join("/");
    const transfer = uploadSharedAttachment({
      objectKey,
      signedUploadToken: "signed-token",
      contentType: "image/png",
      sha256: "b".repeat(64),
      sizeBytes: 12,
      supabaseUrl: "https://project.supabase.co",
      readRange: vi.fn(async (start, end) => new Uint8Array(end - start)),
    });
    const instance = mocks.instances[0]!;

    await expect(transfer.promise).resolves.toBe(objectKey);
    expect(instance.options.endpoint).toBe(
      "https://project.storage.supabase.co/storage/v1/upload/resumable/sign",
    );
    expect(instance.options.headers).toEqual({
      "x-signature": "signed-token",
    });
    expect(instance.options.metadata).toMatchObject({
      bucketName: "shared-note-attachments",
      objectName: objectKey,
      contentType: "image/png",
      cacheControl: "0",
    });
    expect(JSON.parse(instance.options.metadata.metadata)).toEqual({
      plaintextSha256: "b".repeat(64),
    });
  });
});
