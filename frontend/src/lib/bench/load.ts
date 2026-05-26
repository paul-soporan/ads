import { normalizeBenchmarkArtifact } from "./normalize";
import type { NormalizedBenchmarkDataset, RawBenchmarkArtifact } from "./types";

const DEFAULT_DATA_PATH = "/aggregated_benchmarks.json";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function validateArtifact(input: unknown): asserts input is RawBenchmarkArtifact {
  if (!isObject(input)) {
    throw new Error("Benchmark artifact is not an object");
  }

  if (!Array.isArray(input.criterion)) {
    if (!Array.isArray(input.operations)) {
      throw new Error("Missing operations in benchmark artifact");
    }
  }

  if (Array.isArray(input.operations)) {
    for (const operation of input.operations) {
      if (!isObject(operation) || !isObject(operation.join)) {
        throw new Error("Invalid operation group in benchmark artifact");
      }
    }
  }

  if (typeof input.generated_at_unix_secs !== "number") {
    throw new Error("Invalid generated_at_unix_secs in benchmark artifact");
  }

  if (typeof input.operation_count !== "number") {
    throw new Error("Invalid operation_count in benchmark artifact");
  }
}

export async function loadBenchmarkDataset(dataPath = DEFAULT_DATA_PATH): Promise<NormalizedBenchmarkDataset> {
  const requestPath =
    typeof window === "undefined"
      ? dataPath
      : `${dataPath}${dataPath.includes("?") ? "&" : "?"}t=${Date.now()}`;

  const response = await fetch(requestPath, {
    cache: "no-store",
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Unable to fetch benchmark artifact from ${dataPath} (HTTP ${response.status}). ` +
        "Run `yarn frontend:sync-data` to copy data into frontend/public/.",
    );
  }

  const payload = (await response.json()) as unknown;
  validateArtifact(payload);

  return normalizeBenchmarkArtifact(payload);
}
