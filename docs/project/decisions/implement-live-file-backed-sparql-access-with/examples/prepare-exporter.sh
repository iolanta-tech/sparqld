examples="$PWD/docs/project/decisions/implement-live-file-backed-sparql-access-with/examples"
git clone https://github.com/The-Knowledge-Graph-Guys/vault-ld.git ../vault-ld
cd ../vault-ld
git checkout 025e71be8d810e387dc451c920e05472d1c47170
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r scripts/requirements.txt
