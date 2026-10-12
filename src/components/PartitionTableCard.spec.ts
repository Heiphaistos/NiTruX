import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PartitionTableCard from "./PartitionTableCard.vue";

const invoke = vi.fn().mockResolvedValue("ok");
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const button = (w: ReturnType<typeof mount>, text: string) => w.findAll("button").find((b) => b.text() === text)!;

describe("PartitionTableCard", () => {
  it("creates a partition from a disk and a size, no retyping needed", async () => {
    const wrapper = mount(PartitionTableCard);
    const [target, size] = wrapper.findAll("input");
    await target!.setValue("/dev/sdb");
    await size!.setValue("500");
    await button(wrapper, "Créer").trigger("click");
    expect(invoke).toHaveBeenCalledWith("create_partition", { disk: "/dev/sdb", sizeMib: 500 });
    await vi.waitFor(() => expect(wrapper.emitted("changed")).toBeTruthy());
  });

  it("deletes only after the partition is typed again", async () => {
    const wrapper = mount(PartitionTableCard);
    await wrapper.find("select").setValue("delete");
    const [target, confirm] = wrapper.findAll("input");
    await target!.setValue("/dev/sdb2");
    expect(button(wrapper, "Supprimer").attributes("disabled")).toBeDefined();
    await confirm!.setValue("/dev/sdb2");
    await button(wrapper, "Supprimer").trigger("click");
    expect(invoke).toHaveBeenCalledWith("delete_partition", { device: "/dev/sdb2" });
  });
});
