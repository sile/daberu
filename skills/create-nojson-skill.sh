#! /bin/sh

set -eux

daberu -x clean-files

cd ~/rust/nojson/

echo 'Create skill files for this Rust crate and upload the final .skill file. [NOTE] always use full qualified name such as nojson::RawJsonValue (not use imports)' | \
  daberu -k skill-creator \
    -r README.md \
    -r Cargo.toml \
    -g 'src/*.rs' \
    -g 'tests/*.rs'
