Start-FileDownload "https://static.rust-lang.org/dist/rust-nightly-i686-pc-windows-msvc.exe"
.\rust-nightly-i686-pc-windows-msvc.exe /VERYSILENT /NORESTART /DIR="C:\Program Files (x86)\Rust"
$env:Path += ";C:\MinGW\bin;C:\Program Files (x86)\Rust\bin"

Start-FileDownload "http://llvm.org/releases/${env:LLVM_VERSION}.0/LLVM-${env:LLVM_VERSION}.0-win32.exe"
7z x LLVM-$env:LLVM_VERSION.0-win32.exe -oC:\LLVM
$env:Path += ";C:\LLVM\bin"
$env:LIBCLANG_PATH = "C:\LLVM\lib"
