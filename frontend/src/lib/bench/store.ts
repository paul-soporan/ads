import { create } from "zustand";

import { assignComparisonColors, type ComparisonColorMap } from "./colorManager";
import { loadBenchmarkDataset } from "./load";
import type {
  CriterionRecord,
  DashboardFilters,
  DashboardSummary,
  DashboardCategory,
  Distribution,
  NormalizedBenchmarkDataset,
  PayloadKind,
} from "./types";

type DashboardStatus = "idle" | "loading" | "ready" | "error";

interface DashboardStore {
  status: DashboardStatus;
  error: string | null;
  dataset: NormalizedBenchmarkDataset | null;
  filters: DashboardFilters;
  selectedImplementations: string[];
  hoveredImplementation: string | null;
  comparisonColors: ComparisonColorMap;
  loadData: () => Promise<void>;
  setFilter: <K extends keyof DashboardFilters>(key: K, value: DashboardFilters[K]) => void;
  setFilters: (next: DashboardFilters) => void;
  clearFilters: () => void;
  toggleImplementation: (implementation: string) => void;
  setSelectedImplementations: (implementations: string[]) => void;
  setHoveredImplementation: (implementation: string | null) => void;
  resetComparison: () => void;
}

interface ContextualFilterOptions {
  categories: DashboardCategory[];
  operations: string[];
  distributions: Distribution[];
  payloads: PayloadKind[];
  sizes: number[];
  implementations: string[];
}

const defaultFilters: DashboardFilters = {
  category: "maps",
  operation: "insert",
  distribution: "uniform",
  payload: "u64",
  size: 10_000,
  search: "",
};

const EMPTY_RECORDS: CriterionRecord[] = [];
const EMPTY_OPTIONS = {
  categories: [],
  operations: [],
  distributions: [],
  payloads: [],
  sizes: [],
  implementations: [],
};

const EMPTY_CONTEXTUAL_OPTIONS: ContextualFilterOptions = {
  categories: [],
  operations: [],
  distributions: [],
  payloads: [],
  sizes: [],
  implementations: [],
};

const EMPTY_SUMMARY: DashboardSummary = {
  totalRecords: 0,
  visibleRecords: 0,
  implementations: 0,
  sizes: 0,
  operations: 0,
  bestMeanNs: null,
  worstMeanNs: null,
};

function getFirstOption<T>(values: T[], label: string): T {
  const first = values[0];
  if (first === undefined) {
    throw new Error(`Benchmark dataset is missing ${label} options`);
  }

  return first;
}

function searchMatches(record: CriterionRecord, search: string) {
  if (!search) return true;

  const haystack = `${record.implementation} ${record.functionName} ${record.group} ${record.operation}`.toLowerCase();
  return haystack.includes(search);
}

function deriveContextualOptions(
  dataset: NormalizedBenchmarkDataset,
  filters: DashboardFilters,
): ContextualFilterOptions {
  const search = filters.search.trim().toLowerCase();
  const sortStrings = (values: string[]) => [...new Set(values)].sort((a, b) => a.localeCompare(b));
  const sortNumbers = (values: number[]) => [...new Set(values)].sort((a, b) => a - b);

  // Left-to-right cascade prevents control deadlocks (for example, maps-only category lock)
  // while still ensuring downstream controls narrow to valid contextual choices.
  const bySearch = dataset.records.filter((record) => searchMatches(record, search));
  const byCategory =
    filters.category !== "all"
      ? bySearch.filter((record) => record.category === filters.category)
      : bySearch;
  const byOperation =
    filters.operation !== "all"
      ? byCategory.filter((record) => record.operation === filters.operation)
      : byCategory;
  const byDistribution =
    filters.distribution !== "all"
      ? byOperation.filter((record) => record.distribution === filters.distribution)
      : byOperation;
  const byPayload =
    filters.payload !== "all"
      ? byDistribution.filter((record) => record.payload === filters.payload)
      : byDistribution;

  const categories = sortStrings(bySearch.map((record) => record.category));
  const operations = sortStrings(byCategory.map((record) => record.operation));
  const distributions = sortStrings(byOperation.map((record) => record.distribution)) as Distribution[];
  const payloads = sortStrings(byDistribution.map((record) => record.payload)) as PayloadKind[];
  const sizes = sortNumbers(byPayload.map((record) => record.size));

  return {
    categories: categories as DashboardCategory[],
    operations,
    distributions,
    payloads,
    sizes,
    implementations: dataset.options.implementations,
  };
}

