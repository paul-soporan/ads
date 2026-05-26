"use client";

import { AnimatePresence, motion } from "framer-motion";
import { useMemo } from "react";
import type { ReactNode } from "react";

import type { DashboardCategory, DashboardFilters, Distribution, PayloadKind } from "@/lib/bench/types";
import type { LeaderboardSortDirection, LeaderboardSortKey } from "@/lib/bench/urlState";
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
  showProfiling: boolean;
  onShowProfilingChange: (value: boolean) => void;
  leaderboardSortKey: LeaderboardSortKey;
  leaderboardSortDirection: LeaderboardSortDirection;
  onLeaderboardSortKeyChange: (value: LeaderboardSortKey) => void;
  onLeaderboardSortDirectionChange: (value: LeaderboardSortDirection) => void;
  compact: boolean;
  compactSummary: string;
  onExpandRequest: () => void;
}

interface RibbonSelectProps {
  value: string | number;
  ariaLabel: string;
  onValueChange: (value: string) => void;
  children: ReactNode;
}

type SortPreset = {
  id: string;
  label: string;
  key: LeaderboardSortKey;
  direction: LeaderboardSortDirection;
};

const SORT_PRESETS: SortPreset[] = [
  { id: "fastest", label: "Fastest", key: "meanNs", direction: "asc" },
  { id: "leanest", label: "Leanest", key: "peakBytes", direction: "asc" },
  { id: "cache_friendly", label: "Cache-friendly", key: "l1MissRate", direction: "asc" },
  { id: "throughput", label: "Max Throughput", key: "throughput", direction: "desc" },
];

