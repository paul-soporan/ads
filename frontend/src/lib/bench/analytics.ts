import type {
  CriterionRecord,
  Distribution,
  NormalizedBenchmarkDataset,
  PayloadKind,
  VariantKind,
  WorkloadFamily,
} from "./types";

export interface ImplementationAggregate {
  implementation: string;
  variant: VariantKind;
  family: string;
  workload: WorkloadFamily;
  distribution: Distribution;
  payload: PayloadKind;
  meanNs: number;
  ciLowerNs: number;
  ciUpperNs: number;
  stdDevNs: number;
  averageSize: number;
  throughputElements: number;
  sampleCount: number;
  estimatedMemoryBytes: number;
  callgrind: {
    Ir?: number;
    Dr?: number;
    Dw?: number;
    I1mr?: number;
    D1mr?: number;
    D1mw?: number;
    ILmr?: number;
    DLmr?: number;
    DLmw?: number;
  };
  profiling: {
    hasCallgrind: boolean;
    hasDhat: boolean;
    instructions: number | null;
    memoryAccesses: number | null;
    l1DataMissRate: number | null;
    l1InstructionMissRate: number | null;
    llDataMissRate: number | null;
    peakBytes: number | null;
    totalBytes: number | null;
    peakBlocks: number | null;
    totalBlocks: number | null;
    allocationChurnRatio: number | null;
    bytesPerElement: number | null;
  };
}

export interface ParetoPoint {
  implementation: string;
  meanNs: number;
  estimatedMemoryBytes: number;
}

type RadarVariantKey = "safe" | "raw" | "arena";
type RadarTransform = "linear" | "log1p";

export type VariantRadarMetricFormat =
  | "latency_ns"
  | "ratio"
  | "bytes_per_element"
  | "instructions_per_element"
  | "allocation_ratio";

export interface VariantRadarMetric {
  key: string;
  axis: string;
  description: string;
  formatter: VariantRadarMetricFormat;
  lowerIsBetter: boolean;
  contextCount: number;
  contextMedian: number | null;
  safe: number | null;
  raw: number | null;
  arena: number | null;
  safeRaw: number | null;
  rawRaw: number | null;
  arenaRaw: number | null;
}

export interface VariantRadarResult {
  metrics: VariantRadarMetric[];
  omittedAxes: string[];
}

type RadarMetricSpec = {
  key: string;
  axis: string;
  description: string;
  formatter: VariantRadarMetricFormat;
  lowerIsBetter: boolean;
  transform: RadarTransform;
  getValue: (item: ImplementationAggregate) => number | null;
};

function toFamilyName(implementation: string): string {
  return implementation
    .replace(/^(contains_|insert_|remove_|mix_|thrash_|push_pop_|workload_read_heavy_|workload_write_heavy_)/, "")
    .replace(/(^|_)safe(_|$)/g, "_")
    .replace(/(^|_)raw(_|$)/g, "_")
    .replace(/(^|_)arena(_|$)/g, "_")
    .replace(/(^|_)std(_|$)/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "")
    .replace(/_t\d+$/g, "");
}

function normalizeImplementationKey(input: string): string {
  return input.toLowerCase().replace(/_t\d+$/, "").replace(/_+/g, "_").replace(/^_+|_+$/g, "");
}

