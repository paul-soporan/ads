import { forwardRef } from "react";
import type { ButtonHTMLAttributes } from "react";

import { cn } from "@/lib/cn";

type AppButtonVariant = "primary" | "secondary" | "ghost";
type AppButtonSize = "sm" | "md" | "lg";

export interface AppButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: AppButtonVariant;
  size?: AppButtonSize;
}

const variantClasses: Record<AppButtonVariant, string> = {
  primary:
    "bg-primary text-bg shadow-glow hover:brightness-110 active:brightness-95 focus-visible:ring-primary/70",
  secondary:
    "bg-secondary text-text hover:brightness-110 active:brightness-95 focus-visible:ring-secondary/70",
  ghost:
    "bg-bg-elevated/40 text-text hover:bg-bg-elevated/70 active:bg-bg-elevated focus-visible:ring-primary/45",
};

const sizeClasses: Record<AppButtonSize, string> = {
  sm: "h-9 px-3 text-sm",
  md: "h-11 px-4 text-sm",
  lg: "h-12 px-5 text-base",
};

export const AppButton = forwardRef<HTMLButtonElement, AppButtonProps>(function AppButton(
  { className, type = "button", variant = "primary", size = "md", ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={cn(
        "inline-flex items-center justify-center rounded-md font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:cursor-not-allowed disabled:opacity-50",
        variantClasses[variant],
        sizeClasses[size],
        className,
      )}
      {...props}
    />
  );
});
