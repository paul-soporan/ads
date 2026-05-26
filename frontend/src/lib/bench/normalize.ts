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

function normalizeWorkloadName(workload: string, payload: string): string {
  const normalized = workload.toLowerCase();
  const normalizedPayload = payload.toLowerCase();

  if (normalized === "micro_sequences" && normalizedPayload === "u64") {
    return "micro_sequences_u64";
  }

  if (
    normalizedPayload === "u64" &&
    (normalized === "macro_read_heavy" ||
      normalized === "macro_write_heavy" ||
      normalized === "macro_thrashing" ||
      normalized === "micro_maps" ||
      normalized === "micro_heaps" ||
      normalized === "micro_dsu" ||
      normalized === "micro_sequences" ||
      normalized === "micro_sequences_indexing" ||
      normalized === "sweep_btree_cache" ||
      normalized === "sweep_hash_collisions" ||
      normalized === "motivational_heap_merge" ||
      normalized === "motivational_dsu_connectivity")
  ) {
    return `${normalized}_u64`;
  }

  return normalized;
}

function normalizeOperationName(operation: string): string {
  const normalized = operation.toLowerCase();

  if (normalized === "mix_read" || normalized === "mix_write") {
    return "mix";
  }

  return normalized;
}

function normalizeImplementationName(implementation: string): string {
  const normalized = implementation.toLowerCase().replace(/_+/g, "_").replace(/^_+|_+$/g, "");
  if (normalized === "std_binary_heap_min") {
    return "std_binary_heap";
  }
  if (normalized.startsWith("ads_")) {
    return normalized.slice(4);
  }
  return normalized;
}

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
  const payload = (operationGroup.join.payload as string).toLowerCase();
  const workloadName = normalizeWorkloadName(operationGroup.join.workload, payload);
  const operation = normalizeOperationName(operationGroup.join.operation);
  const implementation = normalizeImplementationName(operationGroup.join.implementation);
  const workload = inferWorkloadFamily(workloadName);
  const distribution = operationGroup.join.distribution as Distribution;
  const normalizedPayload = payload as PayloadKind;
  const variant = operationGroup.join.variant as VariantKind;

  return {
    id: `${operationGroup.join.join_key}:${record.sample}`,
    joinKey: operationGroup.join.join_key,
    path: record.path,
    group: workloadName,
    workloadName,
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
    distribution,
    payload: normalizedPayload,
    variant,
    category: inferCategory(workloadName),
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
    const payload = (operationGroup.join.payload as string).toLowerCase();
    const workloadName = normalizeWorkloadName(operationGroup.join.workload, payload);
    const operationName = normalizeOperationName(operationGroup.join.operation);
    const implementationName = normalizeImplementationName(operationGroup.join.implementation);

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
        joinKey: operationGroup.join.join_key,
        path: record.path,
        implementation: implementationName,
        operation: operationName,
        distribution: operationGroup.join.distribution as Distribution,
        workload: workloadName,
        payload,
        size: operationGroup.join.size,
        metrics: record.metrics,
      });
    }

    for (const record of operationGroup.dhat) {
      const bytes = toDhatBytes(record);
      dhat.push({
        joinKey: operationGroup.join.join_key,
        path: record.path,
        implementation: implementationName,
        operation: operationName,
        distribution: operationGroup.join.distribution as Distribution,
        workload: workloadName,
        payload,
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
