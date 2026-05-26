"use client";

import { motion } from "framer-motion";
import { memo, useEffect, useMemo, useState } from "react";
import { ParentSize } from "@visx/responsive";
import { scaleLinear, scaleLog } from "@visx/scale";
import { LinePath } from "@visx/shape";

import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import {
  buildImplementationAggregates,
  buildParetoFront,
  formatBytes,
} from "@/lib/bench/analytics";
import { useDashboardStore } from "@/lib/bench/store";
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
const MIN_CHART_HEIGHT = 300;
const MAX_CHART_HEIGHT = 500;
const CHART_ASPECT = 0.78;

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

function formatCompactCount(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(2)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return Math.round(value).toLocaleString();
}

function formatPercent(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "n/a";
  return `${(value * 100).toFixed(2)}%`;
}

function InfoDot({ help }: { help: string }) {
  return (
    <span className="group relative ml-1 inline-flex align-middle">
      <span className="inline-flex h-3.5 w-3.5 items-center justify-center rounded-full border border-panel-border/90 bg-panel/65 text-[9px] font-semibold text-text-muted">
        i
      </span>
      <span className="pointer-events-none absolute left-1/2 top-full z-30 mt-1.5 w-52 -translate-x-1/2 rounded-md border border-panel-border/85 bg-bg-elevated/95 px-2 py-1.5 text-[11px] normal-case leading-snug text-text-muted opacity-0 shadow-panel backdrop-blur-sm transition-opacity group-hover:opacity-100">
        {help}
      </span>
    </span>
  );
}

function SnapshotMetric({ label, value, help }: { label: string; value: string; help: string }) {
  return (
    <div className="rounded-md border border-panel-border/70 bg-bg-elevated/55 px-2.5 py-2 text-left">
      <p className="text-[10px] uppercase tracking-[0.11em] text-text-muted">
        {label}
        <InfoDot help={help} />
      </p>
      <p className="mt-0.5 font-mono text-sm text-text">{value}</p>
    </div>
  );
}

function StatusMetric({ label, value, help }: { label: string; value: string; help: string }) {
  return (
    <div className="rounded-md border border-panel-border/70 bg-bg-elevated/55 px-2.5 py-2 text-left">
      <p className="text-[10px] uppercase tracking-[0.11em] text-text-muted">
        {label}
        <InfoDot help={help} />
      </p>
      <p className="mt-0.5 font-mono text-sm text-text">{value}</p>
    </div>
  );
}

type OverlayPlacement = {
  left: string;
  top: string;
  transform: string;
};

