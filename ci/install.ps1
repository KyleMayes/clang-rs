Start-FileDownload "https://static.rust-lang.org/dist/rust-1.10.0-i686-pc-windows-msvc.exe"
.\rust-1.10.0-i686-pc-windows-msvc.exe /VERYSILENT /NORESTART /DIR="C:\Rust"
$env:Path += ";C:\MinGW\bin;C:\Rust\bin"
C:\Rust\bin\rustc.exe --version
C:\Rust\bin\cargo.exe --version

Start-FileDownload "http://llvm.org/releases/${env:LLVM_VERSION}.0/LLVM-${env:LLVM_VERSION}.0-win32.exe"
7z x LLVM-$env:LLVM_VERSION.0-win32.exe -oC:\LLVM
$env:Path += ";C:\LLVM\bin"
$env:LIBCLANG_PATH = "C:\LLVM\lib"
