#!/usr/bin/env bash

set -eu

# Print nothing when linking succeeds: rustc reports any linker output as a
# warning (linker_messages lint). Set LINK_WRAP_TRACE to trace the commands.
if [ -n "${LINK_WRAP_TRACE:-}" ]; then
	set -x
fi

LD=${LD:-rust-lld}
# Needed because LLD gets behavior from argv[0]
LD=${LD/-ld/-lld}
${LD} "$@" --emit-relocs

while [ $# -gt 0 -a "$1" != "-o" ];
do
	shift;
done
OUT="$2"

# the relocations for the constants section are required
llvm-objcopy --dump-section .rel.rodata=$OUT-rodata-reloc $OUT /dev/null
# there might not _be_ nonempty .data or .nvm_data sections, so there might be no relocations for it; fail gracefully.
llvm-objcopy --dump-section .rel.data=$OUT-data-reloc $OUT /dev/null 2>/dev/null || true
llvm-objcopy --dump-section .rel.nvm_data=$OUT-nvm-reloc $OUT /dev/null 2>/dev/null || true
# Concatenate the relocation sections; this should still write $OUT-relocs even if $OUT-data-reloc doesn't exist.
cat $OUT-rodata-reloc $OUT-nvm-reloc $OUT-data-reloc > $OUT-relocs 2>/dev/null || true

reloc_allocated_size="$((0x$(llvm-nm $OUT | grep _reloc_size | cut -d' ' -f1)))"
reloc_real_size="$(stat -c %s $OUT-relocs)"
# Check that our relocations _actually_ fit.
if [ "$reloc_real_size" -gt "$reloc_allocated_size" ]
then
	echo "Insufficient size for relocs; This is likely some bug in nanos_sdk's link.ld." >&2
	echo "Available size: " $reloc_allocated_size " Used size: " $reloc_real_size >&2
	exit 1
fi

truncate -s $reloc_allocated_size $OUT-relocs
# and write the relocs to their section in the flash image.
llvm-objcopy --update-section .rel_flash=$OUT-relocs $OUT
