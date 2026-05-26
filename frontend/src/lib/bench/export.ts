import type { CriterionRecord } from "./types";

export interface ComparisonExportRow {
  implementation: string;
  meanNs: number;
  estimatedMemoryBytes: number;
  speedLabel: string;
  memoryLabel: string;
}

function csvEscape(value: string): string {
  if (/[",\n]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

export function leaderboardToCsv(records: CriterionRecord[]): string {
  const header = [
    "implementation",
    "distribution",
    "payload",
    "operation",
    "size",
    "mean_ns",
    "ci_lower_ns",
    "ci_upper_ns",
    "throughput_elements",
  ];
  const lines = records.map((record) =>
    [
      record.implementation,
      record.distribution,
      record.payload,
      record.operation,
      String(record.size),
      String(Math.round(record.meanNs)),
      String(Math.round(record.ciLowerNs)),
      String(Math.round(record.ciUpperNs)),
      String(record.throughputElements ?? ""),
    ]
      .map(csvEscape)
      .join(","),
  );

  return [header.join(","), ...lines].join("\n");
}

export function leaderboardToMarkdown(records: CriterionRecord[]): string {
  const header = [
    "| Implementation | Distribution | Payload | Operation | Size | Mean (ns) | CI Lower | CI Upper | Throughput |",
    "|---|---|---|---|---:|---:|---:|---:|---:|",
  ];

  const rows = records.map(
    (record) =>
      `| ${record.implementation} | ${record.distribution} | ${record.payload} | ${record.operation} | ${record.size.toLocaleString()} | ${Math.round(record.meanNs).toLocaleString()} | ${Math.round(record.ciLowerNs).toLocaleString()} | ${Math.round(record.ciUpperNs).toLocaleString()} | ${(record.throughputElements ?? 0).toLocaleString()} |`,
  );

  return [...header, ...rows].join("\n");
}

export function comparisonToCsv(rows: ComparisonExportRow[]): string {
  const header = ["implementation", "mean_ns", "estimated_memory_bytes", "speed_delta", "memory_delta"];
  const lines = rows.map((row) =>
    [
      row.implementation,
      String(Math.round(row.meanNs)),
      String(Math.round(row.estimatedMemoryBytes)),
      row.speedLabel,
      row.memoryLabel,
    ]
      .map(csvEscape)
      .join(","),
  );

  return [header.join(","), ...lines].join("\n");
}

export function comparisonToMarkdown(rows: ComparisonExportRow[]): string {
  const header = [
    "| Implementation | Mean (ns) | Est. Memory (bytes) | Speed Delta | Memory Delta |",
    "|---|---:|---:|---:|---:|",
  ];

  const lines = rows.map(
    (row) =>
      `| ${row.implementation} | ${Math.round(row.meanNs).toLocaleString()} | ${Math.round(row.estimatedMemoryBytes).toLocaleString()} | ${row.speedLabel} | ${row.memoryLabel} |`,
  );

  return [...header, ...lines].join("\n");
}

export async function copyOrDownload(text: string, fileName: string): Promise<"copied" | "downloaded"> {
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return "copied";
    }
  } catch {
    // fall through to download
  }

  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = fileName;
  link.click();
  URL.revokeObjectURL(link.href);

  return "downloaded";
}
