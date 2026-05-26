"use client";

import { useMemo } from "react";
import type { ReactNode } from "react";

import type { DashboardCategory, DashboardFilters, Distribution, PayloadKind } from "@/lib/bench/types";
import { AppButton } from "@/components/ui/AppButton";
import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import { ExportMenu } from "@/components/dashboard/ExportMenu";

interface ControlRibbonProps {
  filters: DashboardFilters;
  categoryOptions: DashboardCategory[];
  operationOptions: string[];
  distributionOptions: Distribution[];
  payloadOptions: PayloadKind[];
  sizeOptions: number[];
  shortcutHint: string;
  onFilterChange: <K extends keyof DashboardFilters>(key: K, value: DashboardFilters[K]) => void;
  onOpenPalette: () => void;
  onResetView: () => void;
  getShareUrl: () => string;
  onExportVisibleCsv: () => Promise<"copied" | "downloaded">;
  onExportVisibleMarkdown: () => Promise<"copied" | "downloaded">;
  onExportPinnedCsv: () => Promise<"copied" | "downloaded">;
  onExportPinnedMarkdown: () => Promise<"copied" | "downloaded">;
  hasPinnedComparison: boolean;
  onExportAllCsv: () => Promise<"copied" | "downloaded">;
  onExportAllMarkdown: () => Promise<"copied" | "downloaded">;
}

interface RibbonSelectProps {
  value: string | number;
  ariaLabel: string;
  onValueChange: (value: string) => void;
  children: ReactNode;
}

function RibbonSelect({ value, ariaLabel, onValueChange, children }: RibbonSelectProps) {
  return (
    <div className="relative">
      <select
        value={String(value)}
        onChange={(event) => onValueChange(event.target.value)}
        className="h-9 w-full appearance-none rounded-md border border-panel-border/90 bg-bg-elevated/75 px-2.5 pr-8 text-sm text-text shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] transition hover:border-primary/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
        aria-label={ariaLabel}
      >
        {children}
      </select>
      <span className="pointer-events-none absolute inset-y-0 right-2 flex items-center text-primary/75" aria-hidden="true">
        <svg viewBox="0 0 20 20" fill="none" className="h-4 w-4" stroke="currentColor" strokeWidth="1.8">
          <path d="M6 8l4 4 4-4" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </span>
    </div>
  );
}

export function ControlRibbon({
  filters,
  categoryOptions,
  operationOptions,
  distributionOptions,
  payloadOptions,
  sizeOptions,
  shortcutHint,
  onFilterChange,
  onOpenPalette,
  onResetView,
  getShareUrl,
  onExportVisibleCsv,
  onExportVisibleMarkdown,
  onExportPinnedCsv,
  onExportPinnedMarkdown,
  hasPinnedComparison,
  onExportAllCsv,
  onExportAllMarkdown,
}: ControlRibbonProps) {
  const sizeState = useMemo(() => {
    const fallback = sizeOptions[0] ?? 0;
    const selected = typeof filters.size === "number" && sizeOptions.includes(filters.size) ? filters.size : fallback;
    const index = Math.max(sizeOptions.indexOf(selected), 0);

    return {
      selected,
      index,
      min: sizeOptions[0] ?? selected,
      max: sizeOptions[sizeOptions.length - 1] ?? selected,
    };
  }, [filters.size, sizeOptions]);

  return (
    <AppGlassPanel className="space-y-3 border-primary/25 bg-panel/78 p-3 sm:p-4 lg:p-4">
      <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-2">
        <div />
        <button
          type="button"
          onClick={onOpenPalette}
          className="mx-auto inline-flex h-10 min-w-[320px] max-w-full items-center rounded-md border border-primary/35 bg-bg-elevated/50 px-3 text-left text-sm text-text-muted transition hover:bg-bg-elevated/75 hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
          aria-label="Search implementations via command palette"
        >
          Search implementation or family... ({shortcutHint})
        </button>
        <div className="ml-auto flex items-center gap-2">
          <ExportMenu
            getShareUrl={getShareUrl}
            onExportVisibleCsv={onExportVisibleCsv}
            onExportVisibleMarkdown={onExportVisibleMarkdown}
            onExportPinnedCsv={onExportPinnedCsv}
            onExportPinnedMarkdown={onExportPinnedMarkdown}
            hasPinnedComparison={hasPinnedComparison}
            onExportAllCsv={onExportAllCsv}
            onExportAllMarkdown={onExportAllMarkdown}
          />
          <AppButton
            variant="ghost"
            size="sm"
            onClick={onResetView}
            className="border border-panel-border/90 bg-bg-elevated/70 text-text hover:bg-bg-elevated"
          >
            Reset View
          </AppButton>
        </div>
      </div>

      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-5">
        <label className="space-y-1">
          <span className="block text-xs uppercase tracking-[0.12em] text-text-muted">Category</span>
          <RibbonSelect
            value={filters.category}
            onValueChange={(value) => onFilterChange("category", value as DashboardFilters["category"])}
            ariaLabel="Select benchmark category"
          >
            {categoryOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </RibbonSelect>
        </label>

        <label className="space-y-1">
          <span className="block text-xs uppercase tracking-[0.12em] text-text-muted">Operation</span>
          <RibbonSelect
            value={filters.operation}
            onValueChange={(value) => onFilterChange("operation", value)}
            ariaLabel="Select benchmark operation"
          >
            {operationOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </RibbonSelect>
        </label>

        <label className="space-y-1">
          <span className="block text-xs uppercase tracking-[0.12em] text-text-muted">Distribution</span>
          <RibbonSelect
            value={filters.distribution}
            onValueChange={(value) => onFilterChange("distribution", value as DashboardFilters["distribution"])}
            ariaLabel="Select benchmark distribution"
          >
            {distributionOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </RibbonSelect>
        </label>

        <label className="space-y-1">
          <span className="block text-xs uppercase tracking-[0.12em] text-text-muted">Payload</span>
          <RibbonSelect
            value={filters.payload}
            onValueChange={(value) => onFilterChange("payload", value as DashboardFilters["payload"])}
            ariaLabel="Select payload kind"
          >
            {payloadOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </RibbonSelect>
        </label>

        <label className="space-y-1">
          <span className="block text-xs uppercase tracking-[0.12em] text-text-muted">Size</span>
          <div className="rounded-md border border-panel-border bg-bg-elevated/70 px-2 py-1.5">
            <div className="mb-1.5 flex items-center justify-between">
              <span className="font-mono text-sm text-primary [font-variant-numeric:tabular-nums]">
                {sizeState.selected.toLocaleString()}
              </span>
              <span className="text-xs text-text-muted">Discrete slider</span>
            </div>
            <input
              type="range"
              min={0}
              max={Math.max(sizeOptions.length - 1, 0)}
              step={1}
              value={sizeState.index}
              onChange={(event) => {
                const nextIndex = Number(event.target.value);
                const nextSize = sizeOptions[nextIndex] ?? sizeState.selected;
                onFilterChange("size", nextSize);
              }}
              className="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-panel accent-primary"
              aria-label="Select input size"
              disabled={sizeOptions.length <= 1}
            />
            <div className="mt-1.5 flex items-center justify-between text-[11px] text-text-muted">
              <span>{sizeState.min.toLocaleString()}</span>
              <span>{sizeState.max.toLocaleString()}</span>
            </div>
          </div>
        </label>
      </div>
    </AppGlassPanel>
  );
}
