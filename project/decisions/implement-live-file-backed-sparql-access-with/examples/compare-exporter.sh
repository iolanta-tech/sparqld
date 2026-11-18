deactivate
cd "$sparqld_root"
. .venv/bin/activate

# Regenerate the saved comparison files.
j vault-ld --vault-ld ../vault-ld

# Check the saved comparisons without changing them.
j vault-ld --vault-ld ../vault-ld --check
