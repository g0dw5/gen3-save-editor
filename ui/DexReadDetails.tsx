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
  const inconsistent = status.inconsistent_numbers ?? [];
  if (inconsistent.length) {
    rows.push(
      `${t("planDexInconsistent")} ${inconsistent.length} · ${t("planDexNativeChecks")}`,
    );
    rows.push(
      `${t("number")}: ${inconsistent.slice(0, 20).join(", ")}${inconsistent.length > 20 ? "…" : ""}`,
    );
  }
  return rows;
}
