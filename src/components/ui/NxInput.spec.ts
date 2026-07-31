import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxInput from "./NxInput.vue";

describe("NxInput", () => {
  it("binds via v-model", async () => {
    const wrapper = mount(NxInput, { props: { modelValue: "" } });
    await wrapper.find("input").setValue("/dev/sda1");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["/dev/sda1"]);
  });

  it("forwards the placeholder prop to the underlying input", () => {
    const wrapper = mount(NxInput, { props: { modelValue: "", placeholder: "Chemin..." } });
    expect(wrapper.find("input").attributes("placeholder")).toBe("Chemin...");
  });
});
