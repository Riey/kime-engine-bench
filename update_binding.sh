#!/usr/bin/env bash

bindgen --whitelist-function hangul.+ --whitelist-type hangul.+ wrapper.h > src/hangul.rs

