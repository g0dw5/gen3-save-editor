import { useEffect, useState } from "react";
import { api } from "./api";
import { NumberField, Toggle } from "./components";
import { useI18n } from "./i18n";
import type { Catalog } from "./types";

interface Check {
  status: "not_disproved" | "outside_npc_bound";
  minimum_sheen_lower_bound: number | null;
}

export function ContestCondition({
  catalog,
  condition,
  nature,
  npc,
  free,
  onNpcChange,
  onChange,
}: {
  catalog: Catalog;
  condition: number[];
  nature: number;
  npc: boolean;
  free: boolean;
  onNpcChange: (value: boolean) => void;
  onChange: (index: number, value: number) => void;
}) {
  const { t } = useI18n();
  const [result, setResult] = useState<{ key: string; check: Check } | null>(
    null,
  );
  const [errorKey, setErrorKey] = useState<string | null>(null);
  const md5 = catalog.profile.md5;
  const supported = !!catalog.profile.contest?.npc_blender;
  const key = JSON.stringify([md5, nature, condition, npc]);
  const valid =
    condition.length === 6 &&
    condition.every((v) => Number.isInteger(v) && v >= 0 && v <= 255);
  const check = result?.key === key ? result.check : null;
  useEffect(() => {
    if (!npc || !supported || !valid) return;
    let active = true;
    const timer = setTimeout(() => {
      api<Check>("contest_check", { expected_rom_md5: md5, nature, condition })
        .then((check) => {
          if (active) {
            setResult({ key, check });
            setErrorKey(null);
          }
        })
        .catch(() => {
          if (active) setErrorKey(key);
        });
    }, 180);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [key, md5, nature, npc, supported, valid]);
  return (
    <section className="contest-condition">
      <h3>{t("condition")}</h3>
      <p className="muted small">{t("contestRanges")}</p>
      <div className="field-grid">
        {["cool", "beauty", "cute", "smart", "tough"].map((name, i) => (
          <NumberField
            key={name}
            label={t(name)}
            value={condition[i]}
            min={0}
            max={255}
            onChange={(value) => onChange(i, value)}
          />
        ))}
      </div>
      <div className="contest-sheen">
        <NumberField
          label={t("sheen")}
          value={condition[5]}
          min={0}
          max={255}
          onChange={(value) => onChange(5, value)}
        />
        <p
          className={
            condition[5] === 255 ? "warning-text small" : "muted small"
          }
        >
          {t(
            catalog.profile.contest
              ? condition[5] === 255
                ? "contestFull"
                : "contestCanFeed"
              : "contestUnverified",
          )}
        </p>
      </div>
      <p className="muted small">{t("contestFeedingRule")}</p>
      {supported && (
        <>
          <Toggle
            label={t("contestNpcCheck")}
            checked={npc}
            onChange={onNpcChange}
          />
          <p className="muted small">{t("contestNpcScope")}</p>
          {npc && valid && (
            <p
              className={
                check?.status === "outside_npc_bound" || errorKey === key
                  ? "warning-text small"
                  : "muted small"
              }
              role="status"
            >
              {errorKey === key
                ? t("contestCheckFailed")
                : !check
                  ? t("contestChecking")
                  : check.status === "outside_npc_bound"
                    ? t(free ? "contestNpcFree" : "contest_npc_unreachable")
                    : t("contestNotDisproved")}
            </p>
          )}
        </>
      )}
      {(!supported || !npc) && (
        <p className="muted small">{t("contestUnverified")}</p>
      )}
    </section>
  );
}
