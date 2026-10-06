import { useI18n } from "./i18n";
import { locationKey } from "./types";
import type { Catalog, Snapshot } from "./types";

/** Shared stored/simulated selector; choosing a box entry is a query projection. */
export function TrainingIndividualSelect({
  catalog,
  save,
  label,
  value,
  onChange,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  const { t } = useI18n();
  return (
    <label>
      {label}
      <select
        aria-label={label}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="simulated">{t("trainingSimulated")}</option>
        {save?.pokemon
          .filter((p) => !p.pokemon.egg && p.pokemon.species)
          .map((p) => (
            <option
              key={locationKey(p.location)}
              value={locationKey(p.location)}
            >
              {p.pokemon.nickname} ·{" "}
              {catalog.species.find((s) => s.id === p.pokemon.species)?.name} ·{" "}
              {p.location.kind === "party"
                ? `${t("party")} ${p.location.slot + 1}`
                : `${save.boxes[p.location.box_index]?.name} ${p.location.slot + 1}`}
            </option>
          ))}
      </select>
    </label>
  );
}
