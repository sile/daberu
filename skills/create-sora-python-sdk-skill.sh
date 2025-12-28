#! /bin/sh

set -eux

daberu -x clean-files

cd ~/python/sora-python-sdk/

echo 'Create skill files for this Python library and upload the final .skill file. 
      NOTE:
      - Skill name is "sora-python-sdk"' | \
  daberu -k skill-creator \
    -m claude-sonnet-4-5 \
    -r README.md \
    -r pyproject.toml \
    -g 'src/**.py' \
    -g 'src/**.h' \
    -g 'tests/**.py' \
    -r '../sora-python-sdk-examples/README.md' \
    -r '../sora-python-sdk-examples/pyproject.toml' \
    -g '../sora-python-sdk-examples/examples/**.py' \
    -g '../sora-python-sdk-examples/tests/**.py'

daberu -x list-files | jq '.data | map(select(.filename | endswith(".skill"))) | max_by(.created_at)'
