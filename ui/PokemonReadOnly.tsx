import { Sprite, Types } from "./components";
import { natures, statKeys, useI18n } from "./i18n";
import type { Catalog, StoredPokemon } from "./types";

/** Only verified fields are presented; unsupported ribbon/origin schemas stay hidden. */
export function PokemonReadOnly({
  row,
  catalog,
  onReference,
}: {
  row: StoredPokemon;
  catalog: Catalog;
  onReference: (id: number) => void;
}) {
  const { t, locale } = useI18n();
  const p = row.pokemon;
  const species = catalog.species.find((s) => s.id === p.species);
  return (
    <div className="readonly-pokemon">
      <span className="eyebrow">{t("readOnly")}</span>
      <div className="editor-hero">
        <Sprite catalog={catalog} species={p.species} shiny={p.shiny} large />
        <div>
          <h2>{p.nickname || species?.name}</h2>
          <p>
            {species?.name} · Lv. {p.level}
          </p>
          <Types values={species?.types ?? []} />
        </div>
      </div>
      <p className="small muted">{t("representativeSprite")}</p>
      <div className="detail-pairs">
        <span>{t("nature")} · PID</span>
        <strong>{natures[locale][p.nature]}</strong>
        <span>{t("effectiveNature")}</span>
        <strong>{natures[locale][p.effective_nature ?? p.nature]}</strong>
        <span>{t("ability")}</span>
        <strong>
          {catalog.abilities.find((a) => a.id === p.ability_id)?.name ||
            `#${p.ability_id}`}
        </strong>
        <span>{t("held_item")}</span>
        <strong>{catalog.items[p.held_item]?.name || "—"}</strong>
        <span>{t("friendship")}</span>
        <strong>{p.friendship}</strong>
        <span>{t("ball")}</span>
        <strong>#{p.ball}</strong>
        <span>{t("ot_name")}</span>
        <strong>{p.ot_name}</strong>
      </div>
      <table className="data-table">
        <thead>
          <tr>
            <th>{t("stats")}</th>
            <th>{t("value")}</th>
            <th>IV</th>
            <th>EV</th>
          </tr>
        </thead>
        <tbody>
          {statKeys.map((key, i) => (
            <tr key={key}>
              <td>{t(key)}</td>
              <td>{p.stats[i]}</td>
              <td>{p.ivs[i]}</td>
              <td>{p.evs[i]}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <h3>{t("moves")}</h3>
      {p.moves.map((id, i) => {
        const move = catalog.moves[id];
        const max = Math.floor(((move?.pp ?? 0) * (5 + p.pp_ups[i])) / 5);
        return id ? (
          <div className="reference-line" key={i}>
            <span>{move?.name ?? `#${id}`}</span>
            <span>
              PP {p.pps[i]} / {max}
            </span>
          </div>
        ) : null;
      })}
      <button onClick={() => onReference(p.species)}>{t("references")}</button>
    </div>
  );
}
