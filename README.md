# HacalOS
Simple x64 UEFI OS
## Building
> Environment: Linux (or WSL)  
Tested on: WSL Ubuntu 20.04.4 LTS
- Pull submodules `git submodule update --init --recursive`
- Install all required dependencies:
  - Apt: `sudo apt install make gcc build-essential mtools xorriso`
  - Or with specific versions: `sudo apt install make=4.2.1-1.2 gcc=4:9.3.0-1ubuntu2 build-essential mtools=4.0.24-1 xorriso=1.5.2-1`
- Install rust
- Build iso image `make os.iso`
## Run with QEmu
> Tested on: WSL with VcXsrv on Windows 10
- Install all required dependencies:
  - Apt: `sudo apt install qemu-kvm ovmf`
- Run qemu `make run`