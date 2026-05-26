"use client";

import { AnimatePresence, motion } from "framer-motion";
import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

type ToastVariant = "info" | "success" | "warning" | "danger";

interface ToastInput {
  title?: string;
  message: string;
  variant?: ToastVariant;
  durationMs?: number;
}

interface ToastItem extends ToastInput {
  id: number;
  variant: ToastVariant;
}

type ToastFn = (toast: ToastInput) => void;

const ToastContext = createContext<ToastFn | null>(null);

const variantClasses: Record<ToastVariant, string> = {
  info: "border-primary/55 bg-panel/92 text-text",
  success: "border-success/55 bg-panel/92 text-text",
  warning: "border-warning/55 bg-panel/92 text-text",
  danger: "border-danger/60 bg-panel/92 text-text",
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const idRef = useRef(0);

  const removeToast = useCallback((id: number) => {
    setToasts((current) => current.filter((item) => item.id !== id));
  }, []);

  const pushToast = useCallback<ToastFn>(
    ({ title, message, variant = "info", durationMs }) => {
      const resolvedDurationMs = durationMs ?? (variant === "danger" ? 3200 : 1400);
      const id = ++idRef.current;
      setToasts((current) => [...current, { id, title, message, variant, durationMs: resolvedDurationMs }]);
      window.setTimeout(() => removeToast(id), resolvedDurationMs);
    },
    [removeToast],
  );

  const contextValue = useMemo(() => pushToast, [pushToast]);

  return (
    <ToastContext.Provider value={contextValue}>
      {children}

      <div
        className="pointer-events-none fixed inset-x-0 top-3 z-[60] mx-auto flex w-full max-w-xl flex-col gap-2 px-3 sm:top-6 sm:right-6 sm:left-auto sm:max-w-sm"
        aria-live="polite"
        aria-atomic="true"
      >
        <AnimatePresence initial={false}>
          {toasts.map((toast) => (
            <motion.div
              key={toast.id}
              role="status"
              initial={{ opacity: 0, y: 14, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 12, scale: 0.98 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
              className={cn(
                "pointer-events-auto rounded-md border px-3 py-2 shadow-panel backdrop-blur-lg",
                variantClasses[toast.variant],
              )}
            >
              {toast.title ? <p className="font-display text-xs uppercase tracking-[0.12em] text-text-muted">{toast.title}</p> : null}
              <p className="text-sm">{toast.message}</p>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </ToastContext.Provider>
  );
}

export function useToast(): ToastFn {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return context;
}
