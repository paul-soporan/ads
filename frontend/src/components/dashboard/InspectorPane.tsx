"use client";

import { memo } from "react";

import { VariantDrilldown } from "@/components/dashboard/VariantDrilldown";
import { VersusComparison } from "@/components/dashboard/VersusComparison";
import { AppButton } from "@/components/ui/AppButton";
import type { CriterionRecord, NormalizedBenchmarkDataset } from "@/lib/bench/types";

interface InspectorPaneProps {
  records: CriterionRecord[];
  trendRecords: CriterionRecord[];
  dataset: NormalizedBenchmarkDataset;
  selectedImplementations: string[];
  onResetComparison: () => void;
}

function InspectorPaneInner({
  records,
  trendRecords,
  dataset,
  selectedImplementations,
  onResetComparison,
}: InspectorPaneProps) {
  if (selectedImplementations.length === 0) {
    return null;
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-2">
      <div className="flex items-center justify-between px-1">
        <p className="text-sm uppercase tracking-[0.12em] text-text-muted">
          {selectedImplementations.length >= 2 ? "Inspector: Versus Mode" : "Inspector: Family Mode"}
        </p>
        <AppButton variant="ghost" size="sm" onClick={onResetComparison}>
          Clear Inspector
        </AppButton>
      </div>
      <div className="min-h-0 flex-1 overflow-auto">
        {selectedImplementations.length >= 2 ? (
          <VersusComparison
            records={records}
            trendRecords={trendRecords}
            dataset={dataset}
            selectedImplementations={selectedImplementations}
          />
        ) : (
          <VariantDrilldown
            records={trendRecords}
            dataset={dataset}
            selectedImplementations={selectedImplementations}
          />
        )}
      </div>
    </div>
  );
}

export const InspectorPane = memo(InspectorPaneInner);
