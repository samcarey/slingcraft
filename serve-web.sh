#!/bin/bash
export RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\" --cfg web_sys_unstable_apis"
trunk serve