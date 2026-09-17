import type { Catalog } from "./types";

// ROM type IDs omit Normal (0) and Mystery (9).
const types = [1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17];

export function hiddenPower(
  rules: Catalog["profile"]["hidden_power"],
  ivs: readonly number[] | null | undefined,
): { type: number; power: number } | null {
  if (
    rules?.formula !== "gen3_to5" ||
    ivs?.length !== 6 ||
    ivs.some((iv) => !Number.isInteger(iv) || iv < 0 || iv > 31)
  )
    return null;
  // Save order: HP, Attack, Defense, Speed, Sp. Atk, Sp. Def.
  const bits = (shift: number) =>
    ivs.reduce((sum, iv, i) => sum + (((iv >> shift) & 1) << i), 0);
  return {
    type: types[Math.floor((bits(0) * 15) / 63)],
    power: Math.floor((bits(1) * 40) / 63) + 30,
  };
}
