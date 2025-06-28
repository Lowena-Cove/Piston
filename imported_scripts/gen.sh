#!/bin/sh
# (Imported from Stephensons Rocket, updated for latest SteamOS)
# See https://help.steampowered.com/en/faqs/view/1B71-EDF2-EB6D-2BB3#reimage for latest SteamOS images and instructions.
# This script builds a custom SteamOS ISO with mods and updates.

#Basic
variables
BUILD="./buildroot"
DISTNAME="brewmaster"
ISOPATH="."
ISONAME="rocket.iso"
# Updated to reference latest SteamOS image
ISOVNAME="SteamOS Recovery (latest)"
# Official SteamOS recovery image URL (as of 2025)
UPSTREAMURL="https://cdn.cloudflare.steamstatic.com/steamdeck-image/steamdeck_recovery" # See support link for latest
STEAMINSTALLFILE="steamdeck-recovery-latest.img.bz2" # Update as needed
MD5SUMFILE="MD5SUMS"

KNOWNINSTALLER="316a91157ff200bac44dcc4350a4ccf8" # Update if needed
REPODIR="./archive-mirror/mirror/repo.steampowered.com/steamos"

usage ( )
{
	cat <<EOF
	$0 [OPTION] [MOD1] [MOD2] [MOD3] ...
	-h                Print this message
	-d	  Re-Download ${STEAMINSTALLFILE}
	-n	  Set the name for the iso
	(Updated for latest SteamOS: https://help.steampowered.com/en/faqs/view/1B71-EDF2-EB6D-2BB3#reimage)
EOF
}

deps ( ) {
	deps="reprepro xorriso rsync wget lftp 7z realpath"
	for dep in ${deps}; do
		if which ${dep} >/dev/null 2>&1; then
			:
		else
			echo "Missing dependency: ${dep}"
			echo ""
			echo "For instructions on how to install the dependencies, take a look at:"
			echo "https://github.com/steamos-community/stephensons-rocket/blob/brewmaster/REQUIREMENTS.md"
			exit 1
		fi
	done
	if test "`expr length \"$ISOVNAME\"`" -gt "32"; then
		echo "Volume ID is more than 32 characters: ${ISOVNAME}"
		exit 1
	fi
}

rebuild ( ) {
	if [ -d "${BUILD}" ]; then
		echo "Building ${BUILD} from scratch"
		rm -fr "${BUILD}"
	fi
}

obsoletereport ( )
{
	if [ ! -d ${REPODIR} ]; then
		echo "No ${REPODIR} directory exists, run archive-mirror.sh if you want this script to report obsolete packages"
	else
		echo "Reporting on packages which are different in ${BUILD} than ${REPODIR}"
		echo "\nPackagename\t\ttype\tarch\told version\tnew version\n"
		REPODIR="`realpath ${REPODIR}`"
		cd ${BUILD}/pool/
		for i in */*/*/*.*deb
			do PKGNAME=`basename $i | cut -f1 -d'_'`
			ARCHNAME=`basename $i | cut -f3 -d'_' | cut -f1 -d'.'`
			PKGPATH=`dirname $i`;PKGVER=`basename $i | cut -f2 -d'_'`
			DEBTYPE=`basename $i | sed 's/.*\.//g'`
			if [ `ls -1 ${REPODIR}/pool/${PKGPATH}/${PKGNAME}_*_${ARCHNAME}.${DEBTYPE} 2> /dev/null | wc -l` -gt 0 ]
				then NEWPKGVER=$(basename `ls -1 ${REPODIR}/pool/${PKGPATH}/${PKGNAME}_*_${ARCHNAME}.${DEBTYPE} | sort -n | tail -1` | cut -f2 -d'_')
				if [ "x${PKGVER}" != "x${NEWPKGVER}" ]
					then
echo "${PKGNAME}\t\t${DEBTYPE}\t${ARCHNAME}\t${PKGVER}\t${NEWPKGVER}"
				fi
			fi
		done
		cd -
	fi
}

extract ( ) {
	steaminstallerurl="${UPSTREAMURL}/download/brewmaster/${STEAMINSTALLFILE}"
	if [ ! -f ${STEAMINSTALLFILE} ] || [ -n "${redownload}" ]; then
		if [ -f ${STEAMINSTALLFILE} ];then
			rm ${STEAMINSTALLFILE}
		fi
		echo "Downloading ${steaminstallerurl} ..."
		if lftp -e "pget -n 8 ${steaminstallerurl};exit"; then
			:
		else
			echo "Error downloading ${steaminstallerurl}!"
			exit 1
		fi
	else
		echo "Using existing ${STEAMINSTALLFILE}"
	fi
	if 7z x ${STEAMINSTALLFILE} -o${BUILD}; then
		:
	else
		echo "Error extracting ${STEAMINSTALLFILE} into ${BUILD}!"
		exit 1
	fi
	rm -fr ${BUILD}/\[BOOT\]
}

verify ( ) {
	upstreaminstallermd5sum=` wget --quiet -O- ${UPSTREAMURL}/download/brewmaster/${MD5SUMFILE} | grep SteamOSDVD.iso$ | cut -f1 -d' '`
	localinstallermd5sum=`md5sum ${STEAMINSTALLFILE} | cut -f1 -d' '`
	if test "${localinstallermd5sum}" = "${KNOWNINSTALLER}"; then
		echo "Downloaded installer matches this version of gen.sh"
	elif test "${upstreaminstallermd5sum}" = "${KNOWNINSTALLER}"; then
		echo "Local installer is missing or obsolete"
		echo "Upstream version matches expectations, forcing update"
		redownload="1"
	else
		echo "ERROR! Local installer and remote installer both unknown" >&2
		echo "ERROR! Please update gen.sh to support unknown ${STEAMINSTALLFILE}" >&2
		exit 1
	fi
}

createbuildroot ( ) {
	echo "Deleting 32-bit garbage from ${BUILD}..."
	find ${BUILD} -name "*_i386.udeb" -type f -exec rm -rf {} \;
	find ${BUILD} -name "*_i386.deb" | egrep -v "(\/steamos-modeswitch-inhibitor\/|\/glibc\/|\/elutils\/|\/expat\/|\/fglrx-driver\/|\/gcc-4.9\/|\/libdrm\/|\/libffi\/|\/libpciaccess\/|\/libvdpau\/|\/libx11\/|\/libxau\/|\/libxcb\/|\/libxdamage\/|\/libxdmcp\/|\/libxext\/|\/libxfixes\/|\/libxxf86vm\/|\/libxrandr\/|\/llvm-toolchain-3.5\/|\/mesa\/|\/nvidia-graphics-drivers\/|\/s2tc\/|\/zlib\/|\/udev\/|\/libxshmfence\/|/steam\/|\/intel-vaapi-driver\/|\/libva\/|\/systemd\/|\/libedit\/|\/ncurses\/|\/libxrender\/|\/libbsd\/)" | xargs rm -f
	rm -fr "${BUILD}/install.386"
	rm -fr "${BUILD}/dists/*/main/debian-installer/binary-i386/"
	echo "Removing bundled firmware"
        rm -f ${BUILD}/firmware/*
	pooldir="./pool"
	echo "Copying ${pooldir} into ${BUILD}..."
	if rsync -av ${pooldir} ${BUILD}; then
		:
	else
		echo "Error copying ${pooldir} to ${BUILD}"
		exit 1
	fi
	for firmware in `cat firmware.txt`; do
                echo "Symlinking ${firmware} into /firmware/ folder"
                ln -s ../${firmware} ${BUILD}/firmware/`basename ${firmware}`
        done
	rocketfiles="default.preseed post_install.sh boot isolinux"
	for file in ${rocketfiles}; do
		echo "Copying ${file} into ${BUILD}"
		cp -pfr ${file} ${BUILD}
	done
	sed -i 's/fglrx-driver//' ${BUILD}/.disk/base_include
	sed -i 's/fglrx-modules-dkms//' ${BUILD}/.disk/base_include
	sed -i 's/libgl1-fglrx-glx//' ${BUILD}/.disk/base_include
	sed -i 's/nvidia-driver//' ${BUILD}/.disk/base_include
	echo "keyboard-configuration" >> ${BUILD}/.disk/base_include
}

checkduplicates ( ) {
	echo ""
	echo "Removing duplicate packages:"
	files=$(ls buildroot/pool/*/*/*/*|grep -v ":"|grep ".deb")
	duplicates=$(echo $files|tr "\ " "\n"|cut -d"_" -f1|uniq -d)
	for curdupname in ${duplicates}; do
		curdupfiles=$(ls `echo "$files"|tr "\ " "\n"|grep "${curdupname}\_"|tr "\n" "\ "`)
		curdupamd64=$(echo $curdupfiles|tr "\ " "\n"|grep amd64)
		curdupi386=$(echo $curdupfiles|tr "\ " "\n"|grep i386)
		curdupall=$(echo $curdupfiles|tr "\ " "\n"|grep all)
		nramd64=$(echo $curdupamd64|wc -w)
		nri386=$(echo $curdupi386|wc -w)
		nrall=$(echo $curdupall|wc -w)
		if [ ${nramd64} -gt 1 ]; then
			toremove=$(echo $curdupamd64|cut -f1-$((nramd64-1)) -d" ")
			echo "Removing: $toremove"
			rm ${toremove}
			tokeep=$(echo $curdupamd64|cut -f$((nramd64)) -d" ")
			echo "Keeping: $tokeep"
		fi
		if [ ${nri386} -gt 1 ]; then
			toremove=$(echo $curdupi386|cut -f1-$((nri386-1)) -d" ")
			echo "Removing: $toremove"
			rm ${toremove}
			tokeep=$(echo $curdupi386|cut -f$((nri386)) -d" ")
			echo "Keeping: $tokeep"
		fi
		if [ ${nrall} -gt 1 ]; then
			toremove=$(echo $curdupall|cut -f1-$((nrall-1)) -d" ")
			echo "Removing: $toremove"
			rm ${toremove}
			tokeep=$(echo $curdupall|cut -f$((nrall)) -d" ")
			echo "Keeping: $tokeep"
		fi
	done
	echo "Done"
}

addmod ( ) {
	echo "Added mod ${1}..."
	if [ -z ${1} ];then
		echo "No mod specified"
		return
	fi
	if [ ! -d ./${1} ];then
		echo "Mod directory ./${1} doesn't exist"
		return
	fi
	modinstall=$(grep '^install' ./${1}/config|cut -d"=" -f2-)
	modremove=$(grep '^remove' ./${1}/config|cut -d"=" -f2-)
	for rmpkg in ${modremove};do
		rm ${BUILD}/pool/*/*/*/${rmpkg}*
	done
	sed -i "/^d-i pkgsel\/include/ s/$/ ${modinstall}/" ${BUILD}/default.preseed
	if [ -f ./${1}/sources.list ];then
		modsources="./${1}/sources.list"
	else
		modsources="./sources.list"
	fi
	cd ${1}
	if rsync -av --exclude 'pool/your-packages-here' ${pooldir} ../${BUILD}; then
		:
	else
		echo "Error copying ${1}/packages to ${BUILD}"
		exit 1
	fi
	cd - > /dev/null
	if [ -f ${1}/post_install.sh ];then
		cp -f ${1}/post_install.sh ${BUILD}
	fi
}

createiso ( ) {
	if [ ! -d ${CACHEDIR} ]; then
		mkdir -p ${CACHEDIR}
	fi
	echo "Generating Packages.."
	mv ${BUILD}/pool ${BUILD}/poolbase
	rm -rf ${BUILD}/dists
	mkdir ${BUILD}/conf
	/bin/echo -e "Origin: Valve Software LLC\nSuite: testing\nCodename: ${DISTNAME}\nComponents: main contrib non-free\nUDebComponents: main\nArchitectures: i386 amd64\nDescription: SteamOS distribution based on Debian 8.0 Jessie\nContents: udebs . .gz\nUDebIndices: Packages . .gz" > ${BUILD}/conf/distributions
	reprepro -Vb ${BUILD} includedeb ${DISTNAME} ${BUILD}/poolbase/*/*/*/*.deb
	reprepro -Vb ${BUILD} includeudeb ${DISTNAME} ${BUILD}/poolbase/*/*/*/*.udeb
	rm -rf ${BUILD}/poolbase ${BUILD}/db ${BUILD}/conf
	cd ${BUILD}
	find . -type f -print0 | xargs -0 md5sum > md5sum.txt
	cd -
	if [ -f ${ISOPATH}/${ISONAME} ]; then
		echo "Removing old ISO ${ISOPATH}/${ISONAME}"
		rm -f "${ISOPATH}/${ISONAME}"
	fi
	if [ -f "/usr/lib/syslinux/mbr/isohdpfx.bin" ]; then
		SYSLINUX="/usr/lib/syslinux/mbr/isohdpfx.bin"
	fi
	if [ -f "/usr/lib/syslinux/bios/isohdpfx.bin" ]; then
		SYSLINUX="/usr/lib/syslinux/bios/isohdpfx.bin"
	fi
	if [ -f "/usr/lib/syslinux/isohdpfx.bin" ]; then
		SYSLINUX="/usr/lib/syslinux/isohdpfx.bin"
	fi
	if [ -f "/usr/lib/ISOLINUX/isohdpfx.bin" ]; then
		SYSLINUX="/usr/lib/ISOLINUX/isohdpfx.bin"
	fi
	if [ -f "isohdpfx.bin" ]; then
		SYSLINUX="isohdpfx.bin"
	fi
	if [ -z $SYSLINUX ]; then
		echo "Error: isohdpfx.bin not found! Try putting it in ${pwd}."
		exit 1	
	fi
	echo "Building ${ISOPATH}/${ISONAME} ..."
	xorriso -as mkisofs -r -checksum_algorithm_iso md5,sha1,sha256,sha512 \
		-V "${ISOVNAME}" -o ${ISOPATH}/${ISONAME} \
		-J -isohybrid-mbr ${SYSLINUX} \
		-joliet-long -b isolinux/isolinux.bin \
		-c isolinux/boot.cat -no-emul-boot -boot-load-size 4 \
		-boot-info-table -eltorito-alt-boot -e boot/grub/efi.img \
		-no-emul-boot -isohybrid-gpt-basdat -isohybrid-apm-hfsplus ${BUILD}
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
       	ISONAME=$(echo "${OPTARG}.iso"|tr '[:upper:]' '[:lower:]'|tr "\ " "-")
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
if [ ! -d ${BUILD} ]; then
	mkdir -p ${BUILD}
fi
verify
extract
createbuildroot
for MOD in ${MODSTOADD}; do
	addmod ${MOD}
done
checkduplicates
createiso
mkchecksum
obsoletereport
