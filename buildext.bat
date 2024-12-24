cd lib\extensions-wasm
rmdir /s /q ..\..\src-tauri\binaries\
cargo build --release
mkdir ..\..\src-tauri\binaries
for /f "tokens=2 delims=: " %%i in ('rustc -Vv ^| findstr host') do (
    set "HOST=%%i"
)
copy target\release\wasm-extension-runner.exe ..\..\src-tauri\binaries\exthost-wasm-%HOST%.exe

