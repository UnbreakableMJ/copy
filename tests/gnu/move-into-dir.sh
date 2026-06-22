# Ensure `move` relocates files into a directory and removes the sources.
#
# Independent reimplementation in the spirit of the GNU coreutils mv tests.
# Requires a `move` binary on PATH.

set -eu

fail=0
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"

mkdir dest
echo one > a.txt
echo two > b.txt

move a.txt b.txt dest/

test ! -e a.txt || { echo "ERROR: source a.txt was not removed"; fail=1; }
test ! -e b.txt || { echo "ERROR: source b.txt was not removed"; fail=1; }
test "$(cat dest/a.txt)" = one || { echo "ERROR: dest/a.txt has wrong contents"; fail=1; }
test "$(cat dest/b.txt)" = two || { echo "ERROR: dest/b.txt has wrong contents"; fail=1; }

exit $fail
