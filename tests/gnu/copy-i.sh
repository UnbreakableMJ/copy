#!/bin/sh
# Test whether copy -i prompts in the right place.
# Test interactive mode prompting behavior with various flag combinations.
#
# Inspired by GNU coreutils test: tests/cp/cp-i.sh
# Independent reimplementation for Copy.

set -eu
fail=0

command -v copy >/dev/null 2>&1 || exit 77

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

mkdir -p a b/a/c || exit 1
touch a/c || exit 1

# copy should prompt when overwriting in interactive mode
# Answer 'n' should result in non-zero exit and file should not be overwritten
echo n | copy -i -r a b 2>/dev/null && fail=1

# Verify the original file was not overwritten
test -e b/a/c || fail=1

# test basic interactive prompting
touch c d || exit 1

# Store original content of d
echo "original" > d || exit 1
echo "new content" > c || exit 1

# ask for overwrite, answer no - file should remain unchanged
echo n | copy -i c d 2>/dev/null && fail=1
test "$(cat d)" = "original" || fail=1

# ask for overwrite, answer yes - file should be overwritten
echo y | copy -i c d 2>/dev/null || fail=1
test "$(cat d)" = "new content" || fail=1

# Reset for next test
echo "original" > d || exit 1

# -f with -i: should still prompt (last option wins or -i takes precedence)
echo y | copy -f -i c d 2>/dev/null || fail=1
test "$(cat d)" = "new content" || fail=1

exit $fail
