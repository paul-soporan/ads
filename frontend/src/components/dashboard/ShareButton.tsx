"use client";

import { AppButton } from "@/components/ui/AppButton";
import { useToast } from "@/components/ui/ToastProvider";

interface ShareButtonProps {
  getShareUrl: () => string;
}

export function ShareButton({ getShareUrl }: ShareButtonProps) {
  const pushToast = useToast();

  async function handleShare() {
    const url = getShareUrl();

    try {
      await navigator.clipboard.writeText(url);
      pushToast({ title: "Share", message: "Deep link copied to clipboard.", variant: "success" });
    } catch {
      pushToast({ title: "Share", message: "Unable to copy link to clipboard.", variant: "danger" });
    }
  }

  return (
    <div className="flex items-center gap-1">
      <AppButton variant="ghost" size="sm" onClick={() => void handleShare()} className="gap-1 px-2 sm:px-3" aria-label="Share current dashboard link">
        <span aria-hidden="true">↗</span>
        <span className="hidden sm:inline">Share</span>
        <span className="sr-only sm:hidden">Share</span>
      </AppButton>
    </div>
  );
}