function normalizeContextLockedFilters(dataset: NormalizedBenchmarkDataset, candidate: DashboardFilters): DashboardFilters {
  const search = candidate.search;
  const query = search.trim().toLowerCase();
  const bySearch = dataset.records.filter((record) => searchMatches(record, query));

  if (bySearch.length === 0) {
    return {
      ...defaultFilters,
      search,
    };
  }

  const sortStrings = (values: string[]) => [...new Set(values)].sort((a, b) => a.localeCompare(b));
  const sortNumbers = (values: number[]) => [...new Set(values)].sort((a, b) => a - b);

  const categories = sortStrings(bySearch.map((record) => record.category)) as DashboardCategory[];
  const category =
    candidate.category !== "all" && categories.includes(candidate.category)
      ? candidate.category
      : getFirstOption(categories, "category");

  const byCategory = bySearch.filter((record) => record.category === category);
  const operations = sortStrings(byCategory.map((record) => record.operation));
  const operation =
    candidate.operation !== "all" && operations.includes(candidate.operation)
      ? candidate.operation
      : getFirstOption(operations, "operation");

  const byOperation = byCategory.filter((record) => record.operation === operation);
  const distributions = sortStrings(byOperation.map((record) => record.distribution)) as Distribution[];
  const distribution =
    candidate.distribution !== "all" && distributions.includes(candidate.distribution)
      ? candidate.distribution
      : getFirstOption(distributions, "distribution");

  const byDistribution = byOperation.filter((record) => record.distribution === distribution);
  const payloads = sortStrings(byDistribution.map((record) => record.payload)) as PayloadKind[];
  const payload =
    candidate.payload !== "all" && payloads.includes(candidate.payload)
      ? candidate.payload
      : getFirstOption(payloads, "payload");

  const byPayload = byDistribution.filter((record) => record.payload === payload);
  const sizes = sortNumbers(byPayload.map((record) => record.size));
  const size =
    candidate.size !== "all" && sizes.includes(candidate.size)
      ? candidate.size
      : getFirstOption(sizes, "size");

  return {
    category,
    operation,
    distribution,
    payload,
    size,
    search,
  };
}

