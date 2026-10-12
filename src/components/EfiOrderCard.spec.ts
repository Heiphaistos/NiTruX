import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import EfiOrderCard from "./EfiOrderCard.vue";

const invoke = vi.fn().mockResolvedValue("ok");
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("EfiOrderCard", () => {
  it("puts Linux back first and applies the new order", async () => {
    const wrapper = mount(EfiOrderCard, {
      props: {
        entries: [
          { id: "0002", label: "Windows Boot Manager", is_current: true },
          { id: "0003", label: "debian", is_current: true },
        ],
        order: ["0002", "0003"],
      },
    });
    expect(wrapper.text()).not.toContain("Appliquer cet ordre");
    await wrapper.find('button[aria-label="Monter debian"]').trigger("click");
    expect(wrapper.text()).toMatch(/1\. debian.*2\. Windows/s);
    await wrapper.findAll("button").find((b) => b.text() === "Appliquer cet ordre")!.trigger("click");
    expect(invoke).toHaveBeenCalledWith("set_efi_boot_order", { order: ["0003", "0002"] });
    await vi.waitFor(() => expect(wrapper.emitted("changed")).toBeTruthy());
  });
});
