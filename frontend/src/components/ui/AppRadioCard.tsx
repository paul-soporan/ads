"use client";

import { motion } from "framer-motion";

import { cn } from "@/lib/cn";

export interface AppRadioCardOption {
  value: string;
  label: string;
  hint?: string;
}

interface AppRadioCardProps {
  label: string;
  description?: string;
  value: string;
  options: AppRadioCardOption[];
  onChange: (value: string) => void;
  name: string;
  compact?: boolean;
}

export function AppRadioCard({
  label,
  description,
  value,
  options,
  onChange,
  name,
  compact = false,
}: AppRadioCardProps) {
  return (
    <fieldset className={compact ? "space-y-1" : "space-y-3"}>
      <legend className="font-display text-sm uppercase tracking-[0.14em] text-text/80">{label}</legend>
      {description ? <p className="text-[10px] leading-tight text-text/76">{description}</p> : null}
      <div className={cn("grid sm:grid-cols-2", compact ? "gap-1 lg:grid-cols-5 2xl:grid-cols-6" : "gap-3 lg:grid-cols-4")}>
        {options.map((option, index) => {
          const isActive = option.value === value;

          return (
            <motion.label
              key={option.value}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.04, duration: 0.28 }}
              className={cn(
                "group relative block cursor-pointer rounded-md border transition focus-within:ring-2 focus-within:ring-primary/70 focus-within:ring-offset-2 focus-within:ring-offset-bg",
                compact ? "px-2 py-1.5" : "p-4",
                isActive
                  ? "border-primary bg-primary/10 shadow-glow"
                  : "border-panel-border bg-bg-elevated hover:border-primary/50 hover:bg-bg-elevated/80",
              )}
            >
              <input
                type="radio"
                name={name}
                value={option.value}
                checked={isActive}
                onChange={() => onChange(option.value)}
                className="sr-only"
                aria-label={option.label}
              />
              <span
                className={cn(
                  "block font-body font-semibold",
                  compact ? "text-[0.78rem] leading-tight" : "text-base",
                  isActive ? "text-text" : "text-text-muted group-hover:text-text/92",
                )}
              >
                {option.label}
              </span>
              {option.hint ? (
                <span
                  className={cn(
                    "block",
                    compact ? "mt-0.5 text-[10px] leading-tight" : "mt-1 text-sm",
                    isActive ? "text-text/72" : "text-text-muted/95 group-hover:text-text-muted",
                  )}
                >
                  {option.hint}
                </span>
              ) : null}
            </motion.label>
          );
        })}
      </div>
    </fieldset>
  );
}
