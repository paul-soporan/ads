"use client";

import { memo, useMemo, useState } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import { AppButton } from "@/components/ui/AppButton";
import { useToast } from "@/components/ui/ToastProvider";
import { buildImplementationAggregates, formatBytes, toPercentChange } from "@/lib/bench/analytics";
import { COMPARISON_COLOR_PALETTE } from "@/lib/bench/colorManager";
import { comparisonToCsv, comparisonToMarkdown, copyOrDownload } from "@/lib/bench/export";
import { useDashboardStore } from "@/lib/bench/store";
import { CHART_COLORS } from "@/lib/bench/visualTokens";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";

interface VersusComparisonProps {
  records: CriterionRecord[];
  trendRecords: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset;
  selectedImplementations: string[];
  onToggleImplementation: (implementation: string) => void;
}

type LatencyScale = {
  unit: "ns" | "us" | "ms" | "s";
  divisor: number;
};

type TrendMetricKey = "latency" | "memory" | "instructions" | "l1MissRate";

type TrendMetricOption = {
  key: TrendMetricKey;
  label: string;
  axisLabel: string;
  description: string;
  showBand: boolean;
};

type TrendSeries = {
  implementation: string;
  color: string;
};

type TrendPoint = {
  size: number;
  [key: string]: number | null;
};

const TREND_METRICS: TrendMetricOption[] = [
  {
    key: "latency",
    label: "Mean Time",
    axisLabel: "Mean Latency",
    description: "See how execution time changes as the input size grows.",
    showBand: true,
  },
  {
    key: "memory",
    label: "Memory",
    axisLabel: "Peak Memory",
    description: "See how much memory each implementation uses at each size.",
    showBand: false,
  },
  {
    key: "instructions",
    label: "Instructions",
    axisLabel: "Instructions (Ir)",
    description: "See how much work the CPU performs as size changes.",
    showBand: false,
  },
  {
    key: "l1MissRate",
    label: "L1 Miss",
    axisLabel: "L1 Data Miss Rate",
    description: "See how often data misses the fast cache layer.",
    showBand: false,
  },
];

