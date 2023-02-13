#!/usr/bin/env bash

set -eu

set -x

LD=${LD:-rust-lld}
# Needed because LLD gets behavior from argv[0]
LD=${LD/-ld/-lld}
${LD} "$@" --emit-relocs

echo RUST_LLD DONE

while [ $# -gt 0 -a "$1" != "-o" ];
do
	shift;
done
OUT="$2"

echo OUT IS $OUT

# the relocations for the constants section are required
${OBJCOPY} --dump-section .rel.rodata=$OUT-rodata-reloc $OUT /dev/null
# there might not _be_ nonempty .data or .nvm_data sections, so there might be no relocations for it; fail gracefully.
${OBJCOPY} --dump-section .rel.data=$OUT-data-reloc $OUT /dev/null || true
${OBJCOPY} --dump-section .rel.nvm_data=$OUT-nvm-reloc $OUT /dev/null || true
# Concatenate the relocation sections; this should still write $OUT-relocs even if $OUT-data-reloc doesn't exist.
cat $OUT-rodata-reloc $OUT-nvm-reloc $OUT-data-reloc > $OUT-relocs || true
# pad the relocs out to size - we should probably make some way to adjust this size from the source.

reloc_allocated_size="$((0x$(${NM} $OUT | grep _reloc_size | cut -d' ' -f1)))"
reloc_real_size="$(stat --format %s $OUT-relocs)"
# Check that our relocations _actually_ fit.
if [ "$reloc_real_size" -gt "$reloc_allocated_size" ]
then
	echo "Insufficient size for relocs; increase it in build.rs."
	echo "Available size: " $reloc_allocated_size " Used size: " $reloc_real_size
	exit 1
else
	echo "Sufficient size:" $reloc_allocated_size $reloc_real_size
fi

truncate -s $reloc_allocated_size $OUT-relocs
# and write the relocs to their section in the flash image.
${OBJCOPY} --update-section .rel_flash=$OUT-relocs $OUT
