// Run with node --experimental-strip-types scripts/test_hidden_power.mjs.
import assert from "node:assert/strict";
import { hiddenPower } from "../ui/hiddenPower.ts";

const rules = { move_id: 237, formula: "gen3_to5" };
assert.deepEqual(hiddenPower(rules, [31, 31, 31, 31, 31, 31]), { type: 17, power: 70 });
assert.deepEqual(hiddenPower(rules, [30, 30, 30, 30, 30, 30]), { type: 1, power: 70 });
assert.deepEqual(hiddenPower(rules, [31, 30, 30, 31, 31, 31]), { type: 15, power: 70 });
assert.deepEqual(hiddenPower(rules, [0, 0, 0, 0, 0, 0]), { type: 1, power: 30 });
// Speed is the fourth IV, not the last one. Sp. Def has the greatest weight.
assert.deepEqual(hiddenPower(rules, [0, 0, 0, 1, 0, 0]), { type: 2, power: 30 });
assert.deepEqual(hiddenPower(rules, [0, 0, 0, 0, 0, 1]), { type: 8, power: 30 });
for (const ivs of [null, undefined, [], [31], [0, 0, 0, 0, 0, 32],
  [-1, 0, 0, 0, 0, 0], [NaN, 0, 0, 0, 0, 0], [1.5, 0, 0, 0, 0, 0]]) {
  assert.equal(hiddenPower(rules, ivs), null);
}
assert.equal(hiddenPower(null, [31, 31, 31, 31, 31, 31]), null);
assert.equal(hiddenPower({ move_id: 237, formula: "unknown" }, [31, 31, 31, 31, 31, 31]), null);
for (let mask = 0; mask < 64; mask++) {
  const ivs = Array.from({length: 6}, (_, i) => 30 + ((mask >> i) & 1));
  const classic = hiddenPower(rules, ivs);
  assert.deepEqual(hiddenPower({move_id:237, formula:"gen6_fixed60"}, ivs), {type:classic.type, power:60});
}
console.log("Hidden Power known vectors, stat ordering and unknown/invalid input checks passed");
