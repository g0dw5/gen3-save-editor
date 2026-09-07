import type { Catalog, Opponent, World } from "./types";

// Search aliases describe encounters, not embedded ROM names or parties.
export const trainerGroupAliases: Record<string, string> = {
  league_first:
    "一周目联盟 一周目冠军 一周目四天王 一周目 首次 初次 联盟 四天王 冠军 first league elite four champion",
  league_stronger: "联盟 强化 再战 重赛 league stronger rematch",
};

export function trainerGroups(world: World | null, id: number) {
  return world?.trainer_groups.filter((g) => g.trainer_ids.includes(id)) ?? [];
}

export function searchTrainers(
  trainers: Opponent[],
  world: World | null,
  catalog: Catalog,
  query: string,
  group: string,
) {
  const selectedGroup = world?.trainer_groups.find((g) => g.id === group);
  const tokens = query.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
  return trainers
    .filter((trainer) => {
      if (selectedGroup && !selectedGroup.trainer_ids.includes(trainer.id))
        return false;
      const context = trainerGroups(world, trainer.id)
        .map((g) => trainerGroupAliases[g.id] ?? g.id)
        .join(" ");
      const maps =
        world?.trainer_locations.locations
          .filter((l) => l.trainer_id === trainer.id)
          .map((l) => `${l.map_id} ${l.map_name}`)
          .join(" ") ?? "";
      const party = trainer.party
        .map((p) => catalog.species.find((s) => s.id === p.species)?.name ?? "")
        .join(" ");
      const text =
        `${trainer.id} ${trainer.name} ${context} ${maps} ${party}`.toLocaleLowerCase();
      return tokens.every((token) => text.includes(token));
    })
    .sort((a, b) =>
      selectedGroup
        ? selectedGroup.trainer_ids.indexOf(a.id) -
          selectedGroup.trainer_ids.indexOf(b.id)
        : a.id - b.id,
    );
}
