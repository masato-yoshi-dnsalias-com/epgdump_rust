#!/bin/bash

cargo build --release
install target/release/epgdump /usr/local/bin
mkdir -p /usr/local/etc/epgdump
install conf/tsid.conf /usr/local/etc/epgdump/tsid.conf
