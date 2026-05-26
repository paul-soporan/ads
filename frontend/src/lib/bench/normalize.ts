import type {
  CallgrindRecord,
  CriterionRecord,
  DashboardCategory,
  DashboardIndexes,
  DashboardOptionLists,
  DhatRecord,
  Distribution,
  NormalizedBenchmarkDataset,
  PayloadKind,
  RawBenchmarkArtifact,
  RawCriterionRecord,
  RawOperationGroup,
  VariantKind,
  WorkloadFamily,
} from "./types";

function inferWorkloadFamily(group: string): WorkloadFamily {
  const normalizedGroup = group.toLowerCase();

  if (
    normalizedGroup.startsWith("micro_") ||
    normalizedGroup.startsWith("data_types_") ||
    normalizedGroup.startsWith("traversal_and_bulk")
  ) {
    return "micro";
  }

  if (normalizedGroup.startsWith("macro_") || normalizedGroup.startsWith("ordered_maps_")) {
    return "macro";
  }

  if (
    normalizedGroup.startsWith("sweep_") ||
    normalizedGroup.startsWith("btree_degree") ||
    normalizedGroup.startsWith("collision_")
  ) {
    return "sweeps";
  }

  return "other";
}

function inferOperation(record: RawCriterionRecord): string {
  const blob = `${record.function} ${record.path}`.toLowerCase();

  if (blob.includes("contains_zipf")) return "contains_zipf";
  if (blob.includes("contains_temporal")) return "contains_temporal";
  if (blob.includes("contains_mixed") || blob.includes("read_heavy")) return "contains_mixed";
  if (blob.includes("thrash")) return "thrash";
  if (blob.includes("bulk")) return "bulk_insert";
  if (blob.includes("insert")) return "insert";
  if (blob.includes("remove")) return "remove";
  if (blob.includes("contains")) return "contains";
  if (blob.includes("write_heavy") || blob.includes("mix")) return "mix";
  if (blob.includes("push_pop")) return "push_pop";

  return "unknown";
}

function inferDistribution(record: RawCriterionRecord): Distribution {
  const operation = inferOperation(record);
  const workload = inferWorkloadFamily(record.group);
  const blob = `${record.group} ${record.function} ${record.path} ${operation}`.toLowerCase();

  if (blob.includes("zipf")) return "zipfian";
  if (blob.includes("temporal")) return "temporal";
  if (blob.includes("sorted")) return "sorted";
  if (blob.includes("uniform")) return "uniform";
  if (blob.includes("mixed") || blob.includes("thrash") || blob.includes("read_heavy") || blob.includes("write_heavy")) {
    return "mixed";
  }

  if (operation === "contains_zipf") return "zipfian";
  if (operation === "contains_temporal") return "temporal";
  if (operation === "contains_mixed" || operation === "mix" || operation === "thrash") return "mixed";

  if (workload === "micro" || workload === "sweeps") {
    return "uniform";
  }

  return "other";
}

function inferPayload(record: RawCriterionRecord): PayloadKind {
  const blob = `${record.group} ${record.function} ${record.path}`.toLowerCase();

  if (blob.includes("string")) return "string";
  if (blob.includes("large_payload") || blob.includes("large")) return "large_payload";
  if (blob.includes("u64") || blob.includes("n1k") || blob.includes("n10k") || blob.includes("micro_sequences")) {
    return "u64";
  }

  return "other";
}

function inferVariant(implementation: string): VariantKind {
  if (implementation.includes("_safe") || implementation.startsWith("safe_")) return "safe";
  if (implementation.includes("_raw") || implementation.startsWith("raw_")) return "raw";
  if (implementation.includes("_arena") || implementation.startsWith("arena_")) return "arena";
  if (implementation.startsWith("std_") || implementation.includes("std_")) return "std";

  return "other";
}

function inferCategory(workloadName: string): DashboardCategory {
  const value = workloadName.toLowerCase();

  if (value.includes("map") || value.includes("btree") || value.includes("collision") || value.includes("read_heavy") || value.includes("write_heavy") || value.includes("thrashing")) {
    return "maps";
  }

  if (value.includes("sequence") || value.includes("list") || value.includes("deque") || value.includes("ring")) {
    return "sequences";
  }

  if (value.includes("heap")) {
    return "heaps";
  }

  return "other";
}

function pickBestCriterionSample(records: RawCriterionRecord[]): RawCriterionRecord[] {
  return [...records].sort((a, b) => {
    if (a.sample === "new" && b.sample !== "new") return -1;
    if (b.sample === "new" && a.sample !== "new") return 1;
    return 0;
  }).slice(0, 1);
}

