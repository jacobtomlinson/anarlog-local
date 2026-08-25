import { create } from "zustand";

import { id } from "~/shared/utils";

type ChatSelection = {
  groupId: string | undefined;
  sessionId: string;
};

interface ChatContextState {
  chat: ChatSelection;
}

interface ChatContextActions {
  setGroupId: (groupId: string | undefined) => void;
  rollbackFailedGroup: (failedGroupId: string) => void;
  startNewChat: () => void;
  selectChat: (groupId: string) => void;
}

export const useChatContext = create<ChatContextState & ChatContextActions>(
  (set) => ({
    chat: createChatSelection(),
    setGroupId: (groupId) =>
      set((state) => ({ chat: { ...state.chat, groupId } })),
    // Compares against the live groupId, not a value captured when the send
    // started — the failure lands after onGroupCreated already updated it.
    rollbackFailedGroup: (failedGroupId) =>
      set((state) => {
        const selection = state.chat;
        if (selection.groupId !== failedGroupId) {
          return state;
        }

        return {
          chat: { ...selection, groupId: undefined },
        };
      }),
    startNewChat: () => set({ chat: createChatSelection() }),
    selectChat: (groupId) => set({ chat: { groupId, sessionId: groupId } }),
  }),
);

function createChatSelection(): ChatSelection {
  return { groupId: undefined, sessionId: id() };
}
