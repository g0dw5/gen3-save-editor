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
  natures: { id: number; name: string; stat_changes: number[] }[];
  type_names: string[];
  form_families?: { species: number[]; offset: number }[];
  battle_forms?: NonNullable<SpeciesDetail["battle_forms"]>;
  editor_rules?: {
    balls: number[];
    ball_options?: { value: number; item: number }[];
    hyper_training?: boolean;
    pokemon_checksum?: boolean;
    origin_game_max?: number;
    met_level_max?: number;
    nature_override: boolean;
    contest_ranks: number[];
  };
  profile: {
    capabilities?: {
      save_edit: boolean;
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
    sprite_rules?: { unown_species: number };
    hardware_clock?: unknown;
    clock?: {
      starts: number[];
      native_predicates: number[];
      forced_night_flag: number | null;
    } | null;
    breeding?: {
      pending_width: number;
      production?: { modifier: unknown | null } | null;
      saved?: object | null;
    } | null;
    native_trainers?: { constructor: number } | null;
    fishing_rods?: number[];
    save?: { pockets: { id: string; category: number }[] };
    feebas?: { map_id: string } | null;
    teaching?: { shared_lists?: number | null };
    hidden_power?: {
      move_id: number;
      formula: "gen3_to5" | "gen6_fixed60";
    } | null;
    contest?: {
      flavor_preferences: number;
      npc_blender: object | null;
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
  hyper_trained?: boolean[];
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
  periods?: string[];
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
  relations?: {
    species: number[];
    evolutions: (SpeciesDetail["evolutions"][number] & { source: number })[];
    battle_forms: NonNullable<SpeciesDetail["battle_forms"]>;
    form_families: { species: number[]; offset: number }[];
    name_relations: { source: number; target: number }[];
  };
  teaching_list_present?: boolean;
  encounters_verified?: boolean;
  battle_forms?: {
    source: number;
    target: number;
    kind: "mega" | "primal" | "gigantamax" | "transformation";
    trigger: {
      kind: "held_item" | "known_move" | "battle_command";
      id: number;
    };
    offset: number;
  }[];
  origins: OriginOptions;
  species: Species;
  evolutions: {
    condition?: string;
    requirements?: { kind: string; value: number }[];
    method: number;
    parameter: number;
    auxiliary?: number;
    target: number;
    offset: number;
  }[];
  learnset: LearnSource[];
  encounters: Encounter[];
}
export interface ReceiptEvidence {
  flag: number;
  root: number;
  award_offset: number;
  success_set_offsets: number[];
}
export interface ItemReward {
  receipt: ReceiptEvidence | null;
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
export interface PokemonSource {
  species: number;
  level: number | null;
  held_item: number | null;
  method: string;
  offset: number;
  member: number;
  conditions: ItemReward["conditions"];
  trade: {
    index: number;
    record_offset: number;
    requested_species: number;
    level_rule: "offered_pokemon";
  } | null;
}
export interface TeachingSource {
  move_id: number;
  parameter: number;
  offset: number;
  conditions: ItemReward["conditions"];
}
export interface DaycareSource {
  offset: number;
  conditions: ItemReward["conditions"];
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
  underfoot: boolean | null;
  flag: number | null;
  receipt_flag: number | null;
  offset: number;
  script: number | null;
  rewards: ItemReward[];
  pokemon: PokemonSource[];
  teaching?: TeachingSource[];
  daycare?: DaycareSource[];
  stopped_at: number[];
}
export interface MapEventReport {
  map_id: string;
  markers: MapMarker[];
  unplaced_rewards: ItemReward[];
  unplaced_pokemon: PokemonSource[];
  unplaced_teaching?: TeachingSource[];
  unplaced_daycare?: DaycareSource[];
  stopped_at: number[];
}
export interface GameMap {
  invalid_events?: boolean;
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
    level_rule: "fixed" | "party_max" | "difficulty";
    generation: {
      context?: "ordinary" | "ultimate_template" | "ultimate_plain";
      ev_increment?: number | null;
      gender: string;
      nature: number;
      ability_id: number;
      ivs: number[] | null;
      evs: number[] | null;
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
export type TrainerDifficulty = 1 | 2 | 3 | 4;
export interface TrainerEvPreview {
  trainer_id: number;
  difficulty: TrainerDifficulty;
  player_count: number;
  mons: {
    species: number;
    level: number;
    ivs: number[] | null;
    alternate_ivs: number[] | null;
    evs: number[] | null;
    alternate_evs: number[] | null;
  }[];
}
export interface TrainerBattlePreview {
  trainer_id: number;
  difficulty: TrainerDifficulty;
  player_max_level: number | null;
  mons: {
    species: number;
    base_level: number;
    level: number | null;
    level_source:
      | "rom"
      | "player_max"
      | "lunatic_scaled"
      | "needs_player_max"
      | "unsupported_raw";
    moves: number[] | null;
    ivs: number[] | null;
    evs: number[] | null;
  }[];
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
  | "species"
  | "moves"
  | "items"
  | "abilities"
  | "maps"
  | "trainers"
  | "events"
  | "collection"
  | "clock";
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

export interface MapLink {
  from: string;
  to: string | null;
  kind: "warp" | "connection";
  x: number | null;
  y: number | null;
  target_x: number | null;
  target_y: number | null;
  warp_index: number | null;
  target_warp: number | null;
  direction: number | null;
  displacement: number | null;
  offset: number;
  unresolved: string | null;
}
export interface MapNavigation {
  map_id: string;
  outgoing: MapLink[];
  incoming: MapLink[];
  approaches: MapLink[][];
  truncated: boolean;
  diagnostics: string[];
}
export interface MapFocus {
  x: number;
  y: number;
}

export interface QueryTarget {
  kind: "species" | "item" | "move";
  id: number;
}
export interface AcquisitionSource {
  underfoot: boolean | null;
  in_scenario: boolean | null;
  kind: string;
  map_id: string | null;
  region: number | null;
  x: number | null;
  y: number | null;
  related: QueryTarget[];
  quantity: number | null;
  min_level: number | null;
  max_level: number | null;
  encounter_percent: number | null;
  held_percent: number | null;
  encounter_method?: string | null;
  held_issue?: string | null;
  held_context?: {
    species: number;
    layout: number;
    routine: number;
    baseline: HeldDistribution;
    current_party: HeldDistribution | null;
  } | null;
  periods: string[];
  conditions: {
    condition: ItemReward["conditions"][number];
    satisfied: boolean | null;
    actual: number | null;
    unresolved?: string | null;
  }[];
  requirements: { kind: string; value: number }[];
  evolution: SpeciesDetail["evolutions"][number] | null;
  status: "completed" | "available" | "blocked" | "unknown";
  receipt_flag: number | null;
  receipt: ReceiptEvidence | null;
  script_source: PokemonSource | null;
  trade_context: { party_levels: number[]; box_levels: number[] } | null;
  teaching_source?: TeachingSource | null;
  repeatable: boolean | null;
  offset: number;
  partial: boolean;
}
export interface NativeTime {
  days: number;
  hour: number;
  minute: number;
  second: number;
}
export interface RtcProjection {
  rom_md5: string;
  source: "rtc_scenario";
  input: {
    year: number;
    month: number;
    day: number;
    hour: number;
    minute: number;
    second: number;
  };
  offset: NativeTime;
  offset_source: "save" | "scenario";
  local_time: NativeTime;
  weekday: null;
  period: null;
  current_clock_verified: false;
  partial: true;
}
export interface ClockReport {
  rom_md5?: string;
  hardware?: {
    offset: NativeTime;
    last_update: NativeTime;
    offset_valid: boolean;
    last_update_valid: boolean;
  } | null;
  source: "scenario" | "unresolved" | "save_virtual";
  effective_hour: number | null;
  weekday: number | null;
  period: string | null;
  next_period_hour: number | null;
  seconds_until_next_period: number | null;
  saved: {
    year: number;
    month: number;
    day: number;
    weekday: number;
    hour: number;
    minute: number;
    second: number;
    speed: number;
  } | null;
  forced_night: boolean | null;
  issue: string | null;
  current_clock_verified: boolean;
  forced_night_state_verified: boolean;
}
export interface AcquisitionReport {
  clock: ClockReport | null;
  target: QueryTarget;
  sources: AcquisitionSource[];
  partial: boolean;
}

export interface CollectionTask {
  target: QueryTarget;
  family: number[];
  existing_family_members: number[];
  source: AcquisitionSource | null;
  alternatives: number;
  preparation?: CollectionPreparation | null;
}
export interface SavedDaycareState {
  rom_md5: string;
  source: string;
  status: string;
  parents: {
    slot: number;
    present: boolean;
    pokemon: Pokemon | null;
    accumulated_steps: number;
    issue: string | null;
  }[];
  egg_available: boolean;
  compatibility: number | null;
  next_check_steps: number | null;
  native_service_state: number | null;
  legacy_pending_value: number;
  partial: boolean;
}
export interface CollectionPreparation {
  origin: number;
  current_count: number;
  source: AcquisitionSource | null;
  steps: {
    from: number;
    evolution: SpeciesDetail["evolutions"][number];
    related: QueryTarget[];
  }[];
  needs_hatching: boolean;
  breeding?: CollectionBreedingRoute | null;
  truncated: boolean;
  partial: boolean;
}
export interface CollectionBreedingRoute {
  parents: {
    location: Location;
    species: number;
    nickname: string;
    gender: string;
    held_item: number;
  }[];
  compatibility: number;
  seed: number;
  offspring_pid: number;
  partial: boolean;
}
export interface CollectionPlan {
  prerequisites?: {
    reports: EventDependencyReport[];
    entrances: { map_id: string; chains: MapLink[][]; truncated: boolean }[];
    skipped_conditions: number;
    truncated: boolean;
    partial: boolean;
  } | null;
  clock: ClockReport | null;
  rom_md5: string;
  basis: "dex" | "individuals";
  families: boolean;
  owned_count: number;
  missing_count: number;
  regions: { region: number | null; tasks: CollectionTask[] }[];
  entrances: { map_id: string; chains: MapLink[][]; truncated: boolean }[];
  breeding_coverage?: {
    parent_count: number;
    checked_pairs: number;
    total_pairs: number;
    failed_pairs: number;
    truncated: boolean;
    sampled: boolean;
    issue: string | null;
  } | null;
  partial: boolean;
}

export interface NativeTrainerPreview {
  rom_md5: string;
  trainer_id: number;
  seed: number;
  scenario: string;
  partial: boolean;
  mons: Pokemon[];
}

export interface HeldDistribution {
  outcomes: { item: number; count: number }[];
  denominator: number;
  lead_species: number | null;
  lead_ability: number | null;
  lead_egg: boolean;
}

export interface EventDependencyReport {
  rom_md5: string;
  condition: AcquisitionSource["conditions"][number];
  writers: {
    effect: {
      kind: string;
      id: number;
      operation: string;
      operand: number | null;
      value: number | null;
      offset: number;
      conditions: ItemReward["conditions"];
    };
    reference: {
      map_id: string;
      map_name: string;
      region: number;
      kind: string;
      x: number | null;
      y: number | null;
      local_id: number | null;
      offset: number;
      root: number;
      entry_unresolved: boolean;
    };
    conditions: AcquisitionSource["conditions"];
    text: { offset: number; text: string }[];
    stopped_at: number[];
    path_complete: boolean;
  }[];
  coverage: {
    checked_scripts: number;
    total_scripts: number;
    failed_scripts: number;
    truncated: boolean;
  };
  total_matches: number;
  next_offset: number | null;
  partial: boolean;
}

export interface EventClue {
  id: string;
  reference: EventDependencyReport["writers"][number]["reference"];
  text: { offset: number; text: string }[];
  visibility: AcquisitionSource["conditions"];
  effects: {
    effect: EventDependencyReport["writers"][number]["effect"];
    conditions: AcquisitionSource["conditions"];
    observed: boolean | null;
  }[];
  effects_truncated: boolean;
  battles?: ScriptBattleSource[];
  stopped_at: number[];
  path_complete: boolean;
}
export interface ScriptBattleSource {
  trainer_id: number;
  battle_type: number;
  offset: number;
  role: string;
  conditions: ItemReward["conditions"];
}
export interface TrainerReferenceReport {
  rom_md5: string;
  trainer_id: number;
  references: {
    clue_id: string;
    battle: ScriptBattleSource;
    reference: EventDependencyReport["writers"][number]["reference"];
    conditions: AcquisitionSource["conditions"];
    visibility: AcquisitionSource["conditions"];
    text: EventClue["text"];
    stopped_at: number[];
  }[];
  total_matches: number;
  next_offset: number | null;
  coverage: EventDependencyReport["coverage"];
  partial: boolean;
}
export interface EventClueReport {
  rom_md5: string;
  entries: EventClue[];
  selected: EventClue | null;
  total_matches: number;
  next_offset: number | null;
  coverage: EventDependencyReport["coverage"];
  partial: boolean;
}
