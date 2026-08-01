#!/bin/sh
# src-tauri/packaging/nitrux-postrm-cleanup.sh
#
# Runs as root during package removal (registered as both deb's
# postRemoveScript and rpm's postRemoveScript). Deletes NiTruX's per-user
# runtime data (~/.local/share/org.heiphaistos.nitrux, keyed by the Tauri
# `identifier` -- this is WebKitGTK's local-storage/preferences directory,
# NOT anything dpkg/rpm itself tracks, so it survives a plain removal by
# design on every Linux system unless a maintainer script explicitly
# removes it) -- but ONLY on a genuine full removal, never on a plain
# "remove" (which conventionally preserves user data so a reinstall keeps
# preferences) or on an upgrade step.
#
# Argument conventions differ between package managers:
#   - dpkg/apt: postrm is invoked as "postrm <action> ..." where <action>
#     is "remove", "purge", "upgrade", "failed-upgrade", "abort-install",
#     or "abort-upgrade". Only "purge" means "the user wants everything
#     gone, including config/data" -- this is the standard, well-understood
#     Debian distinction (`apt remove` keeps data, `apt purge` doesn't).
#   - rpm: the post-uninstall scriptlet receives a single integer argument:
#     the count of this package's versions that will remain installed
#     after this operation. 0 means "this was the last/only version, it is
#     now fully gone" (the rpm equivalent of a full removal); any other
#     value means this run is part of an upgrade replacing an old version,
#     not a real uninstall.
set -eu

action="${1:-}"

should_clean=false
case "$action" in
  purge) should_clean=true ;;
  0) should_clean=true ;;
  *) : ;;
esac

if [ "$should_clean" != "true" ]; then
  exit 0
fi

# Scoped deletion: only ever touches the exact, identifier-named data
# directory for THIS app under each real user's home (and root's), never a
# broader path. Iterates /home/*/ plus /root explicitly rather than trying
# to be clever about which UIDs are "real users" -- a home directory that
# doesn't exist or doesn't contain our data dir is silently skipped, never
# an error (a system with no such user, or a user who never ran the app,
# is the common case, not a failure).
for home_dir in /root /home/*; do
  [ -d "$home_dir" ] || continue
  data_dir="$home_dir/.local/share/org.heiphaistos.nitrux"
  if [ -d "$data_dir" ]; then
    rm -rf -- "$data_dir"
  fi
done

exit 0
