import type { DashboardFilters, DashboardOptionLists } from "./types";

const PARAMS = {
  category: "cat",
  operation: "op",
  distribution: "dist",
  payload: "payload",
  size: "size",
  search: "q",
  compare: "cmp",
  matrixScale: "mx",
  matrixOutliers: "out",
  advancedFilters: "adv",
  profiling: "pf",
  leaderboardSort: "ls",
  leaderboardSortDir: "ld",
} as const;

export type MatrixScaleMode = "log" | "linear";
export type LeaderboardSortKey =
  | "implementation"
  | "operation"
  | "size"
  | "meanNs"
  | "instructions"
  | "l1MissRate"
  | "peakBytes"
  | "throughput";
export type LeaderboardSortDirection = "asc" | "desc";

export interface DashboardViewState {
  matrixScale: MatrixScaleMode;
  hideOutliers: boolean;
  showAdvancedFilters: boolean;
  showProfiling: boolean;
  leaderboardSortKey: LeaderboardSortKey;
  leaderboardSortDirection: LeaderboardSortDirection;
}

export interface ParsedUrlState {
  filters: DashboardFilters;
  selectedImplementations: string[];
  view: DashboardViewState;
}

const DEFAULT_FILTERS: DashboardFilters = {
  category: "maps",
  operation: "insert",
  distribution: "uniform",
  payload: "u64",
  size: 10_000,
  search: "",
};

const DEFAULT_VIEW: DashboardViewState = {
  matrixScale: "log",
  hideOutliers: false,
  showAdvancedFilters: false,
  showProfiling: false,
  leaderboardSortKey: "meanNs",
  leaderboardSortDirection: "asc",
};

function parseLeaderboardSortKey(value: string | null): LeaderboardSortKey {
  if (
    value === "implementation" ||
    value === "operation" ||
    value === "size" ||
    value === "instructions" ||
    value === "l1MissRate" ||
    value === "peakBytes" ||
    value === "throughput"
  ) {
    return value;
  }

  return "meanNs";
}

function parseLeaderboardSortDirection(value: string | null): LeaderboardSortDirection {
  return value === "desc" ? "desc" : "asc";
}

function normalizeSearch(value: string | null): string {
  if (!value) return "";
  return value.trim().slice(0, 120);
}

function parseSize(size: string | null, options: DashboardOptionLists): DashboardFilters["size"] {
  if (!size || size === "all") return "all";

  const parsed = Number(size);
  if (!Number.isFinite(parsed)) return "all";
  if (!options.sizes.includes(parsed)) return "all";
  return parsed;
}

export function parseUrlState(params: URLSearchParams, options: DashboardOptionLists): ParsedUrlState {
  const category = params.get(PARAMS.category);
  const operation = params.get(PARAMS.operation);
  const distribution = params.get(PARAMS.distribution);
  const payload = params.get(PARAMS.payload);
  const parsedSize = parseSize(params.get(PARAMS.size), options);
  const search = normalizeSearch(params.get(PARAMS.search));

  const parsed: ParsedUrlState = {
    filters: {
      category:
        category && options.categories.includes(category as typeof options.categories[number])
          ? (category as DashboardFilters["category"])
          : DEFAULT_FILTERS.category,
      operation: operation && options.operations.includes(operation) ? operation : DEFAULT_FILTERS.operation,
      distribution:
        distribution && options.distributions.includes(distribution as typeof options.distributions[number])
          ? (distribution as DashboardFilters["distribution"])
          : DEFAULT_FILTERS.distribution,
      payload:
        payload && options.payloads.includes(payload as typeof options.payloads[number])
          ? (payload as DashboardFilters["payload"])
          : DEFAULT_FILTERS.payload,
      size: parsedSize === "all" ? DEFAULT_FILTERS.size : parsedSize,
      search,
    },
    selectedImplementations: [],
    view: {
      matrixScale: params.get(PARAMS.matrixScale) === "linear" ? "linear" : DEFAULT_VIEW.matrixScale,
      hideOutliers: params.get(PARAMS.matrixOutliers) === "1",
      showAdvancedFilters: params.get(PARAMS.advancedFilters) === "1",
      showProfiling: params.get(PARAMS.profiling) === "1",
      leaderboardSortKey: parseLeaderboardSortKey(params.get(PARAMS.leaderboardSort)),
      leaderboardSortDirection: parseLeaderboardSortDirection(params.get(PARAMS.leaderboardSortDir)),
    },
  };

  const cmp = params.get(PARAMS.compare);
  if (cmp) {
    const selected = cmp
      .split(",")
      .map((item) => item.trim())
      .filter((item) => item.length > 0)
      .filter((item) => options.implementations.includes(item));

    parsed.selectedImplementations = [...new Set(selected)].slice(0, 4);
  }

  return parsed;
}

export function serializeUrlState(
  filters: DashboardFilters,
  selectedImplementations: string[],
  view: DashboardViewState,
): string {
  const params = new URLSearchParams();

  if (filters.category !== DEFAULT_FILTERS.category) params.set(PARAMS.category, filters.category);
  if (filters.operation !== DEFAULT_FILTERS.operation) params.set(PARAMS.operation, filters.operation);
  if (filters.distribution !== DEFAULT_FILTERS.distribution) params.set(PARAMS.distribution, filters.distribution);
  if (filters.payload !== DEFAULT_FILTERS.payload) params.set(PARAMS.payload, filters.payload);
  if (filters.size !== DEFAULT_FILTERS.size) params.set(PARAMS.size, String(filters.size));
  if (filters.search.trim()) params.set(PARAMS.search, filters.search.trim());

  if (selectedImplementations.length > 0) {
    params.set(PARAMS.compare, selectedImplementations.slice(0, 4).join(","));
  }

  if (view.matrixScale !== DEFAULT_VIEW.matrixScale) params.set(PARAMS.matrixScale, view.matrixScale);
  if (view.hideOutliers !== DEFAULT_VIEW.hideOutliers) params.set(PARAMS.matrixOutliers, view.hideOutliers ? "1" : "0");
  if (view.showAdvancedFilters) params.set(PARAMS.advancedFilters, "1");
  if (view.showProfiling !== DEFAULT_VIEW.showProfiling) params.set(PARAMS.profiling, view.showProfiling ? "1" : "0");
  if (view.leaderboardSortKey !== DEFAULT_VIEW.leaderboardSortKey) params.set(PARAMS.leaderboardSort, view.leaderboardSortKey);
  if (view.leaderboardSortDirection !== DEFAULT_VIEW.leaderboardSortDirection) {
    params.set(PARAMS.leaderboardSortDir, view.leaderboardSortDirection);
  }

  return params.toString();
}
