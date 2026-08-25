import { useHotkeys } from "react-hotkeys-hook";

import { useChatContext } from "./chat-context";

import { useTabs } from "~/store/zustand/tabs";

export type { ChatEvent, ChatMode } from "~/store/zustand/tabs";

export function useChatMode() {
  const mode = useTabs((state) => state.chatMode);
  const transitionChatMode = useTabs((state) => state.transitionChatMode);
  const selection = useChatContext((state) => state.chat);
  const setGroupId = useChatContext((state) => state.setGroupId);
  const rollbackFailedGroup = useChatContext(
    (state) => state.rollbackFailedGroup,
  );
  const startNewChat = useChatContext((state) => state.startNewChat);
  const selectChat = useChatContext((state) => state.selectChat);

  useHotkeys(
    "mod+j",
    () => {
      transitionChatMode({ type: "TOGGLE" });
    },
    {
      preventDefault: true,
      enableOnFormTags: true,
      enableOnContentEditable: true,
    },
    [transitionChatMode],
  );

  return {
    mode,
    sendEvent: transitionChatMode,
    groupId: selection.groupId,
    sessionId: selection.sessionId,
    setGroupId,
    rollbackFailedGroup,
    startNewChat,
    selectChat,
  };
}
