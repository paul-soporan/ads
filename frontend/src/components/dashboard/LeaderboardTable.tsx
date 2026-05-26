"use client";

import { memo, useMemo, useState } from "react";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import type { CriterionRecord } from "@/lib/bench/types";

type SortKey = "implementation" | "operation" | "size" | "meanNs" | "throughput";
type SortDirection = "asc" | "desc";

interface LeaderboardTableProps {
  records: CriterionRecord[];
  trendRecords?: CriterionRecord[];
  selectedImplementations: string[];
  onToggleImplementation: (implementation: string) => void;
}

const ROW_HEIGHT = 46;
const VIEWPORT_HEIGHT = 520;
const OVERSCAN = 8;

function quantile(values: number[], q: number): number {
  if (values.length === 0) return 0;

  const sorted = values.slice().sort((a, b) => a - b);
  const position = (sorted.length - 1) * q;
  const lower = Math.floor(position);
  const upper = Math.ceil(position);

  if (lower === upper) return sorted[lower] ?? 0;

  const lowerValue = sorted[lower] ?? 0;
  const upperValue = sorted[upper] ?? lowerValue;
  return lowerValue + (upperValue - lowerValue) * (position - lower);
}

function trendKey(record: CriterionRecord): string {
  return `${record.workloadName}|${record.implementation}|${record.operation}|${record.distribution}|${record.payload}`;
}

