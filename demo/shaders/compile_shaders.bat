@echo off
setlocal
cd /D "%~dp0"

cargo run --package rafx-shader-processor -- --glsl-path glsl/*.vert glsl/*.frag glsl/*.comp --rs-path src --metal-generated-src-path generated_msl --cooked-shaders-path ../assets/shaders && cargo fmt && cargo test --package shaders
REM cargo run --package rafx-shader-processor -- --trace --glsl-path glsl/*.vert glsl/*.frag glsl/*.comp --cooked-shaders-path ../assets/shaders