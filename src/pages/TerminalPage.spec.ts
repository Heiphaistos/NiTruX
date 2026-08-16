import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import TerminalPage from "./TerminalPage.vue";

const mockTerm = {
  open: vi.fn(),
  write: vi.fn(),
  onData: vi.fn(),
  loadAddon: vi.fn(),
  dispose: vi.fn(),
  rows: 24,
  cols: 80,
};

vi.mock("@xterm/xterm", () => ({
  Terminal: vi.fn(function () {
    return mockTerm;
  }),
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: vi.fn(function () {
    return { fit: vi.fn() };
  }),
}));

const { FakeChannel } = vi.hoisted(() => {
  class FakeChannel {
    onmessage: ((data: string) => void) | null = null;
  }
  return { FakeChannel };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
  Channel: FakeChannel,
}));

describe("TerminalPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("spawns a terminal session on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(TerminalPage);
    expect(invoke).toHaveBeenCalledWith(
      "spawn_terminal",
      expect.objectContaining({ id: expect.any(String), onData: expect.any(FakeChannel) }),
    );
  });

  it("relays user keystrokes to write_to_terminal", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(TerminalPage);
    const onDataCallback = mockTerm.onData.mock.calls[0][0] as (data: string) => void;
    onDataCallback("ls\n");
    const call = (invoke as ReturnType<typeof vi.fn>).mock.calls.find((c) => c[0] === "spawn_terminal")!;
    const id = call[1].id as string;
    expect(invoke).toHaveBeenCalledWith("write_to_terminal", { id, data: "ls\n" });
  });

  it("attaches a rejection handler to write_to_terminal's promise (dead pty must not go unhandled)", async () => {
    // Regression guard: write_to_terminal fires on every keystroke with no
    // try/catch (a bare `.catch(() => {})` guards it instead) -- before
    // that fix, a dead pty would leave one unhandled promise rejection per
    // character typed, since nothing awaited or caught the call's result.
    // Asserted by spying directly on the returned promise's `.catch`
    // (deterministic) rather than Node's `unhandledRejection` event, whose
    // firing is timing-dependent across Vitest's worker boundary and does
    // not reliably reproduce here even without the fix.
    const { invoke } = await import("@tauri-apps/api/core");
    const catchSpy = vi.fn();
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "write_to_terminal") {
        const rejected = Promise.reject("pty closed");
        const originalCatch = rejected.catch.bind(rejected);
        rejected.catch = (...args: Parameters<typeof originalCatch>) => {
          catchSpy();
          return originalCatch(...args);
        };
        return rejected;
      }
      return Promise.resolve(null);
    });

    mount(TerminalPage);
    const onDataCallback = mockTerm.onData.mock.calls[0][0] as (data: string) => void;
    onDataCallback("x");

    expect(catchSpy).toHaveBeenCalled();
  });

  it("writes incoming channel data into the terminal", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(TerminalPage);
    const call = (invoke as ReturnType<typeof vi.fn>).mock.calls.find((c) => c[0] === "spawn_terminal")!;
    const channel = call[1].onData as InstanceType<typeof FakeChannel>;
    channel.onmessage!("hello from shell");
    expect(mockTerm.write).toHaveBeenCalledWith("hello from shell");
  });

  it("shows an error instead of a silently blank terminal when spawn_terminal fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementationOnce(() => Promise.reject("shell introuvable : /bin/bash"));
    const wrapper = mount(TerminalPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("shell introuvable"));
  });

  it("closes the session on unmount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TerminalPage);
    const call = (invoke as ReturnType<typeof vi.fn>).mock.calls.find((c) => c[0] === "spawn_terminal")!;
    const id = call[1].id as string;
    wrapper.unmount();
    expect(invoke).toHaveBeenCalledWith("close_terminal", { id });
  });
});
