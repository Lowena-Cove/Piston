#!/bin/sh
# (Imported from Stephensons Rocket, updated for latest SteamOS)
# See https://help.steampowered.com/en/faqs/view/1B71-EDF2-EB6D-2BB3#reimage for latest SteamOS images and instructions.
# This script builds a custom SteamOS image with mods and updates.

#Basic variables
BUILD="./buildroot"
DISTNAME="steamos-recovery"
ISOPATH="."
ISONAME="steamdeck-recovery-latest.img"
ISOVNAME="SteamOS Recovery (latest)"
# Official SteamOS recovery image URL (as of 2025)
UPSTREAMURL="https://cdn.cloudflare.steamstatic.com/steamdeck-image/steamdeck_recovery"
STEAMINSTALLFILE="steamdeck-recovery-latest.img.bz2"
MD5SUMFILE="MD5SUMS"

KNOWNINSTALLER="316a91157ff200bac44dcc4350a4ccf8" # Update if needed
REPODIR="./archive-mirror/mirror/repo.steampowered.com/steamos"

usage ( )
{
	cat <<EOF
	$0 [OPTION] [MOD1] [MOD2] [MOD3] ...
	-h                Print this message
	-d	  Re-Download ${STEAMINSTALLFILE}
	-n	  Set the name for the image
	(Updated for latest SteamOS: https://help.steampowered.com/en/faqs/view/1B71-EDF2-EB6D-2BB3#reimage)
EOF
}

deps ( ) {
	deps="bzip2 rsync wget 7z realpath"
	for dep in ${deps}; do
		if which ${dep} >/dev/null 2>&1; then
			:
		else
			echo "Missing dependency: ${dep}"
			echo "Install with your package manager."
			exit 1
		fi
	done
}

rebuild ( ) {
	if [ -d "${BUILD}" ]; then
		echo "Building ${BUILD} from scratch"
		rm -fr "${BUILD}"
	fi
}

extract ( ) {
	steaminstallerurl="${UPSTREAMURL}/${STEAMINSTALLFILE}"
	if [ ! -f ${STEAMINSTALLFILE} ] || [ -n "${redownload}" ]; then
		if [ -f ${STEAMINSTALLFILE} ];then
			rm ${STEAMINSTALLFILE}
		fi
		echo "Downloading ${steaminstallerurl} ..."
		if wget -O ${STEAMINSTALLFILE} ${steaminstallerurl}; then
			:
		else
			echo "Error downloading ${steaminstallerurl}!"
			exit 1
		fi
	else
		echo "Using existing ${STEAMINSTALLFILE}"
	fi
	# Decompress the .bz2 image
	if [ ! -f ${ISONAME} ]; then
		echo "Decompressing ${STEAMINSTALLFILE} ..."
		if bzip2 -dk ${STEAMINSTALLFILE}; then
			:
		else
			echo "Error decompressing ${STEAMINSTALLFILE}!"
			exit 1
		fi
	fi
}

# Placeholder for modding logic: mount image, inject files, etc.
mod_image ( ) {
	echo "Modding image: ${ISONAME} (not yet implemented)"
	# TODO: Use guestmount, kpartx, or loopback to mount partitions and inject mods
	# Example: guestmount -a ${ISONAME} -i --rw /mnt/steamos
	# rsync -av mods/ /mnt/steamos/
	# umount /mnt/steamos
}

mkchecksum ( ) {
	echo "Generating checksum..."
	md5sum ${ISONAME} > "${ISONAME}.md5"
	if [ -f ${ISONAME}.md5 ]; then
		echo "Checksum saved in ${ISONAME}.md5"
	else
		echo "Failed to save checksum"
	fi
}

while getopts "hdn:" OPTION; do
        case ${OPTION} in
      h)
                usage
                exit 1
        ;;
        d)
            redownload="1"
        ;;
        n)
        	ISOVNAME="${OPTARG}"
       	ISONAME=$(echo "${OPTARG}.img"|tr '[:upper:]' '[:lower:]'|tr "\ " "-")
       ;;
        *)
                echo "${OPTION} - Unrecongnized option"
             usage
                exit 1
        ;;
        esac
done
shift $((OPTIND-1))
MODSTOADD="$@"
deps
rebuild
extract
mod_image
mkchecksum
# End of script
