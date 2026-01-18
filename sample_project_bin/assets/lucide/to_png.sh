#!/bin/bash

inkscape $1.svg --export-type=png --export-filename=$1.svg --export-dpi=300;
convert $1.png -channel RGB -negate $1-white.png;
