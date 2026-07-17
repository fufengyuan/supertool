#!/usr/bin/env python3
import re, sys

for f in sys.argv[1:]:
    with open(f, 'r') as fh:
        content = fh.read()
    content = re.sub(
        r'  agentSettings: \{.*?    refresh: .*?\n  \},\n',
        '', content, flags=re.DOTALL
    )
    with open(f, 'w') as fh:
        fh.write(content)
    print(f'cleaned {f}')
