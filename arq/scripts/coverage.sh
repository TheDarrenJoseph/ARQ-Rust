#!/bin/bash
export RUSTFLAGS="-Cinstrument-coverage"
rm -r ./target/coverage
mkdir -P "./target/coverage"
export LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw"
cargo test

grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing --ignore "/*" -o ./target/coverage/lcov.info
lcov --remove ./target/coverage/lcov.info -o ./target/coverage/lcov.info\
    'src/test'

genhtml -o ./target/coverage/ --show-details --highlight --ignore-errors source --legend ./target/coverage/lcov.info

TOTAL_LINES_FOUND=$(grep "LF" target/coverage/lcov.info | cut -d: -f2 | xargs |  sed -e 's/\ /+/g' | bc)
TOTAL_LINES_HIT=$(grep "LH" target/coverage/lcov.info | cut -d: -f2 | xargs |  sed -e 's/\ /+/g' | bc)
echo "Total lines found: $TOTAL_LINES_FOUND"
echo "Total lines hit: $TOTAL_LINES_HIT"
echo "$TOTAL_LINES_HIT / $TOTAL_LINES_FOUND"
TOTAL_LINE_COVERAGE_PERCENTAGE=$(echo "scale=6; $TOTAL_LINES_HIT / $TOTAL_LINES_FOUND * 100" | bc -l)
BADGE_PERCENTAGE=$(echo $TOTAL_LINE_COVERAGE_PERCENTAGE | grep -Eo "[0-9]*\.[0-9]{2}")
echo "Total lines covered (percentage): $BADGE_PERCENTAGE"

wget https://img.shields.io/badge/Line_Coverage-$BADGE_PERCENTAGE%-green -O ../images/badge.svg
