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
import { comparisonToCsv, comparisonToMarkdown, copyOrDownload } from "@/lib/bench/export";
import { CHART_COLORS } from "@/lib/bench/visualTokens";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";

interface VersusComparisonProps {
  records: CriterionRecord[];
  trendRecords: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset;
  selectedImplementations: string[];
}

type LatencyScale = {
  unit: "ns" | "us" | "ms" | "s";
  divisor: number;
};

const SERIES_COLORS = ["#21d4fd", "#ff6cc3", "#45e3a1", "#f9bc60"];

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

function sanitizeSeriesKey(implementation: string): string {
  return implementation.replace(/[^a-zA-Z0-9_]/g, "_");
}

function VersusComparisonInner({ records, trendRecords, dataset, selectedImplementations }: VersusComparisonProps) {
  const [exportScope, setExportScope] = useState<"compared" | "top">("compared");
  const [exportFormat, setExportFormat] = useState<"csv" | "markdown">("markdown");
  const pushToast = useToast();
  const aggregates = useMemo(() => buildImplementationAggregates(records, dataset), [dataset, records]);

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
    () => compared.map((item, index) => ({ implementation: item.implementation, color: SERIES_COLORS[index % SERIES_COLORS.length] })),
    [compared],
  );

  const versusLineData = useMemo(() => {
    const trendSource = trendRecords.length > 0 ? trendRecords : records;
    const included = new Set(compared.map((item) => item.implementation));
    const bySeriesAndSize = new Map<string, Map<number, { sum: number; lowerSum: number; upperSum: number; count: number }>>();

    for (const row of trendSource) {
      if (!included.has(row.implementation)) continue;

      const seriesKey = sanitizeSeriesKey(row.implementation);
      const bySize = bySeriesAndSize.get(seriesKey) ?? new Map<number, { sum: number; lowerSum: number; upperSum: number; count: number }>();
      const current = bySize.get(row.size) ?? { sum: 0, lowerSum: 0, upperSum: 0, count: 0 };
      current.sum += row.meanNs;
      current.lowerSum += row.ciLowerNs;
      current.upperSum += row.ciUpperNs;
      current.count += 1;
      bySize.set(row.size, current);
      bySeriesAndSize.set(seriesKey, bySize);
    }

    const sizes = [...new Set(trendSource.filter((row) => included.has(row.implementation)).map((row) => row.size))].sort((a, b) => a - b);

    return sizes.map((size) => {
      const row: Record<string, number | null> & { size: number } = { size };
      for (const series of comparedSeries) {
        const key = sanitizeSeriesKey(series.implementation);
        const stats = bySeriesAndSize.get(key)?.get(size);
        row[`${key}Mean`] = stats ? stats.sum / stats.count : null;
        row[`${key}Lower`] = stats ? stats.lowerSum / stats.count : null;
        row[`${key}Band`] = stats ? Math.max(0, (stats.upperSum - stats.lowerSum) / stats.count) : null;
      }
      return row;
    });
  }, [compared, comparedSeries, records, trendRecords]);

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
  const comparedExportRows = compared.map((item, index) => {
    const deltaSpeed = toPercentChange(item.meanNs, baseline?.meanNs ?? item.meanNs);
    const deltaMemory = toPercentChange(item.estimatedMemoryBytes, baseline?.estimatedMemoryBytes ?? item.estimatedMemoryBytes);

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

  const topExportRows = aggregates.slice(0, 12).map((item, index) => {
    const deltaSpeed = toPercentChange(item.meanNs, baseline?.meanNs ?? item.meanNs);
    const deltaMemory = toPercentChange(item.estimatedMemoryBytes, baseline?.estimatedMemoryBytes ?? item.estimatedMemoryBytes);

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
          Pin at least two implementations from the matrix or leaderboard to unlock head-to-head deltas.
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
          <p className="text-xs text-text-muted/80">First pinned implementation is locked as the baseline.</p>
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
              className={`rounded-md border bg-bg-elevated/70 p-3.5 ${
                index === 0 ? "border-primary/60 shadow-[0_0_0_1px_rgba(33,212,253,0.2)]" : "border-panel-border"
              }`}
            >
              <p className="font-mono text-sm uppercase tracking-[0.12em] text-text-muted">{item.implementation}</p>
              {index === 0 ? (
                <p className="mt-1 text-xs uppercase tracking-[0.12em] text-primary">Baseline (Locked)</p>
              ) : null}
              <p className="mt-2 font-mono text-lg text-primary [font-variant-numeric:tabular-nums]">{Math.round(item.meanNs).toLocaleString()} ns</p>
              <p className={`text-sm ${index === 0 ? "text-text-muted" : speedTone}`}>{speedLabel}</p>
              <p className="mt-2 font-mono text-sm text-secondary [font-variant-numeric:tabular-nums]">{formatBytes(item.estimatedMemoryBytes)}</p>
              <p className={`text-sm ${index === 0 ? "text-text-muted" : memoryTone}`}>{memoryLabel}</p>
            </article>
          );
        })}
      </div>

      {versusLineData.length > 0 ? (
        <div className="rounded-md border border-panel-border bg-bg-elevated/65 p-2.5">
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
                tickFormatter={(value: number) => formatLatencyTick(value, latencyScale)}
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
              <Tooltip />
              <Legend />
              {comparedSeries.map((series) => {
                const key = sanitizeSeriesKey(series.implementation);
                return (
                  <Area
                    key={`${key}-ci-base`}
                    type="monotone"
                    dataKey={`${key}Lower`}
                    stackId={`ci-${key}`}
                    stroke="none"
                    fill="transparent"
                    connectNulls
                    isAnimationActive
                    legendType="none"
                  />
                );
              })}
              {comparedSeries.map((series) => {
                const key = sanitizeSeriesKey(series.implementation);
                return (
                  <Area
                    key={`${key}-ci-band`}
                    type="monotone"
                    dataKey={`${key}Band`}
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
              })}
              {comparedSeries.map((series) => {
                const key = sanitizeSeriesKey(series.implementation);
                return (
                  <Line
                    key={`${key}-mean`}
                    type="monotone"
                    dataKey={`${key}Mean`}
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
        </div>
      ) : null}
    </AppGlassPanel>
  );
}

export const VersusComparison = memo(VersusComparisonInner);
