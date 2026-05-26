"use client";

import { memo, useMemo } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import { buildImplementationAggregates, formatBytes } from "@/lib/bench/analytics";
import { CHART_COLORS, VARIANT_COLORS as GLOBAL_VARIANT_COLORS } from "@/lib/bench/visualTokens";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";

interface VariantDrilldownProps {
  records: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset;
  selectedImplementations: string[];
}

type VariantKey = "safe" | "raw" | "arena";

type RadarDatum = {
  axis: string;
  safe: number | null;
  raw: number | null;
  arena: number | null;
};

const VARIANT_COLORS: Record<VariantKey, string> = {
  safe: GLOBAL_VARIANT_COLORS.safe,
  raw: GLOBAL_VARIANT_COLORS.raw,
  arena: GLOBAL_VARIANT_COLORS.arena,
};

function familyKey(implementation: string): string {
  return implementation
    .replace(/^(contains_zipf_|contains_temporal_|contains_mixed_|contains_|insert_|remove_|mix_|thrash_|push_pop_|workload_read_heavy_|workload_write_heavy_)/, "")
    .replace(/(^|_)safe(_|$)/g, "_")
    .replace(/(^|_)raw(_|$)/g, "_")
    .replace(/(^|_)arena(_|$)/g, "_")
    .replace(/(^|_)std(_|$)/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "")
    .replace(/_t\d+$/g, "");
}

function normalizeSpeed(value: number, min: number, max: number): number {
  if (max <= min) return 1;
  const normalized = (value - min) / (max - min);
  return 1 - normalized;
}

function scoreOrNull(value: number | null, min: number, max: number): number | null {
  if (value == null) return null;
  return normalizeSpeed(value, min, max);
}

function clamp01(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, value));
}

function formatRadarScore(value: number): string {
  return `${(clamp01(value) * 100).toFixed(1)}%`;
}

