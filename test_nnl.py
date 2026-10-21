import importlib.util

spec = importlib.util.spec_from_file_location('srv', 'server.py')
srv = importlib.util.module_from_spec(spec)
spec.loader.exec_module(srv)

print('se cfg:', srv._get_se_config(1))
item = {'source': 'a.mp4', 'secondary_events': [
    {'template': 'NNL', 'in': 0.0, 'out': 10.0, 'text': 'LIVE'}]}
ev = item['secondary_events'][0]
off = max(float(ev.get('in', ev.get('offset', 0) or 0) or 0), 0.0)
print('off:', off)
