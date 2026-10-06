import { useI18n } from "./i18n";
import type { ClockReport } from "./types";

export function clockSummary(
  report: ClockReport,
  t: (key: string) => string,
): string {
  const saved = report.saved;
  const time = saved
    ? `${saved.year}-${String(saved.month).padStart(2, "0")}-${String(saved.day).padStart(2, "0")} · ${t(`clockWeekday${saved.weekday}`)} · ${String(saved.hour).padStart(2, "0")}:${String(saved.minute).padStart(2, "0")}:${String(saved.second).padStart(2, "0")}`
    : report.effective_hour !== null
      ? `${report.effective_hour}:00`
      : "";
  const period = report.period
    ? t(
        (
          {
            morning: "encounterMorning",
            day: "encounterDay",
            dusk: "encounterDusk",
            night: "encounterNight",
          } as Record<string, string>
        )[report.period] ?? "unresolved",
      )
    : "";
  return [
    t(
      report.source === "save_virtual"
        ? "clockSavedVirtual"
        : report.source === "scenario"
          ? "clockSimulated"
          : (report.issue ?? "clockUnknown"),
    ),
    time,
    !saved && report.weekday !== null ? t(`clockWeekday${report.weekday}`) : "",
    period,
  ]
    .filter(Boolean)
    .join(" · ");
}
export function ClockDetails({ report }: { report: ClockReport }) {
  const { t } = useI18n();
  const seconds = report.seconds_until_next_period;
  return (
    <div className="clock-report">
      <p>{clockSummary(report, t)}</p>
      {report.forced_night === true ? (
        <p>{t("clockForcedNight")}</p>
      ) : (
        report.next_period_hour !== null && (
          <p>
            {t("clockNextPeriod")} {report.next_period_hour}:00
          </p>
        )
      )}
      {report.saved && (
        <p className="small muted">
          {t("clockSnapshotHelp")} · {t("clockSpeed")} ×{report.saved.speed}
        </p>
      )}
      {seconds !== null && (
        <p className="small muted">
          {t("clockUntilNext")} {Math.floor(seconds / 3600)}:
          {String(Math.floor((seconds % 3600) / 60)).padStart(2, "0")}:
          {String(seconds % 60).padStart(2, "0")}
        </p>
      )}
      {report.hardware && (
        <div className="clock-hardware-snapshot">
          <p>
            {t("clockOffsetSave")} · {t("clockDayCounter")}{" "}
            {report.hardware.offset.days} · {report.hardware.offset.hour}:
            {String(report.hardware.offset.minute).padStart(2, "0")}:
            {String(report.hardware.offset.second).padStart(2, "0")}
          </p>
          {!report.hardware.offset_valid && <p>{t("clockInvalidOffset")}</p>}
          <p>
            {t("clockLastUpdate")} · {t("clockDayCounter")}{" "}
            {report.hardware.last_update.days} ·{" "}
            {report.hardware.last_update.hour}:
            {String(report.hardware.last_update.minute).padStart(2, "0")}:
            {String(report.hardware.last_update.second).padStart(2, "0")}
          </p>
          {!report.hardware.last_update_valid && (
            <p>{t("clockInvalidCheckpoint")}</p>
          )}
          <p className="small muted">{t("clockHardwareSnapshotHelp")}</p>
        </div>
      )}
      <details>
        <summary>{t("evidence")}</summary>
        <pre>{JSON.stringify(report, null, 2)}</pre>
      </details>
    </div>
  );
}
