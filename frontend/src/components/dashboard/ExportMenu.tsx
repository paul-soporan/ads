"use client";

import { useEffect, useRef, useState } from "react";

import { AppButton } from "@/components/ui/AppButton";
import { useToast } from "@/components/ui/ToastProvider";

interface ExportMenuProps {
  getShareUrl: () => string;
  onExportVisibleCsv: () => Promise<"copied" | "downloaded">;
  onExportVisibleMarkdown: () => Promise<"copied" | "downloaded">;
  onExportPinnedCsv: () => Promise<"copied" | "downloaded">;
  onExportPinnedMarkdown: () => Promise<"copied" | "downloaded">;
  hasPinnedComparison: boolean;
  onExportAllCsv: () => Promise<"copied" | "downloaded">;
  onExportAllMarkdown: () => Promise<"copied" | "downloaded">;
}

export function ExportMenu({
  getShareUrl,
  onExportVisibleCsv,
  onExportVisibleMarkdown,
  onExportPinnedCsv,
  onExportPinnedMarkdown,
  hasPinnedComparison,
  onExportAllCsv,
  onExportAllMarkdown,
}: ExportMenuProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const pushToast = useToast();

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };

    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
      }
    };

    window.addEventListener("mousedown", onPointerDown);
    window.addEventListener("keydown", onEscape);

    return () => {
      window.removeEventListener("mousedown", onPointerDown);
      window.removeEventListener("keydown", onEscape);
    };
  }, [open]);

  async function handleShare() {
    try {
      await navigator.clipboard.writeText(getShareUrl());
      pushToast({ title: "Share", message: "Deep link copied to clipboard.", variant: "success" });
      setOpen(false);
    } catch {
      pushToast({ title: "Share", message: "Unable to copy link to clipboard.", variant: "danger" });
    }
  }

  async function handleExport(
    action: () => Promise<"copied" | "downloaded">,
    label: string,
  ) {
    const result = await action();
    pushToast({
      title: "Export",
      message: result === "copied" ? `${label} copied to clipboard.` : `${label} downloaded.`,
      variant: "success",
    });
    setOpen(false);
  }

  return (
    <div className="relative z-[80]" ref={rootRef}>
      <AppButton
        variant="ghost"
        size="sm"
        onClick={() => setOpen((value) => !value)}
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label="Open share and export menu"
        className="border border-panel-border/90 bg-bg-elevated/70 text-text hover:bg-bg-elevated"
      >
        Share / Export
      </AppButton>

      {open ? (
        <div
          role="menu"
          aria-label="Share and export"
          className="absolute right-0 top-11 z-[120] min-w-64 rounded-md border border-panel-border bg-panel/95 p-1.5 shadow-panel backdrop-blur-[16px]"
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleShare()}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text"
          >
            Copy Deep Link
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleExport(onExportVisibleCsv, "Visible CSV")}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text"
          >
            Export Visible Data - CSV
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleExport(onExportVisibleMarkdown, "Visible Markdown")}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text"
          >
            Export Visible Data - Markdown
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleExport(onExportPinnedCsv, "Pinned CSV")}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text disabled:cursor-not-allowed disabled:opacity-45"
            disabled={!hasPinnedComparison}
          >
            Export Pinned Comparison - CSV
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleExport(onExportPinnedMarkdown, "Pinned Markdown")}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text disabled:cursor-not-allowed disabled:opacity-45"
            disabled={!hasPinnedComparison}
          >
            Export Pinned Comparison - Markdown
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleExport(onExportAllCsv, "All CSV")}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text"
          >
            Export Full Dataset - CSV
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => void handleExport(onExportAllMarkdown, "All Markdown")}
            className="block w-full rounded px-2.5 py-2 text-left text-sm text-text-muted transition hover:bg-bg-elevated/70 hover:text-text"
          >
            Export Full Dataset - Markdown
          </button>
        </div>
      ) : null}
    </div>
  );
}
