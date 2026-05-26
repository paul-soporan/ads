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
import {
  buildImplementationAggregates,
  buildVariantRadarMetrics,
  formatBytes,
  type VariantRadarMetric,
} from "@/lib/bench/analytics";
import { CHART_COLORS, VARIANT_COLORS as GLOBAL_VARIANT_COLORS } from "@/lib/bench/visualTokens";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";

interface VariantDrilldownProps {
  snapshotRecords: CriterionRecord[];
  trendRecords: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset;
  selectedImplementations: string[];
}

type VariantKey = "safe" | "raw" | "arena";

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

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(3)}%`;
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

function formatVariantLabel(variant: VariantKey): string {
  return variant[0].toUpperCase() + variant.slice(1);
}

function variantFromImplementation(implementation: string): VariantKey | null {
  const normalized = implementation.toLowerCase();
  if (normalized.includes("_safe") || normalized.startsWith("safe_")) return "safe";
  if (normalized.includes("_raw") || normalized.startsWith("raw_")) return "raw";
  if (normalized.includes("_arena") || normalized.startsWith("arena_")) return "arena";
  return null;
}

function formatRadarRawValue(metric: VariantRadarMetric, value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";

  switch (metric.formatter) {
    case "latency_ns": {
      const scale = pickLatencyScale(value);
      return formatLatencyWithScale(value, scale);
    }
    case "ratio":
      return `${(value * 100).toFixed(2)}%`;
    case "bytes_per_element":
      return `${value >= 1024 ? formatBytes(value) : `${value.toFixed(2)} B`}/elem`;
    case "instructions_per_element":
      return `${value >= 1000 ? formatCompactTick(value) : value.toFixed(2)} Ir/elem`;
    case "allocation_ratio":
      return `${value.toFixed(2)}x`;
    default:
      return value.toFixed(2);
  }
}

function formatRadarBand(value: number): string {
  const score = clamp01(value);
  if (score >= 0.8) return "Leading";
  if (score >= 0.62) return "Strong";
  if (score >= 0.38) return "Mid-pack";
  return "Trailing";
}

function rawMetricValueForVariant(metric: VariantRadarMetric, variant: string | undefined): number | null {
  if (variant === "safe") return metric.safeRaw;
  if (variant === "raw") return metric.rawRaw;
  if (variant === "arena") return metric.arenaRaw;
  return null;
}

function RadarTooltipContent({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: Array<{ name?: string; value?: number; color?: string; payload?: VariantRadarMetric }>;
  label?: string;
}) {
  if (!active || !payload || payload.length === 0) return null;

  const rows = payload.filter((entry) => typeof entry.value === "number");
  if (rows.length === 0) return null;

  const metric = rows[0]?.payload;
  if (!metric) return null;

  return (
    <div className="min-w-[280px] rounded-lg border border-panel-border bg-panel/95 px-3 py-2.5 shadow-panel backdrop-blur-[8px]">
      <p className="text-xs uppercase tracking-[0.1em] text-text-muted">{label}</p>
      <p className="mt-1 text-xs text-text-muted">{metric.description}</p>
      <div className="mt-2 space-y-1.5">
        {rows.map((entry) => (
          <div key={entry.name} className="grid grid-cols-[auto_1fr_auto] items-center gap-2 text-sm">
            <span
              className="h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: entry.color ?? "currentColor" }}
              aria-hidden="true"
            />
            <span className="capitalize text-text">{entry.name}</span>
            <div className="text-right">
              <p className="font-mono text-text-muted">{formatRadarScore(entry.value ?? 0)}</p>
              <p className="font-mono text-[11px] text-text-muted/80">
                {formatRadarRawValue(metric, rawMetricValueForVariant(metric, entry.name))}
              </p>
            </div>
          </div>
        ))}
      </div>
      <div className="mt-2 space-y-1 text-[11px] text-text-muted">
        <p>
          {metric.lowerIsBetter ? "Lower raw values score higher." : "Higher raw values score higher."} Scores are smoothed against the visible workload context.
        </p>
        <p>
          Context median: <span className="font-mono">{formatRadarRawValue(metric, metric.contextMedian)}</span>
          {metric.contextCount > 0 ? ` across ${metric.contextCount} implementations` : ""}.
        </p>
        <p>Band: <span className="font-mono">{formatRadarBand(rows[0]?.value ?? 0)}</span></p>
      </div>
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

function ProfilingTooltipContent({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: Array<{ dataKey?: string; value?: number; color?: string }>;
  label?: string | number;
}) {
  if (!active || !payload || payload.length === 0) return null;

  const rows = new Map<VariantKey, { ir: number | null; miss: number | null; color: string }>();
  const ensure = (variant: VariantKey): { ir: number | null; miss: number | null; color: string } => {
    const existing = rows.get(variant);
    if (existing) return existing;
    const created = { ir: null, miss: null, color: VARIANT_COLORS[variant] };
    rows.set(variant, created);
    return created;
  };

  for (const entry of payload) {
    const key = entry.dataKey;
    if (!key) continue;

    for (const variant of ["safe", "raw", "arena"] as VariantKey[]) {
      if (key === `${variant}Ir`) {
        const row = ensure(variant);
        row.ir = typeof entry.value === "number" ? entry.value : null;
        if (entry.color) row.color = entry.color;
      }

      if (key === `${variant}Miss`) {
        const row = ensure(variant);
        row.miss = typeof entry.value === "number" ? entry.value : null;
        if (entry.color) row.color = entry.color;
      }
    }
  }

  const ordered = (["safe", "raw", "arena"] as VariantKey[])
    .map((variant) => ({ variant, ...rows.get(variant) }))
    .filter((row) => row.ir != null || row.miss != null);

  if (ordered.length === 0) return null;

  const sizeLabel = typeof label === "number" ? formatCompactTick(label) : String(label ?? "");

  return (
    <div className="min-w-[260px] rounded-lg border border-panel-border bg-panel/95 px-3 py-2.5 shadow-panel backdrop-blur-[8px]">
      <p className="text-xs uppercase tracking-[0.1em] text-text-muted">Input Size</p>
      <p className="font-mono text-sm text-text">{sizeLabel}</p>
      <div className="mt-2 space-y-1.5">
        {ordered.map((row) => (
          <div key={row.variant} className="space-y-0.5">
            <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2 text-sm">
              <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: row.color }} aria-hidden="true" />
              <span className="capitalize text-text">{row.variant}</span>
              <span className="font-mono text-text-muted">Ir {row.ir != null ? formatCompactTick(row.ir) : "n/a"}</span>
            </div>
            <p className="pl-4 text-xs text-text-muted">L1 miss {row.miss != null ? formatPercent(row.miss) : "n/a"}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function VariantDrilldownInner({ snapshotRecords, trendRecords, dataset, selectedImplementations }: VariantDrilldownProps) {
  const snapshotAggregates = useMemo(
    () => buildImplementationAggregates(snapshotRecords, dataset),
    [dataset, snapshotRecords],
  );

  const focusImplementation =
    selectedImplementations[0] ??
    snapshotAggregates.find((item) => item.variant === "safe")?.implementation ??
    snapshotAggregates[0]?.implementation;

  const focusFamily = focusImplementation ? familyKey(focusImplementation) : "";

  const variantRows = snapshotAggregates.filter((item) => {
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
      trendRecords.filter(
        (record) =>
          familyKey(record.implementation) === focusFamily &&
          (record.variant === "safe" || record.variant === "raw" || record.variant === "arena"),
      ),
    [focusFamily, trendRecords],
  );

  const radar = useMemo(
    () => buildVariantRadarMetrics(snapshotAggregates, focusFamily),
    [focusFamily, snapshotAggregates],
  );
  const radarData = radar.metrics;
  const omittedRadarAxes = radar.omittedAxes;

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

  const callgrindByJoinKey = useMemo(() => {
    const buckets = new Map<string, Array<Record<string, number>>>();

    for (const row of dataset.callgrind) {
      const values = buckets.get(row.joinKey) ?? [];
      values.push(row.metrics);
      buckets.set(row.joinKey, values);
    }

    const averaged = new Map<string, Record<string, number>>();
    for (const [joinKey, values] of buckets.entries()) {
      const totals = new Map<string, number>();
      const counts = new Map<string, number>();

      for (const metrics of values) {
        for (const [key, value] of Object.entries(metrics)) {
          totals.set(key, (totals.get(key) ?? 0) + value);
          counts.set(key, (counts.get(key) ?? 0) + 1);
        }
      }

      const metrics: Record<string, number> = {};
      for (const [key, total] of totals.entries()) {
        const count = counts.get(key) ?? 1;
        metrics[key] = total / count;
      }

      averaged.set(joinKey, metrics);
    }

    return averaged;
  }, [dataset.callgrind]);

  const profilingLineData = useMemo(() => {
    const byVariantAndSize = new Map<VariantKey, Map<number, { irSum: number; irCount: number; missSum: number; missCount: number }>>();

    for (const variant of ["safe", "raw", "arena"] as VariantKey[]) {
      byVariantAndSize.set(variant, new Map<number, { irSum: number; irCount: number; missSum: number; missCount: number }>());
    }

    for (const record of familyRecords) {
      const variant = variantFromImplementation(record.implementation);
      if (!variant) continue;

      const metrics = callgrindByJoinKey.get(record.joinKey);
      if (!metrics) continue;

      const bySize = byVariantAndSize.get(variant);
      if (!bySize) continue;

      const current = bySize.get(record.size) ?? { irSum: 0, irCount: 0, missSum: 0, missCount: 0 };

      const ir = metrics.Ir;
      if (typeof ir === "number" && Number.isFinite(ir)) {
        current.irSum += ir;
        current.irCount += 1;
      }

      const dr = metrics.Dr;
      const dw = metrics.Dw;
      const d1mr = metrics.D1mr;
      const d1mw = metrics.D1mw;
      if (
        typeof dr === "number" && Number.isFinite(dr) &&
        typeof dw === "number" && Number.isFinite(dw) &&
        typeof d1mr === "number" && Number.isFinite(d1mr) &&
        typeof d1mw === "number" && Number.isFinite(d1mw)
      ) {
        const accesses = dr + dw;
        if (accesses > 0) {
          current.missSum += (d1mr + d1mw) / accesses;
          current.missCount += 1;
        }
      }

      bySize.set(record.size, current);
    }

    const sizes = [...new Set(familyRecords.map((record) => record.size))].sort((a, b) => a - b);
    return sizes.map((size) => {
      const row: {
        size: number;
        safeIr: number | null;
        rawIr: number | null;
        arenaIr: number | null;
        safeMiss: number | null;
        rawMiss: number | null;
        arenaMiss: number | null;
      } = {
        size,
        safeIr: null,
        rawIr: null,
        arenaIr: null,
        safeMiss: null,
        rawMiss: null,
        arenaMiss: null,
      };

      for (const variant of ["safe", "raw", "arena"] as VariantKey[]) {
        const entry = byVariantAndSize.get(variant)?.get(size);
        if (!entry) continue;

        row[`${variant}Ir`] = entry.irCount > 0 ? entry.irSum / entry.irCount : null;
        row[`${variant}Miss`] = entry.missCount > 0 ? entry.missSum / entry.missCount : null;
      }

      return row;
    });
  }, [callgrindByJoinKey, familyRecords]);

  const hasProfilingTrendData = useMemo(
    () =>
      profilingLineData.some(
        (point) =>
          point.safeIr != null ||
          point.rawIr != null ||
          point.arenaIr != null ||
          point.safeMiss != null ||
          point.rawMiss != null ||
          point.arenaMiss != null,
      ),
    [profilingLineData],
  );

  const instructionTickScale = useMemo(() => {
    const values = profilingLineData
      .flatMap((point) => [point.safeIr, point.rawIr, point.arenaIr])
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));
    return values.length > 0 ? Math.max(...values) : 0;
  }, [profilingLineData]);

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
        <p className="text-sm text-text-muted">Pin one implementation to see how its family compares across the safe, raw, and arena variants.</p>
      </AppGlassPanel>
    );
  }

  return (
    <AppGlassPanel className="space-y-4">
      <div className="space-y-3">
        <div>
          <h3 className="font-display text-xl">Variant Micro-Drilldown</h3>
          <p className="text-sm text-text-muted">Looking at the {focusFamily} family across the visible variants.</p>
          <p className="text-xs text-text-muted/90">
            The radar is locked to the exact current workload slice and scores each variant against the broader visible context, so near-equal variants stay near the middle instead of collapsing to 0 or 100.
          </p>
          {omittedRadarAxes.length > 0 ? (
            <p className="text-xs text-text-muted/75">
              Some radar spokes are hidden until every visible variant has matching profiling coverage: {omittedRadarAxes.join(", ")}.
            </p>
          ) : null}
        </div>

        <div className="grid gap-2 sm:grid-cols-3">
          {availableVariants.map((variant) => {
            const item = variantByType.get(variant);
            return (
              <div key={variant} className="rounded-md border border-panel-border bg-bg-elevated/55 px-3 py-2.5">
                <p className="text-xs uppercase tracking-[0.12em] text-text-muted">{formatVariantLabel(variant)}</p>
                <p className="mt-1 text-sm text-text">Average run time: <span className="font-mono text-primary">{item ? formatLatencyWithScale(item.meanNs, latencyScale) : "n/a"}</span></p>
                <p className="text-sm text-text">Memory footprint: <span className="font-mono text-text-muted">{item ? formatBytes(item.estimatedMemoryBytes) : "n/a"}</span></p>
              </div>
            );
          })}
        </div>
      </div>

      <div className="space-y-3">
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

        <div className="grid gap-3 xl:grid-cols-2">
          <div className="rounded-md border border-panel-border bg-bg-elevated/65 p-2.5">
            <div className="mb-2">
              <p className="text-xs uppercase tracking-[0.12em] text-text-muted">Latency Trend</p>
              <p className="text-xs text-text-muted/80">Mean latency over input size with confidence interval bands.</p>
            </div>
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
                    dy: 0,
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

          <div className="rounded-md border border-panel-border bg-bg-elevated/65 p-2.5">
            <div className="mb-2">
              <p className="text-xs uppercase tracking-[0.12em] text-text-muted">Profiling Trend</p>
              <p className="text-xs text-text-muted/80">Solid lines: instructions (Ir). Dashed lines: L1 data miss rate.</p>
            </div>
            {hasProfilingTrendData ? (
              <ResponsiveContainer width="100%" height={360}>
                <ComposedChart data={profilingLineData} margin={{ top: 10, right: 42, left: 40, bottom: 8 }}>
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
                    yAxisId="ir"
                    tickFormatter={(value: number) => formatCompactTick(Number(value))}
                    tick={{
                      fill: CHART_COLORS.label,
                      fontSize: 12,
                      fontFamily: "var(--font-ibm-plex-mono), monospace",
                    }}
                    label={{
                      value: instructionTickScale >= 1_000_000 ? "Instructions (M)" : "Instructions",
                      angle: -90,
                      position: "insideLeft",
                      dx: -30,
                      dy: 0,
                      fill: CHART_COLORS.label,
                      fontSize: 12,
                      fontFamily: "var(--font-ibm-plex-mono), monospace",
                    }}
                  />
                  <YAxis
                    yAxisId="miss"
                    orientation="right"
                    tickFormatter={(value: number) => `${(Number(value) * 100).toFixed(2)}%`}
                    tick={{
                      fill: CHART_COLORS.label,
                      fontSize: 11,
                      fontFamily: "var(--font-ibm-plex-mono), monospace",
                    }}
                    label={{
                      value: "L1 Data Miss %",
                      angle: 90,
                      position: "insideRight",
                      dx: 24,
                      dy: 0,
                      fill: CHART_COLORS.label,
                      fontSize: 12,
                      fontFamily: "var(--font-ibm-plex-mono), monospace",
                    }}
                  />
                  <Tooltip content={<ProfilingTooltipContent />} />
                  <Legend />
                  {availableVariants.map((variant) => (
                    <Line
                      key={`${variant}-ir`}
                      yAxisId="ir"
                      type="monotone"
                      dataKey={`${variant}Ir`}
                      name={`${variant} Ir`}
                      stroke={VARIANT_COLORS[variant]}
                      strokeWidth={2.2}
                      dot={false}
                      connectNulls
                      isAnimationActive
                    />
                  ))}
                  {availableVariants.map((variant) => (
                    <Line
                      key={`${variant}-miss`}
                      yAxisId="miss"
                      type="monotone"
                      dataKey={`${variant}Miss`}
                      name={`${variant} L1 miss`}
                      stroke={VARIANT_COLORS[variant]}
                      strokeWidth={1.7}
                      strokeDasharray="6 4"
                      opacity={0.8}
                      dot={false}
                      connectNulls
                      isAnimationActive
                    />
                  ))}
                </ComposedChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex h-[360px] items-center justify-center rounded-md border border-panel-border/70 bg-bg-elevated/35 p-4 text-center text-sm text-text-muted">
                No callgrind trend data is available for this family in the current context.
              </div>
            )}
          </div>
        </div>
      </div>
    </AppGlassPanel>
  );
}

export const VariantDrilldown = memo(VariantDrilldownInner);
