#!/bin/bash
#
# Copyright (C) 2017 Petar Mihaylov (petarmihaylov.me)
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is furnished
# to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.
#
# Credit: Adapted from https://github.com/smallmuou/ios-icon-generator

set -e

SRC_FILE="$1"
DST_PATH="$2"

VERSION=1.0.0

info() {
     local green="\033[1;32m"
     local normal="\033[0m"
     echo -e "[${green}INFO${normal}] $1"
}

info-l() {
     local green="\033[1;32m"
     local normal="\033[0m"
     echo -en "[${green}INFO${normal}] $1"
}

error() {
     local red="\033[1;31m"
     local normal="\033[0m"
     echo -e "[${red}ERROR${normal}] $1"
}

usage() {
cat << EOF
VERSION: $VERSION
USAGE:
    $0 srcfile dstpath

DESCRIPTION:
    This script aim to generate icons for Electron projects easier.

    srcfile - The source png image. Minimum 1024x1024 pixels.
    dstpath - The destination path where the icons will be generated.

PREREQUISITES:
    This script is dependent on ImageMagick. You must install ImageMagick first

    ImageMagick Website:
    http://www.imagemagick.org

    See https://github.com/petarmihaylov/electron-icon-generator for more details.

    You can use Homebrew on a Mac to get ImageMagick.

AUTHOR:
    Petar Mihaylov (petarmihaylov.me)

LICENSE:
    MIT

EXAMPLE:
    $0 1024.png ~/123
EOF
}

# Check ImageMagick
command -v convert >/dev/null 2>&1 || { error >&2 "ImageMagick is not installed. Please use brew to install it first."; exit -1; }

# Display help
if ([ $# = 1 ]) && ([ $1 == "-h" ] || [$1 == "-help"]);then
    usage
    exit -1
fi


# Check parameters
if [ $# != 2 ];then
    usage
    exit -1
fi

# Check the file size is at least 1024x1024 pixels
info-l 'Checking source image size... '

WIDTH=$(identify -format %w $SRC_FILE)
HEIGHT=$(identify -format %h $SRC_FILE)

if [ $WIDTH -lt 1024 ] || [ $HEIGHT -lt 1024 ];then
	echo "Width: ${WIDTH}px"
	echo "Height: ${HEIGHT}px"
  error 'The source file must be at least 1024x1024 pixels.'
  exit -1
fi

echo '[OK]'

# Check whether the dst path exists.
if [ ! -d "$DST_PATH" ];then
    mkdir -p "$DST_PATH"
fi

# For optimal icon set info refer to: https://developer.apple.com/library/content/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html

info 'Generating MacOS X Icons'
# Create the icon.iconset path if it doesn't exist
convert "$SRC_FILE" -resize 1024x1024 "$DST_PATH/icon.png"

info 'Done generating MacOS X icons.'
echo ''

info 'Generating Mac Png Icon'


# Create the Electron Icons
info 'Generating Electron Icons'

# Create the icon path if it doesn't exist
if [ ! -d "$DST_PATH/icons" ];then
    mkdir -p "$DST_PATH/icons"
fi
info 'Generating icons/icon_16.png ...'
convert "$SRC_FILE" -resize 16x16 "$DST_PATH/icons/16x16.png"
info 'Generating icons/icon_24.png ...'
convert "$SRC_FILE" -resize 24x24 "$DST_PATH/icons/24x24.png"
info 'Generating icons/icon_32.png ...'
convert "$SRC_FILE" -resize 32x32 "$DST_PATH/icons/32x32.png"
info 'Generating icons/icon_48.png ...'
convert "$SRC_FILE" -resize 48x48 "$DST_PATH/icons/48x48.png"
info 'Generating icons/icon_64.png ...'
convert "$SRC_FILE" -resize 64x64 "$DST_PATH/icons/64x64.png"
info 'Generating icons/icon_96.png ...'
convert "$SRC_FILE" -resize 96x96 "$DST_PATH/icons/96x96.png"
info 'Generating icons/icon_128.png ...'
convert "$SRC_FILE" -resize 128x128 "$DST_PATH/icons/128x128.png"
info 'Generating icons/icon_256.png ...'
convert "$SRC_FILE" -resize 256x256 "$DST_PATH/icons/256x256.png"
info 'Generating icons/icon_512.png ...'
convert "$SRC_FILE" -resize 512x512 "$DST_PATH/icons/512x512.png"
info 'Generating icons/icon_1024.png ...'
convert "$SRC_FILE" -resize 1024x1024 "$DST_PATH/icons/1024x1024.png"

convert "$DST_PATH/icons/16x16.png" "$DST_PATH/icons/32x32.png" "$DST_PATH/icons/64x64.png" "$DST_PATH/icons/128x128.png" "$DST_PATH/icons/256x256.png" "$DST_PATH/icons/icon.ico"

info 'Done generating Electron icons.'
