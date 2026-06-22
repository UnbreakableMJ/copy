# Ensure `move -i` prompts and a declined prompt preserves the destination.
#
# Independent reimplementation in the spirit of the GNU coreutils mv tests.
# Requires a `move` binary on PATH.

set -eu

fail=0
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

echo new > a.txt
echo old > b.txt

printf 'n\n' | move -i a.txt b.txt

test "$(cat b.txt)" = old || { echo "ERROR: a declined prompt overwrote the destination"; fail=1; }
test -e a.txt || { echo "ERROR: a declined prompt removed the source"; fail=1; }

exit $fail
