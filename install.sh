#!/bin/sh
if ! type cargo 2>&1 > /dev/null; then
	echo "This plugin requires an installation of rust cargo"
	exit 1
fi

cargo build --release
