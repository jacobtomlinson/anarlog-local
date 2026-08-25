import { beforeEach, describe, expect, test } from "vitest";

import { useChatContext } from "./chat-context";

describe("chat context", () => {
  beforeEach(() => {
    useChatContext.setState({
      chat: { groupId: undefined, sessionId: "initial" },
    });
  });

  test("startNewChat resets the group and rotates the session id", () => {
    useChatContext.setState({
      chat: { groupId: "group-1", sessionId: "session-1" },
    });

    useChatContext.getState().startNewChat();

    const selection = useChatContext.getState().chat;
    expect(selection.groupId).toBeUndefined();
    expect(selection.sessionId).not.toBe("session-1");
  });

  test("selectChat syncs the selected group and session id", () => {
    useChatContext.getState().selectChat("group-2");

    expect(useChatContext.getState().chat).toEqual({
      groupId: "group-2",
      sessionId: "group-2",
    });
  });
});
