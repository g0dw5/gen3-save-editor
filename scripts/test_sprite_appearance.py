"""Browser regression for PID-aware sprite requests, caching and refresh.

Start Vite and run with Playwright/Chrome. Uses synthetic data and no ROM/save.
"""
import json
import os
from playwright.sync_api import sync_playwright


def main():
    requests = []
    errors = []
    pixel = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII='
    def respond(route):
        request = route.request.post_data_json
        if request['command'] == 'state':
            data = {'catalog': None, 'save': None}
        elif request['command'] == 'sprite':
            requests.append(request['payload'])
            data = {'url': pixel}
        else:
            raise AssertionError(request)
        route.fulfill(content_type='application/json', body=json.dumps({'ok': True, 'data': data}))
    with sync_playwright() as p:
        browser = p.chromium.launch(channel='chrome', headless=True)
        page = browser.new_page()
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.route('**/api', respond)
        page.goto(os.environ.get('GEN3_UI_URL', 'http://127.0.0.1:5173'))
        page.evaluate('''async () => {
            const {default: React} = await import('/node_modules/.vite/deps/react.js');
            const {default: ReactDOM} = await import('/node_modules/.vite/deps/react-dom_client.js');
            const {Sprite} = await import('/ui/components.tsx');
            const el = document.createElement('div'); el.id = 'sprite-test'; document.body.append(el);
            const root = ReactDOM.createRoot(el);
            window.renderSprites = (rows) => root.render(React.createElement(React.Fragment, null,
                ...rows.map((row, i) => React.createElement(Sprite, {
                    key: i, catalog: {profile: {md5: 'synthetic'}}, ...row
                }))));
        }''')
        rows = [{'species': 201, 'pid': 0}, {'species': 201, 'pid': 1},
                {'species': 308, 'pid': 0}, {'species': 308, 'pid': 0x88888888}]
        page.evaluate('(rows) => window.renderSprites(rows)', rows)
        page.wait_for_function("document.querySelectorAll('#sprite-test img').length === 4")
        assert len(requests) == 4, requests
        assert {(r['id'], r['pid']) for r in requests} == {(r['species'], r['pid']) for r in rows}
        rows[0]['pid'] = 0x10000
        rows[1]['shiny'] = True
        page.evaluate('(rows) => window.renderSprites(rows)', rows)
        page.wait_for_function("document.querySelectorAll('#sprite-test img').length === 4")
        assert len(requests) == 6, requests
        assert any(r['pid'] == 0x10000 for r in requests)
        assert any(r['id'] == 201 and r['pid'] == 1 and r['shiny'] for r in requests)
        rows[0]['pid'] = 0
        rows[1]['shiny'] = False
        page.evaluate('(rows) => window.renderSprites(rows)', rows)
        page.wait_for_function("document.querySelectorAll('#sprite-test img').length === 4")
        assert len(requests) == 6, requests  # Original appearances use their own cache entries.
        assert not errors, errors
        browser.close()
    print('Distinct PIDs, shiny variants, live PID changes and cache reuse verified')


if __name__ == '__main__':
    main()
