import type { DexReadStatus } from "./types";

/** Shared read-only interpretation for the planner and independent HTML. */
export function dexReadSummary(
  status: DexReadStatus | null | undefined,
  t: (key: string) => string,
): string[] {
  if (!status) return [];
  const rows = status.read_only ? [t("planDexReadOnly")] : [];
  for (const range of status.uninitialized_ranges) {
    rows.push(
      `${t("planDexUninitialized")} ${range.first}–${range.first + range.count - 1} · ${t("planDexProjection")}`,
    );
  }
  return rows;
}
