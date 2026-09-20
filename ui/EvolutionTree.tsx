import { speciesDisplayName } from "./speciesDisplay";
import { Sprite } from "./components";
import { evolutionLabel } from "./referenceLabels";
import { evolutionGraph } from "./evolutionGraph";
import { useI18n } from "./i18n";
import type { Catalog, SpeciesDetail } from "./types";

export function EvolutionTree({
  detail,
  catalog,
  onNavigate,
}: {
  detail: SpeciesDetail;
  catalog: Catalog;
  onNavigate: (id: number) => void;
}) {
  const { t } = useI18n();
  const id = detail.species.id;
  const graph = evolutionGraph(detail);
  const label = (value: number) =>
    speciesDisplayName(catalog, value, t, undefined, detail);
  const sourceLink = (value: number, arrow = "→") => (
    <button className="link-button" onClick={() => onNavigate(value)}>
      {label(value)} {arrow}
    </button>
  );
  const card = (value: number) => (
    <article className="evolution-card" key={value} data-species={value}>
      <button
        type="button"
        className={`evolution-node ${value === id ? "current" : ""}`}
        onClick={() => onNavigate(value)}
        aria-current={value === id ? "true" : undefined}
      >
        <Sprite catalog={catalog} species={value} />
        <span>
          {label(value)}
          <small>
            #{value}
            {value === id ? ` · ${t("currentPokemon")}` : ""}
          </small>
        </span>
      </button>
      {graph.evolutions
        .filter((e) => e.target === value)
        .map((edge) => (
          <div
            className="evolution-condition"
            key={`e:${edge.source}:${edge.method}:${edge.parameter}`}
          >
            {sourceLink(edge.source)}
            <span>{evolutionLabel(edge, catalog, catalog.type_names, t)}</span>
          </div>
        ))}
      {graph.battles
        .filter((e) => e.target === value)
        .map((form) => (
          <div
            className="evolution-condition"
            key={`b:${form.source}:${form.kind}:${form.trigger.kind}:${form.trigger.id}`}
          >
            {sourceLink(form.source)}
            <span>
              {t(form.kind)} ·{" "}
              {(form.trigger.kind === "held_item"
                ? catalog.items
                : catalog.moves
              ).find((entry) => entry.id === form.trigger.id)?.name ??
                t("unresolved")}
            </span>
          </div>
        ))}
      {graph.names
        .filter((e) => e.target === value)
        .map((edge) => (
          <div className="evolution-condition" key={`n:${edge.source}`}>
            {sourceLink(edge.source, "⋯")}
            <span>{t("nameLinkOnly")}</span>
          </div>
        ))}
    </article>
  );
  return (
    <section className="evolution-tree" aria-label={t("evolutionTree")}>
      <h3>{t("evolutionTree")}</h3>
      <p className="small muted">{t("evolutionTreeHelp")}</p>
      {!graph.evolutions.length && (
        <p className="small muted">{t("noEvolutionRecorded")}</p>
      )}
      <div className="evolution-cards">{graph.main.map(card)}</div>
      {!!graph.battleIds.length && (
        <>
          <h4>{t("battleForms")}</h4>
          <p className="small muted">{t("battleFormsHelp")}</p>
          <div className="evolution-cards">{graph.battleIds.map(card)}</div>
        </>
      )}
      {graph.families.map((family) => (
        <details
          className="evolution-forms"
          key={`${family.offset}:${id}`}
          open={family.members.includes(id)}
        >
          <summary>
            {t("formFamily")} · {label(family.species[0])} ·{" "}
            {family.members.length}
          </summary>
          <p className="small muted">{t("formFamilyHelp")}</p>
          <div className="evolution-cards">{family.members.map(card)}</div>
        </details>
      ))}
      {!!graph.nameIds.length && (
        <>
          <h4>{t("relatedNames")}</h4>
          <p className="small muted">{t("relatedNamesHelp")}</p>
          <div className="evolution-cards name-edge">
            {graph.nameIds.map(card)}
          </div>
        </>
      )}
    </section>
  );
}
