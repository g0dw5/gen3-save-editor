export type Location =
  | { kind: "party"; slot: number }
  | { kind: "box"; box_index: number; slot: number };
export const locationKey = (loc: Location) =>
  loc.kind === "party" ? `p:${loc.slot}` : `${loc.box_index}:${loc.slot}`;
export const fromKey = (key: string): Location => {
  const [box, slot] = key.split(":");
  return box === "p"
    ? { kind: "party", slot: +slot }
    : { kind: "box", box_index: +box, slot: +slot };
};
export interface Species {
  dex_number: number;
  id: number;
  name: string;
  stats: number[];
  types: number[];
  catch_rate: number;
  exp_yield: number;
  ev_yield: number;
  items: number[];
  gender_ratio: number;
  egg_cycles: number;
  friendship: number;
  growth: number;
  egg_groups: number[];
  abilities: number[];
  offset: number;
}
export interface Move {
  id: number;
  name: string;
  description: string;
  effect: number;
  power: number;
  move_type: number;
  category: number;
  accuracy: number;
  pp: number;
  chance: number;
  target: number;
  priority: number;
  flags: number;
  offset: number;
}
export interface Item {
  id: number;
  name: string;
  description: string;
  price: number;
  hold_effect: number;
  hold_param: number;
  pocket: number;
  item_type: number;
  tm_move: number | null;
  offset: number;
}
export interface Ability {
  id: number;
  name: string;
  description: string;
}
export interface Catalog {
  editor_rules?: {
    balls: number[];
    nature_override: boolean;
    contest_ranks: number[];
  };
  profile: {
    capabilities?: {
      save_edit: boolean;
      rom_edit: boolean;
      world: boolean;
      dex: boolean;
      complete_learnsets: boolean;
      individual_sprites: boolean;
      battle_forms: boolean;
    };
    id: string;
    label: string;
    md5: string;
    size: number;
    max_level?: number;
    fishing_rods?: number[];
    save?: { pockets: { id: string; category: number }[] };
    feebas?: { map_id: string } | null;
    teaching?: { shared_lists?: number | null };
    hidden_power?: {
      move_id: number;
      formula: "gen3_to5" | "gen6_fixed60";
    } | null;
  };
  species: Species[];
  moves: Move[];
  items: Item[];
  abilities: Ability[];
  met_locations: { id: number; name: string }[];
}
export interface Pokemon {
  pid: number;
  ot_id: number;
  nickname: string;
  ot_name: string;
  language: number;
  markings: number;
  species: number;
  held_item: number;
  experience: number;
  pp_ups: number[];
  friendship: number;
  moves: number[];
  pps: number[];
  evs: number[];
  condition: number[];
  ivs: number[];
  ability_slot: number;
  ability_id: number;
  egg: boolean;
  pokerus: number;
  met_location: number;
  met_level: number;
  origin_game: number;
  ball: number;
  ot_gender: number;
  ribbons: number;
  nature: number;
  effective_nature?: number;
  nature_override?: number | null;
  gender: string;
  shiny: boolean;
  level: number;
  stats: number[];
  current_hp: number | null;
  status: number | null;
  checksum_ok: boolean;
}
export interface StoredPokemon {
  location: Location;
  pokemon: Pokemon;
}
export interface Trainer {
  name: string;
  gender: number;
  tid: number;
  sid: number;
  hours: number;
  minutes: number;
  seconds: number;
  money: number;
  coins: number;
  registered_item: number;
}
export interface BoxInfo {
  index: number;
  name: string;
  wallpaper: number;
  count: number;
}
export interface BagEntry {
  pocket: string;
  slot: number;
  item: number;
  quantity: number;
}
export interface Finding {
  code: string;
  field: string;
  severity: string;
  detail: string;
}
export interface Change {
  fields: { path: string; before: unknown; after: unknown }[];
  action: Record<string, unknown>;
  bytes_changed: number;
  findings: Finding[];
}
export interface Snapshot {
  trainer: Trainer;
  pokemon: StoredPokemon[];
  boxes: BoxInfo[];
  bag: BagEntry[];
  dex: { number: number; seen: boolean; owned: boolean }[];
  active_slot: number;
  counter: number;
  backup_valid: boolean;
  dirty: boolean;
  can_undo: boolean;
  can_redo: boolean;
  changes: Change[];
}
export interface LearnSource {
  move_id: number;
  source: string;
  species: number;
  level: number | null;
  index: number | null;
  offset: number;
}
export interface Encounter {
  selector?: { variable: number; value: number; fallback: boolean } | null;
  species: number;
  map_id: string;
  map_name: string;
  region: number;
  method: string;
  min_level: number;
  max_level: number;
  weight: number | null;
  encounter_rate: number | null;
  slot: number | null;
  offset: number;
  conditional: boolean;
}
export interface OriginOptions {
  ancestors: number[];
  encounters: Encounter[];
  can_hatch: boolean;
  hatch_regions: number[];
}
export interface SpeciesDetail {
  teaching_list_present?: boolean;
  encounters_verified?: boolean;
  battle_forms?: {
    source: number;
    target: number;
    kind: "mega" | "primal";
    trigger: { kind: "held_item" | "known_move"; id: number };
    offset: number;
  }[];
  origins: OriginOptions;
  species: Species;
  evolutions: {
    condition?: string;
    method: number;
    parameter: number;
    target: number;
    offset: number;
  }[];
  learnset: LearnSource[];
  encounters: Encounter[];
}
export interface ItemReward {
  item: number;
  quantity: number | null;
  offset: number;
  via: string;
  conditions: {
    kind: string;
    id: number;
    value: number;
    comparison: number;
    taken: boolean;
  }[];
}
export interface MapMarker {
  id: string;
  kind: "pickup" | "hidden" | "gift" | "npc" | "event";
  x: number;
  y: number;
  elevation: number;
  local_id: number | null;
  graphics_id: number | null;
  movement_type: number | null;
  flag: number | null;
  offset: number;
  script: number | null;
  rewards: ItemReward[];
  stopped_at: number[];
}
export interface MapEventReport {
  map_id: string;
  markers: MapMarker[];
  unplaced_rewards: ItemReward[];
  stopped_at: number[];
}
export interface GameMap {
  id: string;
  group: number;
  number: number;
  name: string;
  region: number;
  width: number;
  height: number;
  map_type: number;
  header: number;
  layout: number;
  scripts: number[];
}
export interface Opponent {
  diagnostics: string[];
  id: number;
  name: string;
  class: number;
  class_name: string | null;
  portrait: number;
  female: boolean;
  double_battle: boolean;
  items: number[];
  ai: number;
  party: {
    species: number;
    level: number;
    iv_quality: number;
    level_rule: "fixed" | "party_max";
    generation: {
      gender: string;
      nature: number;
      ability_id: number;
      ivs: number[] | null;
      evs: number[];
      personality_parameter: number;
      ability_options?: number[];
    } | null;
    held_item: number;
    moves: number[];
    moves_explicit: boolean;
    offset: number;
  }[];
  offset: number;
}
export interface World {
  map_events: MapEventReport[];
  map_groups: { kind: string; map_ids: string[] }[];
  trainer_locations: {
    locations: {
      trainer_id: number;
      map_id: string;
      map_name: string;
      battle_offsets: number[];
      actors: {
        local_id: number;
        graphics_id: number;
        script: number | null;
      }[];
    }[];
    unresolved_maps: { map_id: string; stopped_at: number[] }[];
  };
  maps: GameMap[];
  encounters: Encounter[];
  trainers: Opponent[];
}
export type RefTab =
  "species" | "moves" | "items" | "abilities" | "maps" | "trainers";
export interface RefWindow {
  id: number;
  tab: RefTab;
  selected?: number | string;
}
export interface Template {
  species: number;
  level: number;
  met_location?: number;
  egg?: boolean;
}

export interface FishingReport {
  map_id: string;
  species: number;
  min_level: number;
  max_level: number;
  percent: number;
  seed: number | null;
  spots: { x: number; y: number; spot_id: number }[];
}
