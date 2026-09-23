import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { Sprite, Types } from "./components";
import { statKeys, useI18n } from "./i18n";
import { hiddenPower } from "./hiddenPower";
import type {
  Catalog,
  Opponent,
  Snapshot,
  TrainerDifficulty,
  TrainerBattlePreview,
  TrainerEvPreview,
} from "./types";

type BattleMon = {
  species: number;
  held_item: number;
  ability_slot: number;
  nature: number;
  speed: number;
  current_hp: number;
};
type ManualMon = BattleMon & { speciesText: string };
const newManualMon = (): ManualMon => ({
  species: 0,
  speciesText: "",
  held_item: 0,
  ability_slot: 0,
  nature: 0,
  speed: 0,
  current_hp: 1,
});

function speciesId(value: string, catalog: Catalog): number {
  const number = Number(value.match(/#(\d+)/)?.[1] ?? value);
  if (
    Number.isInteger(number) &&
    number > 0 &&
    catalog.species.some((s) => s.id === number)
  )
    return number;
  return catalog.species.find((s) => s.name === value)?.id ?? 0;
}

export function TrainerParty({
  trainer,
  catalog,
  save,
  difficulty,
  onSpecies,
  onAbility,
}: {
  trainer: Opponent;
  catalog: Catalog;
  save: Snapshot | null;
  difficulty: TrainerDifficulty | null;
  onSpecies: (id: number) => void;
  onAbility: (id: number) => void;
}) {
  const { t } = useI18n();
  const party = save?.pokemon.filter((p) => p.location.kind === "party") ?? [];
  const [manual, setManual] = useState<ManualMon[]>([newManualMon()]);
  const [source, setSource] = useState<"save" | "manual">("save");
  const [levels, setLevels] = useState<number[]>([]);
  const [manualMaxLevel, setManualMaxLevel] = useState<number>(0);
  const [battlePreview, setBattlePreview] =
    useState<TrainerBattlePreview | null>(null);
  const [battleError, setBattleError] = useState("");
  const [preview, setPreview] = useState<TrainerEvPreview | null>(null);
  const [previewError, setPreviewError] = useState("");
  const [loading, setLoading] = useState(false);
  useEffect(() => setLevels([]), [trainer.id]);
  const hasTemplate = trainer.party.some(
    (p) => p.generation?.context === "ultimate_template",
  );
  const savedParty = useMemo<BattleMon[]>(
    () =>
      (save?.pokemon ?? [])
        .filter((p) => p.location.kind === "party" && !p.pokemon.egg)
        .sort((a, b) => a.location.slot - b.location.slot)
        .map(({ pokemon: p }) => ({
          species: p.species,
          held_item: p.held_item,
          ability_slot: p.ability_slot,
          nature: p.effective_nature ?? p.nature,
          speed: p.stats[3],
          current_hp: p.current_hp ?? 0,
        })),
    [save],
  );
  const useSave = savedParty.length > 0 && (!hasTemplate || source === "save");
  const saveMaxLevel = useMemo(() => {
    const levels = (save?.pokemon ?? [])
      .filter((p) => p.location.kind === "party" && !p.pokemon.egg)
      .map((p) => p.pokemon.level);
    return levels.length ? Math.max(...levels) : null;
  }, [save]);
  const playerMaxLevel = useSave
    ? saveMaxLevel
    : manualMaxLevel > 0
      ? manualMaxLevel
      : null;
  const needsMaxLevel = trainer.party.some(
    (p) =>
      p.level === 0 ||
      (difficulty === 4 &&
        p.level > 1 &&
        p.level < (catalog.profile.max_level ?? 100)),
  );
  const battleKey = JSON.stringify({
    trainer_id: trainer.id,
    difficulty,
    player_max_level: playerMaxLevel,
  });
  useEffect(() => {
    if (difficulty === null) {
      setBattlePreview(null);
      setBattleError("");
      return;
    }
    let alive = true;
    setBattlePreview(null);
    setBattleError("");
    const timer = window.setTimeout(() => {
      api<TrainerBattlePreview>("trainer_battle_preview", JSON.parse(battleKey))
        .then((result) => {
          if (alive) setBattlePreview(result);
        })
        .catch((error) => {
          if (alive)
            setBattleError(
              `${error?.code ?? "error"}: ${error?.detail ?? error}`,
            );
        });
    }, 120);
    return () => {
      alive = false;
      window.clearTimeout(timer);
    };
  }, [difficulty, battleKey]);
  const currentBattle =
    battlePreview?.trainer_id === trainer.id &&
    battlePreview.difficulty === difficulty &&
    battlePreview.player_max_level === playerMaxLevel
      ? battlePreview
      : null;
  const playerParty = useSave
    ? savedParty
    : manual.map((m) => ({
        species: speciesId(m.speciesText, catalog),
        held_item: m.held_item,
        ability_slot: m.ability_slot,
        nature: m.nature,
        speed: m.speed,
        current_hp: m.current_hp,
      }));
  const scenario =
    playerParty.length > 0 &&
    playerParty.every(
      (p) => p.species > 0 && p.speed > 0 && p.nature >= 0 && p.nature < 25,
    );
  const scenarioKey = JSON.stringify({
    trainer_id: trainer.id,
    difficulty,
    player_party: playerParty,
    opponent_levels: trainer.party.map(
      (p, i) => levels[i] || currentBattle?.mons[i]?.level || p.level,
    ),
  });
  useEffect(() => {
    if (
      difficulty === null ||
      !hasTemplate ||
      !scenario ||
      !currentBattle ||
      trainer.party.some(
        (p, i) =>
          p.generation?.context === "ultimate_template" &&
          !(levels[i] || currentBattle.mons[i]?.level),
      )
    ) {
      setPreview(null);
      setLoading(false);
      setPreviewError("");
      return;
    }
    let alive = true;
    setLoading(true);
    setPreview(null);
    setPreviewError("");
    const timer = window.setTimeout(() => {
      api<TrainerEvPreview>("trainer_ev_preview", JSON.parse(scenarioKey))
        .then((result) => {
          if (alive) setPreview(result);
        })
        .catch((error) => {
          if (alive) {
            setPreview(null);
            setPreviewError(
              `${error?.code ?? "error"}: ${error?.detail ?? error}`,
            );
          }
        })
        .finally(() => {
          if (alive) setLoading(false);
        });
    }, 180);
    return () => {
      alive = false;
      window.clearTimeout(timer);
    };
  }, [
    difficulty,
    hasTemplate,
    scenario,
    scenarioKey,
    currentBattle,
    trainer.party,
    levels,
  ]);
  const editManual = (index: number, change: Partial<ManualMon>) =>
    setManual((old) =>
      old.map((m, i) => (i === index ? { ...m, ...change } : m)),
    );
  const highestLevel = party.length
    ? Math.max(...party.map((p) => p.pokemon.level))
    : null;
  return (
    <div className="trainer-party">
      {difficulty !== null && (
        <div className="trainer-mode-note">
          <strong>
            {t("trainerPartyInMode").replace(
              "{mode}",
              t(`trainerDifficulty_${difficulty}`),
            )}
          </strong>
          <p>{t(`trainerDifficultyHelp_${difficulty}`)}</p>
          <p>{t("trainerDifficultyPartySource")}</p>
          {!hasTemplate && (
            <p className="small">{t("trainerRomOnlyPreview")}</p>
          )}
          {needsMaxLevel && !useSave && (
            <label className="trainer-ev-level">
              {t("trainerPlayerMaxLevel")}
              <input
                type="number"
                min="1"
                max={catalog.profile.max_level ?? 100}
                value={manualMaxLevel || ""}
                onChange={(e) => setManualMaxLevel(Number(e.target.value))}
              />
            </label>
          )}
          {needsMaxLevel && playerMaxLevel === null && (
            <p className="small muted">{t("trainerMaxLevelNeeded")}</p>
          )}
          {battleError && (
            <p className="small" role="alert">
              {battleError}
            </p>
          )}
          {hasTemplate && (
            <div className="trainer-ev-scenario">
              <strong>{t("trainerEvScenario")}</strong>
              {savedParty.length > 0 && (
                <div className="trainer-ev-source">
                  <label>
                    <input
                      type="radio"
                      checked={useSave}
                      onChange={() => setSource("save")}
                    />
                    {t("trainerEvCurrentParty")}
                  </label>
                  <label>
                    <input
                      type="radio"
                      checked={!useSave}
                      onChange={() => setSource("manual")}
                    />
                    {t("trainerEvManualParty")}
                  </label>
                </div>
              )}
              {useSave ? (
                <p className="small">
                  {savedParty
                    .map(
                      (m) =>
                        catalog.species.find((s) => s.id === m.species)?.name ??
                        `#${m.species}`,
                    )
                    .join(" · ")}
                </p>
              ) : (
                <>
                  <p className="small muted">{t("trainerEvManualHelp")}</p>
                  {manual.map((m, i) => (
                    <div className="trainer-ev-manual-row" key={i}>
                      <input
                        list="trainer-ev-species"
                        aria-label={`${t("pokemon")} ${i + 1}`}
                        placeholder={t("pokemon")}
                        value={m.speciesText}
                        onChange={(e) =>
                          editManual(i, { speciesText: e.target.value })
                        }
                      />
                      <label>
                        {t("speed")}
                        <input
                          type="number"
                          min="1"
                          max="9999"
                          value={m.speed || ""}
                          onChange={(e) =>
                            editManual(i, { speed: Number(e.target.value) })
                          }
                        />
                      </label>
                      <label>
                        {t("nature")}
                        <select
                          value={m.nature}
                          onChange={(e) =>
                            editManual(i, { nature: Number(e.target.value) })
                          }
                        >
                          {catalog.natures.map((n) => (
                            <option value={n.id} key={n.id}>
                              {n.name}
                            </option>
                          ))}
                        </select>
                      </label>
                      <label>
                        {t("held_item")}
                        <select
                          value={m.held_item}
                          onChange={(e) =>
                            editManual(i, { held_item: Number(e.target.value) })
                          }
                        >
                          <option value="0">{t("noHeldItem")}</option>
                          {catalog.items
                            .filter((item) => item.id && item.name)
                            .map((item) => (
                              <option value={item.id} key={item.id}>
                                {item.name}
                              </option>
                            ))}
                        </select>
                      </label>
                      <label>
                        {t("trainerEvAbilitySlot")}
                        <select
                          value={m.ability_slot}
                          onChange={(e) =>
                            editManual(i, {
                              ability_slot: Number(e.target.value),
                            })
                          }
                        >
                          <option value="0">1</option>
                          <option value="1">2</option>
                          <option value="2">3</option>
                        </select>
                      </label>
                      <label>
                        {t("trainerEvCurrentHp")}
                        <input
                          type="number"
                          min="0"
                          max="9999"
                          value={m.current_hp}
                          onChange={(e) =>
                            editManual(i, {
                              current_hp: Number(e.target.value),
                            })
                          }
                        />
                      </label>
                      <button
                        type="button"
                        disabled={manual.length === 1}
                        onClick={() =>
                          setManual((old) => old.filter((_, j) => j !== i))
                        }
                      >
                        −
                      </button>
                    </div>
                  ))}
                  <datalist id="trainer-ev-species">
                    {catalog.species
                      .filter((s) => s.id && s.name)
                      .map((s) => (
                        <option value={`${s.name} (#${s.id})`} key={s.id} />
                      ))}
                  </datalist>
                  <button
                    type="button"
                    disabled={manual.length >= 6}
                    onClick={() => setManual((old) => [...old, newManualMon()])}
                  >
                    {t("trainerEvAddMon")}
                  </button>
                </>
              )}
              {!scenario && (
                <p className="small muted">{t("trainerEvNeedsParty")}</p>
              )}
              {loading && (
                <p className="small muted">{t("trainerEvCalculating")}</p>
              )}
              {previewError && (
                <p className="small" role="alert">
                  {previewError}
                </p>
              )}
              {preview && <p className="small">{t("trainerEvComputed")}</p>}
            </div>
          )}
        </div>
      )}
      <p className="small muted">{t("trainerGenerationHelp")}</p>
      {trainer.party.map((p, i) => {
        const species = catalog.species.find((s) => s.id === p.species);
        const g = p.generation;
        const template = g?.context === "ultimate_template";
        const simulated =
          template &&
          preview?.trainer_id === trainer.id &&
          preview.difficulty === difficulty
            ? preview.mons[i]
            : null;
        const battleMon = currentBattle?.mons[i];
        const resolvedIvs = simulated?.ivs ?? battleMon?.ivs ?? g?.ivs;
        const resolvedEvs = simulated?.evs ?? battleMon?.evs ?? g?.evs;
        const hpRules = catalog.profile.hidden_power;
        const hp = hiddenPower(hpRules, resolvedIvs);
        const hpAlternate = hiddenPower(hpRules, simulated?.alternate_ivs);
        const dynamic = p.level_rule === "party_max";
        const level = battleMon
          ? battleMon.level
          : dynamic
            ? highestLevel
            : p.level;
        const moveIds = battleMon?.moves ?? p.moves;
        const gender = g?.gender;
        const abilities = [
          ...new Set(g?.ability_options ?? (g ? [g.ability_id] : [])),
        ];
        return (
          <article className="trainer-mon-card" key={i}>
            {template && (
              <p className="small muted">{t("ultimateTrainerTemplate")}</p>
            )}
            <div className="trainer-mon-heading">
              <Sprite catalog={catalog} species={p.species} />
              <div>
                <div className="trainer-mon-title">
                  <button
                    className="link-button"
                    onClick={() => onSpecies(p.species)}
                  >
                    {species?.name ?? `#${p.species}`}
                  </button>
                  <span className={`gender-badge ${gender ?? ""}`}>
                    {t("gender")} · {gender ? t(gender) : t("unresolved")}
                  </span>
                  <strong>
                    {battleMon && level === null
                      ? t(
                          battleMon.level_source === "needs_player_max"
                            ? "trainerMaxLevelNeeded"
                            : "dynamicLevel",
                        )
                      : level === null
                        ? t("dynamicLevel")
                        : p.level_rule === "difficulty"
                          ? battleMon
                            ? `Lv. ${level}`
                            : `${t("trainerBaseLevel")} ${level}`
                          : `Lv. ${level}`}
                  </strong>
                </div>
                {species && <Types catalog={catalog} values={species.types} />}
                {p.level_rule === "difficulty" && (
                  <p className="small muted">
                    {battleMon
                      ? t(`trainerLevelSource_${battleMon.level_source}`)
                      : t("ultimateTrainerLevel")}
                  </p>
                )}
                {template && (
                  <label className="trainer-ev-level">
                    {t("trainerEvLevel")}
                    <input
                      type="number"
                      min="1"
                      max={catalog.profile.max_level ?? 100}
                      value={levels[i] || p.level || ""}
                      onChange={(e) =>
                        setLevels((old) => {
                          const next = [...old];
                          next[i] = Number(e.target.value);
                          return next;
                        })
                      }
                    />
                  </label>
                )}
                {dynamic && (
                  <div className="small muted">
                    {t(
                      highestLevel === null
                        ? "partyMaxNoSave"
                        : "partyMaxWithSave",
                    )}
                  </div>
                )}
              </div>
            </div>
            <dl className="trainer-mon-facts">
              <div>
                <dt>{t("ability")}</dt>
                <dd>
                  {g
                    ? abilities.map((id) => (
                        <button
                          key={id}
                          className="link-button"
                          onClick={() => onAbility(id)}
                          title={catalog.abilities[id]?.description}
                        >
                          {catalog.abilities[id]?.name ?? `#${id}`} ↗
                        </button>
                      ))
                    : t("unresolved")}
                  {g && (
                    <span className="small muted">
                      {t(
                        abilities.length > 1 ? "abilityChoice" : "abilityFixed",
                      )}
                    </span>
                  )}
                </dd>
              </div>
              <div>
                <dt>{t("nature")}</dt>
                <dd>{g ? catalog.natures[g.nature]?.name : t("unresolved")}</dd>
              </div>
              <div>
                <dt>{t("held_item")}</dt>
                <dd>
                  {p.held_item
                    ? (catalog.items[p.held_item]?.name ?? p.held_item)
                    : t("noHeldItem")}
                </dd>
              </div>
            </dl>
            <div className="trainer-moves">
              <span className="small muted">
                {t("moves")} ·{" "}
                {t(p.moves_explicit ? "explicitMoves" : "levelSource")}
              </span>
              <div>
                {(dynamic || (battleMon && battleMon.moves === null)) &&
                !p.moves_explicit
                  ? t("dynamicMoves")
                  : moveIds.filter(Boolean).map((id) => (
                      <span key={id}>
                        {catalog.moves[id]?.name ?? id}
                        {id === hpRules?.move_id && (
                          <>
                            {" "}
                            ·{" "}
                            {hp
                              ? `${catalog.type_names[hp.type]} · ${t("power")} ${hp.power}${hpAlternate && (hpAlternate.type !== hp.type || hpAlternate.power !== hp.power) ? ` / ${catalog.type_names[hpAlternate.type]} · ${t("power")} ${hpAlternate.power}` : ""}`
                              : t("hiddenPowerUnknown")}
                          </>
                        )}
                      </span>
                    ))}
              </div>
            </div>
            <table className="trainer-stat-table">
              <thead>
                <tr>
                  <th>{t("stat")}</th>
                  {statKeys.map((key) => (
                    <th key={key}>{t(key)}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                <tr>
                  <th>{t("ivs")}</th>
                  {statKeys.map((key, s) => (
                    <td key={key}>
                      {resolvedIvs?.[s] ?? "?"}
                      {simulated?.alternate_ivs &&
                      simulated.alternate_ivs[s] !== simulated.ivs?.[s]
                        ? ` / ${simulated.alternate_ivs[s]}`
                        : ""}
                    </td>
                  ))}
                </tr>
                <tr>
                  <th>{t("evs")}</th>
                  {statKeys.map((key, s) => (
                    <td key={key}>
                      {resolvedEvs?.[s] ?? "?"}
                      {simulated?.alternate_evs &&
                      simulated.alternate_evs[s] !== simulated.evs?.[s]
                        ? ` / ${simulated.alternate_evs[s]}`
                        : ""}
                    </td>
                  ))}
                </tr>
              </tbody>
            </table>
            {simulated?.evs && (
              <p className="small muted">
                {t("trainerEvTotal")}:{" "}
                {simulated.evs.reduce((a, b) => a + b, 0)}
                {(simulated.alternate_evs || simulated.alternate_ivs) &&
                  ` · ${t("trainerEvAlternate")}`}
              </p>
            )}
            {template && (
              <p className="small muted">
                {t("ultimateTrainerEvs").replace("{n}", String(g.ev_increment))}
              </p>
            )}
            {g && !g.ivs && <p className="small muted">{t("randomIVs")}</p>}
            <details className="trainer-evidence">
              <summary>{t("rawParameters")}</summary>
              <p className="small muted">
                {t(
                  template
                    ? "ultimateTemplateEvidence"
                    : g?.context === "ultimate_plain"
                      ? "ultimatePlainEvidence"
                      : "ivQualityHelp",
                )}
              </p>
              <pre>
                {JSON.stringify(
                  {
                    offset: `0x${p.offset.toString(16).toUpperCase()}`,
                    raw_level: p.level,
                    iv_quality: p.iv_quality,
                    personality_parameter: g?.personality_parameter,
                    level_rule: p.level_rule,
                  },
                  null,
                  2,
                )}
              </pre>
            </details>
          </article>
        );
      })}
    </div>
  );
}
