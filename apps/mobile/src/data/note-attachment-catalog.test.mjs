import assert from "node:assert/strict";
import test from "node:test";

import { mapNoteAttachmentRows } from "./note-attachment-catalog-model.ts";

const row = {
  id: "2c617e36-a3b9-49aa-a240-19c3aa542f50",
  source_id: "9cfb2f08-a02f-41cf-a13e-07f36b87ef2b",
  filename: "product brief.pdf",
  relative_path: "attachments/9cfb2f08-a02f-41cf-a13e-07f36b87ef2b",
  content_type: "application/pdf",
  size_bytes: 42,
  sha256: "a".repeat(64),
  created_at: "2026-08-17T00:00:00.000Z",
  available_locally: 1,
  local_relative_path: "attachments/9cfb2f08-a02f-41cf-a13e-07f36b87ef2b",
};

test("maps a local note attachment to its canonical device path", () => {
  assert.deepEqual(mapNoteAttachmentRows([row]), [
    {
      attachmentId: "2c617e36-a3b9-49aa-a240-19c3aa542f50",
      sourceId: "9cfb2f08-a02f-41cf-a13e-07f36b87ef2b",
      filename: "product brief.pdf",
      relativePath: "attachments/9cfb2f08-a02f-41cf-a13e-07f36b87ef2b",
      contentType: "application/pdf",
      sizeBytes: 42,
      sha256: "a".repeat(64),
      createdAt: "2026-08-17T00:00:00.000Z",
      availableLocally: true,
      localRelativePath: "attachments/9cfb2f08-a02f-41cf-a13e-07f36b87ef2b",
    },
  ]);
});

test("does not expose a mismatched local path", () => {
  const [attachment] = mapNoteAttachmentRows([
    { ...row, local_relative_path: "attachments/other" },
  ]);
  assert.equal(attachment.availableLocally, false);
  assert.equal(attachment.localRelativePath, null);
});

test("drops attachment metadata with an unsafe canonical path", () => {
  assert.deepEqual(
    mapNoteAttachmentRows([
      {
        ...row,
        source_id: "../product brief.pdf",
        relative_path: "attachments/../product brief.pdf",
      },
    ]),
    [],
  );
});
