import assert from 'node:assert/strict';
import fs from 'node:fs';
import { resolveMapping } from '../ui/speciesMappings.ts';
const rows=JSON.parse(fs.readFileSync('ui/data/official-stats.json')).rows;
const keys=new Set(rows.map(r=>`${r[0]}:${r[2]}`));
const files=['bw','dp','rocket'].map(n=>JSON.parse(fs.readFileSync(`config/species-mapping-reviews/${n}.json`)));
assert.equal(new Set(files.map(f=>f.rom_md5)).size,3);
for(const config of files){
 assert.equal(config.schema,1);assert.match(config.rom_md5,/^[a-f0-9]{32}$/);
 for(const [id,entry]of Object.entries(config.entries)){
  assert.match(id,/^[1-9][0-9]*$/);assert.ok(['pending','direct','comparison','none'].includes(entry.status));
  for(const key of entry.candidates)assert.ok(keys.has(key),key);
  if(entry.target!==null)assert.ok(keys.has(entry.target));
  if(['direct','comparison'].includes(entry.status))assert.ok(entry.target);
  if(entry.status==='none')assert.equal(entry.target,null);
 }
}
const rocket=files[2];
assert.equal(rocket.entries[990].candidates[0],'150:超级超梦Ｘ');
assert.equal(rocket.entries[991].candidates[0],'150:超级超梦Ｙ');
assert.equal(rocket.entries[978].candidates[0],'6:超级喷火龙Ｘ');
assert.equal(rocket.entries[979].candidates[0],'6:超级喷火龙Ｙ');
assert.equal(files[0].entries[12].candidates[0],'12:');
assert.equal(files[0].entries[219].target,'555:普通模式');
assert.equal(rocket.entries[555].target,'555:普通模式');
assert.equal(rocket.entries[899].suggested_mode,'comparison');
const fixtures=[{schema:1,profile_id:'A',rom_md5:'a',entries:{1:{status:'direct',target:'150:超级超梦Ｘ'},2:{status:'pending',target:'9:'},3:{status:'none',target:null},4:{status:'comparison',target:'9:'}}},{schema:1,profile_id:'B',rom_md5:'b',entries:{1:{status:'direct',target:'150:超级超梦Ｙ'}}}];
assert.equal(resolveMapping(fixtures,'a',1).target,'150:超级超梦Ｘ');
assert.equal(resolveMapping(fixtures,'b',1).target,'150:超级超梦Ｙ');
assert.equal(resolveMapping(fixtures,'a',2),undefined);
assert.equal(resolveMapping(fixtures,'a',3).target,null);
assert.equal(resolveMapping(fixtures,'a',4).status,'comparison');
assert.equal(resolveMapping(fixtures,'unknown',1),undefined);
console.log('Passed: proposal keys, X/Y branches, semantic aliases, pending isolation, null decisions and cross-ROM mapping.');
