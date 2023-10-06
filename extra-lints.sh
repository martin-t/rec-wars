#!/usr/bin/env bash

# Exit on error, unset variables and pipeline errors
set -euo pipefail

RET=0

# Grep returns success when found and failure when not found.
# The `[F]` is the usual trick to avoid matching this line itself
# without excluding this whole file so it's still checked.
if grep --recursive --color=auto --exclude-dir=.git --exclude-dir=target --exclude-dir=screenshots [F]IXME . ; then
    echo "The lines above this message must be fixed (or marked as todo/later in uppercase, not fixme).

Fixmes are to be fixed before committing or at least before merging to master so they can be used during development for things that must not be forgotten and grep's output is not littered with other people's fixmes."
    RET=1
fi

if grep --recursive --color=auto --exclude-dir=.git --exclude-dir=target --exclude-dir=screenshots --exclude=debug.rs cvars\.dbg . ; then
    echo "The dbg* cvars should not be used in committed code. Maybe you forgot to remove debug code?

Similar to fixmes - cvars like dbg* should not be committed to master so they're always available to be used by other devs for quick testing."
    RET=1
fi

# Find all files that contain frame context methods.
# In them, find all methods (except new/ctx/update),
# count them and print those that are present on multiple contexts.
FILES_WITH_FRAME_CTX=$(grep --recursive --files-with-matches "impl [A-Za-z]*FrameCtx" src)
if grep --extended-regexp --no-filename --only-matching "pub fn +[a-z_]+" $FILES_WITH_FRAME_CTX | grep --extended-regexp --invert-match "pub fn (new|ctx|update)$" | sort | uniq --count | grep --invert-match " *1 " | grep --color=auto ".*"; then
    echo "Frame context methods present on multiple contexts should have different names to prevent confusion between client/server/common.

The goal is to prevent accidentally shadowing a common method or calling a method on a client/server context and have it autoderef to the common context. Inheritance is nice but some things are better explicit."
    RET=1
fi

if find data | grep [A-Z] ; then
    echo "Asset names/paths must be all lowercase or we might have issues when switching between case-sensitive and case-insensitive filesystems."
    RET=1
fi

exit $RET

