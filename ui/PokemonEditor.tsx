import { useEffect, useMemo, useState } from "react";
import {
  BookOpen,
  Download,
  Trash2,
  Copy,
  MoveRight,
  Sparkles,
} from "lucide-react";
import { api } from "./api";
import { useI18n, natures, statKeys } from "./i18n";
import { NumberField, SelectField, Sprite, Toggle, Types } from "./components";
import type {
  Catalog,
  Location,
  Pokemon,
  SpeciesDetail,
  StoredPokemon,
} from "./types";

interface Props {
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
  const [tab, setTab] = useState("overview");
  const [patch, setPatch] = useState<Record<string, unknown>>({});
  const [detail, setDetail] = useState<SpeciesDetail | null>(null);
  const [allMoves, setAllMoves] = useState(false);
  const merged = { ...p, ...patch } as Pokemon;
  const species = catalog.species.find((s) => s.id === merged.species)!;
  const change = (key: string, value: unknown) =>
    setPatch((old) => ({ ...old, [key]: value }));
  useEffect(() => {
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
      .map((m) => ({
        value: m.id,
        label: `${m.id ? m.name : t("emptyMove")}${m.id ? ` · #${m.id}` : ""}${m.id && !known.has(m.id) ? ` — ${future.has(m.id) ? t("futureMove") : t("unknownSource")}` : ""}`,
      }));
  const itemOptions = catalog.items.map((i) => ({
    value: i.id,
    label: i.id
      ? `${i.name}${i.tm_move ? ` · ${catalog.moves[i.tm_move]?.name ?? ""}` : ""} #${i.id}`
      : t("emptyMove"),
  }));
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
      <div className="editor-hero">
        <Sprite
          catalog={catalog}
          species={merged.species}
          shiny={merged.shiny}
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
        {["overview", "stats", "moves", "origin", "advanced"].map((key) => (
          <button
            type="button"
            key={key}
            className={tab === key ? "active" : ""}
            onClick={() => setTab(key)}
          >
            {t(key)}
          </button>
        ))}
      </div>
      <div className="editor-fields">
        {tab === "overview" && (
          <>
            <SelectField
              label={t("species")}
              value={merged.species}
              onChange={(v) => change("species", +v)}
              options={catalog.species
                .filter((s) => s.stats[0] > 0)
                .map((s) => ({ value: s.id, label: `${s.name} #${s.id}` }))}
            />
            <label className="field">
              <span>{t("nickname")}</span>
              <input
                value={merged.nickname}
                onChange={(e) => change("nickname", e.target.value)}
              />
            </label>
            <div className="field-grid">
              {num("level", 100, 1)}
              {num("experience", 0xffffffff)}
              <SelectField
                label={t("nature")}
                value={merged.nature}
                onChange={(v) => change("nature", +v)}
                options={natures[locale].map((label, value) => ({
                  value,
                  label,
                }))}
              />
              <SelectField
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
                    <th>{t(key)}</th>
                    <td>
                      <input
                        type="number"
                        aria-label={`${t(key)} ${t("ivs")}`}
                        min="0"
                        max="31"
                        required
                        value={merged.ivs[i]}
                        onChange={(e) => arrayChange("ivs", i, +e.target.value)}
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
                        onChange={(e) => arrayChange("evs", i, +e.target.value)}
                      />
                    </td>
                    <td>{p.stats[i]}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            <div
              className={`ev-total ${merged.evs.reduce((a, b) => a + b, 0) > 510 ? "warning-text" : ""}`}
            >
              {t("evs")} · {t("total")} {merged.evs.reduce((a, b) => a + b, 0)}{" "}
              / 510
            </div>
            {p.current_hp !== null && (
              <div className="field-grid">
                {num("current_hp", 65535)}
                {num("status", 255)}
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
            {merged.moves.map((id, i) => (
              <div className="move-card" key={i}>
                <SelectField
                  label={`${t("move")} ${i + 1}`}
                  value={id}
                  onChange={(v) => moveChange(i, +v)}
                  options={moveOptions(id)}
                />
                <div className="field-grid">
                  <NumberField
                    label={t("pp")}
                    value={merged.pps[i]}
                    onChange={(v) => arrayChange("pps", i, v)}
                    max={255}
                  />
                  <NumberField
                    label={t("ppUps")}
                    value={merged.pp_ups[i]}
                    onChange={(v) => {
                      const pp_ups = [...merged.pp_ups];
                      pp_ups[i] = v;
                      const pps = [...merged.pps];
                      pps[i] = Math.floor(
                        ((catalog.moves[id]?.pp ?? 0) * (5 + v)) / 5,
                      );
                      setPatch((old) => ({ ...old, pp_ups, pps }));
                    }}
                    max={3}
                  />
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
            <label className="field">
              <span>{t("ot_name")}</span>
              <input
                value={merged.ot_name}
                onChange={(e) => change("ot_name", e.target.value)}
              />
            </label>
            <div className="field-grid">
              {num("ot_id", 0xffffffff)}
              {num("ot_gender", 1)}
              {num("met_location")}
              {num("met_level", 127)}
              {num("origin_game", 15)}
              {num("ball", 15)}
              {num("language", 7, 1)}
            </div>
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
            <div className="field-grid">
              {num("pid", 0xffffffff)}
              {num("markings", 15)}
              {num("pokerus")}
              {num("ribbons", 0xffffffff)}
            </div>
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
    </form>
  );
}
