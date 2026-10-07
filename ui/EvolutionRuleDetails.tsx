import { evolutionLabel } from "./referenceLabels";
import { speciesDisplayName } from "./speciesDisplay";
import { useI18n } from "./i18n";
import type {
  Catalog,
  GameMap,
  MapFocus,
  QueryTarget,
  SpeciesDetail,
} from "./types";

type Rule = SpeciesDetail["evolutions"][number];

/** Only decoded positive location requirements select maps. Raw parameters do not. */
export function evolutionLocationMaps(rule: Rule, maps: GameMap[]) {
  const requirements = [...(rule.requirements ?? [])];
  if (rule.condition === "region")
    requirements.push({ kind: "region", value: rule.parameter });
  const hasLocation = requirements.some((r) =>
    ["map", "region"].includes(r.kind),
  );
  return {
    hasLocation,
    maps: hasLocation
      ? maps.filter((map) =>
          requirements.every((r) =>
            r.kind === "map"
              ? map.id === `${r.value >>> 8}-${r.value & 255}`
              : r.kind === "region"
                ? map.region === r.value
                : r.kind === "outside_region"
                  ? map.region !== r.value
                  : true,
          ),
        )
      : [],
  };
}

export function EvolutionRuleDetails({
  rule,
  catalog,
  maps = [],
  related = [],
  onTarget,
  onMap,
}: {
  rule: Rule;
  catalog: Catalog;
  maps?: GameMap[];
  related?: QueryTarget[];
  onTarget: (target: QueryTarget) => void;
  onMap?: (id: string, focus?: MapFocus) => void;
}) {
  const { t } = useI18n();
  const locations = evolutionLocationMaps(rule, maps);
  const name = (target: QueryTarget) =>
    target.kind === "species"
      ? speciesDisplayName(catalog, target.id)
      : ((target.kind === "item" ? catalog.items : catalog.moves).find(
          (row) => row.id === target.id,
        )?.name ?? `#${target.id}`);
  return (
    <div className="evolution-rule-details">
      <span>{evolutionLabel(rule, catalog, catalog.type_names, t)}</span>
      {!!related.length && (
        <div className="source-related">
          <small>{t("evolutionRequirements")}</small>
          {related.map((target) => (
            <button
              className="link-button"
              key={`${target.kind}:${target.id}`}
              onClick={() => onTarget(target)}
            >
              {name(target)} ↗
            </button>
          ))}
        </div>
      )}
      {locations.hasLocation && (
        <details className="evolution-locations">
          <summary>
            {t("evolutionMatchingMaps")} · {locations.maps.length}
          </summary>
          <p className="small muted">{t("evolutionLocationHelp")}</p>
          {!locations.maps.length && (
            <p className="small muted">{t("evolutionNoMatchingMap")}</p>
          )}
          <div className="evolution-location-options">
            {locations.maps.map((map) => (
              <div key={map.id}>
                <button
                  className="link-button"
                  disabled={!onMap}
                  onClick={() => onMap?.(map.id)}
                >
                  {map.name || t("unresolved")} · {map.id} ↗
                </button>
              </div>
            ))}
          </div>
        </details>
      )}
    </div>
  );
}
