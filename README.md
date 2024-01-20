# HacalOS
Simple x64 UEFI OS
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