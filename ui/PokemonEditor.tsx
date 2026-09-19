import { useEffect, useLayoutEffect, useMemo, useState } from "react";
import {
  BookOpen,
  Download,
  Trash2,
  Copy,
  MoveRight,
  Sparkles,
} from "lucide-react";
import { api } from "./api";
import { romOption, itemOption } from "./names";
import { hiddenPower } from "./hiddenPower";
import { HiddenPowerSummary } from "./HiddenPowerSummary";
import { PokemonOrigin, PokemonAdvanced } from "./PokemonMetadata";
import {
  useI18n,
  natures,
  statKeys,
  typeNames,
  moveCategoryNames,
} from "./i18n";
import { NumberField, SelectField, Sprite, Toggle, Types } from "./components";
import type {
  Catalog,
  Location,
  Pokemon,
  SpeciesDetail,
  StoredPokemon,
} from "./types";

const editorTabs = [
  "overview",
  "stats",
  "moves",
  "origin",
  "advanced",
] as const;
export type PokemonEditorTab = (typeof editorTabs)[number];

interface Props {
  tab: PokemonEditorTab;
  onTabChange: (tab: PokemonEditorTab) => void;
  row: StoredPokemon;
  catalog: Catalog;
  free: boolean;
  setFree: (v: boolean) => void;
  onDirty: (v: boolean) => void;
  onApply: (patch: Record<string, unknown>) => Promise<boolean>;
  onReference: (id: number) => void;
  onTransfer: (copy: boolean) => void;
  onDelete: () => void;
  onExport: () => void;
  batchLocations: Location[];
}
export function PokemonEditor({
  tab,
  onTabChange,
  row,
  catalog,
  free,
  setFree,
  onDirty,
  onApply,
  onReference,
  onTransfer,
  onDelete,
  onExport,
  batchLocations,
}: Props) {
  const { t, locale } = useI18n();
  const p = row.pokemon;
  const [patch, setPatch] = useState<Record<string, unknown>>({});
  useLayoutEffect(() => {
    setPatch({});
  }, [p]);
  const [detail, setDetail] = useState<SpeciesDetail | null>(null);
  const [allMoves, setAllMoves] = useState(false);
  const merged = { ...p, ...patch } as Pokemon;
  const hpRules = catalog.profile.hidden_power;
  const hp = hiddenPower(hpRules, merged.ivs);
  const hasNatureOverride = catalog.editor_rules?.nature_override ?? false;
  const effectiveNature =
    hasNatureOverride &&
    merged.nature_override != null &&
    merged.nature_override < 25
      ? merged.nature_override
      : patch.pid !== undefined
        ? merged.pid % 25
        : merged.nature;
  const natureUp = Math.floor(effectiveNature / 5) + 1;
  const natureDown = (effectiveNature % 5) + 1;
  const metadataProps = {
    pokemon: merged,
    catalog,
    change: (key: string, value: unknown) => change(key, value),
    free,
    pidLocked: ["nature", "gender", "shiny"].some((key) => key in patch),
  };
  const species = catalog.species.find((s) => s.id === merged.species)!;
  const change = (key: string, value: unknown) =>
    setPatch((old) => ({ ...old, [key]: value }));
  // Publish the guard before the edited form becomes interactive.
  useLayoutEffect(() => {
    onDirty(Object.keys(patch).length > 0);
  }, [patch, onDirty]);
  useEffect(() => {
    let current = true;
    setDetail(null);
    api<SpeciesDetail>("species", { id: merged.species })
      .then((d) => {
        if (current) setDetail(d);
      })
      .catch(() => {});
    return () => {
      current = false;
    };
  }, [merged.species]);
  const known = useMemo(
    () =>
      new Set(
        detail?.learnset
          .filter((s) => s.level === null || s.level <= merged.level)
          .map((s) => s.move_id),
      ),
    [detail, merged.level],
  );
  const future = useMemo(
    () => new Set(detail?.learnset.map((s) => s.move_id)),
    [detail],
  );
  const moveOptions = (current: number) =>
    catalog.moves
      .filter(
        (m) =>
          m.id === 0 || m.id === current || free || allMoves || known.has(m.id),
      )
      .map((m) => {
        const isHiddenPower = m.id === hpRules?.move_id;
        const moveType = isHiddenPower
          ? hp
            ? typeNames[locale][hp.type]
            : t("unresolved")
          : (typeNames[locale][m.move_type] ?? "?");
        const power = isHiddenPower ? (hp?.power ?? "?") : m.power || "—";
        return {
          ...romOption(m),
          label: `${m.id ? `【${moveCategoryNames[locale][m.category] ?? "?"}】【${moveType}】【${power}】${romOption(m).label}` : t("emptyMove")}${m.id && !known.has(m.id) ? ` — ${future.has(m.id) ? t("futureMove") : t("unknownSource")}` : ""}`,
        };
      });
  const itemOptions = catalog.items.map((i) =>
    i.id ? itemOption(catalog, i) : { value: 0, label: t("emptyMove") },
  );
  const num = (key: keyof Pokemon, max = 255, min = 0) => (
    <NumberField
      key={key}
      label={t(key)}
      value={Number(merged[key] ?? 0)}
      onChange={(v) => change(key, v)}
      min={min}
      max={max}
    />
  );
  const arrayChange = (
    key: "ivs" | "evs" | "condition" | "moves" | "pps" | "pp_ups",
    i: number,
    value: number,
  ) => {
    const a = [...merged[key]];
    a[i] = value;
    change(key, a);
  };
  const moveChange = (i: number, id: number) => {
    const moves = [...merged.moves];
    const pps = [...merged.pps];
    moves[i] = id;
    pps[i] = Math.floor(
      ((catalog.moves[id]?.pp ?? 0) * (5 + merged.pp_ups[i])) / 5,
    );
    setPatch((old) => ({ ...old, moves, pps }));
  };
  const submit = async () => {
    if (await onApply(patch)) {
      setPatch({});
      onDirty(false);
    }
  };
  return (
    <form
      className="pokemon-editor"
      onSubmit={(e) => {
        e.preventDefault();
        void submit();
      }}
    >
      <div className="editor-header">
        <div className="editor-hero">
          <Sprite
            catalog={catalog}
            species={merged.species}
            shiny={merged.shiny}
            pid={merged.pid}
            large
          />
          <div>
            <div className="eyebrow">
              {batchLocations.length > 1
                ? `${t("batch")} · ${batchLocations.length}`
                : row.location.kind === "party"
                  ? `${t("party")} ${row.location.slot + 1}`
                  : `${t("box")} ${row.location.box_index + 1} · ${row.location.slot + 1}`}
            </div>
            <h2>
              {species?.name} {merged.shiny && <Sparkles size={17} />}
            </h2>
            <Types values={species?.types ?? []} />
          </div>
        </div>
        <div className="editor-tabs">
          {editorTabs.map((key) => (
            <button
              type="button"
              key={key}
              className={tab === key ? "active" : ""}
              onClick={() => onTabChange(key)}
              aria-pressed={tab === key}
            >
              {t(key)}
            </button>
          ))}
        </div>
      </div>
      <div className="editor-body">
        <div className="editor-fields">
          {tab === "overview" && (
            <>
              <SelectField
                searchable
                label={t("species")}
                value={merged.species}
                onChange={(v) => change("species", +v)}
                options={catalog.species
                  .filter((s) => s.stats[0] > 0)
                  .map((s) => romOption(s))}
              />
              <label className="field">
                <span>{t("nickname")}</span>
                <input
                  value={merged.nickname}
                  onChange={(e) => change("nickname", e.target.value)}
                />
              </label>
              <div className="field-grid">
                {num("level", catalog.profile.max_level ?? 100, 1)}
                {num("experience", 0xffffffff)}
                <SelectField
                  searchable
                  label={t("nature")}
                  value={
                    hasNatureOverride
                      ? (merged.nature_override ?? 26)
                      : effectiveNature
                  }
                  onChange={(v) =>
                    change(hasNatureOverride ? "nature_override" : "nature", +v)
                  }
                  options={[
                    ...natures[locale].map((label, value) => ({
                      value,
                      label,
                    })),
                    ...(hasNatureOverride
                      ? [
                          {
                            value: 26,
                            label: `${t("pidNature")} · ${natures[locale][merged.pid % 25]}`,
                          },
                        ]
                      : []),
                  ]}
                />
                <SelectField
                  searchable
                  label={t("gender")}
                  value={merged.gender}
                  onChange={(v) => change("gender", v)}
                  options={["male", "female", "genderless"].map((v) => ({
                    value: v,
                    label: t(v),
                  }))}
                />
              </div>
              <SelectField
                searchable
                label={t("ability")}
                value={merged.ability_slot}
                onChange={(v) => change("ability_slot", +v)}
                options={(species?.abilities ?? [0, 0]).map((id, value) => ({
                  value,
                  label: `${value + 1} · ${id ? catalog.abilities[id]?.name : t("emptyMove")}`,
                  disabled: value === 1 && !id && !free,
                }))}
              />
              <SelectField
                searchable
                label={t("held_item")}
                value={merged.held_item}
                onChange={(v) => change("held_item", +v)}
                options={itemOptions}
              />
              <div className="toggle-row">
                <Toggle
                  label={t("shiny")}
                  checked={merged.shiny}
                  onChange={(v) => change("shiny", v)}
                />
                <Toggle
                  label={t("egg")}
                  checked={merged.egg}
                  onChange={(v) => change("egg", v)}
                />
              </div>
              {num("friendship")}
            </>
          )}
          {tab === "stats" && (
            <>
              <p className="nature-summary">
                {t("nature")} · {natures[locale][effectiveNature]}
                {natureUp === natureDown ? ` · ${t("neutralNature")}` : ""}
              </p>
              <table className="stat-table">
                <thead>
                  <tr>
                    <th></th>
                    <th>{t("ivs")}</th>
                    <th>{t("evs")}</th>
                    <th>{t("calculated")}</th>
                  </tr>
                </thead>
                <tbody>
                  {statKeys.map((key, i) => (
                    <tr key={key}>
                      <th>
                        {t(key)}
                        {natureUp !== natureDown && i === natureUp && (
                          <small
                            className="nature-modifier increase"
                            aria-label={`${t(key)} ${t("natureIncrease")}`}
                          >
                            ↑ 10%
                          </small>
                        )}
                        {natureUp !== natureDown && i === natureDown && (
                          <small
                            className="nature-modifier decrease"
                            aria-label={`${t(key)} ${t("natureDecrease")}`}
                          >
                            ↓ 10%
                          </small>
                        )}
                      </th>
                      <td>
                        <input
                          type="number"
                          aria-label={`${t(key)} ${t("ivs")}`}
                          min="0"
                          max="31"
                          required
                          value={merged.ivs[i]}
                          onChange={(e) =>
                            arrayChange("ivs", i, +e.target.value)
                          }
                        />
                      </td>
                      <td>
                        <input
                          type="number"
                          aria-label={`${t(key)} ${t("evs")}`}
                          min="0"
                          max="255"
                          required
                          value={merged.evs[i]}
                          onChange={(e) =>
                            arrayChange("evs", i, +e.target.value)
                          }
                        />
                      </td>
                      <td className="calculated-stat">{p.stats[i]}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <HiddenPowerSummary catalog={catalog} ivs={merged.ivs} />
              <div
                className={`ev-total ${merged.evs.reduce((a, b) => a + b, 0) > 510 ? "warning-text" : ""}`}
              >
                {t("evs")} · {t("total")}{" "}
                {merged.evs.reduce((a, b) => a + b, 0)} / 510
              </div>
              {p.current_hp !== null && (
                <div className="field-grid">
                  {num("current_hp", 65535)}
                  <SelectField
                    searchable
                    label={t("status")}
                    value={merged.status ?? 0}
                    onChange={(v) => change("status", +v)}
                    options={[
                      ...[0, 8, 16, 32, 64, 128].map((value) => ({
                        value,
                        label: t(`status_${value}`),
                      })),
                      ...Array.from({ length: 7 }, (_, i) => ({
                        value: i + 1,
                        label: `${t("status_sleep")} · ${i + 1}`,
                      })),
                      ...(![
                        0, 1, 2, 3, 4, 5, 6, 7, 8, 16, 32, 64, 128,
                      ].includes(merged.status ?? 0)
                        ? [
                            {
                              value: merged.status ?? 0,
                              label: `${t("unknownValue")} #${merged.status}`,
                              disabled: true,
                            },
                          ]
                        : []),
                    ]}
                  />
                </div>
              )}
              <p className="muted small">
                {t("calculated")} → {t("apply")}
              </p>
            </>
          )}
          {tab === "moves" && (
            <>
              <Toggle
                label={t("allMoves")}
                checked={allMoves || free}
                onChange={setAllMoves}
              />
              <details className="pp-help small muted">
                <summary>{t("ppHelpTitle")}</summary>
                <p>{t("ppStorageHelp")}</p>
              </details>
              {hpRules && merged.moves.includes(hpRules.move_id) && (
                <HiddenPowerSummary catalog={catalog} ivs={merged.ivs} />
              )}
              {merged.moves.map((id, i) => (
                <div className="move-card" key={i}>
                  <SelectField
                    searchable
                    label={`${t("move")} ${i + 1}`}
                    value={id}
                    onChange={(v) => moveChange(i, +v)}
                    options={moveOptions(id)}
                  />
                  <div className="move-pp-row">
                    <NumberField
                      label={t("currentPp")}
                      value={merged.pps[i]}
                      onChange={(v) => arrayChange("pps", i, v)}
                      max={
                        free
                          ? 255
                          : Math.floor(
                              ((catalog.moves[id]?.pp ?? 0) *
                                (5 + merged.pp_ups[i])) /
                                5,
                            )
                      }
                      disabled={!id}
                    />
                    <NumberField
                      label={t("maximumPp")}
                      value={Math.floor(
                        ((catalog.moves[id]?.pp ?? 0) *
                          (5 + merged.pp_ups[i])) /
                          5,
                      )}
                      onChange={() => {}}
                      disabled
                    />
                    <div className="field">
                      <span>{t("ppUps")}</span>
                      <div
                        className="pp-up-control"
                        role="group"
                        aria-label={t("ppUps")}
                      >
                        {[0, 1, 2, 3].map((value) => (
                          <button
                            key={value}
                            type="button"
                            aria-pressed={merged.pp_ups[i] === value}
                            disabled={!id}
                            onClick={() => {
                              if (merged.pp_ups[i] === value) return;
                              const pp_ups = [...merged.pp_ups];
                              pp_ups[i] = value;
                              const pps = [...merged.pps];
                              pps[i] = Math.floor(
                                ((catalog.moves[id]?.pp ?? 0) * (5 + value)) /
                                  5,
                              );
                              setPatch((old) => ({ ...old, pp_ups, pps }));
                            }}
                          >
                            +{value}
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>
                  {id > 0 && (
                    <div
                      className={`source-note ${known.has(id) ? "" : "warning-text"}`}
                    >
                      {known.has(id) ? t("knownSource") : t("unknownSource")}
                    </div>
                  )}
                </div>
              ))}
            </>
          )}
          {tab === "origin" && (
            <>
              <PokemonOrigin
                {...metadataProps}
                origins={
                  detail?.species.id === merged.species ? detail.origins : null
                }
              />
              <button
                type="button"
                className="link-button"
                onClick={() => onReference(merged.species)}
              >
                <BookOpen size={15} />
                {t("encounter")}
              </button>
            </>
          )}
          {tab === "advanced" && (
            <>
              <PokemonAdvanced {...metadataProps} />
              <h3>{t("condition")}</h3>
              <div className="field-grid">
                {["cool", "beauty", "cute", "smart", "tough", "sheen"].map(
                  (key, i) => (
                    <NumberField
                      key={key}
                      label={t(key)}
                      value={merged.condition[i]}
                      onChange={(v) => arrayChange("condition", i, v)}
                    />
                  ),
                )}
              </div>
            </>
          )}
        </div>
        <div className="policy">
          <Toggle label={t("free")} checked={free} onChange={setFree} />
          <p>{t(free ? "freeHelp" : "standardHelp")}</p>
        </div>
      </div>
      <div className="editor-footer">
        <div className="editor-submit">
          <button
            className="primary"
            type="submit"
            disabled={!Object.keys(patch).length}
          >
            {t("apply")}
            {batchLocations.length > 1 ? ` · ${batchLocations.length}` : ""}
          </button>
          {!!Object.keys(patch).length && (
            <button
              type="button"
              onClick={() => {
                setPatch({});
                onDirty(false);
              }}
            >
              {t("cancel")}
            </button>
          )}
        </div>
        <div className="object-actions">
          <button type="button" onClick={() => onTransfer(false)}>
            <MoveRight size={14} />
            {t("moveTo")}
          </button>
          <button type="button" onClick={() => onTransfer(true)}>
            <Copy size={14} />
            {t("clone")}
          </button>
          <button type="button" onClick={onExport}>
            <Download size={14} />
            {t("exportPokemon")}
          </button>
          <button type="button" className="danger" onClick={onDelete}>
            <Trash2 size={14} />
            {t("delete")}
          </button>
        </div>
      </div>
    </form>
  );
}
