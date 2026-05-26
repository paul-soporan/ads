"use client";

import { memo, useEffect, useMemo, useRef, useState } from "react";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import { useDashboardStore } from "@/lib/bench/store";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";
import type { LeaderboardSortDirection, LeaderboardSortKey } from "@/lib/bench/urlState";

type SortKey = LeaderboardSortKey;
type SortDirection = LeaderboardSortDirection;

interface LeaderboardTableProps {
  records: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset | null;
  showProfiling: boolean;
  sortKey: SortKey;
  sortDirection: SortDirection;
  onSortChange: (key: SortKey, direction: SortDirection) => void;
  trendRecords?: CriterionRecord[];
  selectedImplementations: string[];
  onToggleImplementation: (implementation: string) => void;
  matchedHeightPx?: number | null;
}

const ROW_HEIGHT = 46;
const FALLBACK_VIEWPORT_HEIGHT = 520;
const OVERSCAN = 8;

function formatCompactCount(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(2)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return Math.round(value).toLocaleString();
}

function formatPercent(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";
  return `${(value * 100).toFixed(3)}%`;
}

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

function LeaderboardTableInner({
  records,
  dataset,
  showProfiling,
  sortKey,
  sortDirection,
  onSortChange,
  trendRecords,
  selectedImplementations,
  onToggleImplementation,
  matchedHeightPx,
}: LeaderboardTableProps) {
  const [scrollTop, setScrollTop] = useState(0);
  const [hint, setHint] = useState<{ recordId: string; text: string; x: number; y: number } | null>(null);
  const hintTimerRef = useRef<number | null>(null);
  const hoverPointRef = useRef<{ x: number; y: number } | null>(null);
  const scrollViewportRef = useRef<HTMLDivElement | null>(null);
  const hoveredImplementation = useDashboardStore((state) => state.hoveredImplementation);
  const setHoveredImplementation = useDashboardStore((state) => state.setHoveredImplementation);
  const trendSourceRecords = trendRecords && trendRecords.length > 0 ? trendRecords : records;

  useEffect(() => {
    return () => {
      if (hintTimerRef.current != null) {
        window.clearTimeout(hintTimerRef.current);
      }
    };
  }, []);

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

  const profilingByJoinKey = useMemo(() => {
    const instructionsBuckets = new Map<string, number[]>();
    const l1MissRateBuckets = new Map<string, number[]>();
    const peakBytesBuckets = new Map<string, number[]>();
    if (!dataset) {
      return {
        instructions: new Map<string, number>(),
        l1MissRate: new Map<string, number>(),
        peakBytes: new Map<string, number>(),
      };
    }

    for (const row of dataset.callgrind) {
      const instructions = row.metrics.Ir;
      if (Number.isFinite(instructions)) {
        const values = instructionsBuckets.get(row.joinKey) ?? [];
        values.push(instructions);
        instructionsBuckets.set(row.joinKey, values);
      }

      const dr = row.metrics.Dr;
      const dw = row.metrics.Dw;
      const d1mr = row.metrics.D1mr;
      const d1mw = row.metrics.D1mw;
      if (Number.isFinite(dr) && Number.isFinite(dw) && Number.isFinite(d1mr) && Number.isFinite(d1mw)) {
        const accesses = dr + dw;
        if (accesses > 0) {
          const missRate = (d1mr + d1mw) / accesses;
          const values = l1MissRateBuckets.get(row.joinKey) ?? [];
          values.push(missRate);
          l1MissRateBuckets.set(row.joinKey, values);
        }
      }
    }

    for (const row of dataset.dhat) {
      const peakBytes = row.maxBytes ?? row.totalBytes;
      if (typeof peakBytes !== "number" || !Number.isFinite(peakBytes)) continue;
      const values = peakBytesBuckets.get(row.joinKey) ?? [];
      values.push(peakBytes);
      peakBytesBuckets.set(row.joinKey, values);
    }

    const toMeanMap = (input: Map<string, number[]>) => {
      const output = new Map<string, number>();
      for (const [key, values] of input.entries()) {
        if (values.length === 0) continue;
        output.set(key, values.reduce((sum, value) => sum + value, 0) / values.length);
      }
      return output;
    };

    return {
      instructions: toMeanMap(instructionsBuckets),
      l1MissRate: toMeanMap(l1MissRateBuckets),
      peakBytes: toMeanMap(peakBytesBuckets),
    };
  }, [dataset]);

  const sortedRecords = useMemo(() => {
    const copy = records.slice();

    copy.sort((a, b) => {
      let cmp = 0;

      if (sortKey === "implementation") cmp = a.implementation.localeCompare(b.implementation);
      if (sortKey === "operation") cmp = a.operation.localeCompare(b.operation);
      if (sortKey === "size") cmp = a.size - b.size;
      if (sortKey === "meanNs") cmp = a.meanNs - b.meanNs;
      if (sortKey === "instructions") {
        const aValue = profilingByJoinKey.instructions.get(a.joinKey);
        const bValue = profilingByJoinKey.instructions.get(b.joinKey);

        if (aValue == null && bValue == null) {
          cmp = 0;
        } else if (aValue == null) {
          cmp = 1;
        } else if (bValue == null) {
          cmp = -1;
        } else {
          cmp = aValue - bValue;
        }
      }
      if (sortKey === "l1MissRate") {
        const aValue = profilingByJoinKey.l1MissRate.get(a.joinKey);
        const bValue = profilingByJoinKey.l1MissRate.get(b.joinKey);

        if (aValue == null && bValue == null) {
          cmp = 0;
        } else if (aValue == null) {
          cmp = 1;
        } else if (bValue == null) {
          cmp = -1;
        } else {
          cmp = aValue - bValue;
        }
      }
      if (sortKey === "peakBytes") {
        const aValue = profilingByJoinKey.peakBytes.get(a.joinKey);
        const bValue = profilingByJoinKey.peakBytes.get(b.joinKey);

        if (aValue == null && bValue == null) {
          cmp = 0;
        } else if (aValue == null) {
          cmp = 1;
        } else if (bValue == null) {
          cmp = -1;
        } else {
          cmp = aValue - bValue;
        }
      }
      if (sortKey === "throughput") cmp = (a.throughputElements ?? -1) - (b.throughputElements ?? -1);

      return sortDirection === "asc" ? cmp : -cmp;
    });

    return copy;
  }, [profilingByJoinKey, records, sortDirection, sortKey]);

  const total = sortedRecords.length;
  const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const viewportHeight = Math.max(FALLBACK_VIEWPORT_HEIGHT, scrollViewportRef.current?.clientHeight ?? FALLBACK_VIEWPORT_HEIGHT);
  const endIndex = Math.min(total, Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + OVERSCAN);
  const visibleRows = sortedRecords.slice(startIndex, endIndex);

  const padTop = startIndex * ROW_HEIGHT;
  const padBottom = Math.max((total - endIndex) * ROW_HEIGHT, 0);

  function applySort(nextKey: SortKey) {
    if (sortKey === nextKey) {
      onSortChange(nextKey, sortDirection === "asc" ? "desc" : "asc");
      return;
    }

    onSortChange(nextKey, "asc");
  }

  function ariaSortValue(key: SortKey): "none" | "ascending" | "descending" {
    if (sortKey !== key) return "none";
    return sortDirection === "asc" ? "ascending" : "descending";
  }

  function getHintPosition(clientX: number, clientY: number): { x: number; y: number } {
    const viewport = scrollViewportRef.current?.getBoundingClientRect();
    const viewportNode = scrollViewportRef.current;
    if (!viewport) {
      return {
        x: 14,
        y: 14,
      };
    }

    const scrollTopOffset = viewportNode?.scrollTop ?? 0;
    const localX = clientX - viewport.left;
    const localY = clientY - viewport.top + scrollTopOffset;

    const tooltipWidth = 250;
    const tooltipHeight = 28;
    const topOffset = 24;
    const bottomOffset = 18;

    const clampedX = Math.min(Math.max(localX + 12, 8), viewport.width - tooltipWidth - 8);
    const preferredTop = localY - topOffset;
    const top = localY - scrollTopOffset < 24 ? localY + bottomOffset : preferredTop;
    const clampedY = Math.min(
      Math.max(top, scrollTopOffset + 8),
      scrollTopOffset + viewport.height - tooltipHeight - 8,
    );

    return {
      x: clampedX,
      y: clampedY,
    };
  }

  function scheduleHint(recordId: string, text: string) {
    if (hintTimerRef.current != null) {
      window.clearTimeout(hintTimerRef.current);
    }

    hintTimerRef.current = window.setTimeout(() => {
      const hoverPoint = hoverPointRef.current;
      if (!hoverPoint) return;

      const nextPosition = getHintPosition(hoverPoint.x, hoverPoint.y);
      setHint({
        recordId,
        text,
        x: nextPosition.x,
        y: nextPosition.y,
      });
    }, 450);
  }

  function clearHint(recordId?: string) {
    if (hintTimerRef.current != null) {
      window.clearTimeout(hintTimerRef.current);
      hintTimerRef.current = null;
    }

    setHint((current) => {
      if (recordId == null) return null;
      return current?.recordId === recordId ? null : current;
    });
  }

  return (
    <AppGlassPanel
      className="flex min-h-0 flex-col gap-4"
      style={matchedHeightPx && matchedHeightPx > 0 ? { height: `${matchedHeightPx}px` } : undefined}
    >
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="font-display text-xl">Global Leaderboard</h2>
          <p className="text-sm text-text-muted">Scan the current results and click any row to add or remove it from the Inspector.</p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <p className="font-mono text-sm text-primary">Rows: {total.toLocaleString()}</p>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-x-auto">
        <div className={`space-y-1.5 ${showProfiling ? "min-w-[980px]" : "min-w-[720px]"}`}>
          <table className="w-full border-b border-panel-border text-sm uppercase tracking-[0.12em] text-text-muted">
            <thead>
              <tr className={`grid ${showProfiling ? "grid-cols-[2fr_1fr_0.8fr_1fr_1fr_1fr_1fr_110px]" : "grid-cols-[2fr_1fr_0.8fr_1fr_110px]"} gap-2 px-3 pb-1.5`}>
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
                {showProfiling ? (
                  <th scope="col" aria-sort={ariaSortValue("instructions")} className="text-left font-medium">
                    <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("instructions")} aria-label="Sort by instruction count">Instr.</button>
                  </th>
                ) : null}
                {showProfiling ? (
                  <th scope="col" aria-sort={ariaSortValue("l1MissRate")} className="text-left font-medium">
                    <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("l1MissRate")} aria-label="Sort by L1 data miss rate">L1 Miss</button>
                  </th>
                ) : null}
                {showProfiling ? (
                  <th scope="col" aria-sort={ariaSortValue("peakBytes")} className="text-left font-medium">
                    <button className="text-left hover:text-text focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/70" onClick={() => applySort("peakBytes")} aria-label="Sort by peak bytes">Peak B</button>
                  </th>
                ) : null}
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
              ref={scrollViewportRef}
              className="relative h-full min-h-[520px] overflow-y-auto rounded-md border border-panel-border"
              onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
            >
              <div style={{ paddingTop: `${padTop}px`, paddingBottom: `${padBottom}px` }}>
                {visibleRows.map((record, index) => {
                  const active = selectedImplementations.includes(record.implementation);
                  const crossHighlighted = hoveredImplementation === record.implementation;
                  const absoluteIndex = startIndex + index;
                  const hintText = active ? "Click to remove from the Inspector" : "Click to add to the Inspector";
                  return (
                    <button
                      key={record.id}
                      type="button"
                      onClick={() => onToggleImplementation(record.implementation)}
                      style={{ height: `${ROW_HEIGHT}px` }}
                      aria-label={`Toggle comparison pin for ${record.implementation}`}
                      onMouseEnter={(event) => {
                        setHoveredImplementation(record.implementation);
                        hoverPointRef.current = { x: event.clientX, y: event.clientY };
                        scheduleHint(record.id, hintText);
                      }}
                      onMouseMove={(event) => {
                        hoverPointRef.current = { x: event.clientX, y: event.clientY };
                        if (hint?.recordId !== record.id) return;

                        const nextPosition = getHintPosition(event.clientX, event.clientY);
                        setHint((current) => {
                          if (!current || current.recordId !== record.id) return current;
                          return {
                            ...current,
                            x: nextPosition.x,
                            y: nextPosition.y,
                            text: hintText,
                          };
                        });
                      }}
                      onMouseLeave={() => {
                        setHoveredImplementation(null);
                        hoverPointRef.current = null;
                        clearHint(record.id);
                      }}
                      onFocus={(event) => {
                        setHoveredImplementation(record.implementation);
                        const bounds = event.currentTarget.getBoundingClientRect();
                        const x = bounds.left + 24;
                        const y = bounds.top + 16;
                        hoverPointRef.current = { x, y };
                        const nextPosition = getHintPosition(x, y);
                        setHint({
                          recordId: record.id,
                          text: hintText,
                          x: nextPosition.x,
                          y: nextPosition.y,
                        });
                      }}
                      onBlur={() => {
                        setHoveredImplementation(null);
                        clearHint(record.id);
                      }}
                      className={`relative grid w-full ${showProfiling ? "grid-cols-[2fr_1fr_0.8fr_1fr_1fr_1fr_1fr_110px]" : "grid-cols-[2fr_1fr_0.8fr_1fr_110px]"} items-center gap-2 border-b border-panel-border px-3 text-left text-sm transition ${
                        crossHighlighted
                          ? "bg-primary/18 ring-1 ring-inset ring-primary/70"
                          : active
                            ? "bg-primary/14 ring-1 ring-inset ring-primary/50"
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
                      {showProfiling ? (
                        <span className="font-mono text-text-muted [font-variant-numeric:tabular-nums]">
                          {(() => {
                            const instructions = profilingByJoinKey.instructions.get(record.joinKey);
                            return instructions != null ? formatCompactCount(instructions) : "n/a";
                          })()}
                        </span>
                      ) : null}
                      {showProfiling ? (
                        <span className="font-mono text-text-muted [font-variant-numeric:tabular-nums]">
                          {formatPercent(profilingByJoinKey.l1MissRate.get(record.joinKey) ?? null)}
                        </span>
                      ) : null}
                      {showProfiling ? (
                        <span className="font-mono text-text-muted [font-variant-numeric:tabular-nums]">
                          {formatCompactCount(profilingByJoinKey.peakBytes.get(record.joinKey) ?? null)}
                        </span>
                      ) : null}
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

              {hint ? (
                <div
                  className="pointer-events-none absolute z-20 max-w-[250px] whitespace-nowrap rounded-md border border-panel-border/80 bg-panel/96 px-2.5 py-1 text-[11px] text-text-muted shadow-panel backdrop-blur-[8px]"
                  style={{
                    left: `${hint.x}px`,
                    top: `${hint.y}px`,
                  }}
                >
                  {hint.text}
                </div>
              ) : null}
            </div>
          )}
        </div>
      </div>
    </AppGlassPanel>
  );
}

export const LeaderboardTable = memo(LeaderboardTableInner);
