import { describe, expect, it, vi } from "vitest";
import { mockNuxtImport, mountSuspended } from "@nuxt/test-utils/runtime";
import { ref } from "vue";
import ConnectPage from "~/pages/connect.vue";

const { useFetchMock } = vi.hoisted(() => ({ useFetchMock: vi.fn() }));

mockNuxtImport("useFetch", () => useFetchMock);

describe("connect page", () => {
  it("shows the response from the backend on success", async () => {
    useFetchMock.mockReturnValue({
      data: ref("Hello frontend! This is Rust. Nou2X^mZwMq!4F$t"),
      error: ref(null),
      status: ref("success"),
    });

    const page = await mountSuspended(ConnectPage);

    expect(page.text()).toContain(
      "Hello frontend! This is Rust. Nou2X^mZwMq!4F$t",
    );
  });

  it("shows an error message when the request fails", async () => {
    useFetchMock.mockReturnValue({
      data: ref(null),
      error: ref(new Error("Failed to fetch")),
      status: ref("error"),
    });

    const page = await mountSuspended(ConnectPage);

    expect(page.text()).toContain("Failed to fetch");
  });

  it("shows an error message when the backend is offline", async () => {
    const page = await mountSuspended(ConnectPage);
    expect(page.find(".text-red-500").exists()).toBe(true);
  });
});
