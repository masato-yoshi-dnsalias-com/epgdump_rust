#!/bin/bash

cargo build --release
install target/release/epgdump /usr/local/bin
