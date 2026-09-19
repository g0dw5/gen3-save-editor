import { Sprite, Types } from "./components";
import { natures, statKeys, typeNames, useI18n } from "./i18n";
import { hiddenPower } from "./hiddenPower";
import type { Catalog, Opponent, Snapshot } from "./types";

export function TrainerParty({
  trainer,
  catalog,
  save,
  onSpecies,
  onAbility,
}: {
  trainer: Opponent;
  catalog: Catalog;
  save: Snapshot | null;
  onSpecies: (id: number) => void;
  onAbility: (id: number) => void;
}) {
  const { t, locale } = useI18n();
  const party = save?.pokemon.filter((p) => p.location.kind === "party") ?? [];
  const highestLevel = party.length
    ? Math.max(...party.map((p) => p.pokemon.level))
    : null;
  return (
    <div className="trainer-party">
      <p className="small muted">{t("trainerGenerationHelp")}</p>
      {trainer.party.map((p, i) => {
        const species = catalog.species.find((s) => s.id === p.species);
        const g = p.generation;
        const hpRules = catalog.profile.hidden_power;
        const hp = hiddenPower(hpRules, g?.ivs);
        const dynamic = p.level_rule === "party_max";
        const level = dynamic ? highestLevel : p.level;
        const gender = g?.gender;
        return (
          <article className="trainer-mon-card" key={i}>
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
                    {level === null ? t("dynamicLevel") : `Lv. ${level}`}
                  </strong>
                </div>
                {species && <Types values={species.types} />}
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
                    ? (g.ability_options ?? [g.ability_id]).map((id) => (
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
                </dd>
              </div>
              <div>
                <dt>{t("nature")}</dt>
                <dd>{g ? natures[locale][g.nature] : t("unresolved")}</dd>
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
                {dynamic && !p.moves_explicit
                  ? t("dynamicMoves")
                  : p.moves.filter(Boolean).map((id) => (
                      <span key={id}>
                        {catalog.moves[id]?.name ?? id}
                        {id === hpRules?.move_id && (
                          <>
                            {" "}
                            ·{" "}
                            {hp
                              ? `${typeNames[locale][hp.type]} · ${t("power")} ${hp.power}`
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
                    <td key={key}>{g?.ivs?.[s] ?? "?"}</td>
                  ))}
                </tr>
                <tr>
                  <th>{t("evs")}</th>
                  {statKeys.map((key, s) => (
                    <td key={key}>{g?.evs[s] ?? "?"}</td>
                  ))}
                </tr>
              </tbody>
            </table>
            {g && !g.ivs && <p className="small muted">{t("randomIVs")}</p>}
            <details className="trainer-evidence">
              <summary>{t("rawParameters")}</summary>
              <p className="small muted">{t("ivQualityHelp")}</p>
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
