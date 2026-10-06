import { useContext, useEffect, useRef, useState } from "react";
import { api } from "./api";
import { ClockDetails } from "./ClockDetails";
import { ConditionQueryRevision } from "./ConditionDetails";
import { useI18n } from "./i18n";
import type { Catalog, ClockReport, RtcProjection, Snapshot } from "./types";

export function ClockPanel({
  catalog,
  save,
}: {
  catalog: Catalog;
  save: Snapshot | null;
}) {
  const { t } = useI18n();
  const revision = useContext(ConditionQueryRevision);
  const generation = useRef(0);
  const projectionGeneration = useRef(0);
  const [report, setReport] = useState<ClockReport | null>(null);
  const [projection, setProjection] = useState<RtcProjection | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [queryBusy, setBusy] = useState(false);
  const [projectionBusy, setProjectionBusy] = useState(false);
  const busy = queryBusy || projectionBusy;
  const clearProjection = () => {
    projectionGeneration.current++;
    setProjection(null);
    setProjectionBusy(false);
  };
  const [hour, setHour] = useState("");
  const [weekday, setWeekday] = useState("");
  const [date, setDate] = useState("2000-01-01");
  const [time, setTime] = useState("12:00:00");
  const [offsetSource, setOffsetSource] = useState<"save" | "manual">(
    save ? "save" : "manual",
  );
  const [offset, setOffset] = useState({
    days: "0",
    hour: "0",
    minute: "0",
    second: "0",
  });
  const md5 = catalog.profile.md5;
  useEffect(() => {
    const ticket = ++generation.current;
    projectionGeneration.current++;
    setProjectionBusy(false);
    setReport(null);
    setProjection(null);
    setError(null);
    setBusy(true);
    api<ClockReport>("clock_query", {
      hour: hour === "" ? null : +hour,
      weekday: weekday === "" ? null : +weekday,
    })
      .then((r) => {
        if (generation.current === ticket && r.rom_md5 === md5) setReport(r);
      })
      .catch((e) => {
        if (generation.current === ticket) setError(e);
      })
      .finally(() => {
        if (generation.current === ticket) setBusy(false);
      });
    return () => {
      generation.current++;
      projectionGeneration.current++;
    };
  }, [md5, revision, hour, weekday]);
  useEffect(() => {
    if (!save) setOffsetSource("manual");
  }, [save]);
  const project = async () => {
    const ticket = ++projectionGeneration.current;
    setProjectionBusy(true);
    setProjection(null);
    setError(null);
    try {
      const [year, month, day] = date.split("-").map(Number);
      const [h, minute, second = 0] = time.split(":").map(Number);
      const r = await api<RtcProjection>("clock_rtc_preview", {
        expected_rom_md5: md5,
        rtc: { year, month, day, hour: h, minute, second },
        offset:
          offsetSource === "save"
            ? null
            : Object.fromEntries(
                Object.entries(offset).map(([k, v]) => [k, Number(v)]),
              ),
      });
      if (projectionGeneration.current === ticket && r.rom_md5 === md5)
        setProjection(r);
    } catch (e) {
      if (projectionGeneration.current === ticket) setError(e);
    } finally {
      if (projectionGeneration.current === ticket) setProjectionBusy(false);
    }
  };
  const errorCode = (error as { code?: string } | null)?.code;
  const errorLabel =
    errorCode === "clock_rtc_invalid" || errorCode === "clock_rtc_range"
      ? "clockInvalidRtc"
      : errorCode === "clock_save_required"
        ? "clockNeedsOffset"
        : errorCode === "clock_saved_offset"
          ? "clockInvalidOffset"
          : "clockQueryError";
  return (
    <section className="clock-panel">
      <h2>{t("clockPage")}</h2>
      <p className="small muted">{t("clockPageHelp")}</p>
      {busy && <p role="status">{t("loading")}</p>}
      {report && <ClockDetails report={report} />}
      {catalog.profile.clock && (
        <fieldset className="clock-scenario">
          <legend>{t("clockScenario")}</legend>
          <label>
            {t("clockHour")}{" "}
            <select
              aria-label={t("clockHour")}
              value={hour}
              onChange={(e) => setHour(e.target.value)}
            >
              <option value="">
                {t(weekday === "" ? "clockUseSave" : "clockUnspecified")}
              </option>
              {Array.from({ length: 24 }, (_, h) => (
                <option key={h} value={h}>
                  {h}:00
                </option>
              ))}
            </select>
          </label>
          <label>
            {t("clockWeekday")}{" "}
            <select
              aria-label={t("clockWeekday")}
              value={weekday}
              onChange={(e) => setWeekday(e.target.value)}
            >
              <option value="">
                {t(hour === "" ? "clockUseSave" : "clockUnspecified")}
              </option>
              {Array.from({ length: 7 }, (_, d) => (
                <option key={d} value={d}>
                  {t(`clockWeekday${d}`)}
                </option>
              ))}
            </select>
          </label>
          <p className="small muted">{t("clockIndependentScenario")}</p>
        </fieldset>
      )}
      {!!catalog.profile.hardware_clock && (
        <fieldset className="clock-scenario">
          <legend>{t("clockRtcScenario")}</legend>
          <p className="small muted">{t("clockRtcScenarioHelp")}</p>
          <label>
            {t("clockRtcDate")}{" "}
            <input
              type="date"
              min="2000-01-01"
              max="2099-12-31"
              value={date}
              onChange={(e) => {
                setDate(e.target.value);
                clearProjection();
              }}
            />
          </label>
          <label>
            {t("clockRtcTime")}{" "}
            <input
              type="time"
              step="1"
              value={time}
              onChange={(e) => {
                setTime(e.target.value);
                clearProjection();
              }}
            />
          </label>
          <label>
            {t("clockOffsetSource")}{" "}
            <select
              aria-label={t("clockOffsetSource")}
              value={offsetSource}
              onChange={(e) => {
                setOffsetSource(e.target.value as "save" | "manual");
                clearProjection();
              }}
            >
              <option value="save" disabled={!save}>
                {t("clockOffsetSave")}
              </option>
              <option value="manual">{t("clockOffsetManual")}</option>
            </select>
          </label>
          {offsetSource === "manual" && (
            <div className="clock-offsets">
              {(["days", "hour", "minute", "second"] as const).map((key) => (
                <label key={key}>
                  {t(`clockOffset_${key}`)}{" "}
                  <input
                    type="number"
                    step="1"
                    min={key === "days" ? -32768 : 0}
                    max={key === "days" ? 32767 : key === "hour" ? 23 : 59}
                    value={offset[key]}
                    onChange={(e) => {
                      setOffset({ ...offset, [key]: e.target.value });
                      clearProjection();
                    }}
                  />
                </label>
              ))}
            </div>
          )}
          <button
            onClick={project}
            disabled={
              busy ||
              !date ||
              !time ||
              (offsetSource === "manual" &&
                Object.values(offset).some((v) => v === ""))
            }
          >
            {t("clockRtcProject")}
          </button>
          {projection && (
            <div className="clock-projection">
              <p>
                <strong>{t("clockSimulated")}</strong> ·{" "}
                {String(projection.local_time.hour).padStart(2, "0")}:
                {String(projection.local_time.minute).padStart(2, "0")}:
                {String(projection.local_time.second).padStart(2, "0")}
              </p>
              <p>
                {t("clockDayCounter")} {projection.local_time.days} ·{" "}
                {t(
                  projection.offset_source === "save"
                    ? "clockOffsetSave"
                    : "clockOffsetManual",
                )}
              </p>
              <p className="small muted">{t("clockRtcResultHelp")}</p>
              <details>
                <summary>{t("evidence")}</summary>
                <pre>{JSON.stringify(projection, null, 2)}</pre>
              </details>
            </div>
          )}
        </fieldset>
      )}
      {!!error && (
        <div>
          <p role="alert">{t(errorLabel)}</p>
          <details>
            <summary>{t("evidence")}</summary>
            <pre>{JSON.stringify(error, null, 2)}</pre>
          </details>
        </div>
      )}
    </section>
  );
}
