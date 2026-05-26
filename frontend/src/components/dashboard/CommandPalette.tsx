"use client";

import { useEffect, useMemo, useState } from "react";

import { cn } from "@/lib/cn";

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  suggestions: string[];
  shortcutHint: string;
  onSelect: (value: string) => void;
}

export function CommandPalette({ open, onOpenChange, suggestions, shortcutHint, onSelect }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return suggestions.slice(0, 10);
    return suggestions.filter((item) => item.toLowerCase().includes(q)).slice(0, 10);
  }, [query, suggestions]);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setActiveIndex(0);
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        onOpenChange(false);
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        onOpenChange(false);
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((index) => (filtered.length === 0 ? 0 : (index + 1) % filtered.length));
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex((index) => (filtered.length === 0 ? 0 : (index - 1 + filtered.length) % filtered.length));
      }

      if (event.key === "Enter" && filtered.length > 0) {
        event.preventDefault();
        onSelect(filtered[activeIndex] ?? filtered[0]);
        onOpenChange(false);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activeIndex, filtered, onOpenChange, onSelect, open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-[120] flex items-start justify-center bg-bg/72 px-4 pt-[12vh] backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-label="Search implementations or families"
      onClick={() => onOpenChange(false)}
    >
      <div
        className="w-full max-w-2xl rounded-lg border border-panel-border bg-bg-elevated shadow-panel"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="border-b border-panel-border p-3">
          <input
            autoFocus
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search Implementation: ... or Family: ..."
            className="w-full rounded-md border border-panel-border bg-panel/60 px-3 py-2 text-sm text-text outline-none ring-primary/60 transition focus:ring-2"
          />
        </div>

        <ul className="max-h-80 overflow-y-auto p-2">
          {filtered.length === 0 ? (
            <li className="rounded-md px-3 py-4 text-sm text-text-muted">No results for this query.</li>
          ) : (
            filtered.map((item, index) => (
              <li key={item}>
                <button
                  type="button"
                  className={cn(
                    "w-full rounded-md px-3 py-2 text-left text-sm transition",
                    index === activeIndex
                      ? "bg-primary/85 font-semibold text-bg"
                      : "text-text-muted hover:bg-bg-elevated/75 hover:text-text",
                  )}
                  onMouseEnter={() => setActiveIndex(index)}
                  onClick={() => {
                    onSelect(item);
                    onOpenChange(false);
                  }}
                >
                  {item}
                </button>
              </li>
            ))
          )}
        </ul>

        <div className="flex items-center justify-between border-t border-panel-border px-3 py-2 text-sm text-text-muted">
          <span>Use ↑/↓ to navigate, Enter to apply</span>
          <span className="font-mono">{shortcutHint}</span>
        </div>
      </div>
    </div>
  );
}
