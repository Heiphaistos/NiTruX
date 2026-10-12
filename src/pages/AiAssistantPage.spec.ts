import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import AiAssistantPage from "./AiAssistantPage.vue";
import { pendingTerminalCommand } from "@/lib/terminalCommand";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("AiAssistantPage", () => {
  it("sends the conversation to the chosen model and shows the reply", async () => {
    invoke.mockImplementation((cmd: string) =>
      cmd === "ollama_status"
        ? Promise.resolve({ installed: true, running: true, models: ["llama3.2:3b"], error: null })
        : Promise.resolve("Votre disque est plein à cause des journaux."),
    );
    const wrapper = mount(AiAssistantPage);
    await vi.waitFor(() => expect(wrapper.find("textarea").exists()).toBe(true));
    await wrapper.find("textarea").setValue("Pourquoi mon disque est plein ?");
    await wrapper.find("form").trigger("submit");
    await vi.waitFor(() => expect(wrapper.text()).toContain("à cause des journaux"));
    expect(invoke).toHaveBeenCalledWith("ollama_chat", {
      model: "llama3.2:3b",
      messages: [{ role: "user", content: "Pourquoi mon disque est plein ?" }, { role: "assistant", content: "Votre disque est plein à cause des journaux." }],
    });
  });

  it("offers to download a model when Ollama has none", async () => {
    invoke.mockResolvedValue({ installed: true, running: true, models: [], error: null });
    const wrapper = mount(AiAssistantPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("aucun modèle"));
    await wrapper.findAll("button").find((b) => b.text().startsWith("Télécharger"))!.trigger("click");
    expect(pendingTerminalCommand.value).toBe("ollama pull llama3.2:3b");
  });
});
