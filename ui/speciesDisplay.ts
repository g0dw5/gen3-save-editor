import { configuredReference } from "./configuredReferences";
import { officialStats } from "./officialStats";
import formRules from "./data/species-form-rules.json";
import type { VerifiedFormFamily } from "./speciesForms";
import { formLabel } from "./speciesForms";
import type { Catalog, SpeciesDetail } from "./types";
import type { Key } from "./i18n";

export function speciesFormLabel(
  catalog: Catalog,
  id: number,
  t: (key: Key) => string,
  pid?: number,
  detail?: SpeciesDetail,
) {
  const decision = configuredReference(catalog.profile.md5, id);
  const row =
    decision?.status === "direct"
      ? officialStats.find((row) => row.key === decision.target)
      : undefined;
  return formLabel(catalog, id, t, {
    verified: formRules as VerifiedFormFamily[],
    pid,
    detail,
    approved: decision && row ? { decision, form: row.form } : undefined,
  });
}
export function speciesDisplayName(
  catalog: Catalog,
  id: number,
  t: (key: Key) => string,
  pid?: number,
  detail?: SpeciesDetail,
) {
  const name = catalog.species.find((row) => row.id === id)?.name ?? `#${id}`;
  const form = speciesFormLabel(catalog, id, t, pid, detail);
  return form ? `${name} · ${form}` : name;
}