function averageMetrics(metricsList: Record<string, number>[]): Record<string, number> {
  const totals = new Map<string, number>();
  const counts = new Map<string, number>();

  for (const metrics of metricsList) {
    for (const [key, value] of Object.entries(metrics)) {
      totals.set(key, (totals.get(key) ?? 0) + value);
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }

  const averaged: Record<string, number> = {};
  for (const [key, total] of totals.entries()) {
    const count = counts.get(key) ?? 1;
    averaged[key] = total / count;
  }

  return averaged;
}

function extractCallgrindMetadata(dataset: NormalizedBenchmarkDataset) {
  const byJoinKey = new Map<string, Record<string, number>>();
  const joinKeyBuckets = new Map<string, Record<string, number>[]>();

  for (const record of dataset.callgrind) {
    const joinKey = record.joinKey;
    if (!joinKey) continue;

    const metricsForJoinKey = joinKeyBuckets.get(joinKey) ?? [];
    metricsForJoinKey.push(record.metrics);
    joinKeyBuckets.set(joinKey, metricsForJoinKey);
  }

  for (const [joinKey, metrics] of joinKeyBuckets.entries()) {
    byJoinKey.set(joinKey, averageMetrics(metrics));
  }

  return { byJoinKey };
}

function average(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function quantile(values: number[], q: number): number {
  if (values.length === 0) return 0;

  const sorted = values.slice().sort((a, b) => a - b);
  const position = (sorted.length - 1) * clamp(q, 0, 1);
  const lowerIndex = Math.floor(position);
  const upperIndex = Math.ceil(position);
  const lower = sorted[lowerIndex] ?? sorted[0] ?? 0;
  const upper = sorted[upperIndex] ?? sorted[sorted.length - 1] ?? lower;

  if (lowerIndex === upperIndex) return lower;

  const weight = position - lowerIndex;
  return lower + (upper - lower) * weight;
}

function transformForRadar(value: number, mode: RadarTransform): number {
  if (mode === "log1p") {
    return Math.log1p(Math.max(0, value));
  }

  return value;
}

function robustRadarScore(value: number, contextualValues: number[], lowerIsBetter: boolean, transform: RadarTransform): number {
  if (contextualValues.length < 2) return 0.5;

  const transformed = contextualValues.map((item) => transformForRadar(item, transform));
  const median = quantile(transformed, 0.5);
  const q1 = quantile(transformed, 0.25);
  const q3 = quantile(transformed, 0.75);
  const range = Math.max(...transformed) - Math.min(...transformed);
  const spread = Math.max(q3 - q1, range / 4, 1e-9);
  const offset = (transformForRadar(value, transform) - median) / spread;
  const direction = lowerIsBetter ? -offset : offset;
  const sigmoid = 1 / (1 + Math.exp(-direction * 1.35));

  return clamp(sigmoid, 0.06, 0.94);
}

function isRadarVariant(variant: VariantKind): variant is RadarVariantKey {
  return variant === "safe" || variant === "raw" || variant === "arena";
}

function averageNullable(values: Array<number | null | undefined>): number | null {
  const present = values.filter((value): value is number => typeof value === "number" && Number.isFinite(value));
  if (present.length === 0) return null;
  return average(present);
}

function buildDhatLookup(dataset: NormalizedBenchmarkDataset): {
  byJoinKey: Map<string, {
    peakBytes: number | null;
    totalBytes: number | null;
    peakBlocks: number | null;
    totalBlocks: number | null;
  }>;
} {
  const joinKeyBuckets = new Map<string, Array<{
    peakBytes: number | null;
    totalBytes: number | null;
    peakBlocks: number | null;
    totalBlocks: number | null;
  }>>();

  for (const row of dataset.dhat) {
    const joinKey = row.joinKey;
    if (!joinKey) continue;

    const valuesForJoinKey = joinKeyBuckets.get(joinKey) ?? [];
    valuesForJoinKey.push({
      peakBytes: row.maxBytes,
      totalBytes: row.totalBytes,
      peakBlocks: row.maxBlocks,
      totalBlocks: row.totalBlocks,
    });
    joinKeyBuckets.set(joinKey, valuesForJoinKey);
  }

  const byJoinKey = new Map<string, {
    peakBytes: number | null;
    totalBytes: number | null;
    peakBlocks: number | null;
    totalBlocks: number | null;
  }>();
  for (const [joinKey, values] of joinKeyBuckets.entries()) {
    byJoinKey.set(joinKey, {
      peakBytes: averageNullable(values.map((value) => value.peakBytes)),
      totalBytes: averageNullable(values.map((value) => value.totalBytes)),
      peakBlocks: averageNullable(values.map((value) => value.peakBlocks)),
      totalBlocks: averageNullable(values.map((value) => value.totalBlocks)),
    });
  }

  return { byJoinKey };
}

export function buildImplementationAggregates(
  records: CriterionRecord[],
  dataset: NormalizedBenchmarkDataset,
): ImplementationAggregate[] {
  const grouped = new Map<string, CriterionRecord[]>();

  for (const record of records) {
    const bucket = grouped.get(record.implementation);
    if (bucket) {
      bucket.push(record);
    } else {
      grouped.set(record.implementation, [record]);
    }
  }

  const callgrindMetadata = extractCallgrindMetadata(dataset);
  const dhatLookup = buildDhatLookup(dataset);

  const provisional = [...grouped.entries()].map(([implementation, list]) => {
    const sampleCount = list.length;

    const sumMean = list.reduce((acc, item) => acc + item.meanNs, 0);
    const sumLower = list.reduce((acc, item) => acc + item.ciLowerNs, 0);
    const sumUpper = list.reduce((acc, item) => acc + item.ciUpperNs, 0);
    const sumStd = list.reduce((acc, item) => acc + item.stdDevNs, 0);
    const avgSize = list.reduce((acc, item) => acc + item.size, 0) / sampleCount;
    const throughput = list.reduce((acc, item) => acc + (item.throughputElements ?? item.size), 0) / sampleCount;
    const variant = list[0]?.variant ?? "other";

    const workloadCounts = new Map<WorkloadFamily, number>();
    const distributionCounts = new Map<Distribution, number>();
    const payloadCounts = new Map<PayloadKind, number>();

    for (const row of list) {
      workloadCounts.set(row.workload, (workloadCounts.get(row.workload) ?? 0) + 1);
      distributionCounts.set(row.distribution, (distributionCounts.get(row.distribution) ?? 0) + 1);
      payloadCounts.set(row.payload, (payloadCounts.get(row.payload) ?? 0) + 1);
    }

    const mode = <T,>(counts: Map<T, number>, fallback: T): T => {
      let best: T = fallback;
      let bestCount = -1;
      for (const [key, count] of counts.entries()) {
        if (count > bestCount) {
          best = key;
          bestCount = count;
        }
      }
      return best;
    };

    const workload = mode(workloadCounts, "other");
    const distribution = mode(distributionCounts, "other");
    const payload = mode(payloadCounts, "other");

    const callgrindValues = list.map((item) => {
      const value = callgrindMetadata.byJoinKey.get(item.joinKey);
      return value ?? null;
    });

    const dhatValues = list.map((item) => {
      const value = dhatLookup.byJoinKey.get(item.joinKey);
      return value ?? null;
    });

    const presentCallgrind = callgrindValues.filter((value): value is Record<string, number> => value !== null);
    const presentDhat = dhatValues.filter((value): value is {
      peakBytes: number | null;
      totalBytes: number | null;
      peakBlocks: number | null;
      totalBlocks: number | null;
    } => value !== null);

    const estimatedMemoryBytes = averageNullable(presentDhat.map((value) => value.peakBytes)) ?? 0;
    const callgrind = presentCallgrind.length > 0 ? averageMetrics(presentCallgrind) : {};

    const instructions = callgrind.Ir ?? null;
    const dataReads = callgrind.Dr ?? null;
    const dataWrites = callgrind.Dw ?? null;
    const i1Misses = callgrind.I1mr ?? null;
    const d1ReadMisses = callgrind.D1mr ?? null;
    const d1WriteMisses = callgrind.D1mw ?? null;
    const llReadMisses = callgrind.DLmr ?? null;
    const llWriteMisses = callgrind.DLmw ?? null;

    const memoryAccesses =
      dataReads != null && dataWrites != null
        ? dataReads + dataWrites
        : null;

    const l1DataMissRate =
      memoryAccesses != null && memoryAccesses > 0 && d1ReadMisses != null && d1WriteMisses != null
        ? (d1ReadMisses + d1WriteMisses) / memoryAccesses
        : null;

    const l1InstructionMissRate =
      instructions != null && instructions > 0 && i1Misses != null
        ? i1Misses / instructions
        : null;

    const llDataMissRate =
      memoryAccesses != null && memoryAccesses > 0 && llReadMisses != null && llWriteMisses != null
        ? (llReadMisses + llWriteMisses) / memoryAccesses
        : null;

    const peakBytes = averageNullable(presentDhat.map((value) => value.peakBytes));
    const totalBytes = averageNullable(presentDhat.map((value) => value.totalBytes));
    const peakBlocks = averageNullable(presentDhat.map((value) => value.peakBlocks));
    const totalBlocks = averageNullable(presentDhat.map((value) => value.totalBlocks));

    const allocationChurnRatio =
      peakBytes != null && peakBytes > 0 && totalBytes != null
        ? totalBytes / peakBytes
        : null;

    const bytesPerElement =
      peakBytes != null && throughput > 0
        ? peakBytes / throughput
        : null;

    return {
      implementation,
      variant,
      family: toFamilyName(implementation),
      workload,
      distribution,
      payload,
      meanNs: sumMean / sampleCount,
      ciLowerNs: sumLower / sampleCount,
      ciUpperNs: sumUpper / sampleCount,
      stdDevNs: sumStd / sampleCount,
      averageSize: avgSize,
      throughputElements: throughput,
      sampleCount,
      estimatedMemoryBytes,
      callgrind,
      profiling: {
        hasCallgrind: presentCallgrind.length > 0,
        hasDhat: presentDhat.length > 0,
        instructions,
        memoryAccesses,
        l1DataMissRate,
        l1InstructionMissRate,
        llDataMissRate,
        peakBytes,
        totalBytes,
        peakBlocks,
        totalBlocks,
        allocationChurnRatio,
        bytesPerElement,
      },
    } satisfies ImplementationAggregate;
  });

  return provisional.sort((a, b) => a.meanNs - b.meanNs);
}

export function buildParetoFront(points: ParetoPoint[]): ParetoPoint[] {
  const sorted = points
    .slice()
    .sort((a, b) => a.estimatedMemoryBytes - b.estimatedMemoryBytes || a.meanNs - b.meanNs);

  const frontier: ParetoPoint[] = [];
  let bestSpeed = Number.POSITIVE_INFINITY;

  for (const point of sorted) {
    if (point.meanNs < bestSpeed) {
      frontier.push(point);
      bestSpeed = point.meanNs;
    }
  }

  return frontier;
}

export function buildVariantRadarMetrics(
  aggregates: ImplementationAggregate[],
  focusFamily: string,
): VariantRadarResult {
  const familyAggregates = aggregates.filter(
    (item) => item.family === focusFamily && isRadarVariant(item.variant),
  );

  const byVariant = new Map<RadarVariantKey, ImplementationAggregate>();
  for (const item of familyAggregates) {
    if (isRadarVariant(item.variant)) {
      byVariant.set(item.variant, item);
    }
  }

  const availableVariants = (["safe", "raw", "arena"] as const).filter((variant) => byVariant.has(variant));

  const specs: RadarMetricSpec[] = [
    {
      key: "context_speed",
      axis: "Context Speed Score",
      description: "Mean latency in the exact current workload slice. Lower latency scores higher.",
      formatter: "latency_ns",
      lowerIsBetter: true,
      transform: "log1p",
      getValue: (item) => item.meanNs,
    },
    {
      key: "consistency",
      axis: "Consistency Score",
      description: "Relative runtime stability using coefficient of variation. Lower spread scores higher.",
      formatter: "ratio",
      lowerIsBetter: true,
      transform: "linear",
      getValue: (item) => (item.meanNs > 0 ? item.stdDevNs / item.meanNs : null),
    },
    {
      key: "memory_efficiency",
      axis: "Memory Efficiency Score",
      description: "Peak memory normalized by work done when available. Lower bytes per element score higher.",
      formatter: "bytes_per_element",
      lowerIsBetter: true,
      transform: "log1p",
      getValue: (item) => item.profiling.bytesPerElement ?? item.profiling.peakBytes ?? item.estimatedMemoryBytes,
    },
    {
      key: "cpu_efficiency",
      axis: "CPU Efficiency Score",
      description: "Instruction cost per processed element. Lower instruction overhead scores higher.",
      formatter: "instructions_per_element",
      lowerIsBetter: true,
      transform: "log1p",
      getValue: (item) => {
        const instructions = item.profiling.instructions;
        return instructions != null && item.throughputElements > 0
          ? instructions / item.throughputElements
          : null;
      },
    },
    {
      key: "cache_locality",
      axis: "Cache Locality Score",
      description: "L1 data miss rate in the current context. Lower miss rate scores higher.",
      formatter: "ratio",
      lowerIsBetter: true,
      transform: "linear",
      getValue: (item) => item.profiling.l1DataMissRate,
    },
    {
      key: "allocation_discipline",
      axis: "Allocation Discipline Score",
      description: "Allocation churn relative to peak memory. Lower churn scores higher.",
      formatter: "allocation_ratio",
      lowerIsBetter: true,
      transform: "log1p",
      getValue: (item) => item.profiling.allocationChurnRatio,
    },
  ];

  const metrics: VariantRadarMetric[] = [];
  const omittedAxes: string[] = [];

  for (const spec of specs) {
    const contextualValues = aggregates
      .map((item) => spec.getValue(item))
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value));

    const rawValues = {
      safe: byVariant.get("safe") ? spec.getValue(byVariant.get("safe") as ImplementationAggregate) : null,
      raw: byVariant.get("raw") ? spec.getValue(byVariant.get("raw") as ImplementationAggregate) : null,
      arena: byVariant.get("arena") ? spec.getValue(byVariant.get("arena") as ImplementationAggregate) : null,
    };

    const visibleValues = availableVariants.map((variant) => rawValues[variant]);
    const hasCompleteCoverage = visibleValues.every(
      (value) => typeof value === "number" && Number.isFinite(value),
    );

    if (!hasCompleteCoverage || contextualValues.length === 0) {
      omittedAxes.push(spec.axis);
      continue;
    }

    metrics.push({
      key: spec.key,
      axis: spec.axis,
      description: spec.description,
      formatter: spec.formatter,
      lowerIsBetter: spec.lowerIsBetter,
      contextCount: contextualValues.length,
      contextMedian: contextualValues.length > 0 ? quantile(contextualValues, 0.5) : null,
      safeRaw: rawValues.safe,
      rawRaw: rawValues.raw,
      arenaRaw: rawValues.arena,
      safe:
        rawValues.safe != null
          ? robustRadarScore(rawValues.safe, contextualValues, spec.lowerIsBetter, spec.transform)
          : null,
      raw:
        rawValues.raw != null
          ? robustRadarScore(rawValues.raw, contextualValues, spec.lowerIsBetter, spec.transform)
          : null,
      arena:
        rawValues.arena != null
          ? robustRadarScore(rawValues.arena, contextualValues, spec.lowerIsBetter, spec.transform)
          : null,
    });
  }

  return { metrics, omittedAxes };
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${Math.round(bytes)} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  return `${mb.toFixed(2)} MB`;
}

export function toPercentChange(current: number, baseline: number): number {
  if (baseline === 0) return 0;
  return ((current - baseline) / baseline) * 100;
}
