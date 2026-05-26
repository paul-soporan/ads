export type BenchmarkSample = "base" | "new" | string;

export interface ConfidenceInterval {
  confidence_level: number;
  lower_bound: number;
  upper_bound: number;
}

export interface EstimateStat {
  point_estimate: number;
  standard_error: number;
  confidence_interval: ConfidenceInterval;
}

export interface RawCriterionRecord {
  path: string;
  group: string;
  function: string;
  sample: BenchmarkSample;
  mean: EstimateStat;
  median: EstimateStat;
  slope: EstimateStat | null;
  std_dev: EstimateStat;
  throughput_elements: number | null;
}

export interface RawCallgrindRecord {
  path: string;
  events: string[];
  metrics: Record<string, number>;
}

export interface RawDhatRecord {
  path: string;
  total_bytes: number | null;
  total_blocks: number | null;
  max_bytes: number | null;
  max_blocks: number | null;
  extra_numeric_fields?: Record<string, number>;
  raw?: {
    pps?: Array<{
      mb?: number;
      tb?: number;
      mbk?: number;
      tbk?: number;
    }>;
  };
}

export interface RawBenchmarkArtifact {
  generated_at_unix_secs: number;
  operation_count: number;
  operations: RawOperationGroup[];
}

export interface RawJoinKeys {
  workload: string;
  payload: string;
  operation: string;
  implementation: string;
  size: number;
  variant: VariantKind;
  join_key: string;
}

export interface RawOperationGroup {
  join: RawJoinKeys;
  criterion: RawCriterionRecord[];
  callgrind: RawCallgrindRecord[];
  dhat: RawDhatRecord[];
}

export type WorkloadFamily = "micro" | "macro" | "sweeps" | "other";
export type DashboardCategory = "maps" | "sequences" | "heaps" | "other";
export type Distribution = "uniform" | "zipfian" | "temporal" | "sorted" | "mixed" | "other";
export type PayloadKind = "u64" | "string" | "large_payload" | "other";
export type VariantKind = "safe" | "raw" | "arena" | "std" | "other";

export interface CriterionRecord {
  id: string;
  path: string;
  group: string;
  workloadName: string;
  functionName: string;
  operation: string;
  implementation: string;
  size: number;
  sample: BenchmarkSample;
  meanNs: number;
  standardErrorNs: number;
  ciLowerNs: number;
  ciUpperNs: number;
  stdDevNs: number;
  throughputElements: number | null;
  workload: WorkloadFamily;
  distribution: Distribution;
  payload: PayloadKind;
  variant: VariantKind;
  category: DashboardCategory;
}

export interface CallgrindRecord {
  path: string;
  implementation: string;
  operation: string;
  workload: string;
  payload: string;
  size: number;
  metrics: Record<string, number>;
}

export interface DhatRecord {
  path: string;
  implementation: string;
  operation: string;
  workload: string;
  payload: string;
  size: number;
  totalBytes: number | null;
  maxBytes: number | null;
  totalBlocks: number | null;
  maxBlocks: number | null;
}

export interface DashboardOptionLists {
  categories: DashboardCategory[];
  operations: string[];
  distributions: Distribution[];
  payloads: PayloadKind[];
  sizes: number[];
  implementations: string[];
}

export interface DashboardIndexes {
  byCategory: Map<DashboardCategory, number[]>;
  byOperation: Map<string, number[]>;
  byDistribution: Map<Distribution, number[]>;
  byPayload: Map<PayloadKind, number[]>;
  bySize: Map<number, number[]>;
  byImplementation: Map<string, number[]>;
}

export interface NormalizedBenchmarkDataset {
  version: string;
  generatedAtUnixSecs: number;
  records: CriterionRecord[];
  callgrind: CallgrindRecord[];
  dhat: DhatRecord[];
  options: DashboardOptionLists;
  indexes: DashboardIndexes;
}

export interface DashboardFilters {
  category: "all" | DashboardCategory;
  operation: "all" | string;
  distribution: "all" | Distribution;
  payload: "all" | PayloadKind;
  size: "all" | number;
  search: string;
}

export interface DashboardSummary {
  totalRecords: number;
  visibleRecords: number;
  implementations: number;
  sizes: number;
  operations: number;
  bestMeanNs: number | null;
  worstMeanNs: number | null;
}
