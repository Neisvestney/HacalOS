# HacalOS
Blazingly fast simple x64 UEFI OS
## Building
> Environment: Linux (or WSL)  
Tested on: WSL Ubuntu 20.04.4 LTS
- Install all required dependencies:
  - Apt: `sudo apt install make build-essential mtools xorriso`
  - Or with specific versions: `sudo apt install make=4.2.1-1.2 build-essential mtools=4.0.24-1 xorriso=1.5.2-1`
- Install rust nightly
  - `rustup toolchain install nightly`
- Install rust targets
  - `rustup target add x86_64-unknown-uefi`
  - `rustup target add x86_64-unknown-none`
- Build iso image `make os.iso`
## Run with QEmu
> Tested on: WSL with VcXsrv on Windows 10
- Install all required dependencies:
  - Apt: `sudo apt install qemu-kvm ovmf`
- Run qemu `make run`
## Setup and debug in RustRover
- Set `File | Settings | Rust -> Toolchain Location` to `\\wsl$\Ubuntu\home\[user]\.cargo\bin`
- Set `Enviroments variables` to `CARGO_UNSTABLE_JSON_TARGET_SPEC=true`
- Add remote debug configuration with GDB, target to `localhost:1234` and symbols to `$ProjectFileDir$/src/kernel/target/x86_64-hacal_os/debug/kernel`
- Run `make DEBUG=1 run` and then run debug configuration in RustRover
## Debug with CLion and gdb
- Add new WSL toolchain (`File | Settings | Build, Execution, Deployment | Toolchains`) with custom gdb path `\home\[user]\.cargo\bin\rust-gdb` 
- Add remote debug configuration with this toolchain
- Run `make DEBUG=1 run` and then run debug configuration in CLion