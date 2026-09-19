/** Game-scoped identity decisions. Pending proposals never change comparisons. */
export interface MappingEntry {
  status: "pending" | "direct" | "comparison" | "none";
  target: string | null;
}
export interface MappingConfig {
  schema: number;
  profile_id: string;
  rom_md5: string;
  entries: Record<string, MappingEntry>;
}
export function resolveMapping(
  configs: MappingConfig[],
  md5: string,
  species: number,
): MappingEntry | undefined {
  const config = configs.find((row) => row.schema === 1 && row.rom_md5 === md5);
  const entry = config?.entries[String(species)];
  return entry && ["direct", "comparison", "none"].includes(entry.status)
    ? entry
    : undefined;
}
