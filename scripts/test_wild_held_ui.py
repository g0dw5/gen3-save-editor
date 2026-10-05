"""Public read-only fixture: native chance context -> Pokémon/map/back and HTML."""
import copy,json,os
from pathlib import Path
from playwright.sync_api import sync_playwright,expect
from test_reference_navigation import CATALOG
from test_editor_navigation import pokemon

def main():
 expect.set_options(timeout=30000)
 catalog=copy.deepcopy(CATALOG);catalog['profile']['capabilities']=dict(world=True,save_edit=True,dex=True)
 catalog['items']=[dict(id=0,name='',tm_move=None),dict(id=1,name='Wild stone <script>',tm_move=None,description='',price=100,pocket=1)]
 catalog['abilities']=[dict(id=14,name='ROM-only ability',description='')]
 maps=[dict(id='0-0',name='Wild cave',region=1,width=4,height=4,map_type=4)]
 world=dict(maps=maps,map_events=[],encounters=[],trainers=[],trainer_locations=dict(locations=[]),map_groups=[])
 baseline=dict(outcomes=[dict(item=0,count=62261),dict(item=1,count=3275)],denominator=65536,lead_species=None,lead_ability=None,lead_egg=False)
 current=dict(outcomes=[dict(item=0,count=52436),dict(item=1,count=13100)],denominator=65536,lead_species=2,lead_ability=14,lead_egg=False)
 source=dict(underfoot=None,kind='wild_held',encounter_method='cave',map_id='0-0',region=1,x=None,y=None,related=[dict(kind='species',id=1)],quantity=1,min_level=10,max_level=12,encounter_percent=20,held_percent=13100/65536*100,held_context=dict(species=1,layout=0,routine=0x0806ea68,baseline=baseline,current_party=current),held_issue=None,periods=['night'],conditions=[],requirements=[],evolution=None,status='unknown',receipt_flag=None,repeatable=True,offset=100,partial=True,in_scenario=None)
 unreferenced=dict(source,kind='wild_held_unreferenced',map_id=None,encounter_method=None,held_context=None,held_percent=None,repeatable=None)
 save=dict(trainer={'name':'TEST'},pokemon=[pokemon(2,dict(kind='party',slot=0))],boxes=[dict(index=i,name=f'Box {i}',count=0,wallpaper=0) for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
 task=dict(target=dict(kind='item',id=1),family=[],existing_family_members=[],source=source,alternatives=2)
 plan=dict(clock=None,rom_md5='test',basis='individuals',families=True,owned_count=1,missing_count=0,regions=[dict(region=1,tasks=[task])],entrances=[],partial=True)
 errors=[];requests=[]
 def respond(route):
  req=route.request.post_data_json;requests.append(req);command,payload=req['command'],req['payload']
  if command=='state':data=dict(catalog=catalog,save=save)
  elif command=='world':data=world
  elif command=='species':data=dict(species=catalog['species'][payload['id']-1],evolutions=[],learnset=[],encounters=[],origins={})
  elif command=='acquisition':data=dict(target=payload,sources=[source,unreferenced] if payload['kind']=='item' else [],partial=True,clock=None)
  elif command=='map_navigation':data=dict(map_id=payload['id'],outgoing=[],incoming=[],approaches=[],truncated=False,diagnostics=[])
  elif command=='collection':data=plan
  elif command in ('map_image','sprite','object_sprite','trainer_sprite'):data=dict(url='')
  else:raise AssertionError(req)
  route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
 with sync_playwright() as p:
  browser=p.chromium.launch(channel='chrome',headless=True);page=browser.new_page(viewport=dict(width=1050,height=780),accept_downloads=True)
  page.add_init_script("localStorage.setItem('gen3.locale','en')");page.on('pageerror',lambda e:errors.append(str(e)));page.route('**/api',respond)
  page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'));page.get_by_role('button',name='ROM reference',exact=True).click()
  page.locator('.reference-tabs').get_by_role('button',name='Items',exact=True).click()
  pane=page.locator('.acquisition-panel');expect(pane).to_contain_text('No-modifier baseline (simulated): ≈ 5.00%');expect(pane).to_contain_text('ROM-only ability → ≈ 19.99%');expect(pane).to_contain_text('Encounter slot probability 20%');expect(pane).to_contain_text('no random encounter reference was found')
  pane.get_by_role('button',name='Test species 1 ↗',exact=True).first.click();expect(page.locator('.reference-detail h2')).to_contain_text('Test species 1')
  page.get_by_role('button',name='Back to previous reference',exact=False).click();expect(pane).to_contain_text('≈ 19.99%')
  pane.get_by_role('button',name='Wild cave',exact=False).click();expect(page.locator('.reference-detail h2')).to_contain_text('Wild cave');page.get_by_role('button',name='Back to previous reference',exact=False).click()
  page.get_by_role('button',name='简体中文',exact=True).click();expect(pane).to_contain_text('无修正基准（模拟条件）');expect(pane).to_contain_text('当前同行首位');expect(pane).to_contain_text('不能据此断言')
  page.get_by_role('button',name='收集规划',exact=True).click();expect(page.locator('.collection-panel')).to_contain_text('≈ 19.99%')
  with page.expect_download() as info:page.get_by_role('button',name='导出独立 HTML',exact=True).click()
  html=Path(info.value.path()).read_text();assert '≈ 19.99%' in html and 'Wild stone &lt;script&gt;' in html and '<script>' not in html and '不预测下一次相遇' in html
  page.set_viewport_size(dict(width=720,height=740));expect(page.get_by_role('button',name='导出独立 HTML',exact=True)).to_be_visible()
  assert not errors,errors;assert not any(r['command'] in ('action','export_save','save_bytes') for r in requests)
  print('Wild held chance contexts, item/species/map/back, bilingual collection HTML and compact window passed');browser.close()

if __name__=='__main__':main()
