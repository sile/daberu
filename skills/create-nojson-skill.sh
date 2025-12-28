#! /bin/sh

set -eux

daberu -x clean-files

cd ~/rust/nojson/

echo 'Create skill files for this Rust crate and upload the final .skill file. 
      NOTE:
      - Skill name is "nojson"
      - Do not use `use`. Please use full qualified names such as nojson::RawJsonValue in the code' | \
  daberu -k skill-creator \
    -m claude-sonnet-4-5 \
    -r README.md \
    -r Cargo.toml \
    -g 'src/*.rs' \
    -g 'tests/*.rs'

daberu -x list-files | jq '.data | map(select(.filename | endswith(".skill"))) | max_by(.created_at)'
