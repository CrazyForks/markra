import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";
import { MarkdownTabsBar } from "./MarkdownTabsBar";

describe("MarkdownTabsBar", () => {
  it("marks titlebar tab empty space as a window drag region", () => {
    const { container } = render(
      <MarkdownTabsBar
        activeTabId="tab-a"
        placement="titlebar"
        tabs={[
          {
            dirty: false,
            id: "tab-a",
            name: "Alpha.md",
            path: "/synthetic/alpha.md"
          }
        ]}
        onCloseTab={() => {}}
        onNewTab={() => {}}
        onSelectTab={() => {}}
      />
    );

    expect(screen.getByRole("tablist", { name: "Open documents" })).toBeInTheDocument();
    expect(container.querySelector(".document-tabs-titlebar")).toHaveAttribute("data-tauri-drag-region");
    expect(container.querySelector(".document-tabs-drag-spacer")).toHaveAttribute("data-tauri-drag-region");
  });

  it("opens tab actions from right click and closes related tabs", async () => {
    const onCloseTab = vi.fn();

    render(
      <MarkdownTabsBar
        activeTabId="tab-b"
        tabs={[
          {
            dirty: false,
            id: "tab-a",
            name: "Alpha.md",
            path: "/synthetic/alpha.md"
          },
          {
            dirty: false,
            id: "tab-b",
            name: "Beta.md",
            path: "/synthetic/beta.md"
          },
          {
            dirty: false,
            id: "tab-c",
            name: "Gamma.md",
            path: "/synthetic/gamma.md"
          }
        ]}
        onCloseTab={onCloseTab}
        onNewTab={() => {}}
        onSelectTab={() => {}}
      />
    );

    fireEvent.contextMenu(screen.getByRole("tab", { name: /Beta\.md/ }));

    const menu = screen.getByRole("menu", { name: "Beta.md" });
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Close other tabs" }));

    await waitFor(() => expect(onCloseTab).toHaveBeenCalledTimes(2));
    expect(onCloseTab).toHaveBeenNthCalledWith(1, "tab-a");
    expect(onCloseTab).toHaveBeenNthCalledWith(2, "tab-c");

    onCloseTab.mockClear();
    fireEvent.contextMenu(screen.getByRole("tab", { name: /Beta\.md/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Close tabs to the right" }));

    await waitFor(() => expect(onCloseTab).toHaveBeenCalledTimes(1));
    expect(onCloseTab).toHaveBeenCalledWith("tab-c");
  });
});