function formatCompactTick(value: number): string {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(0)}k`;
  return Math.round(value).toLocaleString();
}

function pickLatencyScale(maxNs: number): LatencyScale {
  if (maxNs >= 1_000_000_000) return { unit: "s", divisor: 1_000_000_000 };
  if (maxNs >= 1_000_000) return { unit: "ms", divisor: 1_000_000 };
  if (maxNs >= 1_000) return { unit: "us", divisor: 1_000 };
  return { unit: "ns", divisor: 1 };
}

function formatLatencyTick(valueNs: number, scale: LatencyScale): string {
  const scaled = valueNs / scale.divisor;
  if (scaled >= 100) return scaled.toFixed(0);
  if (scaled >= 10) return scaled.toFixed(1);
  return scaled.toFixed(2);
}

function formatLatencyWithScale(valueNs: number, scale: LatencyScale): string {
  return `${formatLatencyTick(valueNs, scale)} ${scale.unit}`;
}

function formatCompactCount(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(2)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return Math.round(value).toLocaleString();
}

function formatMissRate(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";
  return `${(value * 100).toFixed(3)}%`;
}

function formatMetricValue(value: number | null, metric: TrendMetricKey, latencyScale: LatencyScale): string {
  if (value == null || !Number.isFinite(value)) return "n/a";

  if (metric === "latency") {
    return formatLatencyWithScale(value, latencyScale);
  }

  if (metric === "memory") {
    return formatBytes(value);
  }

  if (metric === "instructions") {
    return formatCompactCount(value);
  }

  return formatMissRate(value);
}

function formatMetricTick(value: number, metric: TrendMetricKey, latencyScale: LatencyScale): string {
  if (metric === "latency") return formatLatencyTick(value, latencyScale);
  if (metric === "memory") return formatCompactTick(value);
  if (metric === "instructions") return formatCompactTick(value);
  return `${(value * 100).toFixed(value >= 1 ? 0 : 1)}%`;
}

function trendMetricDataKey(metric: TrendMetricKey): "LatencyMean" | "Memory" | "Instructions" | "L1MissRate" {
  if (metric === "latency") return "LatencyMean";
  if (metric === "memory") return "Memory";
  if (metric === "instructions") return "Instructions";
  return "L1MissRate";
}

function trendMetricBandKey(metric: TrendMetricKey): "LatencyBand" | null {
  return metric === "latency" ? "LatencyBand" : null;
}

function lowerBetterDeltaLabel(current: number | null, baseline: number | null, noun: string): string {
  if (baseline == null || current == null) return "n/a";
  if (baseline === 0) return "n/a";

  const delta = toPercentChange(current, baseline);
  if (Math.abs(delta) < 0.05) return "no material change";
  if (delta < 0) return `${Math.abs(delta).toFixed(1)}% lower ${noun}`;
  return `${delta.toFixed(1)}% higher ${noun}`;
}

function sanitizeSeriesKey(implementation: string): string {
  return implementation.replace(/[^a-zA-Z0-9_]/g, "_");
}

function VersusLineTooltipContent({
  active,
  payload,
  label,
  latencyScale,
  comparedSeries,
  metric,
}: {
  active?: boolean;
  payload?: Array<{ dataKey?: string; value?: number }>;
  label?: string | number;
  latencyScale: LatencyScale;
  comparedSeries: Array<TrendSeries>;
  metric: TrendMetricKey;
}) {
  if (!active || !payload || payload.length === 0) return null;

  const dataKey = trendMetricDataKey(metric);
  const bandKey = trendMetricBandKey(metric);

  const bySeries = new Map<string, { mean: number | null; band: number | null }>();
  for (const series of comparedSeries) {
    bySeries.set(series.implementation, { mean: null, band: null });
  }

  for (const entry of payload) {
    const key = entry.dataKey;
    if (!key) continue;

    for (const series of comparedSeries) {
      const seriesKey = sanitizeSeriesKey(series.implementation);
      const row = bySeries.get(series.implementation);
      if (!row) continue;

      if (key === `${seriesKey}${dataKey}`) {
        row.mean = typeof entry.value === "number" ? entry.value : null;
      }

      if (bandKey && key === `${seriesKey}${bandKey}`) {
        row.band = typeof entry.value === "number" ? entry.value : null;
      }
    }
  }

  const rows = comparedSeries
    .map((series) => ({ series, ...bySeries.get(series.implementation) }))
    .filter((row) => row.mean != null || row.band != null);

  if (rows.length === 0) return null;

  const sizeLabel = typeof label === "number" ? formatCompactTick(label) : String(label ?? "");

  return (
    <div className="min-w-[260px] rounded-lg border border-panel-border bg-panel/95 px-3 py-2.5 shadow-panel backdrop-blur-[8px]">
      <p className="text-xs uppercase tracking-[0.1em] text-text-muted">Input Size</p>
      <p className="font-mono text-sm text-text">{sizeLabel}</p>
      <div className="mt-2 space-y-1.5">
        {rows.map((row) => (
          <div key={row.series.implementation} className="space-y-0.5">
            <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2 text-sm">
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: row.series.color }} aria-hidden="true" />
              <span className="font-mono text-text">{row.series.implementation}</span>
              <span className="font-mono text-text-muted">
                {row.mean != null ? formatMetricValue(row.mean, metric, latencyScale) : "n/a"}
              </span>
            </div>
            {row.band != null ? (
              <p className="pl-4 text-xs text-text-muted">CI span {formatMetricValue(row.band, metric, latencyScale)}</p>
            ) : null}
          </div>
        ))}
      </div>
    </div>
  );
}

function VersusComparisonInner({
  records,
  trendRecords,
  dataset,
  selectedImplementations,
  onToggleImplementation,
}: VersusComparisonProps) {
  const [exportScope, setExportScope] = useState<"compared" | "top">("compared");
  const [exportFormat, setExportFormat] = useState<"csv" | "markdown">("markdown");
  const [trendMetric, setTrendMetric] = useState<TrendMetricKey>("latency");
  const pushToast = useToast();
  const comparisonColors = useDashboardStore((state) => state.comparisonColors);
  const aggregates = useMemo(() => buildImplementationAggregates(records, dataset), [dataset, records]);

  const profilingLookups = useMemo(() => {
    const callgrindBuckets = new Map<string, Record<string, number>[]>().set("__seed__", []);
    callgrindBuckets.delete("__seed__");
    const dhatBuckets = new Map<string, Array<{ peakBytes: number | null }>>();

    for (const row of dataset.callgrind) {
      const bucket = callgrindBuckets.get(row.joinKey) ?? [];
      bucket.push(row.metrics);
      callgrindBuckets.set(row.joinKey, bucket);
    }

    for (const row of dataset.dhat) {
      const bucket = dhatBuckets.get(row.joinKey) ?? [];
      bucket.push({ peakBytes: row.maxBytes ?? row.totalBytes });
      dhatBuckets.set(row.joinKey, bucket);
    }

    const averageNumberMaps = (input: Map<string, Record<string, number>[]>) => {
      const output = new Map<string, Record<string, number>>();
      for (const [joinKey, metricsList] of input.entries()) {
        const totals = new Map<string, number>();
        const counts = new Map<string, number>();

        for (const metrics of metricsList) {
          for (const [key, value] of Object.entries(metrics)) {
            if (!Number.isFinite(value)) continue;
            totals.set(key, (totals.get(key) ?? 0) + value);
            counts.set(key, (counts.get(key) ?? 0) + 1);
          }
        }

        const averaged: Record<string, number> = {};
        for (const [key, total] of totals.entries()) {
          const count = counts.get(key) ?? 1;
          averaged[key] = total / count;
        }

        output.set(joinKey, averaged);
      }
      return output;
    };

    const averageNullable = (values: Array<number | null | undefined>) => {
      const present = values.filter((value): value is number => typeof value === "number" && Number.isFinite(value));
      if (present.length === 0) return null;
      return present.reduce((sum, value) => sum + value, 0) / present.length;
    };

    const peakBytesByJoinKey = new Map<string, number | null>();
    for (const [joinKey, values] of dhatBuckets.entries()) {
      peakBytesByJoinKey.set(joinKey, averageNullable(values.map((value) => value.peakBytes)));
    }

    return {
      callgrindByJoinKey: averageNumberMaps(callgrindBuckets),
      peakBytesByJoinKey,
    };
  }, [dataset.callgrind, dataset.dhat]);

  const compared = useMemo(() => {
    if (selectedImplementations.length > 0) {
      const aggregateByImplementation = new Map(aggregates.map((item) => [item.implementation, item] as const));
      return selectedImplementations
        .map((implementation) => aggregateByImplementation.get(implementation))
        .filter((item): item is (typeof aggregates)[number] => Boolean(item))
        .slice(0, 4);
    }

    return aggregates.slice(0, 4);
  }, [aggregates, selectedImplementations]);

  const baseline = compared[0];

  const comparedSeries = useMemo(
    () =>
      compared.map((item, index) => ({
        implementation: item.implementation,
        color: comparisonColors[item.implementation] ?? COMPARISON_COLOR_PALETTE[index % COMPARISON_COLOR_PALETTE.length],
      })),
    [compared, comparisonColors],
  );

  const colorByImplementation = useMemo(
    () => new Map(comparedSeries.map((series) => [series.implementation, series.color] as const)),
    [comparedSeries],
  );

  const versusLineData = useMemo(() => {
    const trendSource = trendRecords.length > 0 ? trendRecords : records;
    const included = new Set(compared.map((item) => item.implementation));
    const bySeriesAndSize = new Map<string, Map<number, {
      latencySum: number;
      latencyLowerSum: number;
      latencyUpperSum: number;
      latencyCount: number;
      memorySum: number;
      memoryCount: number;
      instructionsSum: number;
      instructionsCount: number;
      l1MissSum: number;
      l1MissCount: number;
    }>>();

    const getBucket = (seriesKey: string, size: number) => {
      const bySize = bySeriesAndSize.get(seriesKey) ?? new Map<number, {
        latencySum: number;
        latencyLowerSum: number;
        latencyUpperSum: number;
        latencyCount: number;
        memorySum: number;
        memoryCount: number;
        instructionsSum: number;
        instructionsCount: number;
        l1MissSum: number;
        l1MissCount: number;
      }>();
      const current = bySize.get(size) ?? {
        latencySum: 0,
        latencyLowerSum: 0,
        latencyUpperSum: 0,
        latencyCount: 0,
        memorySum: 0,
        memoryCount: 0,
        instructionsSum: 0,
        instructionsCount: 0,
        l1MissSum: 0,
        l1MissCount: 0,
      };
      return { bySize, current };
    };

    for (const row of trendSource) {
      if (!included.has(row.implementation)) continue;

      const seriesKey = sanitizeSeriesKey(row.implementation);
      const { bySize, current } = getBucket(seriesKey, row.size);
      current.latencySum += row.meanNs;
      current.latencyLowerSum += row.ciLowerNs;
      current.latencyUpperSum += row.ciUpperNs;
      current.latencyCount += 1;

      const peakBytes = profilingLookups.peakBytesByJoinKey.get(row.joinKey);
      if (peakBytes != null && Number.isFinite(peakBytes)) {
        current.memorySum += peakBytes;
        current.memoryCount += 1;
      }

      const callgrind = profilingLookups.callgrindByJoinKey.get(row.joinKey);
      const instructions = callgrind?.Ir;
      if (instructions != null && Number.isFinite(instructions)) {
        current.instructionsSum += instructions;
        current.instructionsCount += 1;
      }

      const dr = callgrind?.Dr;
      const dw = callgrind?.Dw;
      const d1mr = callgrind?.D1mr;
      const d1mw = callgrind?.D1mw;
      if (
        dr != null && dw != null && d1mr != null && d1mw != null &&
        Number.isFinite(dr) && Number.isFinite(dw) && Number.isFinite(d1mr) && Number.isFinite(d1mw)
      ) {
        const accesses = dr + dw;
        if (accesses > 0) {
          current.l1MissSum += (d1mr + d1mw) / accesses;
          current.l1MissCount += 1;
        }
      }

      bySize.set(row.size, current);
      bySeriesAndSize.set(seriesKey, bySize);
    }

    const sizes = [...new Set(trendSource.filter((row) => included.has(row.implementation)).map((row) => row.size))].sort((a, b) => a - b);

    return sizes.map((size) => {
      const row: TrendPoint = { size };
      for (const series of comparedSeries) {
        const key = sanitizeSeriesKey(series.implementation);
        const stats = bySeriesAndSize.get(key)?.get(size);
        row[`${key}LatencyMean`] = stats && stats.latencyCount > 0 ? stats.latencySum / stats.latencyCount : null;
        row[`${key}LatencyLower`] = stats && stats.latencyCount > 0 ? stats.latencyLowerSum / stats.latencyCount : null;
        row[`${key}LatencyBand`] = stats && stats.latencyCount > 0 ? Math.max(0, (stats.latencyUpperSum - stats.latencyLowerSum) / stats.latencyCount) : null;
        row[`${key}Memory`] = stats && stats.memoryCount > 0 ? stats.memorySum / stats.memoryCount : null;
        row[`${key}Instructions`] = stats && stats.instructionsCount > 0 ? stats.instructionsSum / stats.instructionsCount : null;
        row[`${key}L1MissRate`] = stats && stats.l1MissCount > 0 ? stats.l1MissSum / stats.l1MissCount : null;
      }
      return row;
    });
  }, [compared, comparedSeries, profilingLookups.callgrindByJoinKey, profilingLookups.peakBytesByJoinKey, records, trendRecords]);

  const sizeTicks = useMemo(() => versusLineData.map((item) => item.size).sort((a, b) => a - b), [versusLineData]);

  const latencyScale = useMemo(() => {
    const values = versusLineData
      .flatMap((row) =>
        comparedSeries.flatMap((series) => {
          const key = sanitizeSeriesKey(series.implementation);
          return [row[`${key}Mean`], row[`${key}Lower`], row[`${key}Band`]];
        }),
      )
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));

    const maxNs = values.length > 0 ? Math.max(...values) : 0;
    return pickLatencyScale(maxNs);
  }, [comparedSeries, versusLineData]);

  const selectedTrendMetric = TREND_METRICS.find((option) => option.key === trendMetric) ?? TREND_METRICS[0];

  const selectedTrendMax = useMemo(() => {
    const dataKey = trendMetricDataKey(trendMetric);
    const values = versusLineData
      .flatMap((row) => comparedSeries.map((series) => row[`${sanitizeSeriesKey(series.implementation)}${dataKey}`]))
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));

    return values.length > 0 ? Math.max(...values) : 0;
  }, [comparedSeries, trendMetric, versusLineData]);
  const comparedExportRows = compared.map((item, index) => {
    const deltaSpeed = toPercentChange(item.meanNs, baseline?.meanNs ?? item.meanNs);
    const deltaMemory = toPercentChange(item.estimatedMemoryBytes, baseline?.estimatedMemoryBytes ?? item.estimatedMemoryBytes);
    const instructionLabel =
      index === 0
        ? "baseline"
        : lowerBetterDeltaLabel(item.profiling.instructions, baseline?.profiling.instructions ?? null, "instructions");
    const l1MissRateLabel =
      index === 0
        ? "baseline"
        : lowerBetterDeltaLabel(item.profiling.l1DataMissRate, baseline?.profiling.l1DataMissRate ?? null, "L1 data miss rate");

    return {
      implementation: item.implementation,
      meanNs: item.meanNs,
      estimatedMemoryBytes: item.estimatedMemoryBytes,
      instructions: item.profiling.instructions,
      l1DataMissRate: item.profiling.l1DataMissRate,
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
      instructionLabel,
      l1DataMissRateLabel: l1MissRateLabel,
    };
  });

  const topExportRows = aggregates.slice(0, 12).map((item, index) => {
    const deltaSpeed = toPercentChange(item.meanNs, baseline?.meanNs ?? item.meanNs);
    const deltaMemory = toPercentChange(item.estimatedMemoryBytes, baseline?.estimatedMemoryBytes ?? item.estimatedMemoryBytes);
    const instructionLabel =
      index === 0
        ? "baseline"
        : lowerBetterDeltaLabel(item.profiling.instructions, baseline?.profiling.instructions ?? null, "instructions");
    const l1MissRateLabel =
      index === 0
        ? "baseline"
        : lowerBetterDeltaLabel(item.profiling.l1DataMissRate, baseline?.profiling.l1DataMissRate ?? null, "L1 data miss rate");

    return {
      implementation: item.implementation,
      meanNs: item.meanNs,
      estimatedMemoryBytes: item.estimatedMemoryBytes,
      instructions: item.profiling.instructions,
      l1DataMissRate: item.profiling.l1DataMissRate,
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
      instructionLabel,
      l1DataMissRateLabel: l1MissRateLabel,
    };
  });

  const exportRows = exportScope === "compared" ? comparedExportRows : topExportRows;

  async function exportVersusData() {
    const result =
      exportFormat === "csv"
        ? await copyOrDownload(comparisonToCsv(exportRows), "versus-comparison.csv")
        : await copyOrDownload(comparisonToMarkdown(exportRows), "versus-comparison.md");

    pushToast({
      title: "Export",
      message:
        result === "copied"
          ? `${exportFormat === "csv" ? "CSV" : "Markdown"} copied to clipboard.`
          : `${exportFormat === "csv" ? "CSV" : "Markdown"} downloaded.`,
      variant: "success",
    });
  }

  if (!baseline || compared.length < 2) {
    return (
      <AppGlassPanel className="space-y-2">
        <h3 className="font-display text-xl">Versus Comparison</h3>
        <p className="text-sm text-text-muted">
          Choose two or more implementations from the matrix or leaderboard to compare them side by side in the Inspector.
        </p>
      </AppGlassPanel>
    );
  }

  return (
    <AppGlassPanel className="space-y-4">
      <div className="flex items-end justify-between gap-4">
        <div>
          <h3 className="font-display text-xl">Versus Comparison</h3>
          <p className="text-sm text-text-muted">Baseline: {baseline.implementation}</p>
          <p className="text-xs text-text-muted/80">The first pinned implementation sets the baseline for the deltas shown below.</p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <div className="inline-flex rounded-md border border-panel-border bg-bg-elevated/70 p-1 text-sm">
            <button
              type="button"
              onClick={() => setExportScope("compared")}
              className={`rounded px-2 py-1 ${
                exportScope === "compared" ? "bg-primary/20 text-primary" : "text-text-muted hover:text-text"
              }`}
            >
              Pinned
            </button>
            <button
              type="button"
              onClick={() => setExportScope("top")}
              className={`rounded px-2 py-1 ${
                exportScope === "top" ? "bg-primary/20 text-primary" : "text-text-muted hover:text-text"
              }`}
            >
              Top 12
            </button>
          </div>
          <div className="inline-flex rounded-md border border-panel-border bg-bg-elevated/70 p-1 text-sm">
            <button
              type="button"
              onClick={() => setExportFormat("markdown")}
              className={`rounded px-2 py-1 ${
                exportFormat === "markdown" ? "bg-primary/20 text-primary" : "text-text-muted hover:text-text"
              }`}
            >
              MD
            </button>
            <button
              type="button"
              onClick={() => setExportFormat("csv")}
              className={`rounded px-2 py-1 ${
                exportFormat === "csv" ? "bg-primary/20 text-primary" : "text-text-muted hover:text-text"
              }`}
            >
              CSV
            </button>
          </div>
          <AppButton variant="ghost" size="sm" onClick={() => void exportVersusData()}>
            Export Versus Data
          </AppButton>
        </div>
      </div>

      <div className="grid gap-2.5 md:grid-cols-2 xl:grid-cols-4">
        {compared.map((item, index) => {
          const deltaSpeed = toPercentChange(item.meanNs, baseline.meanNs);
          const deltaMemory = toPercentChange(item.estimatedMemoryBytes, baseline.estimatedMemoryBytes);
          const speedTone = deltaSpeed < 0 ? "text-success" : deltaSpeed > 0 ? "text-danger" : "text-text-muted";
          const memoryTone = deltaMemory < 0 ? "text-success" : deltaMemory > 0 ? "text-danger" : "text-text-muted";
          const instructionLabel =
            index === 0
              ? "baseline"
              : lowerBetterDeltaLabel(item.profiling.instructions, baseline.profiling.instructions, "instructions");
          const instructionTone =
            index === 0
              ? "text-text-muted"
              : item.profiling.instructions != null && baseline.profiling.instructions != null && item.profiling.instructions < baseline.profiling.instructions
                ? "text-success"
                : item.profiling.instructions != null && baseline.profiling.instructions != null && item.profiling.instructions > baseline.profiling.instructions
                  ? "text-danger"
                  : "text-text-muted";
          const missRateLabel =
            index === 0
              ? "baseline"
              : lowerBetterDeltaLabel(item.profiling.l1DataMissRate, baseline.profiling.l1DataMissRate, "L1 data miss rate");
          const missRateTone =
            index === 0
              ? "text-text-muted"
              : item.profiling.l1DataMissRate != null && baseline.profiling.l1DataMissRate != null && item.profiling.l1DataMissRate < baseline.profiling.l1DataMissRate
                ? "text-success"
                : item.profiling.l1DataMissRate != null && baseline.profiling.l1DataMissRate != null && item.profiling.l1DataMissRate > baseline.profiling.l1DataMissRate
                  ? "text-danger"
                  : "text-text-muted";
          const cardColor =
            colorByImplementation.get(item.implementation) ?? COMPARISON_COLOR_PALETTE[index % COMPARISON_COLOR_PALETTE.length];
          const speedLabel =
            index === 0
              ? "baseline"
              : deltaSpeed < 0
                ? `${Math.abs(deltaSpeed).toFixed(1)}% faster`
                : `${deltaSpeed.toFixed(1)}% slower`;

          const memoryLabel =
            index === 0
              ? "baseline"
              : deltaMemory < 0
                ? `${Math.abs(deltaMemory).toFixed(1)}% less memory`
                : `${deltaMemory.toFixed(1)}% more memory`;

          return (
            <article
              key={item.implementation}
              className="relative rounded-md border bg-bg-elevated/70 p-3.5"
              style={{
                borderColor: `${cardColor}${index === 0 ? "bb" : "80"}`,
                boxShadow: `0 0 0 1px ${cardColor}${index === 0 ? "40" : "20"}`,
              }}
            >
              <button
                type="button"
                onClick={() => onToggleImplementation(item.implementation)}
                className="absolute right-2 top-2 rounded border border-panel-border/70 bg-panel/72 px-2 py-1 text-[11px] uppercase tracking-[0.08em] text-text-muted transition hover:border-primary/55 hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                aria-label={`Remove ${item.implementation} from the inspector`}
              >
                Unpin
              </button>
              <p
                className="font-mono text-sm uppercase tracking-[0.12em]"
                style={{ color: index === 0 ? cardColor : "rgba(204, 214, 236, 0.85)" }}
              >
                {item.implementation}
              </p>
              {index === 0 ? (
                <p className="mt-1 text-xs uppercase tracking-[0.12em]" style={{ color: cardColor }}>
                  Baseline (Locked)
                </p>
              ) : null}
              <p className="mt-2 font-mono text-lg [font-variant-numeric:tabular-nums]" style={{ color: cardColor }}>
                {Math.round(item.meanNs).toLocaleString()} ns
              </p>
              <p className={`text-sm ${index === 0 ? "text-text-muted" : speedTone}`}>{speedLabel}</p>
              <p className="mt-2 font-mono text-sm [font-variant-numeric:tabular-nums]" style={{ color: cardColor }}>
                {formatBytes(item.estimatedMemoryBytes)}
              </p>
              <p className={`text-sm ${index === 0 ? "text-text-muted" : memoryTone}`}>{memoryLabel}</p>

              <div className="mt-2 rounded border border-panel-border/65 bg-bg-elevated/35 px-2.5 py-2">
                <p className="font-mono text-xs text-text">
                  Ir: {formatCompactCount(item.profiling.instructions)}
                </p>
                <p className={`text-xs ${instructionTone}`}>{instructionLabel}</p>
                <p className="mt-1 font-mono text-xs text-text">
                  L1 data miss: {formatMissRate(item.profiling.l1DataMissRate)}
                </p>
                <p className={`text-xs ${missRateTone}`}>{missRateLabel}</p>
              </div>
            </article>
          );
        })}
      </div>

      <div className="rounded-md border border-panel-border bg-bg-elevated/65 p-2.5 space-y-3">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <p className="text-xs uppercase tracking-[0.12em] text-text-muted">Trend View</p>
            <p className="text-sm text-text">Choose which metric to compare across the pinned implementations.</p>
          </div>
          <div className="inline-flex flex-wrap gap-1 rounded-md border border-panel-border bg-bg-elevated/55 p-1 text-sm">
            {TREND_METRICS.map((option) => (
              <button
                key={option.key}
                type="button"
                onClick={() => setTrendMetric(option.key)}
                className={`rounded px-2 py-1 transition ${
                  trendMetric === option.key ? "bg-primary/20 text-primary" : "text-text-muted hover:text-text"
                }`}
                aria-pressed={trendMetric === option.key}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>

        {versusLineData.length > 0 ? (
          <ResponsiveContainer width="100%" height={352}>
            <ComposedChart data={versusLineData} margin={{ top: 10, right: 18, left: 48, bottom: 8 }}>
              <CartesianGrid strokeDasharray="4 6" stroke={CHART_COLORS.gridSubtle} />
              <XAxis
                dataKey="size"
                type="number"
                scale="linear"
                domain={["dataMin", "dataMax"]}
                ticks={sizeTicks}
                tickFormatter={(value: number) => formatCompactTick(Number(value))}
                tick={{
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
                label={{
                  value: "Input Size",
                  position: "insideBottom",
                  offset: -2,
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
              />
              <YAxis
                tickFormatter={(value: number) => formatMetricTick(value, trendMetric, latencyScale)}
                domain={
                  trendMetric === "latency"
                    ? ["dataMin", "dataMax"]
                    : [0, selectedTrendMax > 0 ? selectedTrendMax * 1.08 : 1]
                }
                tick={{
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
                label={{
                  value: selectedTrendMetric.axisLabel,
                  angle: -90,
                  position: "insideLeft",
                  dx: -30,
                  dy: 0,
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
              />
              <Tooltip
                content={
                  <VersusLineTooltipContent
                    latencyScale={latencyScale}
                    comparedSeries={comparedSeries}
                    metric={trendMetric}
                  />
                }
              />
              <Legend />
              {comparedSeries.map((series) => {
                const key = sanitizeSeriesKey(series.implementation);
                return trendMetric === "latency" ? (
                  <Area
                    key={`${key}-ci-base`}
                    type="monotone"
                    dataKey={`${key}LatencyLower`}
                    stackId={`ci-${key}`}
                    stroke="none"
                    fill="transparent"
                    connectNulls
                    isAnimationActive
                    legendType="none"
                  />
                ) : null;
              })}
              {trendMetric === "latency"
                ? comparedSeries.map((series) => {
                    const key = sanitizeSeriesKey(series.implementation);
                    return (
                      <Area
                        key={`${key}-ci-band`}
                        type="monotone"
                        dataKey={`${key}LatencyBand`}
                        stackId={`ci-${key}`}
                        name={`${series.implementation} CI band`}
                        stroke="none"
                        fill={series.color}
                        fillOpacity={0.15}
                        connectNulls
                        isAnimationActive
                        legendType="none"
                      />
                    );
                  })
                : null}
              {comparedSeries.map((series) => {
                const key = sanitizeSeriesKey(series.implementation);
                const dataKey = `${key}${trendMetricDataKey(trendMetric)}`;
                return (
                  <Line
                    key={`${key}-mean`}
                    type="monotone"
                    dataKey={dataKey}
                    name={series.implementation}
                    stroke={series.color}
                    strokeWidth={2.2}
                    dot={{ r: 2.5 }}
                    connectNulls
                    isAnimationActive
                  />
                );
              })}
            </ComposedChart>
          </ResponsiveContainer>
        ) : (
          <div className="flex h-[352px] items-center justify-center rounded-md border border-panel-border/70 bg-bg-elevated/35 p-4 text-center text-sm text-text-muted">
            No trend data is available for the current comparison.
          </div>
        )}

        <p className="text-xs text-text-muted/80">{selectedTrendMetric.description}</p>
      </div>
    </AppGlassPanel>
  );
}

export const VersusComparison = memo(VersusComparisonInner);
