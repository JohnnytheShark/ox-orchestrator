import os

files = [
    '.github/workflows/release.yml',
    'Cargo.toml',
    'README.md',
    'crates/ox-cli/src/tui/render.rs',
    'docs/tutorials/01_getting_started.md',
    'install.ps1',
    'install.sh',
    'site/app.js',
    'site/index.html'
]

for f in files:
    if os.path.exists(f):
        with open(f, 'r', encoding='utf-8') as file:
            content = file.read()
        content = content.replace('0.2.0', '0.2.1')
        with open(f, 'w', encoding='utf-8') as file:
            file.write(content)