function computeOverlayPlacement(cx: number, cy: number, width: number, height: number): OverlayPlacement {
  if (!Number.isFinite(cx) || !Number.isFinite(cy)) {
    return {
      left: `${Math.max(12, Math.floor(width * 0.5 - 180))}px`,
      top: `${Math.max(12, Math.floor(height * 0.5 - 72))}px`,
      transform: "translate(0, 0)",
    };
  }

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
  const [hoverPosition, setHoverPosition] = useState<{ x: number; y: number } | null>(null);
  const hoveredImplementation = useDashboardStore((state) => state.hoveredImplementation);
  const setHoveredImplementation = useDashboardStore((state) => state.setHoveredImplementation);

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
  const soleParetoImplementation = paretoFront.length === 1 ? paretoFront[0]?.implementation ?? null : null;

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

  const profilingSnapshot = useMemo(() => {
    const instructions = visibleAggregates
      .map((item) => item.profiling.instructions)
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));
    const l1MissRate = visibleAggregates
      .map((item) => item.profiling.l1DataMissRate)
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));
    const peakBytes = visibleAggregates
      .map((item) => item.profiling.peakBytes)
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));

    return {
      instructionsP50: instructions.length > 0 ? quantile(instructions, 0.5) : null,
      l1MissRateP50: l1MissRate.length > 0 ? quantile(l1MissRate, 0.5) : null,
      peakBytesP50: peakBytes.length > 0 ? quantile(peakBytes, 0.5) : null,
    };
  }, [visibleAggregates]);

  useEffect(() => {
    if (hovered && !visibleAggregates.some((item) => item.implementation === hovered)) {
      setHovered(null);
      setHoverPosition(null);
      if (hoveredImplementation === hovered) {
        setHoveredImplementation(null);
      }
    }
  }, [hovered, hoveredImplementation, setHoveredImplementation, visibleAggregates]);

  const activeDetailsImplementation = hovered;
  const hoveredAggregate = activeDetailsImplementation
    ? visibleAggregates.find((item) => item.implementation === activeDetailsImplementation) ?? null
    : null;

  const pointMotionTransition = useMemo(
    () => ({ type: "spring" as const, stiffness: 220, damping: 26, mass: 0.72 }),
    [],
  );

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

  const xMin = xValues.length > 0 ? Math.max(Math.min(...xValues), 1) : 1;
  const xMax = xValues.length > 0 ? Math.max(...xValues, xMin) : 2;
  const yMin = yValues.length > 0 ? Math.max(Math.min(...yValues), 1) : 1;
  const yMax = yValues.length > 0 ? Math.max(...yValues, yMin) : 2;
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

  return (
    <AppGlassPanel className="space-y-4" data-dashboard-matrix-card="true">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 className="font-display text-2xl">Tradeoff Matrix</h2>
          <p className="text-sm text-text-muted">
            Compare memory use against speed. Hover for details, and click a point to add or remove it from the Inspector.
          </p>
        </div>
        <div className="w-full max-w-[460px] rounded-lg border border-panel-border/70 bg-bg-elevated/42 p-2.5 text-right text-xs text-text/84">
          <div className="grid gap-1.5 sm:grid-cols-2">
            <StatusMetric
              label="Pareto"
              value={String(paretoFront.length)}
              help="Shows how many implementations are currently non-dominated on the speed vs memory frontier."
            />
            <StatusMetric
              label="Visible"
              value={`${visibleAggregates.length}/${aggregates.length}`}
              help="Shows how many points are currently in view after optional outlier trimming."
            />
          </div>
          <div className="mt-2 border-t border-panel-border/60 pt-2">
            <p className="text-left text-[10px] uppercase tracking-[0.12em] text-text-muted">Profiling Snapshot (p50)</p>
            <div className="mt-2 grid gap-1.5 sm:grid-cols-3">
              <SnapshotMetric
                label="Instructions (Ir)"
                value={formatCompactCount(profilingSnapshot.instructionsP50)}
                help="Approximate CPU work per operation. Lower usually means faster execution for similar algorithms."
              />
              <SnapshotMetric
                label="L1 Miss Rate"
                value={formatPercent(profilingSnapshot.l1MissRateP50)}
                help="Fraction of data accesses missing L1 cache. Lower values usually improve latency consistency."
              />
              <SnapshotMetric
                label="Peak Bytes"
                value={profilingSnapshot.peakBytesP50 == null ? "n/a" : formatBytes(profilingSnapshot.peakBytesP50)}
                help="Typical high-water memory footprint while running. Useful for budget-aware deployment limits."
              />
            </div>
          </div>
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
          Hide extreme outliers
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

      <div className="rounded-md border border-panel-border bg-bg-elevated/55 p-0.5">
        <div className="relative w-full" style={{ height: "clamp(300px, 39vw, 500px)" }}>
          <ParentSize>
            {({ width }) => {
              const chartWidth = Math.max(width, MIN_CHART_WIDTH);
              const chartHeight = Math.max(Math.min(chartWidth * CHART_ASPECT, MAX_CHART_HEIGHT), MIN_CHART_HEIGHT);
              const padding = {
                top: 6,
                right: chartWidth < 760 ? 16 : 22,
                bottom: chartWidth < 760 ? 50 : 44,
                left: chartWidth < 760 ? 74 : 98,
              };

              const yAxisCenter = (padding.top + (chartHeight - padding.bottom)) / 2;

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
                    hoverPosition?.x ?? xScale(Math.max(hoveredAggregate.estimatedMemoryBytes, 1)),
                    hoverPosition?.y ?? yScale(Math.max(hoveredAggregate.meanNs, 1)),
                    chartWidth,
                    chartHeight,
                  )
                : null;

              return (
                <>
                  <svg viewBox={`0 0 ${chartWidth} ${chartHeight}`} className="h-full w-full" role="img" aria-label="Memory versus speed scatter matrix with Pareto frontier">
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
                      y={yAxisCenter}
                      fill={CHART_COLORS.label}
                      fontSize="13"
                      textAnchor="middle"
                      transform={`rotate(-90, 26, ${yAxisCenter})`}
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
                      const isCrossHighlighted = hoveredImplementation === point.implementation;
                      const isSolePareto = soleParetoImplementation === point.implementation;
                      const dimmedByCrossHover = hoveredImplementation !== null && !isCrossHighlighted;
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
                          onMouseEnter={() => {
                            setHovered(point.implementation);
                            setHoveredImplementation(point.implementation);
                          }}
                          onMouseMove={(event) => {
                            const bounds = event.currentTarget.ownerSVGElement?.getBoundingClientRect();
                            if (!bounds) return;
                            setHoverPosition({
                              x: event.clientX - bounds.left,
                              y: event.clientY - bounds.top,
                            });
                          }}
                          onMouseLeave={() => {
                            setHovered(null);
                            setHoverPosition(null);
                            setHoveredImplementation(null);
                          }}
                          onClick={(event) => {
                            event.stopPropagation();
                            setHovered(point.implementation);
                            setHoveredImplementation(point.implementation);
                            onToggleImplementation(point.implementation);
                          }}
                          role="button"
                          tabIndex={0}
                          aria-pressed={activeComparison}
                          aria-label={`${point.implementation}, mean latency ${formatTickWithUnit(point.meanNs, latencyUnit)} ${latencyUnit.label}, estimated memory ${formatTickWithUnit(point.estimatedMemoryBytes, memoryUnit)} ${memoryUnit.label}${activeComparison ? ", pinned in the inspector" : ", available to add to the inspector"}`}
                          onKeyDown={(event) => {
                            if (event.key === "Enter" || event.key === " ") {
                              event.preventDefault();
                              setHovered(point.implementation);
                              setHoveredImplementation(point.implementation);
                              onToggleImplementation(point.implementation);
                            }
                          }}
                          style={{
                            opacity: dimmedByCrossHover ? 0.2 : 1,
                          }}
                        >
                          {isSolePareto ? (
                            <motion.circle
                              initial={false}
                              animate={{ r: isCrossHighlighted ? 15.5 : isHovered ? 14.2 : 13.2 }}
                              transition={{ duration: 0.2, ease: "easeOut" }}
                              cx={0}
                              cy={0}
                              fill="url(#paretoGlow)"
                              fillOpacity={dimmedByCrossHover ? 0.12 : 0.18}
                              stroke="url(#paretoGlow)"
                              strokeOpacity={dimmedByCrossHover ? 0.22 : 0.75}
                              strokeWidth={1.6}
                              style={{ filter: "url(#paretoNeon)" }}
                            />
                          ) : null}
                          <motion.circle
                            initial={false}
                            animate={{
                              r: isCrossHighlighted
                                ? isSolePareto
                                  ? 11.2
                                  : 9.8
                                : isHovered
                                  ? isSolePareto
                                    ? 8.9
                                    : 7.4
                                  : activeComparison
                                    ? isSolePareto
                                      ? 7.8
                                      : 6.5
                                    : isSolePareto
                                      ? 7
                                      : 5.6,
                            }}
                            transition={{ duration: 0.2, ease: "easeOut" }}
                            cx={0}
                            cy={0}
                            fill={pointColor}
                            fillOpacity={
                              dimmedByCrossHover
                                ? 0.24
                                : isCrossHighlighted
                                  ? 0.94
                                  : isHovered || activeComparison
                                    ? 0.82
                                    : 0.66
                            }
                            stroke={
                              isSolePareto || isHovered || activeComparison || isCrossHighlighted
                                ? CHART_COLORS.pointActiveStroke
                                : CHART_COLORS.pointStroke
                            }
                            strokeWidth={isCrossHighlighted || isHovered || activeComparison ? 2.6 : isSolePareto ? 2.1 : 1}
                            style={{
                              filter: isCrossHighlighted || isSolePareto
                                ? `drop-shadow(0 0 6px ${pointColor}) drop-shadow(0 0 10px ${pointColor})`
                                : undefined,
                            }}
                          />
                        </motion.g>
                      );
                    })}
                  </svg>

                  {hoveredAggregate ? (
                    <div
                      className="pointer-events-none absolute z-20 min-w-[300px] max-w-[440px] space-y-2 rounded-md border border-primary/45 bg-panel/95 p-3 text-sm text-text shadow-panel backdrop-blur-[10px]"
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

                      <div className="grid gap-1.5 rounded border border-panel-border/70 bg-bg-elevated/45 p-2 text-xs md:grid-cols-2">
                        <p className="break-words">
                          <span className="text-text-muted">Instructions:</span>{" "}
                          <span className="font-mono text-text">{formatCompactCount(hoveredAggregate.profiling.instructions)}</span>
                        </p>
                        <p className="break-words">
                          <span className="text-text-muted">L1 data miss rate:</span>{" "}
                          <span className="font-mono text-text">{formatPercent(hoveredAggregate.profiling.l1DataMissRate)}</span>
                        </p>
                        <p className="break-words">
                          <span className="text-text-muted">Alloc churn:</span>{" "}
                          <span className="font-mono text-text">{formatCompactCount(hoveredAggregate.profiling.totalBytes)} / {formatCompactCount(hoveredAggregate.profiling.peakBytes)}</span>
                        </p>
                        <p className="break-words">
                          <span className="text-text-muted">Churn ratio:</span>{" "}
                          <span className="font-mono text-text">{hoveredAggregate.profiling.allocationChurnRatio != null ? `${hoveredAggregate.profiling.allocationChurnRatio.toFixed(2)}x` : "n/a"}</span>
                        </p>
                      </div>

                      <p className="text-xs text-text-muted">
                        {selectedImplementations.includes(hoveredAggregate.implementation)
                          ? "Click to remove this implementation from the Inspector."
                          : "Click to add this implementation to the Inspector."}
                      </p>
                    </div>
                  ) : null}
                </>
              );
            }}
          </ParentSize>
        </div>
      </div>
    </AppGlassPanel>
  );
}

export const TradeoffMatrix = memo(TradeoffMatrixInner);