const SORT_PRESET_HELP: Record<SortPreset["id"], string> = {
  fastest: "Highlights implementations with the lowest average latency for quick response paths.",
  leanest: "Brings memory-frugal options to the top when footprint is the primary constraint.",
  cache_friendly: "Prioritizes low L1 miss behavior, which often improves stability under load.",
  throughput: "Ranks by maximum elements processed, ideal for high-volume batch scenarios.",
};

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
  showProfiling,
  onShowProfilingChange,
  leaderboardSortKey,
  leaderboardSortDirection,
  onLeaderboardSortKeyChange,
  onLeaderboardSortDirectionChange,
  compact,
  compactSummary,
  onExpandRequest,
}: ControlRibbonProps) {
  const activePresetId = useMemo(() => {
    return (
      SORT_PRESETS.find((preset) => preset.key === leaderboardSortKey && preset.direction === leaderboardSortDirection)
        ?.id ?? null
    );
  }, [leaderboardSortDirection, leaderboardSortKey]);

  const applySortPreset = (preset: SortPreset) => {
    onLeaderboardSortKeyChange(preset.key);
    onLeaderboardSortDirectionChange(preset.direction);
  };

  const sizeState = useMemo(() => {
    const fallback = sizeOptions[0] ?? 0;
    const selected = typeof filters.size === "number" && sizeOptions.includes(filters.size) ? filters.size : fallback;
    const index = Math.max(sizeOptions.indexOf(selected), 0);

    return {
      selected,
      index,
      min: sizeOptions[0] ?? selected,
      max: sizeOptions[sizeOptions.length - 1] ?? selected,
      progressPercent:
        sizeOptions.length > 1 ? (index / Math.max(sizeOptions.length - 1, 1)) * 100 : 100,
    };
  }, [filters.size, sizeOptions]);

  return (
    <AppGlassPanel className={`border-panel-border/45 bg-panel/70 ${compact ? "p-2" : "p-2.5 sm:p-3"}`}>
      <AnimatePresence mode="wait" initial={false}>
        {compact ? (
            <motion.div
            key="compact"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
              className="flex min-h-[38px] items-center justify-between gap-2"
          >
            <span className="truncate font-mono text-sm text-text/88">{compactSummary}</span>
            <div className="flex items-center gap-2">
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
                onClick={onExpandRequest}
                className="border border-panel-border/90 bg-bg-elevated/70 text-text hover:bg-bg-elevated"
              >
                Modify
              </AppButton>
            </div>
          </motion.div>
        ) : (
            <motion.div
            key="expanded"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.22, ease: "easeOut" }}
              className="space-y-2.5"
          >
              <div className="flex items-center justify-between gap-2">
              <button
                type="button"
                onClick={onOpenPalette}
                  className="inline-flex h-9 w-full max-w-[560px] items-center rounded-md border border-panel-border/70 bg-bg-elevated/55 px-3 text-left text-sm text-text-muted transition hover:bg-bg-elevated/72 hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                aria-label="Search implementations via command palette"
              >
                Search implementation or family... ({shortcutHint})
              </button>
                <div className="ml-auto flex shrink-0 items-center gap-2">
                <label className="inline-flex items-center gap-2 rounded-md border border-panel-border/90 bg-bg-elevated/70 px-2.5 py-1.5 text-xs text-text-muted">
                  <input
                    type="checkbox"
                    checked={showProfiling}
                    onChange={(event) => onShowProfilingChange(event.currentTarget.checked)}
                    className="h-3.5 w-3.5 rounded border-panel-border bg-bg-elevated"
                    aria-label="Toggle profiling metrics visibility"
                  />
                  <span>Profiling</span>
                </label>
                <label className="inline-flex items-center gap-2 rounded-md border border-panel-border/90 bg-bg-elevated/70 px-2 py-1.5 text-xs text-text-muted">
                  <span>Sort</span>
                  <select
                    value={leaderboardSortKey}
                    onChange={(event) => onLeaderboardSortKeyChange(event.target.value as LeaderboardSortKey)}
                    className="h-7 rounded border border-panel-border/80 bg-bg-elevated px-1.5 text-xs text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                    aria-label="Choose leaderboard sort metric"
                  >
                    <option value="implementation">Implementation</option>
                    <option value="operation">Operation</option>
                    <option value="meanNs">Latency</option>
                    <option value="throughput">Throughput</option>
                    <option value="size">Size</option>
                    <option value="instructions">Instructions</option>
                    <option value="l1MissRate">L1 Miss Rate</option>
                    <option value="peakBytes">Peak Bytes</option>
                  </select>
                  <select
                    value={leaderboardSortDirection}
                    onChange={(event) => onLeaderboardSortDirectionChange(event.target.value as LeaderboardSortDirection)}
                    className="h-7 rounded border border-panel-border/80 bg-bg-elevated px-1.5 text-xs text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                    aria-label="Choose leaderboard sort direction"
                  >
                    <option value="asc">Asc</option>
                    <option value="desc">Desc</option>
                  </select>
                </label>
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

            <div className="flex flex-wrap items-center gap-1.5 rounded-md border border-panel-border/75 bg-bg-elevated/45 px-2 py-1.5">
              <span className="pr-1 text-[11px] uppercase tracking-[0.12em] text-text-muted">Sort Presets</span>
              {SORT_PRESETS.map((preset) => {
                const isActive = preset.id === activePresetId;
                const tooltip = SORT_PRESET_HELP[preset.id];

                return (
                  <div key={preset.id} className="group relative">
                    <button
                      type="button"
                      onClick={() => applySortPreset(preset)}
                      className={`inline-flex h-7 items-center rounded-full border px-2.5 text-xs transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60 ${
                        isActive
                          ? "border-primary/70 bg-primary/18 text-text"
                          : "border-panel-border/80 bg-panel/55 text-text-muted hover:border-primary/45 hover:bg-bg-elevated/72 hover:text-text"
                      }`}
                      aria-pressed={isActive}
                      aria-label={`Apply ${preset.label} sort preset`}
                    >
                      {preset.label}
                    </button>
                    <div className="pointer-events-none absolute left-1/2 top-full z-30 mt-1.5 w-56 -translate-x-1/2 rounded-md border border-panel-border/85 bg-bg-elevated/95 px-2 py-1.5 text-[11px] leading-snug text-text-muted opacity-0 shadow-panel backdrop-blur-sm transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
                      {tooltip}
                    </div>
                  </div>
                );
              })}
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
                  <div className="rounded-md border border-panel-border/80 bg-bg-elevated/68 px-2 py-1">
                    <div className="mb-1 flex items-center justify-between">
                      <span className="font-mono text-sm text-primary [font-variant-numeric:tabular-nums]">
                        {sizeState.selected.toLocaleString()}
                      </span>
                      <span className="text-[11px] text-text-muted">
                        {sizeState.min.toLocaleString()} - {sizeState.max.toLocaleString()}
                      </span>
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
                      className="h-2 w-full cursor-pointer rounded-full accent-primary"
                      style={{
                        background: `linear-gradient(90deg, rgba(33,212,253,0.85) 0%, rgba(33,212,253,0.85) ${sizeState.progressPercent}%, rgba(148,163,184,0.3) ${sizeState.progressPercent}%, rgba(148,163,184,0.3) 100%)`,
                      }}
                      aria-label="Select input size"
                      disabled={sizeOptions.length <= 1}
                    />
                  </div>
                </label>
              </div>
          </motion.div>
        )}
      </AnimatePresence>
    </AppGlassPanel>
  );
}
