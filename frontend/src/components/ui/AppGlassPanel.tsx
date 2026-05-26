import type { HTMLAttributes } from "react";

import { cn } from "@/lib/cn";

export interface AppGlassPanelProps extends HTMLAttributes<HTMLDivElement> {}

export function AppGlassPanel({ className, ...props }: AppGlassPanelProps) {
  return (
    <div
      className={cn(
        "rounded-lg border border-panel-border bg-panel/70 p-4 sm:p-5 lg:p-6 shadow-panel backdrop-blur-[16px]",
        className,
      )}
      {...props}
    />
  );
}
