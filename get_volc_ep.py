import urllib.request
import json
import os

req = urllib.request.Request(
    'https://ark.cn-beijing.volces.com/api/v3/endpoints',
    headers={'Authorization': 'Bearer 6ef79179-f1f6-484d-8258-585a9ff61b32'}
)
try:
    with urllib.request.urlopen(req) as response:
        print(response.read().decode())
except Exception as e:
    print(f"Error: {e}")
