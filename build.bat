@echo OFF

cargo build --release
cargo build --target=i686-pc-windows-msvc --release --lib

if not exist build mkdir build

copy "%cd%\target\release\key2joy_rebinder.exe" "%cd%\build\key2joy_rebinder.exe"
copy "%cd%\target\release\xinput_injection.dll" "%cd%\build\xinput_injection_x64.dll"
copy "%cd%\target\i686-pc-windows-msvc\release\xinput_injection.dll" "%cd%\build\xinput_injection_x86.dll"
