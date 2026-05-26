"use client";

import { motion } from "framer-motion";
import { memo, useEffect, useMemo, useRef, useState } from "react";
import { ParentSize } from "@visx/responsive";
import { scaleLinear, scaleLog } from "@visx/scale";
import { LinePath } from "@visx/shape";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import {
  buildImplementationAggregates,
  buildParetoFront,
  formatBytes,
} from "@/lib/bench/analytics";
import { CHART_COLORS, VARIANT_COLORS } from "@/lib/bench/visualTokens";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";

interface TradeoffMatrixProps {
  records: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset;
  selectedImplementations: string[];
  onToggleImplementation: (implementation: string) => void;
  scaleMode: AxisScaleMode;
  hideOutliers: boolean;
  onScaleModeChange: (value: AxisScaleMode) => void;
  onHideOutliersChange: (value: boolean) => void;
}

const MIN_CHART_WIDTH = 320;
const MIN_CHART_HEIGHT = 360;
const MAX_CHART_HEIGHT = 560;
const CHART_ASPECT = 0.66;

type AxisScaleMode = "log" | "linear";
type AxisUnit = { divisor: number; label: string };

function buildLinearTicks(min: number, max: number, steps = 5): number[] {
  if (!Number.isFinite(min) || !Number.isFinite(max)) return [];
  if (max <= min) return [min];

  const step = (max - min) / (steps - 1);
  return Array.from({ length: steps }, (_, index) => min + step * index);
}

function quantile(values: number[], q: number): number {
  if (values.length === 0) return 0;
  const sorted = values.slice().sort((a, b) => a - b);
  const index = Math.max(0, Math.min(sorted.length - 1, Math.floor((sorted.length - 1) * q)));
  return sorted[index] ?? 0;
}

function selectLatencyUnit(maxNs: number): AxisUnit {
  if (maxNs >= 1_000_000) return { divisor: 1_000_000, label: "ms" };
  if (maxNs >= 1_000) return { divisor: 1_000, label: "us" };
  return { divisor: 1, label: "ns" };
}

function selectMemoryUnit(maxBytes: number): AxisUnit {
  if (maxBytes >= 1_000_000) return { divisor: 1_000_000, label: "MB" };
  if (maxBytes >= 1_000) return { divisor: 1_000, label: "KB" };
  return { divisor: 1, label: "bytes" };
}

function formatTickWithUnit(value: number, unit: AxisUnit): string {
  const normalized = value / unit.divisor;
  if (unit.divisor === 1) {
    return Math.round(normalized).toLocaleString();
  }

  if (normalized >= 100) return normalized.toFixed(0);
  if (normalized >= 10) return normalized.toFixed(1);
  return normalized.toFixed(2);
}

type OverlayPlacement = {
  left: string;
  top: string;
  transform: string;
};

function computeOverlayPlacement(cx: number, cy: number, width: number, height: number): OverlayPlacement {
  const margin = 10;
  const overlayWidth = Math.min(440, Math.max(300, width - margin * 2));
  const overlayHeight = 172;

  let left = cx + 14;
  if (left + overlayWidth > width - margin) {
    left = cx - overlayWidth - 14;
  }

  let top = cy + 14;
  if (top + overlayHeight > height - margin) {
    top = cy - overlayHeight - 14;
  }

  const clampedLeft = Math.min(Math.max(left, margin), Math.max(margin, width - overlayWidth - margin));
  const clampedTop = Math.min(Math.max(top, margin), Math.max(margin, height - overlayHeight - margin));

  return {
    left: `${clampedLeft.toFixed(0)}px`,
    top: `${clampedTop.toFixed(0)}px`,
    transform: "translate(0, 0)",
  };
}

