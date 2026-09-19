import { resolveMapping, type MappingConfig } from "./speciesMappings";

const files = import.meta.glob<MappingConfig>(
  "./data/species-mappings/*.json",
  {
    eager: true,
    import: "default",
  },
);
// Configurations are isolated by the exact ROM fingerprint.
const configs = Object.values(files);
export function configuredReference(md5: string, species: number) {
  return resolveMapping(configs, md5, species);
}
