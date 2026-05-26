export const VARIANT_COLORS = {
  safe: "hsl(var(--variant-safe))",
  raw: "hsl(var(--variant-raw))",
  arena: "hsl(var(--variant-arena))",
  std: "hsl(var(--variant-std))",
  other: "hsl(var(--variant-other))",
} as const;

export const CHART_COLORS = {
  grid: "hsl(var(--chart-grid))",
  gridSubtle: "hsl(var(--chart-grid-subtle))",
  axis: "hsl(var(--chart-axis))",
  label: "hsl(var(--chart-label))",
  paretoStart: "hsl(var(--primary))",
  paretoEnd: "hsl(var(--secondary))",
  pointActiveStroke: "hsl(var(--text))",
  pointStroke: "hsl(var(--chart-label) / 0.42)",
} as const;
