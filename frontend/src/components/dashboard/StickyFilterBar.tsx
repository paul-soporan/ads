"use client";

import { useState } from "react";

import { AppButton } from "@/components/ui/AppButton";
import { AppGlassPanel } from "@/components/ui/AppGlassPanel";
import { AppRadioCard } from "@/components/ui/AppRadioCard";
import { cn } from "@/lib/cn";
import type { DashboardFilters, Distribution, PayloadKind } from "@/lib/bench/types";

interface StickyFilterBarProps {
  filters: DashboardFilters;
  operationOptions: string[];
  distributionOptions: Distribution[];
  payloadOptions: PayloadKind[];
  sizeOptions: number[];
  showAdvancedFilters: boolean;
  onFilterChange: <K extends keyof DashboardFilters>(key: K, value: DashboardFilters[K]) => void;
  onResetFilters: () => void;
  onShowAdvancedFiltersChange: (value: boolean) => void;
}

export function StickyFilterBar({
  filters,
  operationOptions,
  distributionOptions,
  payloadOptions,
  sizeOptions,
  showAdvancedFilters,
  onFilterChange,
  onResetFilters,
  onShowAdvancedFiltersChange,
}: StickyFilterBarProps) {
  const [mobileExpanded, setMobileExpanded] = useState(false);

  const operationCards = ["all", ...operationOptions].map((value) => ({
    value,
    label: value === "all" ? "All" : value,
    hint: "Operation",
  }));

  const distributionCards = ["all", ...distributionOptions].map((value) => ({
    value,
    label: value === "all" ? "All" : value,
    hint: "Distribution",
  }));

  const payloadCards = ["all", ...payloadOptions].map((value) => ({
    value,
    label: value === "all" ? "All" : value,
    hint: "Payload",
  }));

  const sizeCards = ["all", ...sizeOptions.slice(0, 8).map((size) => String(size))].map((value) => ({
    value,
    label: value === "all" ? "All" : Number(value).toLocaleString(),
    hint: "Input size",
  }));

  return (
    <div className="sticky top-3 z-20">
      <AppGlassPanel className="space-y-3 border-primary/30 bg-panel/78 p-3 sm:p-4">
        <div className="flex flex-wrap items-center gap-3">
          <div className="space-y-0.5">
            <p className="font-display text-sm uppercase tracking-[0.14em] text-text/86">Workload Simulator</p>
            <p className="text-xs text-text/82">Fast, reversible filters over benchmark scenarios.</p>
          </div>
        </div>

        <div className="md:hidden">
          <AppButton
            variant="secondary"
            size="sm"
            onClick={() => setMobileExpanded((value) => !value)}
            className="w-full border border-panel-border/90 bg-bg-elevated/90 text-text shadow-none hover:bg-bg-elevated"
            aria-expanded={mobileExpanded}
            aria-controls="mobile-filter-groups"
          >
            {mobileExpanded ? "Hide Scenario Filters" : "Show Scenario Filters"}
          </AppButton>
        </div>

        <div id="mobile-filter-groups" className={cn("space-y-2", mobileExpanded ? "block" : "hidden md:block")}>
          <AppRadioCard
            label="Operation"
            description="Choose operation family"
            name="operation-filter"
            options={operationCards}
            value={String(filters.operation)}
            onChange={(value) => onFilterChange("operation", value)}
            compact
          />

          <AppRadioCard
            label="Distribution"
            description="Dataset/query distribution profile"
            name="distribution-filter"
            options={distributionCards}
            value={String(filters.distribution)}
            onChange={(value) => onFilterChange("distribution", value as DashboardFilters["distribution"])}
            compact
          />

          <div className="flex flex-wrap items-center justify-end gap-2">
            <AppButton
              variant="secondary"
              size="sm"
              className="border border-panel-border/90 bg-bg-elevated/90 text-text shadow-none hover:bg-bg-elevated"
              onClick={() => onShowAdvancedFiltersChange(!showAdvancedFilters)}
              aria-expanded={showAdvancedFilters}
              aria-controls="advanced-filter-groups"
            >
              {showAdvancedFilters ? "Hide Secondary Filters" : "Show Secondary Filters"}
            </AppButton>
            <AppButton
              variant="secondary"
              size="sm"
              className="border border-panel-border/90 bg-bg-elevated/90 text-text shadow-none hover:bg-bg-elevated"
              onClick={onResetFilters}
            >
              Clear Filters
            </AppButton>
          </div>

          <div id="advanced-filter-groups" className={cn("space-y-2", showAdvancedFilters ? "block" : "hidden")}>
            <AppRadioCard
              label="Payload"
              description="Key/value payload stress type"
              name="payload-filter"
              options={payloadCards}
              value={String(filters.payload)}
              onChange={(value) => onFilterChange("payload", value as DashboardFilters["payload"])}
              compact
            />

            <AppRadioCard
              label="Size"
              description="Working-set scale"
              name="size-filter"
              options={sizeCards}
              value={String(filters.size)}
              onChange={(value) => onFilterChange("size", value === "all" ? "all" : Number(value))}
              compact
            />
          </div>
        </div>
      </AppGlassPanel>
    </div>
  );
}
