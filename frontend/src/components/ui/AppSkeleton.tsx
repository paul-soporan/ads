import type { HTMLAttributes } from "react";

import { cn } from "@/lib/cn";

export interface AppSkeletonProps extends HTMLAttributes<HTMLDivElement> {}

export function AppSkeleton({ className, ...props }: AppSkeletonProps) {
  return (
    <div
      aria-hidden="true"
      className={cn(
        "rounded-md bg-[linear-gradient(110deg,hsl(var(--skeleton-base))_8%,hsl(var(--skeleton-shimmer))_18%,hsl(var(--skeleton-base))_33%)] bg-[length:220%_100%] animate-shimmer",
        className,
      )}
      {...props}
    />
  );
}
