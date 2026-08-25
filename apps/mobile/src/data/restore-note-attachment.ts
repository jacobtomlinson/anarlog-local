import { Directory, File, Paths } from "expo-file-system";
import * as Sharing from "expo-sharing";

import type { NoteAttachment } from "@/data/note-attachment-catalog";

export async function shareNoteAttachment(
  uri: string,
  attachment: NoteAttachment,
): Promise<void> {
  if (!(await Sharing.isAvailableAsync())) {
    throw new Error("Sharing is not available on this device.");
  }
  const directory = new Directory(
    Paths.cache,
    "note-attachment-shares",
    attachment.attachmentId,
  );
  directory.create({ intermediates: true, idempotent: true });
  const staged = new File(directory, attachment.filename);
  if (staged.exists) staged.delete();
  await new File(uri).copy(staged);
  await Sharing.shareAsync(staged.uri, {
    dialogTitle: `Share ${attachment.filename}`,
    mimeType: attachment.contentType || "application/octet-stream",
  });
}
