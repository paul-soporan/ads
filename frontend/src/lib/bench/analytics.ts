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
    D1mr?: number;
    D1mw?: number;
  };
}

export interface ParetoPoint {
  implementation: string;
  meanNs: number;
  estimatedMemoryBytes: number;
}

function toFamilyName(implementation: string): string {
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
  const byImplementation = new Map<string, Record<string, number>>();
  const implementationBuckets = new Map<string, Record<string, number>[]>();

  for (const record of dataset.callgrind) {
    const implementation = normalizeImplementationKey(record.implementation);
    if (!implementation) continue;

    const implList = implementationBuckets.get(implementation) ?? [];
    implList.push(record.metrics);
    implementationBuckets.set(implementation, implList);
  }

  for (const [implementation, metrics] of implementationBuckets.entries()) {
    byImplementation.set(implementation, averageMetrics(metrics));
  }

  return { byImplementation };
}

function average(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function buildDhatLookup(dataset: NormalizedBenchmarkDataset): {
  byImplementation: Map<string, number>;
  byImplementationAndShape: Map<string, number>;
} {
  const implementationBuckets = new Map<string, number[]>();
  const implementationAndShapeBuckets = new Map<string, number[]>();

  for (const row of dataset.dhat) {
    const maxBytes = row.maxBytes ?? row.totalBytes ?? 0;
    if (maxBytes <= 0) continue;

    const impl = normalizeImplementationKey(row.implementation);
    const byImpl = implementationBuckets.get(impl) ?? [];
    byImpl.push(maxBytes);
    implementationBuckets.set(impl, byImpl);

    const shapeKey = `${impl}|${row.operation}|${row.payload}|${row.size}`;
    const byShape = implementationAndShapeBuckets.get(shapeKey) ?? [];
    byShape.push(maxBytes);
    implementationAndShapeBuckets.set(shapeKey, byShape);
  }

  const byImplementation = new Map<string, number>();
  for (const [key, values] of implementationBuckets.entries()) {
    byImplementation.set(key, average(values));
  }

  const byImplementationAndShape = new Map<string, number>();
  for (const [key, values] of implementationAndShapeBuckets.entries()) {
    byImplementationAndShape.set(key, average(values));
  }

  return { byImplementation, byImplementationAndShape };
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

    const normalizedImplementation = normalizeImplementationKey(implementation);
    const callgrind = callgrindMetadata.byImplementation.get(normalizedImplementation);
    if (!callgrind) {
      throw new Error(`Missing callgrind metrics for implementation: ${implementation}`);
    }

    const dhatShapeValues = list.map((item) => {
      const shapeKey = `${normalizedImplementation}|${item.operation}|${item.payload}|${item.size}`;
      const value = dhatLookup.byImplementationAndShape.get(shapeKey);
      if (value === undefined) {
        throw new Error(`Missing dhat metrics for shape: ${shapeKey}`);
      }

      return value;
    });

    const estimatedMemoryBytes = average(dhatShapeValues);

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