function toCriterionRecord(operationGroup: RawOperationGroup, record: RawCriterionRecord): CriterionRecord {
  const workload = inferWorkloadFamily(operationGroup.join.workload);
  const operation = operationGroup.join.operation;
  const payload = operationGroup.join.payload as PayloadKind;
  const variant = operationGroup.join.variant as VariantKind;
  const implementation = operationGroup.join.implementation;

  return {
    id: `${operationGroup.join.join_key}:${record.sample}`,
    path: record.path,
    group: operationGroup.join.workload,
    workloadName: operationGroup.join.workload,
    functionName: record.function,
    operation,
    implementation,
    size: operationGroup.join.size,
    sample: record.sample,
    meanNs: record.mean.point_estimate,
    standardErrorNs: record.mean.standard_error,
    ciLowerNs: record.mean.confidence_interval.lower_bound,
    ciUpperNs: record.mean.confidence_interval.upper_bound,
    stdDevNs: record.std_dev.point_estimate,
    throughputElements: record.throughput_elements,
    workload,
    distribution: inferDistribution(record),
    payload,
    variant,
    category: inferCategory(operationGroup.join.workload),
  };
}

function toDhatBytes(raw: {
  total_bytes: number | null;
  max_bytes: number | null;
  raw?: { pps?: Array<{ mb?: number; tb?: number }> };
}) {
  const sampledMax = raw.raw?.pps?.reduce((max, point) => {
    const value = typeof point.mb === "number" ? point.mb : 0;
    return Math.max(max, value);
  }, 0);

  const sampledTotal = raw.raw?.pps?.reduce((sum, point) => {
    const value = typeof point.tb === "number" ? point.tb : 0;
    return sum + value;
  }, 0);

  return {
    totalBytes: raw.total_bytes ?? sampledTotal ?? null,
    maxBytes: raw.max_bytes ?? sampledMax ?? null,
  };
}

function buildIndexes(records: CriterionRecord[]): DashboardIndexes {
  const byCategory = new Map<DashboardCategory, number[]>();
  const byOperation = new Map<string, number[]>();
  const byDistribution = new Map<Distribution, number[]>();
  const byPayload = new Map<PayloadKind, number[]>();
  const bySize = new Map<number, number[]>();
  const byImplementation = new Map<string, number[]>();

  records.forEach((record, index) => {
    const add = <T,>(map: Map<T, number[]>, key: T) => {
      const list = map.get(key);
      if (list) {
        list.push(index);
      } else {
        map.set(key, [index]);
      }
    };

    add(byCategory, record.category);
    add(byOperation, record.operation);
    add(byDistribution, record.distribution);
    add(byPayload, record.payload);
    add(bySize, record.size);
    add(byImplementation, record.implementation);
  });

  return {
    byCategory,
    byOperation,
    byDistribution,
    byPayload,
    bySize,
    byImplementation,
  };
}

function sortedStrings(values: Iterable<string>): string[] {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b));
}

function sortedNumbers(values: Iterable<number>): number[] {
  return [...new Set(values)].sort((a, b) => a - b);
}

function buildOptions(records: CriterionRecord[]): DashboardOptionLists {
  return {
    categories: sortedStrings(records.map((record) => record.category)) as DashboardCategory[],
    operations: sortedStrings(records.map((record) => record.operation)),
    distributions: sortedStrings(records.map((record) => record.distribution)) as Distribution[],
    payloads: sortedStrings(records.map((record) => record.payload)) as PayloadKind[],
    sizes: sortedNumbers(records.map((record) => record.size)),
    implementations: sortedStrings(records.map((record) => record.implementation)),
  };
}

export function normalizeBenchmarkArtifact(raw: RawBenchmarkArtifact): NormalizedBenchmarkDataset {
  const criterionByJoinKey = new Map<string, CriterionRecord>();
  const callgrind: CallgrindRecord[] = [];
  const dhat: DhatRecord[] = [];

  for (const operationGroup of raw.operations) {
    const bestCriterion = pickBestCriterionSample(operationGroup.criterion);
    for (const record of bestCriterion) {
      const normalized = toCriterionRecord(operationGroup, record);
      const existing = criterionByJoinKey.get(operationGroup.join.join_key);
      if (!existing || normalized.standardErrorNs < existing.standardErrorNs) {
        criterionByJoinKey.set(operationGroup.join.join_key, normalized);
      }
    }

    for (const record of operationGroup.callgrind) {
      callgrind.push({
        path: record.path,
        implementation: operationGroup.join.implementation,
        operation: operationGroup.join.operation,
        workload: operationGroup.join.workload,
        payload: operationGroup.join.payload,
        size: operationGroup.join.size,
        metrics: record.metrics,
      });
    }

    for (const record of operationGroup.dhat) {
      const bytes = toDhatBytes(record);
      dhat.push({
        path: record.path,
        implementation: operationGroup.join.implementation,
        operation: operationGroup.join.operation,
        workload: operationGroup.join.workload,
        payload: operationGroup.join.payload,
        size: operationGroup.join.size,
        totalBytes: bytes.totalBytes,
        maxBytes: bytes.maxBytes,
        totalBlocks: record.total_blocks,
        maxBlocks: record.max_blocks,
      });
    }
  }

  const criterion = [...criterionByJoinKey.values()];

  const indexes = buildIndexes(criterion);
  const options = buildOptions(criterion);

  return {
    version: `${raw.generated_at_unix_secs}-${raw.operation_count}-${criterion.length}`,
    generatedAtUnixSecs: raw.generated_at_unix_secs,
    records: criterion,
    callgrind,
    dhat,
    options,
    indexes,
  };
}
