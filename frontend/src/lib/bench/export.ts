import type { CriterionRecord } from "./types";

export interface ComparisonExportRow {
  implementation: string;
  meanNs: number;
  estimatedMemoryBytes: number;
  speedLabel: string;
  memoryLabel: string;
  instructions?: number | null;
  instructionLabel?: string;
  l1DataMissRate?: number | null;
  l1DataMissRateLabel?: string;
}

export interface LeaderboardProfilingByJoinKey {
  instructionsByJoinKey: Map<string, number>;
  l1DataMissRateByJoinKey: Map<string, number>;
  peakBytesByJoinKey: Map<string, number>;
}

function csvEscape(value: string): string {
  if (/[",\n]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

export function leaderboardToCsv(
  records: CriterionRecord[],
  profiling?: LeaderboardProfilingByJoinKey,
): string {
  const includeProfiling = Boolean(profiling);
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

  if (includeProfiling) {
    header.push("instructions_ir", "l1_data_miss_rate", "peak_bytes");
  }

  const lines = records.map((record) =>
    (() => {
      const row = [
      record.implementation,
      record.distribution,
      record.payload,
      record.operation,
      String(record.size),
      String(Math.round(record.meanNs)),
      String(Math.round(record.ciLowerNs)),
      String(Math.round(record.ciUpperNs)),
      String(record.throughputElements ?? ""),
      ];

      if (includeProfiling && profiling) {
        const instructions = profiling.instructionsByJoinKey.get(record.joinKey);
        const l1MissRate = profiling.l1DataMissRateByJoinKey.get(record.joinKey);
        const peakBytes = profiling.peakBytesByJoinKey.get(record.joinKey);
        row.push(
          instructions != null ? String(Math.round(instructions)) : "",
          l1MissRate != null ? l1MissRate.toFixed(6) : "",
          peakBytes != null ? String(Math.round(peakBytes)) : "",
        );
      }

      return row.map(csvEscape).join(",");
    })(),
  );

  return [header.join(","), ...lines].join("\n");
}

export function leaderboardToMarkdown(
  records: CriterionRecord[],
  profiling?: LeaderboardProfilingByJoinKey,
): string {
  const includeProfiling = Boolean(profiling);
  const header = includeProfiling
    ? [
        "| Implementation | Distribution | Payload | Operation | Size | Mean (ns) | CI Lower | CI Upper | Throughput | Instructions (Ir) | L1 Data Miss Rate | Peak Bytes |",
        "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|",
      ]
    : [
        "| Implementation | Distribution | Payload | Operation | Size | Mean (ns) | CI Lower | CI Upper | Throughput |",
        "|---|---|---|---|---:|---:|---:|---:|---:|",
      ];

  const rows = records.map(
    (record) => {
      if (!includeProfiling || !profiling) {
        return `| ${record.implementation} | ${record.distribution} | ${record.payload} | ${record.operation} | ${record.size.toLocaleString()} | ${Math.round(record.meanNs).toLocaleString()} | ${Math.round(record.ciLowerNs).toLocaleString()} | ${Math.round(record.ciUpperNs).toLocaleString()} | ${(record.throughputElements ?? 0).toLocaleString()} |`;
      }

      const instructions = profiling.instructionsByJoinKey.get(record.joinKey);
      const l1MissRate = profiling.l1DataMissRateByJoinKey.get(record.joinKey);
      const peakBytes = profiling.peakBytesByJoinKey.get(record.joinKey);

      return `| ${record.implementation} | ${record.distribution} | ${record.payload} | ${record.operation} | ${record.size.toLocaleString()} | ${Math.round(record.meanNs).toLocaleString()} | ${Math.round(record.ciLowerNs).toLocaleString()} | ${Math.round(record.ciUpperNs).toLocaleString()} | ${(record.throughputElements ?? 0).toLocaleString()} | ${instructions != null ? Math.round(instructions).toLocaleString() : "n/a"} | ${l1MissRate != null ? `${(l1MissRate * 100).toFixed(3)}%` : "n/a"} | ${peakBytes != null ? Math.round(peakBytes).toLocaleString() : "n/a"} |`;
    },
  );

  return [...header, ...rows].join("\n");
}

export function comparisonToCsv(rows: ComparisonExportRow[]): string {
  const header = [
    "implementation",
    "mean_ns",
    "estimated_memory_bytes",
    "instructions_ir",
    "l1_data_miss_rate",
    "speed_delta",
    "memory_delta",
    "instruction_delta",
    "l1_data_miss_rate_delta",
  ];
  const lines = rows.map((row) =>
    [
      row.implementation,
      String(Math.round(row.meanNs)),
      String(Math.round(row.estimatedMemoryBytes)),
      row.instructions != null ? String(Math.round(row.instructions)) : "",
      row.l1DataMissRate != null ? row.l1DataMissRate.toFixed(6) : "",
      row.speedLabel,
      row.memoryLabel,
      row.instructionLabel ?? "",
      row.l1DataMissRateLabel ?? "",
    ]
      .map(csvEscape)
      .join(","),
  );

  return [header.join(","), ...lines].join("\n");
}

export function comparisonToMarkdown(rows: ComparisonExportRow[]): string {
  const header = [
    "| Implementation | Mean (ns) | Est. Memory (bytes) | Instructions (Ir) | L1 Data Miss Rate | Speed Delta | Memory Delta | Instruction Delta | L1 Miss Delta |",
    "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
  ];

  const lines = rows.map(
    (row) =>
      `| ${row.implementation} | ${Math.round(row.meanNs).toLocaleString()} | ${Math.round(row.estimatedMemoryBytes).toLocaleString()} | ${row.instructions != null ? Math.round(row.instructions).toLocaleString() : "n/a"} | ${row.l1DataMissRate != null ? `${(row.l1DataMissRate * 100).toFixed(3)}%` : "n/a"} | ${row.speedLabel} | ${row.memoryLabel} | ${row.instructionLabel ?? "n/a"} | ${row.l1DataMissRateLabel ?? "n/a"} |`,
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
