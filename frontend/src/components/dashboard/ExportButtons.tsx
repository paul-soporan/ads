"use client";

import { AppButton } from "@/components/ui/AppButton";
import { useToast } from "@/components/ui/ToastProvider";

interface ExportButtonsProps {
  label: string;
  onExportCsv: () => Promise<"copied" | "downloaded">;
  onExportMarkdown: () => Promise<"copied" | "downloaded">;
}

export function ExportButtons({ label, onExportCsv, onExportMarkdown }: ExportButtonsProps) {
  const pushToast = useToast();

  async function handleExport(format: "csv" | "markdown") {
    const result = format === "csv" ? await onExportCsv() : await onExportMarkdown();
    const descriptor = format === "csv" ? "CSV" : "Markdown";

    pushToast({
      title: "Export",
      message: result === "copied" ? `${descriptor} copied to clipboard.` : `${descriptor} downloaded.`,
      variant: "success",
    });
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="hidden text-sm uppercase tracking-[0.12em] text-text-muted lg:inline">{label}</span>
      <AppButton variant="ghost" size="sm" onClick={() => void handleExport("csv")}>Export CSV</AppButton>
      <AppButton variant="ghost" size="sm" onClick={() => void handleExport("markdown")}>Export MD</AppButton>
    </div>
  );
}
