import { speciesDisplayName } from "./speciesDisplay";
import { Sprite } from "./components";
import { evolutionLabel } from "./referenceLabels";
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
  const relations = detail.relations ?? {
    species: [id],
    evolutions: detail.evolutions.map((e) => ({ ...e, source: id })),
    battle_forms: detail.battle_forms ?? [],
    form_families: [],
    name_relations: [],
  };
  const node = (value: number) => (
    <button
      type="button"
      className={`evolution-node ${value === id ? "current" : ""}`}
      onClick={() => onNavigate(value)}
      aria-current={value === id ? "true" : undefined}
    >
      <Sprite catalog={catalog} species={value} />
      <span>
        {speciesDisplayName(catalog, value, t, undefined, detail)}
        <small>
          #{value}
          {value === id ? ` · ${t("currentPokemon")}` : ""}
        </small>
      </span>
    </button>
  );
  return (
    <section className="evolution-tree" aria-label={t("evolutionTree")}>
      <h3>{t("evolutionTree")}</h3>
      <p className="small muted">{t("evolutionTreeHelp")}</p>
      {relations.evolutions.length ? (
        <div className="evolution-edges">
          {relations.evolutions.map((edge) => (
            <div
              className="evolution-edge"
              key={`${edge.source}:${edge.offset}`}
            >
              {node(edge.source)}
              <div className="evolution-condition">
                <span aria-hidden="true">→</span>
                {evolutionLabel(edge, catalog, catalog.type_names, t)}
              </div>
              {node(edge.target)}
            </div>
          ))}
        </div>
      ) : (
        <p className="small muted">{t("noEvolutionRecorded")}</p>
      )}
      {!!relations.battle_forms.length && (
        <>
          <h4>{t("battleForms")}</h4>
          <p className="small muted">{t("battleFormsHelp")}</p>
          {relations.battle_forms.map((form) => (
            <div className="evolution-edge battle-edge" key={form.offset}>
              {node(form.source)}
              <div className="evolution-condition">
                <span aria-hidden="true">⇄</span>
                {t(form.kind)} ·{" "}
                {(form.trigger.kind === "held_item"
                  ? catalog.items
                  : catalog.moves
                ).find((entry) => entry.id === form.trigger.id)?.name ??
                  t("unresolved")}
              </div>
              {node(form.target)}
            </div>
          ))}
        </>
      )}
      {!!relations.form_families.filter((family) =>
        family.species.some(
          (member) =>
            !relations.battle_forms.some(
              (form) => form.target === member || form.source === member,
            ),
        ),
      ).length && (
        <>
          <h4>{t("formFamily")}</h4>
          <p className="small muted">{t("formFamilyHelp")}</p>
          {relations.form_families
            .filter((family) =>
              family.species.some(
                (member) =>
                  !relations.battle_forms.some(
                    (form) => form.target === member || form.source === member,
                  ),
              ),
            )
            .map((family) => (
              <div className="form-family-nodes" key={family.offset}>
                {family.species.map((member) => (
                  <div key={member}>{node(member)}</div>
                ))}
              </div>
            ))}
        </>
      )}
      {!!relations.name_relations.length && (
        <>
          <h4>{t("relatedNames")}</h4>
          <p className="small muted">{t("relatedNamesHelp")}</p>
          {relations.name_relations.map((edge) => (
            <div
              className="evolution-edge name-edge"
              key={`${edge.source}:${edge.target}`}
            >
              {node(edge.source)}
              <div className="evolution-condition">
                <span aria-hidden="true">⋯</span>
                {t("nameLinkOnly")}
              </div>
              {node(edge.target)}
            </div>
          ))}
        </>
      )}
    </section>
  );
}