function TradeoffMatrixInner({
  records,
  dataset,
  selectedImplementations,
  onToggleImplementation,
  scaleMode,
  hideOutliers,
  onScaleModeChange,
  onHideOutliersChange,
}: TradeoffMatrixProps) {
  const [hovered, setHovered] = useState<string | null>(null);
  const [pinnedDetails, setPinnedDetails] = useState<string | null>(null);
  const overlayRef = useRef<HTMLDivElement | null>(null);

  const aggregates = useMemo(() => buildImplementationAggregates(records, dataset), [dataset, records]);
  const points = useMemo(
    () => aggregates.map((item) => ({
      implementation: item.implementation,
      meanNs: item.meanNs,
      estimatedMemoryBytes: item.estimatedMemoryBytes,
    })),
    [aggregates],
  );

  const visibleAggregates = useMemo(() => {
    if (!hideOutliers || aggregates.length <= 3) return aggregates;

    const p95Memory = quantile(aggregates.map((item) => item.estimatedMemoryBytes), 0.95);
    const p95Mean = quantile(aggregates.map((item) => item.meanNs), 0.95);
    const trimmed = aggregates.filter((item) => item.estimatedMemoryBytes <= p95Memory && item.meanNs <= p95Mean);

    return trimmed.length >= 3 ? trimmed : aggregates;
  }, [aggregates, hideOutliers]);

  const visiblePoints = useMemo(
    () =>
      visibleAggregates.map((item) => ({
        implementation: item.implementation,
        meanNs: item.meanNs,
        estimatedMemoryBytes: item.estimatedMemoryBytes,
      })),
    [visibleAggregates],
  );

  const paretoFront = useMemo(() => buildParetoFront(visiblePoints), [visiblePoints]);

  const presentVariants = useMemo(() => {
    const variants = new Set(visibleAggregates.map((item) => item.variant));
    return {
      safe: variants.has("safe"),
      raw: variants.has("raw"),
      arena: variants.has("arena"),
      std: variants.has("std"),
      other: variants.has("other"),
    };
  }, [visibleAggregates]);

  if (aggregates.length === 0) {
    return (
      <AppGlassPanel className="space-y-2">
        <h2 className="font-display text-2xl">Tradeoff Matrix</h2>
        <p className="text-sm text-text-muted">No implementations match the current filter set.</p>
      </AppGlassPanel>
    );
  }

  const xValues = visiblePoints.map((point) => point.estimatedMemoryBytes).filter((value) => value > 0);
  const yValues = visiblePoints.map((point) => point.meanNs).filter((value) => value > 0);

  const xMin = Math.max(Math.min(...xValues), 1);
  const xMax = Math.max(...xValues, xMin);
  const yMin = Math.max(Math.min(...yValues), 1);
  const yMax = Math.max(...yValues, yMin);
  const latencyUnit = selectLatencyUnit(yMax);
  const memoryUnit = selectMemoryUnit(xMax);

  const xLinearMin = Math.max(0, xMin * 0.9);
  const xLinearMax = Math.max(xMax * 1.1, xLinearMin + 1);
  const yLinearMin = Math.max(0, yMin * 0.9);
  const yLinearMax = Math.max(yMax * 1.1, yLinearMin + 1);

  const xLogMin = Math.max(1, xMin * 0.9);
  const xLogMax = Math.max(xMax * 1.1, xLogMin * 1.05);
  const yLogMin = Math.max(1, yMin * 0.9);
  const yLogMax = Math.max(yMax * 1.1, yLogMin * 1.05);

  useEffect(() => {
    if (pinnedDetails && !visibleAggregates.some((item) => item.implementation === pinnedDetails)) {
      setPinnedDetails(null);
      setHovered(null);
    }
  }, [pinnedDetails, visibleAggregates]);

  useEffect(() => {
    if (!pinnedDetails) return;

    const onPointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) return;

      if (overlayRef.current?.contains(target)) return;
      if (target instanceof Element && target.closest('[data-tradeoff-node="true"]')) return;

      setPinnedDetails(null);
      setHovered(null);
    };

    window.addEventListener("pointerdown", onPointerDown);
    return () => window.removeEventListener("pointerdown", onPointerDown);
  }, [pinnedDetails]);

  const activeDetailsImplementation = hovered ?? pinnedDetails;
  const hoveredAggregate = activeDetailsImplementation
    ? visibleAggregates.find((item) => item.implementation === activeDetailsImplementation) ?? null
    : null;
  const detailsPinned = Boolean(pinnedDetails) && hovered === null;

  const pointMotionTransition = useMemo(
    () => ({ type: "spring" as const, stiffness: 220, damping: 26, mass: 0.72 }),
    [],
  );

  return (
    <AppGlassPanel className="space-y-4">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 className="font-display text-2xl">Tradeoff Matrix</h2>
          <p className="text-sm text-text-muted">
            X-axis shows estimated peak memory; Y-axis uses Criterion mean latency with adaptive units.
          </p>
        </div>
        <div className="flex items-center gap-3 text-right text-sm text-text/82">
          <p>Pareto points: {paretoFront.length}</p>
          <p>Showing: {visibleAggregates.length} / {aggregates.length}</p>
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3 rounded-md border border-panel-border bg-bg-elevated/55 px-3 py-2.5 text-sm">
        <label className="inline-flex items-center gap-2 text-text/82">
          <input
            type="checkbox"
            checked={scaleMode === "log"}
            onChange={(event) => onScaleModeChange(event.currentTarget.checked ? "log" : "linear")}
            className="h-4 w-4 rounded border-panel-border bg-bg-elevated"
          />
          Logarithmic Scale
        </label>
        <label className="inline-flex items-center gap-2 text-text/82">
          <input
            type="checkbox"
            checked={hideOutliers}
            onChange={(event) => onHideOutliersChange(event.currentTarget.checked)}
            className="h-4 w-4 rounded border-panel-border bg-bg-elevated"
          />
          Hide extreme outliers (95th percentile)
        </label>
      </div>

      {hideOutliers && visibleAggregates.length < 4 ? (
        <div className="flex justify-end">
          <button
            type="button"
            className="rounded border border-panel-border bg-panel/50 px-2 py-1 text-xs text-text-muted hover:bg-bg-elevated/65 hover:text-text"
            onClick={() => onHideOutliersChange(false)}
          >
            Show all points
          </button>
        </div>
      ) : null}

      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-md border border-panel-border bg-bg-elevated/48 px-3 py-2 text-xs text-text-muted">
        <span className="font-mono uppercase tracking-[0.12em] text-text/86">Legend</span>
        {presentVariants.safe ? (
          <span className="inline-flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: VARIANT_COLORS.safe }} aria-hidden="true" />
            Safe
          </span>
        ) : null}
        {presentVariants.raw ? (
          <span className="inline-flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: VARIANT_COLORS.raw }} aria-hidden="true" />
            Raw
          </span>
        ) : null}
        {presentVariants.arena ? (
          <span className="inline-flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: VARIANT_COLORS.arena }} aria-hidden="true" />
            Arena
          </span>
        ) : null}
        {presentVariants.std ? (
          <span className="inline-flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: VARIANT_COLORS.std }} aria-hidden="true" />
            Std Baseline
          </span>
        ) : null}
        {presentVariants.other ? (
          <span className="inline-flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: VARIANT_COLORS.other }} aria-hidden="true" />
            Other
          </span>
        ) : null}
        <span className="inline-flex items-center gap-1.5">
          <span
            className="h-0.5 w-6 rounded-full"
            style={{
              background: `linear-gradient(90deg, ${CHART_COLORS.paretoStart} 0%, ${CHART_COLORS.paretoEnd} 100%)`,
            }}
            aria-hidden="true"
          />
          Pareto frontier
        </span>
      </div>

      <div className="rounded-md border border-panel-border bg-bg-elevated/55 p-2">
        <div className="relative h-full w-full" style={{ height: "clamp(360px, 46vw, 560px)" }}>
          <ParentSize>
            {({ width }) => {
              const chartWidth = Math.max(width, MIN_CHART_WIDTH);
              const chartHeight = Math.max(Math.min(chartWidth * CHART_ASPECT, MAX_CHART_HEIGHT), MIN_CHART_HEIGHT);
              const padding = {
                top: 18,
                right: chartWidth < 760 ? 24 : 30,
                bottom: chartWidth < 760 ? 72 : 56,
                left: chartWidth < 760 ? 74 : 98,
              };

              const xScale =
                scaleMode === "log"
                  ? scaleLog<number>({
                      domain: [xLogMin, xLogMax],
                      range: [padding.left, chartWidth - padding.right],
                      clamp: true,
                    })
                  : scaleLinear<number>({
                      domain: [xLinearMin, xLinearMax],
                      range: [padding.left, chartWidth - padding.right],
                      clamp: true,
                    });

              const yScale =
                scaleMode === "log"
                  ? scaleLog<number>({
                      domain: [yLogMin, yLogMax],
                      range: [chartHeight - padding.bottom, padding.top],
                      clamp: true,
                    })
                  : scaleLinear<number>({
                      domain: [yLinearMin, yLinearMax],
                      range: [chartHeight - padding.bottom, padding.top],
                      clamp: true,
                    });

              const xTicks =
                scaleMode === "linear"
                  ? buildLinearTicks(xLinearMin, xLinearMax, 5).filter((value) => value > 0)
                  : [0, 0.25, 0.5, 0.75, 1].map((ratio) => xLogMin * (xLogMax / xLogMin) ** ratio);

              const yTicks =
                scaleMode === "linear"
                  ? buildLinearTicks(yLinearMin, yLinearMax, 5).filter((value) => value > 0)
                  : [0, 0.25, 0.5, 0.75, 1].map((ratio) => yLogMin * (yLogMax / yLogMin) ** ratio);

              const xAxisSpan = chartWidth - padding.left - padding.right;
              const denseXLabels = xTicks.length > 0 && xAxisSpan / xTicks.length < 120;

              const overlayPlacement = hoveredAggregate
                ? computeOverlayPlacement(
                    xScale(Math.max(hoveredAggregate.estimatedMemoryBytes, 1)),
                    yScale(Math.max(hoveredAggregate.meanNs, 1)),
                    chartWidth,
                    chartHeight,
                  )
                : null;

              return (
                <>
                  <svg viewBox={`0 0 ${chartWidth} ${chartHeight}`} className="h-full w-full" role="img" aria-label="Memory versus speed scatter matrix with Pareto frontier">
                    <title>Tradeoff Matrix</title>
                    <desc>Scatter plot of estimated memory and mean latency for each implementation with highlighted Pareto frontier.</desc>
                    <defs>
                      <linearGradient id="paretoGlow" x1="0" x2="1">
                        <stop offset="0%" stopColor={CHART_COLORS.paretoStart} stopOpacity="0.9" />
                        <stop offset="100%" stopColor={CHART_COLORS.paretoEnd} stopOpacity="0.9" />
                      </linearGradient>
                      <filter id="paretoNeon" x="-40%" y="-40%" width="180%" height="180%">
                        <feGaussianBlur stdDeviation="3.2" result="blur" />
                        <feMerge>
                          <feMergeNode in="blur" />
                          <feMergeNode in="SourceGraphic" />
                        </feMerge>
                      </filter>
                    </defs>

                    {yTicks.map((tick, index) => {
                      const y = yScale(Math.max(tick, 1));
                      return (
                        <line
                          key={`y-${index}-${tick}`}
                          x1={padding.left}
                          x2={chartWidth - padding.right}
                          y1={y}
                          y2={y}
                          stroke={CHART_COLORS.grid}
                          strokeDasharray="4 6"
                        />
                      );
                    })}

                    {xTicks.map((tick, index) => {
                      const x = xScale(Math.max(tick, 1));
                      return (
                        <line
                          key={`x-${index}-${tick}`}
                          y1={padding.top}
                          y2={chartHeight - padding.bottom}
                          x1={x}
                          x2={x}
                          stroke={CHART_COLORS.gridSubtle}
                          strokeDasharray="4 8"
                        />
                      );
                    })}

                    <line
                      x1={padding.left}
                      x2={chartWidth - padding.right}
                      y1={chartHeight - padding.bottom}
                      y2={chartHeight - padding.bottom}
                      stroke={CHART_COLORS.axis}
                    />
                    <line
                      x1={padding.left}
                      x2={padding.left}
                      y1={chartHeight - padding.bottom}
                      y2={padding.top}
                      stroke={CHART_COLORS.axis}
                    />

                    <text x={chartWidth / 2} y={chartHeight - (denseXLabels ? 10 : 12)} fill={CHART_COLORS.label} textAnchor="middle" fontSize="13">
                      Estimated Peak Memory ({memoryUnit.label})
                    </text>
                    <text
                      x={26}
                      y={chartHeight / 2}
                      fill={CHART_COLORS.label}
                      fontSize="13"
                      textAnchor="middle"
                      transform={`rotate(-90, 26, ${chartHeight / 2})`}
                    >
                      Mean Execution Time ({latencyUnit.label})
                    </text>

                    {xTicks.map((tick, index) => {
                      const x = xScale(Math.max(tick, 1));
                      const tickY = chartHeight - padding.bottom + (denseXLabels && index % 2 === 1 ? 30 : 16);
                      return (
                        <text
                          key={`x-label-${index}`}
                          x={x}
                          y={tickY}
                          fill={CHART_COLORS.label}
                          textAnchor="middle"
                          fontSize={denseXLabels ? "11" : "12"}
                          fontFamily="var(--font-ibm-plex-mono), monospace"
                        >
                          {formatTickWithUnit(tick, memoryUnit)}
                        </text>
                      );
                    })}

                    {yTicks.map((tick, index) => {
                      const y = yScale(Math.max(tick, 1));
                      return (
                        <text
                          key={`y-label-${index}`}
                          x={padding.left - 10}
                          y={y + 4}
                          fill={CHART_COLORS.label}
                          textAnchor="end"
                          fontSize="12"
                          fontFamily="var(--font-ibm-plex-mono), monospace"
                        >
                          {formatTickWithUnit(tick, latencyUnit)}
                        </text>
                      );
                    })}

                    <LinePath
                      data={paretoFront}
                      x={(item) => xScale(item.estimatedMemoryBytes) ?? 0}
                      y={(item) => yScale(item.meanNs) ?? 0}
                      stroke="url(#paretoGlow)"
                      strokeWidth={6.2}
                      strokeOpacity={0.45}
                      filter="url(#paretoNeon)"
                    />

                    <LinePath
                      data={paretoFront}
                      x={(item) => xScale(item.estimatedMemoryBytes) ?? 0}
                      y={(item) => yScale(item.meanNs) ?? 0}
                      stroke="url(#paretoGlow)"
                      strokeWidth={3.2}
                    />

                    {visibleAggregates.map((point) => {
                      const activeComparison = selectedImplementations.includes(point.implementation);
                      const isHovered = hovered === point.implementation;
                      const isPinnedNode = pinnedDetails === point.implementation;
                      const cx = xScale(Math.max(point.estimatedMemoryBytes, 1));
                      const cy = yScale(Math.max(point.meanNs, 1));

                      const pointColor =
                        point.variant === "safe"
                          ? VARIANT_COLORS.safe
                          : point.variant === "raw"
                            ? VARIANT_COLORS.raw
                            : point.variant === "arena"
                              ? VARIANT_COLORS.arena
                              : point.variant === "std"
                                ? VARIANT_COLORS.std
                                : VARIANT_COLORS.other;

                      return (
                        <motion.g
                          key={point.implementation}
                          initial={false}
                          animate={{ x: cx, y: cy }}
                          transition={pointMotionTransition}
                          data-tradeoff-node="true"
                          onMouseEnter={() => setHovered(point.implementation)}
                          onMouseLeave={() => setHovered(null)}
                          onClick={(event) => {
                            event.stopPropagation();
                            if (pinnedDetails === point.implementation) {
                              setPinnedDetails(null);
                              setHovered(null);
                              return;
                            }

                            setPinnedDetails(point.implementation);
                            setHovered(null);
                          }}
                          role="button"
                          tabIndex={0}
                          aria-pressed={isPinnedNode}
                          aria-label={`${point.implementation}, mean latency ${formatTickWithUnit(point.meanNs, latencyUnit)} ${latencyUnit.label}, estimated memory ${formatTickWithUnit(point.estimatedMemoryBytes, memoryUnit)} ${memoryUnit.label}${isPinnedNode ? ", details pinned" : ""}`}
                          onKeyDown={(event) => {
                            if (event.key === "Enter" || event.key === " ") {
                              event.preventDefault();
                              if (pinnedDetails === point.implementation) {
                                setPinnedDetails(null);
                                setHovered(null);
                                return;
                              }

                              setPinnedDetails(point.implementation);
                              setHovered(null);
                            }
                          }}
                        >
                          <motion.circle
                            initial={false}
                            animate={{ r: isPinnedNode ? 8 : isHovered ? 7 : activeComparison ? 6.3 : 5.6 }}
                            transition={{ duration: 0.2, ease: "easeOut" }}
                            cx={0}
                            cy={0}
                            fill={pointColor}
                            fillOpacity={isPinnedNode ? 0.92 : isHovered || activeComparison ? 0.82 : 0.66}
                            stroke={isPinnedNode || isHovered || activeComparison ? CHART_COLORS.pointActiveStroke : CHART_COLORS.pointStroke}
                            strokeWidth={isPinnedNode || isHovered || activeComparison ? 2 : 1}
                          />
                        </motion.g>
                      );
                    })}
                  </svg>

                  {hoveredAggregate ? (
                    <div
                      ref={overlayRef}
                      className="pointer-events-auto absolute z-20 min-w-[300px] max-w-[440px] space-y-2 rounded-md border border-primary/45 bg-panel/95 p-3 text-sm text-text shadow-panel backdrop-blur-[10px]"
                      style={
                        overlayPlacement
                          ? {
                              left: overlayPlacement.left,
                              top: overlayPlacement.top,
                              transform: overlayPlacement.transform,
                            }
                          : undefined
                      }
                    >
                      <p className="font-mono text-sm uppercase tracking-[0.1em] text-primary">Implementation Details</p>

                      <div className="grid gap-x-4 gap-y-1.5 md:grid-cols-2">
                        <p className="break-words">
                          <span className="text-text-muted">Implementation:</span> {hoveredAggregate.implementation}
                        </p>
                        <p className="break-words">
                          <span className="text-text-muted">Mean latency:</span> {Math.round(hoveredAggregate.meanNs).toLocaleString()} ns
                        </p>
                        <p className="break-words">
                          <span className="text-text-muted">Estimated peak memory:</span> {formatBytes(hoveredAggregate.estimatedMemoryBytes)}
                        </p>
                        <p className="break-words">
                          <span className="text-text-muted">CI range:</span> {Math.round(hoveredAggregate.ciLowerNs).toLocaleString()} - {Math.round(hoveredAggregate.ciUpperNs).toLocaleString()} ns
                        </p>
                      </div>

                      <p className="text-xs text-text-muted">
                        {detailsPinned
                          ? "Click the node again or anywhere outside this panel to close details."
                          : "Move away from the node to dismiss details, or click the node to pin this panel."}
                      </p>
                    </div>
                  ) : null}
                </>
              );
            }}
          </ParentSize>
        </div>
      </div>

      <p className="text-sm text-text-muted">Hover a node to inspect metrics. Click a node to pin details, then click it again or outside the panel to close.</p>
    </AppGlassPanel>
  );
}

export const TradeoffMatrix = memo(TradeoffMatrixInner);