export const useDashboardStore = create<DashboardStore>((set, get) => ({
  status: "idle",
  error: null,
  dataset: null,
  filters: defaultFilters,
  selectedImplementations: [],
  hoveredImplementation: null,
  comparisonColors: {},

  async loadData() {
    const { status } = get();
    if (status === "loading" || status === "ready") return;

    set({ status: "loading", error: null });

    try {
      const dataset = await loadBenchmarkDataset();

      set((state) => ({
        dataset,
        status: "ready",
        error: null,
        filters: {
          ...normalizeContextLockedFilters(dataset, state.filters),
        },
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown error while loading benchmark data";
      set({ status: "error", error: message });
    }
  },

  setFilter(key, value) {
    set((state) => {
      if (state.filters[key] === value) return state;

      const next = {
        ...state.filters,
        [key]: value,
      };

      const resolved = state.dataset ? normalizeContextLockedFilters(state.dataset, next) : next;

      if (
        state.filters.category === resolved.category &&
        state.filters.operation === resolved.operation &&
        state.filters.distribution === resolved.distribution &&
        state.filters.payload === resolved.payload &&
        state.filters.size === resolved.size &&
        state.filters.search === resolved.search
      ) {
        return state;
      }

      return {
        filters: resolved,
      };
    });
  },

  setFilters(next) {
    set((state) => {
      const resolved = state.dataset ? normalizeContextLockedFilters(state.dataset, next) : next;

      if (
        state.filters.category === resolved.category &&
        state.filters.operation === resolved.operation &&
        state.filters.distribution === resolved.distribution &&
        state.filters.payload === resolved.payload &&
        state.filters.size === resolved.size &&
        state.filters.search === resolved.search
      ) {
        return state;
      }

      return { filters: resolved };
    });
  },

  clearFilters() {
    set((state) => ({
      filters: state.dataset ? normalizeContextLockedFilters(state.dataset, defaultFilters) : defaultFilters,
    }));
  },

  toggleImplementation(implementation) {
    set((state) => {
      const selected = state.selectedImplementations;
      if (selected.includes(implementation)) {
        const nextSelected = selected.filter((item) => item !== implementation);
        return {
          selectedImplementations: nextSelected,
          comparisonColors: assignComparisonColors(state.comparisonColors, nextSelected),
        };
      }

      if (selected.length >= 4) {
        return state;
      }

      const nextSelected = [...selected, implementation];
      return {
        selectedImplementations: nextSelected,
        comparisonColors: assignComparisonColors(state.comparisonColors, nextSelected),
      };
    });
  },

  setSelectedImplementations(implementations) {
    const normalized = [...new Set(implementations)].slice(0, 4);

    set((state) => {
      if (
        state.selectedImplementations.length === normalized.length &&
        state.selectedImplementations.every((value, index) => value === normalized[index])
      ) {
        return state;
      }

      return {
        selectedImplementations: normalized,
        comparisonColors: assignComparisonColors(state.comparisonColors, normalized),
      };
    });
  },

  setHoveredImplementation(implementation) {
    set((state) => {
      if (state.hoveredImplementation === implementation) {
        return state;
      }

      return { hoveredImplementation: implementation };
    });
  },

  resetComparison() {
    set({ selectedImplementations: [], comparisonColors: {}, hoveredImplementation: null });
  },
}));

interface CacheEntry<T> {
  key: string;
  value: T;
}

function createSingleEntryMemo<T>() {
  let cache: CacheEntry<T> | null = null;

  return (key: string, factory: () => T): T => {
    if (cache && cache.key === key) {
      return cache.value;
    }

    const value = factory();
    cache = { key, value };
    return value;
  };
}

const memoFiltered = createSingleEntryMemo<CriterionRecord[]>();
const memoComparison = createSingleEntryMemo<CriterionRecord[]>();
const memoSummary = createSingleEntryMemo<DashboardSummary>();
const memoContextualOptions = createSingleEntryMemo<ContextualFilterOptions>();

function getCandidateIndexes(dataset: NormalizedBenchmarkDataset, filters: DashboardFilters): number[] | null {
  const buckets: number[][] = [];

  if (filters.category !== "all") {
    buckets.push(dataset.indexes.byCategory.get(filters.category) ?? []);
  }

  if (filters.operation !== "all") {
    buckets.push(dataset.indexes.byOperation.get(filters.operation) ?? []);
  }

  if (filters.distribution !== "all") {
    buckets.push(dataset.indexes.byDistribution.get(filters.distribution) ?? []);
  }

  if (filters.payload !== "all") {
    buckets.push(dataset.indexes.byPayload.get(filters.payload) ?? []);
  }

  if (filters.size !== "all") {
    buckets.push(dataset.indexes.bySize.get(filters.size) ?? []);
  }

  if (buckets.length === 0) return null;

  buckets.sort((a, b) => a.length - b.length);
  return buckets[0] ?? null;
}

export const selectFilterOptions = (state: DashboardStore) =>
  state.dataset?.options ?? EMPTY_OPTIONS;

export const selectContextualFilterOptions = (state: DashboardStore): ContextualFilterOptions => {
  const dataset = state.dataset;
  if (!dataset) {
    return EMPTY_CONTEXTUAL_OPTIONS;
  }

  const key = `ctx|${dataset.version}|${state.filters.category}|${state.filters.operation}|${state.filters.distribution}|${state.filters.payload}|${state.filters.size}|${state.filters.search.trim().toLowerCase()}`;

  return memoContextualOptions(key, () => deriveContextualOptions(dataset, state.filters));
};

export const selectFilteredRecords = (state: DashboardStore): CriterionRecord[] => {
  const { dataset, filters } = state;
  if (!dataset) return EMPTY_RECORDS;

  const search = filters.search.trim().toLowerCase();
  const key = `${dataset.version}|${filters.category}|${filters.operation}|${filters.distribution}|${filters.payload}|${filters.size}|${search}`;

  return memoFiltered(key, () => {
    const seedIndexes = getCandidateIndexes(dataset, filters);
    const source = seedIndexes ? seedIndexes.map((index) => dataset.records[index]) : dataset.records;

    return source.filter((record) => {
      if (filters.category !== "all" && record.category !== filters.category) return false;
      if (filters.operation !== "all" && record.operation !== filters.operation) return false;
      if (filters.distribution !== "all" && record.distribution !== filters.distribution) return false;
      if (filters.payload !== "all" && record.payload !== filters.payload) return false;
      if (filters.size !== "all" && record.size !== filters.size) return false;

      return searchMatches(record, search);
    });
  });
};

export const selectComparisonRecords = (state: DashboardStore): CriterionRecord[] => {
  const filtered = selectFilteredRecords(state);
  const selected = state.selectedImplementations;
  const key = `cmp|${state.dataset?.version ?? "none"}|${selected.join(",")}|${filtered.length}`;

  return memoComparison(key, () => {
    if (selected.length === 0) return EMPTY_RECORDS;

    const selectedSet = new Set(selected);
    return filtered.filter((record) => selectedSet.has(record.implementation));
  });
};

export const selectDashboardSummary = (state: DashboardStore): DashboardSummary => {
  const dataset = state.dataset;
  const filtered = selectFilteredRecords(state);
  const key = `summary|${dataset?.version ?? "none"}|${filtered.length}`;

  return memoSummary(key, () => {
    if (!dataset) {
      return EMPTY_SUMMARY;
    }

    let best = Number.POSITIVE_INFINITY;
    let worst = Number.NEGATIVE_INFINITY;

    for (const record of filtered) {
      if (record.meanNs < best) best = record.meanNs;
      if (record.meanNs > worst) worst = record.meanNs;
    }

    return {
      totalRecords: dataset.records.length,
      visibleRecords: filtered.length,
      implementations: new Set(filtered.map((record) => record.implementation)).size,
      sizes: new Set(filtered.map((record) => record.size)).size,
      operations: new Set(filtered.map((record) => record.operation)).size,
      bestMeanNs: Number.isFinite(best) ? best : null,
      worstMeanNs: Number.isFinite(worst) ? worst : null,
    };
  });
};
