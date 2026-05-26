export const COMPARISON_COLOR_PALETTE = ["#06b6d4", "#d946ef", "#10b981", "#f59e0b"] as const;

export type ComparisonColorMap = Record<string, string>;

export function assignComparisonColors(
  previous: ComparisonColorMap,
  selectedImplementations: string[],
): ComparisonColorMap {
  const next: ComparisonColorMap = {};
  const used = new Set<string>();

  for (const implementation of selectedImplementations) {
    const existing = previous[implementation];
    if (!existing || !COMPARISON_COLOR_PALETTE.includes(existing as (typeof COMPARISON_COLOR_PALETTE)[number])) {
      continue;
    }

    if (used.has(existing)) continue;

    next[implementation] = existing;
    used.add(existing);
  }

  for (let index = 0; index < selectedImplementations.length; index += 1) {
    const implementation = selectedImplementations[index];
    if (!implementation || next[implementation]) continue;

    const available = COMPARISON_COLOR_PALETTE.find((color) => !used.has(color));
    const fallback = COMPARISON_COLOR_PALETTE[index % COMPARISON_COLOR_PALETTE.length];
    const color = available ?? fallback;

    next[implementation] = color;
    used.add(color);
  }

  return next;
}
