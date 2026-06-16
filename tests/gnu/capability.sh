# Ensure copy --preserves copies capabilities
#
# Inspired by GNU coreutils test: tests/cp/capability.sh
# Independent reimplementation for Copy.

set -eu
fail=0
COPY="${COPY_PATH:-$(command -v copy 2>/dev/null || echo "")}"
if [ -z "$COPY" ]; then
  for candidate in "$HOME/.local/bin/copy" "$HOME/.cargo/bin/copy" "/usr/local/bin/copy" "/usr/bin/copy"; do
    [ -x "$candidate" ] && COPY="$candidate" && break
  done
fi
[ -x "$COPY" ] || { echo "SKIP: copy not found"; exit 0; }

[ "$(id -u)" -eq 0 ] || { echo "SKIP: must run as root"; exit 0; }
command -v setcap >/dev/null 2>&1 || { echo "SKIP: setcap not found"; exit 0; }
command -v getcap >/dev/null 2>&1 || { echo "SKIP: getcap not found"; exit 0; }

NON_ROOT_USERNAME=$(grep -m1 -E '^[^:]+:[^:]+:[0-9]{4,}:' /etc/passwd | cut -d: -f1)
[ -n "$NON_ROOT_USERNAME" ] || { echo "FAIL: no non-root user found"; exit 1; }

touch file || { echo "FAIL: touch failed"; exit 1; }
chown $NON_ROOT_USERNAME file || { echo "FAIL: chown failed"; exit 1; }
setcap 'cap_net_bind_service=ep' file || { echo "SKIP: setcap doesn't work"; exit 0; }
getcap file | grep -q cap_net_bind_service || { echo "SKIP: getcap doesn't work"; exit 0; }

$COPY --preserve=xattr file copy1 || fail=1
# Before coreutils 8.5 the capabilities would not be preserved,
# as the owner was set _after_ copying xattrs, thus clearing any capabilities.
$COPY --preserve=all   file copy2 || fail=1

for file in copy1 copy2; do
  getcap $file | grep -q cap_net_bind_service || fail=1
done

exit $fail
