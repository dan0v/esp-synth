# Prerequisites

## Linux

Install packages and then allow your user to write to serial port, according to [espflash README](https://github.com/esp-rs/espflash/blob/main/espflash/README.md#permissions-on-linux), summarized below.

### Ubuntu 24.04.1

```bash
sudo apt update
sudo apt install curl build-essential libssh-dev pkg-config libudev-dev -y
```
Add your user to the required group to interact with /dev/tty...
```bash
sudo usermod -aG dialout $USER
```

### Fedora 40

```bash
sudo dnf install gcc openssl-devel perl systemd-devel -y
```
Add your user to the required group to interact with /dev/tty...
```bash
sudo usermod -aG dialout $USER
```

Remember to reboot after these changes.

## MacOS

Install Homebrew and Xcode dependencies.

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
xcode-select --install
```

Install pkgconf.

```bash
brew install pkgconf
```
In order to communicate with the board, some additional drivers are required.

1. Download the [CH34 drivers](https://www.wch-ic.com/downloads/CH34XSER_MAC_ZIP.html)
2. Open the .dmg image, drag the application to your Applications folder, and run the application from there.
3. After accepting Apple's various security warnings, click 'install'.
4. Once you see a success message, reboot your computer.
5. When flashing your board, select the ...wch...tty option.

## Windows

We recommend [setting up WSL](https://learn.microsoft.com/en-us/windows/wsl/install) and following the next set of instructions. Make sure to install WSL2.

## WSL

Inside your WSL distro of choice, install the aforementioned Linux prerequisites.

Additionally, you will need to install [usbipd-win](https://github.com/dorssel/usbipd-win) on the host (Windows) machine, to allow WSL to use USB devices.

Some additional steps are required to connect the board to WSL when you plug it in:

1. Plug in the board.
2. Open a powershell window **as administrator** and run the command `usbipd list`.
3. You should see a table with several columns: `BUSID`, `VID:PID`, `DEVICE`, and `STATE`. Find the microcontroller under `DEVICE` and note down the corresponding `BUSID`. If you don't know which row your microcontroller is in the list, try unplugging it and running the command again. One of the entries should disappear from the table. This is probably your microcontroller.
4. In the same powershell window, run `usbipd bind --busid="<BUSID>"`, where `<BUSID>` is the ID you noted down in the previous step. Run `usbipd list` again, and the row corresponding to your microcontroller should say `Shared`.
5. Close the powershell window, and open a new one, **not as administrator**.
6. Run `usbipd attach --wsl --busid "<BUSID>`, where `<BUSID>` is the ID you noted down in step 4. You should see something like the following output:
```
usbipd: info: Using WSL distribution 'Ubuntu-22.04' to attach; the device will be available in all WSL 2 distributions.
usbipd: info: Using IP address 172.21.112.1 to reach the host.
```

# Rust dependencies

Dependencies list for reference: [esp-idf-template](https://github.com/esp-rs/esp-idf-template#prerequisites), summarized below.

Install Rust via RustUp (default `stable` version)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Restart your terminal and run the following commands one by one:
```bash
cargo install cargo-generate
cargo install ldproxy
cargo install espup
cargo install espflash
cargo install espmonitor
espup install
```

# IDE

## VSCode (preferred)

Install the `rust-analyzer` vscode extension
Add to your vscode settings.json
```
"rust-analyzer.server.extraEnv": { "RUSTUP_TOOLCHAIN": "stable" },
"rust-analyzer.cargo.features": "all",
"rust-analyzer.cargo.buildScripts.enable": true,
"rust-analyzer.procMacro.enable": true,
"rust-analyzer.check.allTargets": false,
```

## RustRover
The toolchain can be changed in (Settings -> Rust).
