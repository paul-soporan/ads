"use client";

import { AnimatePresence, motion } from "framer-motion";
import { Suspense, useEffect, useMemo, useRef, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import { AppSkeleton } from "@/components/ui/AppSkeleton";
import { CommandPalette } from "@/components/dashboard/CommandPalette";
import { ControlRibbon } from "@/components/dashboard/ControlRibbon";
import { InspectorPane } from "@/components/dashboard/InspectorPane";
import { LeaderboardTable } from "@/components/dashboard/LeaderboardTable";
import { TradeoffMatrix } from "@/components/dashboard/TradeoffMatrix";
import { buildImplementationAggregates, toPercentChange } from "@/lib/bench/analytics";
import { comparisonToCsv, comparisonToMarkdown, copyOrDownload, leaderboardToCsv, leaderboardToMarkdown } from "@/lib/bench/export";
import {
  selectContextualFilterOptions,
  selectFilterOptions,
  selectFilteredRecords,
  useDashboardStore,
} from "@/lib/bench/store";
import type { MatrixScaleMode } from "@/lib/bench/urlState";
import { parseUrlState, serializeUrlState } from "@/lib/bench/urlState";

const STEP1_DEFAULT_FILTERS = {
  category: "maps" as const,
  operation: "insert",
  distribution: "uniform" as const,
  payload: "u64" as const,
  size: 10_000,
  search: "",
};

function searchMatchesTrend(
  record: { implementation: string; functionName: string; group: string; operation: string },
  search: string,
) {
  if (!search) return true;
  const haystack = `${record.implementation} ${record.functionName} ${record.group} ${record.operation}`.toLowerCase();
  return haystack.includes(search);
}

function inferImplementationFamily(implementation: string): string {
  return implementation
    .toLowerCase()
    .replace(/(^|_)(safe|raw|arena|std)(_|$)/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function HomePageContent() {
  const [isPaletteOpen, setPaletteOpen] = useState(false);
  const [shortcutHint, setShortcutHint] = useState("Ctrl + K");
  const [matrixScaleMode, setMatrixScaleMode] = useState<MatrixScaleMode>("log");
  const [hideMatrixOutliers, setHideMatrixOutliers] = useState(true);
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const syncRef = useRef<{ hydrated: boolean; lastQuery: string; isInternalUpdate: boolean }>({
    hydrated: false,
    lastQuery: "",
    isInternalUpdate: false,
  });
  const debounceRef = useRef<number | null>(null);
  const inspectorWasActiveRef = useRef(false);

  const status = useDashboardStore((state) => state.status);
  const error = useDashboardStore((state) => state.error);
  const dataset = useDashboardStore((state) => state.dataset);
  const filters = useDashboardStore((state) => state.filters);
  const selectedImplementations = useDashboardStore((state) => state.selectedImplementations);
  const setFilter = useDashboardStore((state) => state.setFilter);
  const setFilters = useDashboardStore((state) => state.setFilters);
  const toggleImplementation = useDashboardStore((state) => state.toggleImplementation);
  const setSelectedImplementations = useDashboardStore((state) => state.setSelectedImplementations);
  const resetComparison = useDashboardStore((state) => state.resetComparison);
  const clearFilters = useDashboardStore((state) => state.clearFilters);
  const loadData = useDashboardStore((state) => state.loadData);

  const baseOptions = useDashboardStore(selectFilterOptions);
  const contextualFilterOptions = useDashboardStore(selectContextualFilterOptions);
  const filtered = useDashboardStore(selectFilteredRecords);

  const trendRecords = useMemo(() => {
    if (!dataset) return filtered;

    const search = filters.search.trim().toLowerCase();
    return dataset.records.filter((record) => {
      if (filters.category !== "all" && record.category !== filters.category) return false;
      if (filters.operation !== "all" && record.operation !== filters.operation) return false;
      if (filters.distribution !== "all" && record.distribution !== filters.distribution) return false;
      if (filters.payload !== "all" && record.payload !== filters.payload) return false;
      return searchMatchesTrend(record, search);
    });
  }, [dataset, filtered, filters.category, filters.distribution, filters.operation, filters.payload, filters.search]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  useEffect(() => {
    if (typeof navigator === "undefined") return;
    const isApplePlatform = /Mac|iPhone|iPad|iPod/.test(navigator.platform);
    setShortcutHint(isApplePlatform ? "Cmd + K" : "Ctrl + K");
  }, []);

  const commandSuggestions = useMemo(() => {
    const byImpl = baseOptions.implementations.map((implementation) => `Implementation: ${implementation}`);
    const families = baseOptions.implementations
      .map((implementation) => inferImplementationFamily(implementation))
      .filter((value) => value.length > 0)
      .map((family) => `Family: ${family}`);

    return [...new Set([...byImpl, ...families])].sort((a, b) => a.localeCompare(b));
  }, [baseOptions.implementations]);

  const pinnedComparisonRows = useMemo(() => {
    if (!dataset || selectedImplementations.length < 2) return [];

    const aggregateByImplementation = new Map(
      buildImplementationAggregates(filtered, dataset).map((item) => [item.implementation, item] as const),
    );

    const compared = selectedImplementations
      .map((implementation) => aggregateByImplementation.get(implementation))
      .filter((item): item is NonNullable<typeof item> => Boolean(item))
      .slice(0, 4);

    const baseline = compared[0];
    if (!baseline || compared.length < 2) return [];

    return compared.map((item, index) => {
      const deltaSpeed = toPercentChange(item.meanNs, baseline.meanNs);
      const deltaMemory = toPercentChange(item.estimatedMemoryBytes, baseline.estimatedMemoryBytes);

      return {
        implementation: item.implementation,
        meanNs: item.meanNs,
        estimatedMemoryBytes: item.estimatedMemoryBytes,
        speedLabel:
          index === 0
            ? "baseline"
            : deltaSpeed < 0
              ? `${Math.abs(deltaSpeed).toFixed(1)}% faster`
              : `${deltaSpeed.toFixed(1)}% slower`,
        memoryLabel:
          index === 0
            ? "baseline"
            : deltaMemory < 0
              ? `${Math.abs(deltaMemory).toFixed(1)}% less memory`
              : `${deltaMemory.toFixed(1)}% more memory`,
      };
    });
  }, [dataset, filtered, selectedImplementations]);

  const shareUrl = useMemo(() => {
    if (typeof window === "undefined") return pathname;

    const query = serializeUrlState(filters, selectedImplementations, {
      matrixScale: matrixScaleMode,
      hideOutliers: hideMatrixOutliers,
      showAdvancedFilters: false,
    });
    const url = new URL(window.location.origin + pathname);
    url.search = query;
    return url.toString();
  }, [filters, hideMatrixOutliers, matrixScaleMode, pathname, selectedImplementations]);

  const searchParamString = searchParams.toString();

  useEffect(() => {
    if (status !== "ready") return;

    const currentQuery = searchParamString;

    if (!syncRef.current.hydrated) {
      const parsed = parseUrlState(new URLSearchParams(currentQuery), baseOptions);
      setFilters(parsed.filters);
      setSelectedImplementations(parsed.selectedImplementations);
      setMatrixScaleMode(parsed.view.matrixScale);
      setHideMatrixOutliers(parsed.view.hideOutliers);
      syncRef.current.hydrated = true;
      syncRef.current.lastQuery = currentQuery;
      return;
    }

    if (syncRef.current.isInternalUpdate && syncRef.current.lastQuery === currentQuery) {
      syncRef.current.isInternalUpdate = false;
      return;
    }

    if (syncRef.current.lastQuery !== currentQuery) {
      const parsed = parseUrlState(new URLSearchParams(currentQuery), baseOptions);
      setFilters(parsed.filters);
      setSelectedImplementations(parsed.selectedImplementations);
      setMatrixScaleMode(parsed.view.matrixScale);
      setHideMatrixOutliers(parsed.view.hideOutliers);
      syncRef.current.lastQuery = currentQuery;
    }
  }, [
    baseOptions,
    searchParamString,
    setFilters,
    setSelectedImplementations,
    setMatrixScaleMode,
    setHideMatrixOutliers,
    status,
  ]);

  useEffect(() => {
    if (status !== "ready" || !syncRef.current.hydrated) return;

    const desiredQuery = serializeUrlState(filters, selectedImplementations, {
      matrixScale: matrixScaleMode,
      hideOutliers: hideMatrixOutliers,
      showAdvancedFilters: false,
    });
    const currentQuery = searchParamString;

    if (desiredQuery === currentQuery) {
      syncRef.current.lastQuery = currentQuery;
      return;
    }

    if (debounceRef.current != null) {
      window.clearTimeout(debounceRef.current);
    }

    debounceRef.current = window.setTimeout(() => {
      syncRef.current.isInternalUpdate = true;
      syncRef.current.lastQuery = desiredQuery;
      const href = desiredQuery.length > 0 ? `${pathname}?${desiredQuery}` : pathname;
      router.replace(href, { scroll: false });
    }, 140);

    return () => {
      if (debounceRef.current != null) {
        window.clearTimeout(debounceRef.current);
      }
    };
  }, [
    filters,
    hideMatrixOutliers,
    matrixScaleMode,
    pathname,
    router,
    searchParamString,
    selectedImplementations,
    status,
  ]);

  function resetView() {
    clearFilters();
    setFilters(STEP1_DEFAULT_FILTERS);
    setSelectedImplementations([]);
    setMatrixScaleMode("log");
    setHideMatrixOutliers(true);
  }

  const allRows = dataset?.records ?? filtered;
  const inspectorActive = Boolean(dataset) && selectedImplementations.length > 0;

  useEffect(() => {
    if (!inspectorActive) {
      inspectorWasActiveRef.current = false;
      return;
    }

    if (inspectorWasActiveRef.current) return;
    inspectorWasActiveRef.current = true;

    const timerId = window.setTimeout(() => {
      window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
    }, 100);

    return () => window.clearTimeout(timerId);
  }, [inspectorActive]);

  return (
    <main className="mx-auto flex min-h-screen w-full max-w-[1400px] flex-col gap-3 px-4 pb-8 pt-4 sm:px-5 sm:pb-10 sm:pt-5 lg:px-6 lg:pb-12 lg:pt-6">
      <CommandPalette
        open={isPaletteOpen}
        onOpenChange={setPaletteOpen}
        suggestions={commandSuggestions}
        shortcutHint={shortcutHint}
        onSelect={(value) => {
          const normalized = value.startsWith("Family: ")
            ? value.slice(8)
            : value.startsWith("Implementation: ")
              ? value.slice(16)
              : value;
          setFilter("search", normalized);
        }}
      />
      <motion.section
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="sticky top-0 z-50 -mx-2 shrink-0 rounded-lg border border-panel-border/60 bg-bg/80 px-2 py-2 backdrop-blur-md sm:-mx-3 sm:px-3"
      >
        {status !== "ready" ? (
          <AppSkeleton className="h-[110px] w-full" />
        ) : (
          <ControlRibbon
            filters={filters}
            categoryOptions={contextualFilterOptions.categories}
            operationOptions={contextualFilterOptions.operations}
            distributionOptions={contextualFilterOptions.distributions}
            payloadOptions={contextualFilterOptions.payloads}
            sizeOptions={contextualFilterOptions.sizes}
            shortcutHint={shortcutHint}
            onFilterChange={setFilter}
            onOpenPalette={() => setPaletteOpen(true)}
            onResetView={resetView}
            getShareUrl={() => shareUrl}
            onExportVisibleCsv={async () => copyOrDownload(leaderboardToCsv(filtered), "leaderboard-visible.csv")}
            onExportVisibleMarkdown={async () =>
              copyOrDownload(leaderboardToMarkdown(filtered), "leaderboard-visible.md")
            }
            onExportPinnedCsv={async () => copyOrDownload(comparisonToCsv(pinnedComparisonRows), "comparison-pinned.csv")}
            onExportPinnedMarkdown={async () =>
              copyOrDownload(comparisonToMarkdown(pinnedComparisonRows), "comparison-pinned.md")
            }
            hasPinnedComparison={pinnedComparisonRows.length >= 2}
            onExportAllCsv={async () => copyOrDownload(leaderboardToCsv(allRows), "leaderboard-all.csv")}
            onExportAllMarkdown={async () => copyOrDownload(leaderboardToMarkdown(allRows), "leaderboard-all.md")}
          />
        )}
      </motion.section>

      {status === "error" ? (
        <AppGlassPanel className="rounded-md border border-danger/40 bg-danger/10 p-4 text-sm text-text">
          <p className="font-semibold text-danger">Data loading failed</p>
          <p className="mt-2 text-text-muted">{error}</p>
          <p className="mt-2 text-text-muted">
            Ensure <span className="font-mono">frontend/public/aggregated_benchmarks.json</span> exists.
          </p>
        </AppGlassPanel>
      ) : null}

      {status === "ready" && (
        <div className="flex flex-col gap-3">
          <motion.section
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.45, delay: 0.05, ease: "easeOut" }}
            className="grid min-h-[600px] gap-3 lg:grid-cols-[1.25fr_1fr]"
          >
            {dataset ? (
              <div className="min-h-[600px]">
                <TradeoffMatrix
                  records={filtered}
                  dataset={dataset}
                  selectedImplementations={selectedImplementations}
                  onToggleImplementation={toggleImplementation}
                  scaleMode={matrixScaleMode}
                  hideOutliers={hideMatrixOutliers}
                  onScaleModeChange={setMatrixScaleMode}
                  onHideOutliersChange={setHideMatrixOutliers}
                />
              </div>
            ) : null}

            <div className="min-h-[600px]">
              <LeaderboardTable
                records={filtered}
                trendRecords={trendRecords}
                selectedImplementations={selectedImplementations}
                onToggleImplementation={toggleImplementation}
              />
            </div>
          </motion.section>

          <AnimatePresence initial={false}>
            {inspectorActive && dataset ? (
              <motion.section
                key="inspector"
                initial={{ opacity: 0, y: 50 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 40 }}
                transition={{ duration: 0.32, ease: "easeOut" }}
                className="min-h-[360px]"
              >
              <InspectorPane
                records={filtered}
                trendRecords={trendRecords}
                dataset={dataset}
                selectedImplementations={selectedImplementations}
                onResetComparison={resetComparison}
              />
              </motion.section>
            ) : null}
          </AnimatePresence>
        </div>
      )}
    </main>
  );
}

export default function HomePage() {
  return (
    <Suspense
      fallback={
        <main className="mx-auto flex min-h-screen w-full max-w-[1400px] flex-col gap-3 px-4 pb-8 pt-4 sm:px-5 sm:pb-10 sm:pt-5 lg:px-6 lg:pb-12 lg:pt-6">
          <div className="grid gap-3">
            <AppSkeleton className="h-[110px] w-full" />
            <AppSkeleton className="h-[600px] w-full" />
          </div>
        </main>
      }
    >
      <HomePageContent />
    </Suspense>
  );
}
