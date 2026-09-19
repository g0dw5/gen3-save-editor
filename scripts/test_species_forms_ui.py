"""Display native form identity in both ROM references and stored Pokémon."""
import copy,json,os
from playwright.sync_api import sync_playwright,expect
from test_reference_navigation import CATALOG,species
from test_editor_navigation import pokemon


def main():
    catalog=copy.deepcopy(CATALOG)
    catalog['profile'].update(md5='59c658a1081f542086de1060bb65f0b3',sprite_rules=dict(unown_species=201))
    ids=[386,920,1128,1129,1130,201]
    catalog['species']=[dict(species(i),name='未知图腾' if i==201 else '代欧奇希斯')for i in ids]
    catalog['form_families']=[dict(species=[386,1128,1129,1130],offset=100)]
    catalog['battle_forms']=[]
    catalog['moves']=[dict(id=0,name='',power=0,category=0,move_type=0,pp=0)]
    catalog['items']=[dict(id=0,name='',tm_move=None)]
    stored=[pokemon(1128,dict(kind='party',slot=0)),pokemon(1130,dict(kind='box',box_index=0,slot=0)),pokemon(201,dict(kind='box',box_index=0,slot=1))]
    for row in stored:row['pokemon']['ivs']=[31]*6
    stored[2]['pokemon']['pid']=0x10203
    save=dict(trainer=dict(name='TEST',gender=0,tid=1,sid=0,hours=0,minutes=0,seconds=0,money=0,coins=0,registered_item=0),pokemon=stored,boxes=[dict(index=i,name=f'Box {i+1}',wallpaper=0,count=2 if i==0 else 0)for i in range(14)],bag=[],dex=[],active_slot=0,counter=1,backup_valid=True,dirty=False,can_undo=False,can_redo=False,changes=[])
    before=copy.deepcopy(save);calls=[];errors=[]
    def respond(route):
        req=route.request.post_data_json;cmd=req['command'];calls.append(cmd)
        if cmd=='state':data=dict(catalog=catalog,save=save)
        elif cmd=='sprite':data=dict(url='')
        elif cmd=='species':
            id=req['payload']['id'];data=dict(species=next(s for s in catalog['species']if s['id']==id),evolutions=[],learnset=[],encounters=[],battle_forms=[],relations=dict(species=ids,evolutions=[],battle_forms=[],form_families=catalog['form_families'] if id!=920 else [],name_relations=[]))
        else:raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps(dict(ok=True,data=data)))
    with sync_playwright() as p:
        b=p.chromium.launch(channel='chrome',headless=True);page=b.new_page(viewport=dict(width=1440,height=1000));page.add_init_script("localStorage.setItem('gen3.locale','zh')");page.on('pageerror',lambda e:errors.append(str(e)));page.route('**/api',respond)
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        expect(page.locator('.pokemon-form-label')).to_contain_text('攻击形态')
        expect(page.locator('[data-location="p:0"]')).to_have_attribute('title','代欧奇希斯 · 攻击形态')
        page.locator('[data-location="0:0"]').click();expect(page.locator('.pokemon-form-label')).to_contain_text('速度形态')
        page.locator('[data-location="0:1"]').click();expect(page.locator('.pokemon-form-label')).to_contain_text('?')
        page.get_by_role('button',name='ROM 资料',exact=True).click();dialog=page.get_by_role('dialog')
        dialog.locator('.reference-rows button').filter(has_text='1128代欧奇希斯').click()
        expect(dialog.locator('.reference-detail h2')).to_contain_text('攻击形态')
        for label in ['普通形态','攻击形态','防御形态','速度形态']:
            expect(dialog.locator('.form-family-nodes')).to_contain_text(label)
        dialog.locator('.form-family-nodes button').filter(has_text='防御形态').click();expect(dialog.locator('.reference-detail h2')).to_contain_text('防御形态')
        dialog.locator('.reference-rows button').filter(has_text='920代欧奇希斯').click();expect(dialog.locator('.reference-detail h2')).to_contain_text('同名条目 #920')
        assert 'action' not in calls and before==save and not errors,(calls,errors)
        b.close()
    print('Passed: reference names, linked family navigation, party/box form labels, PID letter, ambiguous duplicate name and zero save actions.')


if __name__=='__main__':main()
