"""Generated-data browser regression for searchable IDs and origin filtering."""
import json
import os
from playwright.sync_api import sync_playwright, expect
from test_editor_navigation import pokemon
from test_reference_navigation import species, WORLD


def main():
    md5='0d9b129f7dd76895f79bb47ad7dec2fe'
    moves=[{'id':i,'name':f'Move{i}','category':0,'move_type':0,'power':40,'pp':10} for i in range(472)]
    for i,name in [(29,'思念头槌'),(206,'刀背打'),(230,'香甜花蜜'),(409,'暗影之刃'),(463,'头槌')]:moves[i]['name']=name
    moves[230].update(category=2, power=0)
    catalog={'profile':{'id':'synthetic','label':'Test','md5':md5,'size':0},'species':[species(1),species(2)],'moves':moves,
             'items':[{'id':i,'name':name,'tm_move':tm} for i,name,tm in [(0,'',None),(178,'剧毒珠',None),(200,'剩饭',None),(289,'技能机器01',206)]],
             'abilities':[], 'met_locations':[{'id':i,'name':name} for i,name in [(1,'Ancestor route'),(2,'Current cave'),(3,'Sibling forest'),(4,'Hatch town'),(99,'Legacy place')]]}
    catalog['type_names'] = ['ROM normal']
    rows=[pokemon(1,{'kind':'party','slot':0}),pokemon(2,{'kind':'box','box_index':0,'slot':0})]
    rows[0]['pokemon'].update({'moves':[206,0,0,0],'met_location':99})
    save={'trainer':{'name':'TEST','gender':0,'tid':1,'sid':0,'hours':0,'minutes':0,'seconds':0,'money':0,'coins':0,'registered_item':0},
          'pokemon':rows,'boxes':[{'index':i,'name':f'Box{i}','wallpaper':0,'count':int(i==0)} for i in range(14)],
          'bag':[{'pocket':'pc','slot':0,'item':0,'quantity':0}], 'dex':[], 'active_slot':0,'counter':1,'backup_valid':True,
          'dirty':False,'can_undo':False,'can_redo':False,'changes':[]}
    actions,errors=[],[]
    def respond(route):
        req=route.request.post_data_json;c,p=req['command'],req['payload']
        if c=='state':d={'catalog':catalog,'save':save}
        elif c=='world':d=WORLD
        elif c=='sprite':d={'url':''}
        elif c=='species':d={'species':species(p['id']),'evolutions':[], 'encounters':[],
                            'learnset':[{'move_id':i,'level':1} for i in [29,206,230,409,463]],
                            'origins':{'ancestors':[1],'encounters':[{'region':1},{'region':2}],
                                       'can_hatch':p['id']==1,'hatch_regions':[1,2,3,4] if p['id']==1 else []}}
        elif c=='action':
            a=p['action'];actions.append(a)
            if a['type']=='pokemon':next(r for r in save['pokemon'] if r['location']==a['location'])['pokemon'].update(a['patch'])
            d={'save':save}
        else:raise AssertionError(req)
        route.fulfill(content_type='application/json',body=json.dumps({'ok':True,'data':d}))
    with sync_playwright() as p:
        browser=p.chromium.launch(channel='chrome',headless=True)
        page=browser.new_page(viewport={'width':1100,'height':760})
        page.add_init_script("localStorage.setItem('gen3.locale','en')")
        page.route('**/api',respond);page.on('pageerror',lambda e:errors.append(str(e)))
        page.goto(os.environ.get('GEN3_UI_URL','http://127.0.0.1:5173'))
        apply=page.locator('.editor-submit button[type=submit]')
        held=page.get_by_role('combobox',name='Held item',exact=True)
        held.fill('剧毒珠')
        expect(page.locator('.select-popup').get_by_role('option')).to_have_count(1)
        expect(page.locator('.select-popup').get_by_role('option')).to_contain_text('剧毒珠')
        held.press('ArrowDown');held.press('Enter');apply.click()
        expect(apply).to_be_disabled();assert actions[-1]['patch']=={'held_item':178}
        held.fill('not an item');held.press('Enter');expect(apply).to_be_disabled()
        held.press('Escape')
        assert '#178' in held.input_value()
        held.fill('剩饭');held.press('Tab');expect(apply).to_be_disabled()
        tabs=page.locator('.editor-tabs')
        tabs.get_by_role('button',name='Moves',exact=True).click()
        move=page.get_by_role('combobox',name='Move 1',exact=True)
        move.fill('香甜花蜜');expect(page.locator('.select-popup').get_by_role('option')).to_have_count(1)
        expect(page.locator('.select-popup').get_by_role('option')).to_contain_text('【Status】【ROM normal】')
        move.evaluate("e=>e.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,isComposing:true}))")
        expect(apply).to_be_disabled()
        move.press('Enter');apply.click();expect(apply).to_be_disabled()
        assert actions[-1]['patch']['moves']==[230,0,0,0]
        move.fill('思念头槌');expect(page.locator('.select-popup').get_by_role('option')).to_contain_text('#29')
        move.press('Escape')
        move.fill('刀背打');expect(page.locator('.select-popup').get_by_role('option')).to_contain_text('#206')
        move.press('Escape')
        move.fill('206');expect(page.locator('.select-popup').get_by_role('option')).to_have_count(1);move.press('Escape')
        for alias in ['点到为止', '甜甜香气', 'False Swipe', '點到為止']:
            move.fill(alias);expect(page.locator('.select-popup').get_by_role('option')).to_have_count(0)
            move.press('Enter');expect(apply).to_be_disabled();move.press('Escape')
        tabs.get_by_role('button',name='Origin',exact=True).click()
        location=page.get_by_role('combobox',name='Met location',exact=True)
        location.click()
        expect(page.locator('.select-popup').get_by_role('option')).to_have_count(3)
        expect(page.locator('.select-popup').get_by_role('option').filter(has_text='Legacy place')).to_have_attribute('aria-disabled','true')
        expect(page.locator('.select-popup').get_by_role('option').filter(has_text='Sibling forest')).to_have_count(0)
        page.locator('.select-popup').get_by_role('option').filter(has_text='Ancestor route').click();apply.click();expect(apply).to_be_disabled()
        assert actions[-1]['patch']=={'met_location':1}
        mode=page.get_by_role('combobox',name='Encounter source',exact=True)
        mode.select_option('hatched')
        expect(page.get_by_role('spinbutton',name='Met level',exact=True)).to_have_value('0')
        location.click();expect(page.locator('.select-popup').get_by_role('option')).to_have_count(4)
        page.locator('.select-popup').get_by_role('option').filter(has_text='Hatch town').click()
        page.locator('.editor-submit').get_by_role('button',name='Cancel',exact=True).click()
        expect(mode).to_have_value('caught');assert '#1' in location.input_value()
        mode.select_option('hatched');location.click();page.locator('.select-popup').get_by_role('option').filter(has_text='Hatch town').click()
        apply.click();expect(apply).to_be_disabled();assert actions[-1]['patch']=={'met_level':0,'met_location':4}
        page.locator('[data-location="0:0"]').click()
        expect(mode.locator('option[value=hatched]')).to_have_attribute('disabled', '')
        page.get_by_role('checkbox',name='Free editing',exact=True).check()
        location.click();expect(page.locator('.select-popup').get_by_role('option')).to_have_count(8);location.press('Escape')
        # PC inventory searches ROM item names and associated ROM TM move names.
        page.locator('.workspace-toolbar nav').get_by_role('button',name='Items',exact=True).click()
        page.get_by_role('button',name='PC items',exact=True).click()
        item=page.get_by_role('combobox',name='Items',exact=True)
        item.fill('刀背打');expect(page.locator('.select-popup').get_by_role('option')).to_have_count(1)
        expect(page.locator('.select-popup').get_by_role('option')).to_contain_text('技能机器01')
        item.press('Escape')
        # Removed aliases must stay absent in both UI languages.
        for alias in ['剧毒宝珠', 'Toxic Orb', '点到为止']:
            item.fill(alias);expect(page.locator('.select-popup').get_by_role('option')).to_have_count(0)
            item.press('Escape')
        item.fill('剧毒珠')
        expect(page.locator('.select-popup').get_by_role('option')).to_have_text('剧毒珠 #178')
        item.press('Escape')
        page.get_by_role('button',name='简体中文',exact=True).click()
        item=page.get_by_role('combobox',name='道具',exact=True)
        item.fill('剧毒珠')
        expect(page.locator('.select-popup').get_by_role('option')).to_have_text('剧毒珠 #178')
        item.press('Escape')
        expect(page).to_have_title('三代改版修改器')
        expect(page.locator('.brand')).to_contain_text('三代改版')
        expect(page.locator('.brand')).to_contain_text('修改器')
        assert not errors,errors
        screenshot=os.environ.get('GEN3_SEARCH_PREVIEW')
        if screenshot:
            item.fill('剧毒珠');page.screenshot(path=screenshot)
        browser.close()
    print('Passed: ROM-only names, removed-alias rejection, IDs and bilingual labels, keyboard/IME/dismissal, PC TM search, ancestor/current/hatch/free origins and cancel.')


if __name__=='__main__':main()
