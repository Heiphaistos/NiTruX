import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxSelect from "./NxSelect.vue";

describe("NxSelect", () => {
  const options = [
    { value: "ext4", label: "ext4" },
    { value: "btrfs", label: "btrfs" },
  ];

  it("renders one <option> per entry", () => {
    const wrapper = mount(NxSelect, { props: { modelValue: "ext4", options } });
    expect(wrapper.findAll("option")).toHaveLength(2);
  });

  it("binds via v-model", async () => {
    const wrapper = mount(NxSelect, { props: { modelValue: "ext4", options } });
    await wrapper.find("select").setValue("btrfs");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["btrfs"]);
  });
});