function Sparkline({
  points,
  domainMin,
  domainMax,
}: {
  points: number[];
  domainMin: number;
  domainMax: number;
}) {
  if (points.length <= 1) {
    return <span className="text-text-muted">-</span>;
  }

  const width = 88;
  const height = 24;
  const min = Math.max(1, domainMin);
  const max = Math.max(domainMax, min * 1.05);
  const logMin = Math.log10(min);
  const logMax = Math.log10(max);
  const logRange = Math.max(logMax - logMin, 0.001);

  const path = points
    .map((value, index) => {
      const x = (index / (points.length - 1)) * width;
      const clamped = Math.min(Math.max(value, min), max);
      const y = height - ((Math.log10(clamped) - logMin) / logRange) * height;
      return `${index === 0 ? "M" : "L"}${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");

  return (
    <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} aria-hidden="true">
      <path d={path} fill="none" stroke="currentColor" strokeWidth="2" className="text-primary" />
    </svg>
  );
}

function LeaderboardTableInner({ records, trendRecords, selectedImplementations, onToggleImplementation }: LeaderboardTableProps) {
  const [sortKey, setSortKey] = useState<SortKey>("meanNs");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");
  const [scrollTop, setScrollTop] = useState(0);
  const trendSourceRecords = trendRecords && trendRecords.length > 0 ? trendRecords : records;

  const { trendByContext, sparklineDomain } = useMemo(() => {
    const grouped = new Map<string, CriterionRecord[]>();

    for (const record of trendSourceRecords) {
      const key = trendKey(record);
      const bucket = grouped.get(key);
      if (bucket) {
        bucket.push(record);
      } else {
        grouped.set(key, [record]);
      }
    }

    const trends = new Map<string, number[]>();
    const allTrendPoints: number[] = [];
    for (const [key, list] of grouped) {
      const finite = list.filter((item) => Number.isFinite(item.meanNs) && Number.isFinite(item.size));

      if (finite.length < 2) {
        trends.set(key, []);
        continue;
      }

      const perSize = new Map<number, number[]>();
      for (const item of finite) {
        const bucket = perSize.get(item.size);
        if (bucket) {
          bucket.push(item.meanNs);
        } else {
          perSize.set(item.size, [item.meanNs]);
        }
      }

      const points = Array.from(perSize.entries())
        .slice()
        .sort((a, b) => a[0] - b[0])
        .map(([, values]) => values.reduce((sum, value) => sum + value, 0) / values.length)
        .filter((value) => Number.isFinite(value));

      allTrendPoints.push(...points.filter((value) => value > 0));
      trends.set(key, points);
    }

    const domainMin = allTrendPoints.length > 0 ? Math.max(1, quantile(allTrendPoints, 0.05)) : 1;
    const domainMax =
      allTrendPoints.length > 0
        ? Math.max(quantile(allTrendPoints, 0.95), domainMin * 1.05)
        : domainMin * 1.05;

    return {
      trendByContext: trends,
      sparklineDomain: {
        min: domainMin,
        max: domainMax,
      },
    };
  }, [trendSourceRecords]);

  const sortedRecords = useMemo(() => {
    const copy = records.slice();

    copy.sort((a, b) => {
      let cmp = 0;

      if (sortKey === "implementation") cmp = a.implementation.localeCompare(b.implementation);
      if (sortKey === "operation") cmp = a.operation.localeCompare(b.operation);
      if (sortKey === "size") cmp = a.size - b.size;
      if (sortKey === "meanNs") cmp = a.meanNs - b.meanNs;
      if (sortKey === "throughput") cmp = (a.throughputElements ?? -1) - (b.throughputElements ?? -1);

      return sortDirection === "asc" ? cmp : -cmp;
    });

    return copy;
  }, [records, sortDirection, sortKey]);

  const total = sortedRecords.length;
  const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const endIndex = Math.min(total, Math.ceil((scrollTop + VIEWPORT_HEIGHT) / ROW_HEIGHT) + OVERSCAN);
  const visibleRows = sortedRecords.slice(startIndex, endIndex);

  const padTop = startIndex * ROW_HEIGHT;
  const padBottom = Math.max((total - endIndex) * ROW_HEIGHT, 0);

  function applySort(nextKey: SortKey) {
    if (sortKey === nextKey) {
      setSortDirection((direction) => (direction === "asc" ? "desc" : "asc"));
      return;
    }

    setSortKey(nextKey);
    setSortDirection(nextKey === "meanNs" ? "asc" : "desc");
  }

  function ariaSortValue(key: SortKey): "none" | "ascending" | "descending" {
    if (sortKey !== key) return "none";
    return sortDirection === "asc" ? "ascending" : "descending";
  }

  return (
    <AppGlassPanel className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="font-display text-xl">Global Leaderboard</h2>
          <p className="text-sm text-text-muted">Virtualized and sortable benchmark results for the active context.</p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <p className="font-mono text-sm text-primary">Rows: {total.toLocaleString()}</p>
        </div>
      </div>

      <div className="overflow-x-auto">
        <div className="min-w-[720px] space-y-1.5">
          <table className="w-full border-b border-panel-border text-sm uppercase tracking-[0.12em] text-text-muted">
            <thead>
              <tr className="grid grid-cols-[2fr_1fr_0.8fr_1fr_110px] gap-2 px-3 pb-1.5">
                <th scope="col" aria-sort={ariaSortValue("implementation")} className="text-left font-medium">
                  <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("implementation")} aria-label="Sort by implementation">Implementation</button>
                </th>
                <th scope="col" aria-sort={ariaSortValue("operation")} className="text-left font-medium">
                  <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("operation")} aria-label="Sort by operation">Operation</button>
                </th>
                <th scope="col" aria-sort={ariaSortValue("size")} className="text-left font-medium">
                  <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("size")} aria-label="Sort by input size">Size</button>
                </th>
                <th scope="col" aria-sort={ariaSortValue("meanNs")} className="text-left font-medium">
                  <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("meanNs")} aria-label="Sort by mean latency">Mean (ns)</button>
                </th>
                <th scope="col" aria-sort={ariaSortValue("throughput")} className="text-left font-medium">
                  <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("throughput")} aria-label="Sort by throughput">Spark</button>
                </th>
              </tr>
            </thead>
          </table>

          {total === 0 ? (
            <div className="rounded-md border border-panel-border bg-bg-elevated p-6 text-sm text-text-muted">
              No benchmark rows match the current filters.
            </div>
          ) : (
            <div
              className="overflow-y-auto rounded-md border border-panel-border"
              style={{ height: `${VIEWPORT_HEIGHT}px` }}
              onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
            >
              <div style={{ paddingTop: `${padTop}px`, paddingBottom: `${padBottom}px` }}>
                {visibleRows.map((record, index) => {
                  const active = selectedImplementations.includes(record.implementation);
                  const absoluteIndex = startIndex + index;
                  return (
                    <button
                      key={record.id}
                      type="button"
                      onClick={() => onToggleImplementation(record.implementation)}
                      style={{ height: `${ROW_HEIGHT}px` }}
                      aria-label={`Toggle comparison pin for ${record.implementation}`}
                      className={`grid w-full grid-cols-[2fr_1fr_0.8fr_1fr_110px] items-center gap-2 border-b border-panel-border px-3 text-left text-sm transition ${
                        active
                          ? "bg-primary/12"
                          : absoluteIndex % 2 === 0
                            ? "bg-panel/24 hover:bg-bg-elevated/75"
                            : "bg-transparent hover:bg-bg-elevated/75"
                      }`}
                    >
                      <span className="space-y-0.5">
                        <span className="block font-mono text-sm text-text">{record.implementation}</span>
                        <span className="block text-xs text-text-muted">{record.distribution} • {record.payload}</span>
                      </span>
                      <span className="text-text-muted">{record.operation}</span>
                      <span className="font-mono text-text-muted [font-variant-numeric:tabular-nums]">{record.size.toLocaleString()}</span>
                      <span className="font-mono text-primary [font-variant-numeric:tabular-nums]">{Math.round(record.meanNs).toLocaleString()}</span>
                      <span>
                        <Sparkline
                          points={trendByContext.get(trendKey(record)) ?? []}
                          domainMin={sparklineDomain.min}
                          domainMax={sparklineDomain.max}
                        />
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      </div>
    </AppGlassPanel>
  );
}

export const LeaderboardTable = memo(LeaderboardTableInner);