function formatCompactTick(value: number): string {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(0)}k`;
  return Math.round(value).toLocaleString();
}

type LatencyScale = {
  unit: "ns" | "us" | "ms" | "s";
  divisor: number;
};

function pickLatencyScale(maxNs: number): LatencyScale {
  if (maxNs >= 1_000_000_000) return { unit: "s", divisor: 1_000_000_000 };
  if (maxNs >= 1_000_000) return { unit: "ms", divisor: 1_000_000 };
  if (maxNs >= 1_000) return { unit: "us", divisor: 1_000 };
  return { unit: "ns", divisor: 1 };
}

function formatLatencyWithScale(valueNs: number, scale: LatencyScale): string {
  const scaled = valueNs / scale.divisor;
  if (scaled >= 100) return `${scaled.toFixed(0)} ${scale.unit}`;
  if (scaled >= 10) return `${scaled.toFixed(1)} ${scale.unit}`;
  return `${scaled.toFixed(2)} ${scale.unit}`;
}

function formatLatencyTickWithScale(valueNs: number, scale: LatencyScale): string {
  const scaled = valueNs / scale.divisor;
  if (scaled >= 100) return scaled.toFixed(0);
  if (scaled >= 10) return scaled.toFixed(1);
  return scaled.toFixed(2);
}

function cacheMissRate(metrics?: {
  Dr?: number;
  Dw?: number;
  D1mr?: number;
  D1mw?: number;
}): number | null {
  if (!metrics) return null;
  const reads = metrics.Dr ?? 0;
  const writes = metrics.Dw ?? 0;
  const misses = (metrics.D1mr ?? 0) + (metrics.D1mw ?? 0);
  const accesses = reads + writes;
  if (accesses <= 0) return null;
  return misses / accesses;
}

function variantFromImplementation(implementation: string): VariantKey | null {
  const normalized = implementation.toLowerCase();
  if (normalized.includes("_safe") || normalized.startsWith("safe_")) return "safe";
  if (normalized.includes("_raw") || normalized.startsWith("raw_")) return "raw";
  if (normalized.includes("_arena") || normalized.startsWith("arena_")) return "arena";
  return null;
}

function computeRange(values: number[], fallbackMin = 0, fallbackMax = 1): { min: number; max: number } {
  const finite = values.filter((value) => Number.isFinite(value));
  if (finite.length === 0) {
    return {
      min: fallbackMin,
      max: fallbackMax > fallbackMin ? fallbackMax : fallbackMin + 1,
    };
  }

  const min = Math.min(...finite);
  const max = Math.max(...finite);

  if (max <= min) {
    return { min, max: min + 1 };
  }

  return { min, max };
}

function metricValuesForVariants(
  variants: VariantKey[],
  getValue: (variant: VariantKey) => number | null,
): Array<number | null> {
  return variants.map((variant) => {
    const value = getValue(variant);
    return value != null && Number.isFinite(value) ? value : null;
  });
}

function allVariantsHaveMetric(values: Array<number | null>): values is number[] {
  return values.every((value) => value != null);
}

function formatOperationAxis(operation: string): string {
  if (operation === "push_pop") return "Push/Pop Latency";
  if (operation === "contains") return "Contains Latency";
  if (operation === "contains_zipf") return "Contains Latency (Zipf)";
  if (operation === "contains_temporal") return "Contains Latency (Temporal)";
  if (operation === "contains_mixed") return "Contains Latency (Mixed)";
  if (operation === "bulk_insert") return "Bulk Insert Latency";

  const base = operation
    .split("_")
    .map((part) => (part.length > 0 ? part[0].toUpperCase() + part.slice(1) : part))
    .join(" ");

  return `${base} Latency`;
}

function RadarTooltipContent({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: Array<{ name?: string; value?: number; color?: string }>;
  label?: string;
}) {
  if (!active || !payload || payload.length === 0) return null;

  const rows = payload.filter((entry) => typeof entry.value === "number");
  if (rows.length === 0) return null;

  return (
    <div className="min-w-[220px] rounded-lg border border-panel-border bg-panel/95 px-3 py-2.5 shadow-panel backdrop-blur-[8px]">
      <p className="text-xs uppercase tracking-[0.1em] text-text-muted">{label}</p>
      <div className="mt-2 space-y-1.5">
        {rows.map((entry) => (
          <div key={entry.name} className="grid grid-cols-[auto_1fr_auto] items-center gap-2 text-sm">
            <span
              className="h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: entry.color ?? "currentColor" }}
              aria-hidden="true"
            />
            <span className="capitalize text-text">{entry.name}</span>
            <span className="font-mono text-text-muted">{formatRadarScore(entry.value ?? 0)}</span>
          </div>
        ))}
      </div>
      <p className="mt-2 text-[11px] text-text-muted">Relative metric score. Higher is better.</p>
    </div>
  );
}

function LineTooltipContent({
  active,
  payload,
  label,
  latencyScale,
}: {
  active?: boolean;
  payload?: Array<{ dataKey?: string; name?: string; value?: number; color?: string }>;
  label?: string | number;
  latencyScale: LatencyScale;
}) {
  if (!active || !payload || payload.length === 0) return null;

  const rows = new Map<VariantKey, { mean: number | null; band: number | null; color: string }>();
  const ensure = (variant: VariantKey): { mean: number | null; band: number | null; color: string } => {
    const existing = rows.get(variant);
    if (existing) return existing;
    const created = { mean: null, band: null, color: VARIANT_COLORS[variant] };
    rows.set(variant, created);
    return created;
  };

  for (const entry of payload) {
    const key = entry.dataKey;
    if (!key) continue;

    if (key === "safe" || key === "raw" || key === "arena") {
      const row = ensure(key);
      row.mean = typeof entry.value === "number" ? entry.value : null;
      if (entry.color) row.color = entry.color;
      continue;
    }

    if (key === "safeBand" || key === "rawBand" || key === "arenaBand") {
      const variant = key.replace("Band", "") as VariantKey;
      const row = ensure(variant);
      row.band = typeof entry.value === "number" ? entry.value : null;
      if (entry.color) row.color = entry.color;
    }
  }

  const ordered = ("safe,raw,arena".split(",") as VariantKey[])
    .map((variant) => ({ variant, ...rows.get(variant) }))
    .filter((row) => row.mean != null || row.band != null);

  if (ordered.length === 0) return null;

  const sizeLabel = typeof label === "number" ? formatCompactTick(label) : String(label ?? "");

  return (
    <div className="min-w-[250px] rounded-lg border border-panel-border bg-panel/95 px-3 py-2.5 shadow-panel backdrop-blur-[8px]">
      <p className="text-xs uppercase tracking-[0.1em] text-text-muted">Input Size</p>
      <p className="font-mono text-sm text-text">{sizeLabel}</p>
      <div className="mt-2 space-y-1.5">
        {ordered.map((row) => (
          <div key={row.variant} className="space-y-0.5">
            <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2 text-sm">
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: row.color }} aria-hidden="true" />
              <span className="capitalize text-text">{row.variant}</span>
              <span className="font-mono text-text-muted">
                {row.mean != null ? formatLatencyWithScale(row.mean, latencyScale) : "n/a"}
              </span>
            </div>
            {row.band != null ? (
              <p className="pl-4 text-xs text-text-muted">CI span {formatLatencyWithScale(row.band, latencyScale)}</p>
            ) : null}
          </div>
        ))}
      </div>
    </div>
  );
}

function VariantDrilldownInner({ records, dataset, selectedImplementations }: VariantDrilldownProps) {
  const aggregates = useMemo(() => buildImplementationAggregates(records, dataset), [dataset, records]);

  const focusImplementation =
    selectedImplementations[0] ??
    aggregates.find((item) => item.variant === "safe")?.implementation ??
    aggregates[0]?.implementation;

  const focusFamily = focusImplementation ? familyKey(focusImplementation) : "";

  const variantRows = aggregates.filter((item) => {
    const key = familyKey(item.implementation);
    return key === focusFamily && (item.variant === "safe" || item.variant === "raw" || item.variant === "arena");
  });

  const variantByType = new Map<VariantKey, (typeof variantRows)[number]>();
  for (const row of variantRows) {
    if (row.variant === "safe" || row.variant === "raw" || row.variant === "arena") {
      variantByType.set(row.variant, row);
    }
  }

  const availableVariants = ["safe", "raw", "arena"].filter((variant) => variantByType.has(variant as VariantKey)) as VariantKey[];

  const familyRecords = useMemo(
    () =>
      records.filter(
        (record) =>
          familyKey(record.implementation) === focusFamily &&
          (record.variant === "safe" || record.variant === "raw" || record.variant === "arena"),
      ),
    [focusFamily, records],
  );

  const callgrindCoverage = useMemo(() => {
    const availableSizes = new Set<number>(familyRecords.map((record) => record.size));
    const byVariant = new Map<VariantKey, Set<number>>([
      ["safe", new Set<number>()],
      ["raw", new Set<number>()],
      ["arena", new Set<number>()],
    ]);

    for (const row of dataset.callgrind) {
      if (familyKey(row.implementation) !== focusFamily) continue;
      const variant = variantFromImplementation(row.implementation);
      if (!variant) continue;
      byVariant.get(variant)?.add(row.size);
    }

    return {
      totalSizes: availableSizes.size,
      safe: byVariant.get("safe")?.size ?? 0,
      raw: byVariant.get("raw")?.size ?? 0,
      arena: byVariant.get("arena")?.size ?? 0,
    };
  }, [dataset.callgrind, familyRecords, focusFamily]);

  const speedValues = variantRows.map((item) => item.meanNs);
  const memoryValues = variantRows.map((item) => item.estimatedMemoryBytes);

  const cacheEfficiencyByVariant = new Map<VariantKey, number>();
  for (const variant of ["safe", "raw", "arena"] as VariantKey[]) {
    const rate = cacheMissRate(variantByType.get(variant)?.callgrind);
    if (rate != null) {
      cacheEfficiencyByVariant.set(variant, 1 - rate);
    }
  }

  const cacheEfficiencyValues = [...cacheEfficiencyByVariant.values()];
  const jitterValues = variantRows.map((item) => item.stdDevNs);

  const speedRange = computeRange(speedValues, 0, 1);
  const memoryRange = computeRange(memoryValues, 0, 1);
  const jitterRange = computeRange(jitterValues, 0, 1);

  const operationByVariant = useMemo(() => {
    const operationCounts = new Map<string, number>();
    const variantBuckets = new Map<VariantKey, Map<string, { sum: number; count: number }>>();

    for (const variant of ["safe", "raw", "arena"] as VariantKey[]) {
      variantBuckets.set(variant, new Map<string, { sum: number; count: number }>());
    }

    for (const record of familyRecords) {
      const variant = record.variant as VariantKey;
      const operation = record.operation;
      operationCounts.set(operation, (operationCounts.get(operation) ?? 0) + 1);

      const bucket = variantBuckets.get(variant);
      if (!bucket) continue;
      const current = bucket.get(operation) ?? { sum: 0, count: 0 };
      current.sum += record.meanNs;
      current.count += 1;
      bucket.set(operation, current);
    }

    const means = new Map<VariantKey, Map<string, number>>();
    for (const variant of ["safe", "raw", "arena"] as VariantKey[]) {
      const bucket = variantBuckets.get(variant) ?? new Map<string, { sum: number; count: number }>();
      const values = new Map<string, number>();
      for (const [operation, stats] of bucket.entries()) {
        if (stats.count > 0) values.set(operation, stats.sum / stats.count);
      }
      means.set(variant, values);
    }

    const topOperations = [...operationCounts.entries()]
      .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
      .slice(0, 2)
      .map(([operation]) => operation);

    return {
      means,
      topOperations,
    };
  }, [familyRecords]);

  const radarData: RadarDatum[] = [];
  const omittedRadarAxes: string[] = [];

  for (const operation of operationByVariant.topOperations) {
    const perVariantValues = metricValuesForVariants(
      availableVariants,
      (variant) => operationByVariant.means.get(variant)?.get(operation) ?? null,
    );

    if (!allVariantsHaveMetric(perVariantValues)) {
      omittedRadarAxes.push(`${formatOperationAxis(operation)} (incomplete variant coverage)`);
      continue;
    }

    const values = perVariantValues;

    const range = computeRange(values, speedRange.min, speedRange.max);
    radarData.push({
      axis: `${formatOperationAxis(operation)} Score`,
      safe: scoreOrNull(operationByVariant.means.get("safe")?.get(operation) ?? null, range.min, range.max),
      raw: scoreOrNull(operationByVariant.means.get("raw")?.get(operation) ?? null, range.min, range.max),
      arena: scoreOrNull(operationByVariant.means.get("arena")?.get(operation) ?? null, range.min, range.max),
    });
  }

  if (radarData.length === 0) {
    radarData.push({
      axis: "Mean Latency Score",
      safe: scoreOrNull(variantByType.get("safe")?.meanNs ?? null, speedRange.min, speedRange.max),
      raw: scoreOrNull(variantByType.get("raw")?.meanNs ?? null, speedRange.min, speedRange.max),
      arena: scoreOrNull(variantByType.get("arena")?.meanNs ?? null, speedRange.min, speedRange.max),
    });
  }

  {
    const memoryPerVariant = metricValuesForVariants(
      availableVariants,
      (variant) => variantByType.get(variant)?.estimatedMemoryBytes ?? null,
    );

    if (allVariantsHaveMetric(memoryPerVariant)) {
      radarData.push({
        axis: "Memory Efficiency Score",
        safe: scoreOrNull(variantByType.get("safe")?.estimatedMemoryBytes ?? null, memoryRange.min, memoryRange.max),
        raw: scoreOrNull(variantByType.get("raw")?.estimatedMemoryBytes ?? null, memoryRange.min, memoryRange.max),
        arena: scoreOrNull(variantByType.get("arena")?.estimatedMemoryBytes ?? null, memoryRange.min, memoryRange.max),
      });
    } else {
      omittedRadarAxes.push("Memory (incomplete variant coverage)");
    }
  }

  if (cacheEfficiencyValues.length > 0) {
    const cachePerVariant = metricValuesForVariants(availableVariants, (variant) => cacheEfficiencyByVariant.get(variant) ?? null);
    if (allVariantsHaveMetric(cachePerVariant)) {
      radarData.push({
        axis: "L1 Hit-Rate Score",
        safe: clamp01(cacheEfficiencyByVariant.get("safe") ?? 0),
        raw: clamp01(cacheEfficiencyByVariant.get("raw") ?? 0),
        arena: clamp01(cacheEfficiencyByVariant.get("arena") ?? 0),
      });
    } else {
      omittedRadarAxes.push("L1 Hit Rate (missing callgrind coverage)");
    }
  }

  if (jitterValues.length >= 2) {
    const jitterPerVariant = metricValuesForVariants(availableVariants, (variant) => variantByType.get(variant)?.stdDevNs ?? null);
    if (allVariantsHaveMetric(jitterPerVariant)) {
      radarData.push({
        axis: "Latency Stability Score",
        safe: scoreOrNull(variantByType.get("safe")?.stdDevNs ?? null, jitterRange.min, jitterRange.max),
        raw: scoreOrNull(variantByType.get("raw")?.stdDevNs ?? null, jitterRange.min, jitterRange.max),
        arena: scoreOrNull(variantByType.get("arena")?.stdDevNs ?? null, jitterRange.min, jitterRange.max),
      });
    } else {
      omittedRadarAxes.push("Latency Stability (incomplete variant coverage)");
    }
  }

  const lineData = useMemo(() => {
    const byVariantAndSize = new Map<string, Map<number, { sum: number; lowerSum: number; upperSum: number; count: number }>>();

    for (const record of familyRecords) {
      const bucketKey = record.variant;
      const bucket = byVariantAndSize.get(bucketKey) ?? new Map<number, { sum: number; lowerSum: number; upperSum: number; count: number }>();
      const current = bucket.get(record.size) ?? { sum: 0, lowerSum: 0, upperSum: 0, count: 0 };
      current.sum += record.meanNs;
      current.lowerSum += record.ciLowerNs;
      current.upperSum += record.ciUpperNs;
      current.count += 1;
      bucket.set(record.size, current);
      byVariantAndSize.set(bucketKey, bucket);
    }

    const sizes = [...new Set(familyRecords.map((record) => record.size))].sort((a, b) => a - b);
    return sizes.map((size) => ({
      size,
      safe: (() => {
        const entry = byVariantAndSize.get("safe")?.get(size);
        return entry ? entry.sum / entry.count : null;
      })(),
      safeLower: (() => {
        const entry = byVariantAndSize.get("safe")?.get(size);
        return entry ? entry.lowerSum / entry.count : null;
      })(),
      safeUpper: (() => {
        const entry = byVariantAndSize.get("safe")?.get(size);
        return entry ? entry.upperSum / entry.count : null;
      })(),
      safeBand: (() => {
        const entry = byVariantAndSize.get("safe")?.get(size);
        return entry ? Math.max(0, (entry.upperSum - entry.lowerSum) / entry.count) : null;
      })(),
      raw: (() => {
        const entry = byVariantAndSize.get("raw")?.get(size);
        return entry ? entry.sum / entry.count : null;
      })(),
      rawLower: (() => {
        const entry = byVariantAndSize.get("raw")?.get(size);
        return entry ? entry.lowerSum / entry.count : null;
      })(),
      rawUpper: (() => {
        const entry = byVariantAndSize.get("raw")?.get(size);
        return entry ? entry.upperSum / entry.count : null;
      })(),
      rawBand: (() => {
        const entry = byVariantAndSize.get("raw")?.get(size);
        return entry ? Math.max(0, (entry.upperSum - entry.lowerSum) / entry.count) : null;
      })(),
      arena: (() => {
        const entry = byVariantAndSize.get("arena")?.get(size);
        return entry ? entry.sum / entry.count : null;
      })(),
      arenaLower: (() => {
        const entry = byVariantAndSize.get("arena")?.get(size);
        return entry ? entry.lowerSum / entry.count : null;
      })(),
      arenaUpper: (() => {
        const entry = byVariantAndSize.get("arena")?.get(size);
        return entry ? entry.upperSum / entry.count : null;
      })(),
      arenaBand: (() => {
        const entry = byVariantAndSize.get("arena")?.get(size);
        return entry ? Math.max(0, (entry.upperSum - entry.lowerSum) / entry.count) : null;
      })(),
    }));
  }, [familyRecords]);

  const latencyScale = useMemo(() => {
    const latencyValues = lineData
      .flatMap((item) => [item.safe, item.safeLower, item.safeUpper, item.raw, item.rawLower, item.rawUpper, item.arena, item.arenaLower, item.arenaUpper])
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));
    const maxNs = latencyValues.length > 0 ? Math.max(...latencyValues) : 0;
    return pickLatencyScale(maxNs);
  }, [lineData]);

  const sizeTicks = useMemo(() => lineData.map((point) => point.size).sort((a, b) => a - b), [lineData]);

  if (!focusImplementation || availableVariants.length === 0) {
    return (
      <AppGlassPanel className="space-y-2">
        <h3 className="font-display text-xl">Variant Drilldown</h3>
        <p className="text-sm text-text-muted">Pin or filter to a structure family with safe/raw/arena variants.</p>
      </AppGlassPanel>
    );
  }

  return (
    <AppGlassPanel className="space-y-4">
      <div className="flex items-end justify-between gap-4">
        <div>
          <h3 className="font-display text-xl">Variant Micro-Drilldown</h3>
          <p className="text-sm text-text-muted">Focused family: {focusFamily}</p>
          <p className="text-xs text-text-muted/90">Axes are metric-only and derived from available benchmark operations for this family.</p>
          {omittedRadarAxes.length > 0 ? (
            <p className="text-xs text-text-muted/75">Radar omitted {omittedRadarAxes.length} axis{omittedRadarAxes.length === 1 ? "" : "es"} when metrics were missing for one or more visible variants.</p>
          ) : null}
          {callgrindCoverage.totalSizes > 0 ? (
            <p className="text-xs text-text-muted/75">
              Cache coverage (callgrind sizes): safe {callgrindCoverage.safe}/{callgrindCoverage.totalSizes}, raw {callgrindCoverage.raw}/{callgrindCoverage.totalSizes}, arena {callgrindCoverage.arena}/{callgrindCoverage.totalSizes}
            </p>
          ) : null}
        </div>
        <div className="text-sm text-text-muted">
          {availableVariants.map((variant) => {
            const item = variantByType.get(variant);
            return (
              <p key={variant}>
                {variant}: {item ? `${formatLatencyWithScale(item.meanNs, latencyScale)}, ${formatBytes(item.estimatedMemoryBytes)}` : "n/a"}
              </p>
            );
          })}
        </div>
      </div>

      <div className="grid gap-3 xl:grid-cols-2">
        <div className="rounded-md border border-panel-border bg-bg-elevated/65 p-2.5">
          <ResponsiveContainer width="100%" height={360}>
            <RadarChart data={radarData} outerRadius="66%" margin={{ top: 24, right: 26, bottom: 18, left: 26 }}>
              <PolarGrid stroke={CHART_COLORS.grid} />
              <PolarAngleAxis
                dataKey="axis"
                tick={{
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
              />
              <Tooltip content={<RadarTooltipContent />} />
              <Legend />
              {availableVariants.map((variant) => (
                <Radar
                  key={variant}
                  name={variant}
                  dataKey={variant}
                  stroke={VARIANT_COLORS[variant]}
                  fill={VARIANT_COLORS[variant]}
                  fillOpacity={0.18}
                  strokeWidth={2}
                  isAnimationActive
                />
              ))}
            </RadarChart>
          </ResponsiveContainer>
        </div>

        <div className="rounded-md border border-panel-border bg-bg-elevated/65 p-2.5">
          <ResponsiveContainer width="100%" height={360}>
            <ComposedChart data={lineData} margin={{ top: 10, right: 16, left: 40, bottom: 8 }}>
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
                tickFormatter={(value: number) => formatLatencyTickWithScale(value, latencyScale)}
                tick={{
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
                label={{
                  value: `Mean Latency (${latencyScale.unit})`,
                  angle: -90,
                  position: "insideLeft",
                  dx: -30,
                  fill: CHART_COLORS.label,
                  fontSize: 12,
                  fontFamily: "var(--font-ibm-plex-mono), monospace",
                }}
              />
              <Tooltip content={<LineTooltipContent latencyScale={latencyScale} />} />
              <Legend />
              {availableVariants.map((variant) => (
                <Area
                  key={`${variant}-ci-base`}
                  type="monotone"
                  dataKey={`${variant}Lower`}
                  stackId={`ci-${variant}`}
                  stroke="none"
                  fill="transparent"
                  connectNulls
                  isAnimationActive
                  legendType="none"
                />
              ))}
              {availableVariants.map((variant) => (
                <Area
                  key={`${variant}-ci-band`}
                  type="monotone"
                  dataKey={`${variant}Band`}
                  stackId={`ci-${variant}`}
                  name={`${variant} CI band`}
                  stroke="none"
                  fill={VARIANT_COLORS[variant]}
                  fillOpacity={0.16}
                  connectNulls
                  isAnimationActive
                  legendType="none"
                />
              ))}
              {availableVariants.map((variant) => (
                <Line
                  key={variant}
                  type="monotone"
                  dataKey={variant}
                  name={`${variant} mean`}
                  stroke={VARIANT_COLORS[variant]}
                  strokeWidth={2.2}
                  dot={false}
                  connectNulls
                  isAnimationActive
                />
              ))}
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      </div>
    </AppGlassPanel>
  );
}

export const VariantDrilldown = memo(VariantDrilldownInner);
