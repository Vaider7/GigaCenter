# GigaCenter
Monitor system info, control fan mode, and battery threshold of your Gigabyte laptop on Linux

![image](https://github.com/user-attachments/assets/82c2bba6-bcc3-4273-ba7d-bf4d1a6ab1c3)

## ⚠️Disclaimer
**This project is not affiliated with Gigabyte Technology (or sub-brand Aorus) in any way**

Currently the software is only tested on **Aorus 16X (2024)**. If you have another laptop model, use it at your own risk. We are now responsible for any damage to your hardware

## 🚀Features
- Hardware sensors monitoring
- Fan mode
- Battery threshold

## 📋Prerequirements
GigaCenter writes to EC (embedded controller). So make sure your kernel support `ec_sys` with `write_support`
```
sudo modprobe ec_sys write_support=1
```

## 📦Installation
The software is built in two flavors: the first is default both CLI and GUI application with almost no dependencies, the second is CLI only statically linked binary. So the steps to install the application:
1. Download acrhive from [Release page](https://github.com/Vaider7/GigaCenter/releases)
2. Unarhieve it
3. Run `./gigacenter -d install`. This will add binary to PATH and install the needed background helper as a systemd service
4. Done

Third step is optional for CLI usage and required for GUI 

## 💡Usage
CLI usage:
```
$ gigacenter --help

Manage your Gigabyte laptop fan speed and battery threshold on Linux

Usage: gigacenter [OPTIONS]

Options:
  -s, --show
          Show current machine state (fan speed, temperature, etc.)

  -f, --fan-mode <FAN_MODE>
          Set fan speed mode

          [possible values: normal, eco, power, turbo, unsupported]

  -b, --bat-threshold <THRESHOLD>
          Set battery threshold. Takes values from 60 to 100 (in percent)

  -d, --daemon <DAEMON_COMMAND>
          Possible values:
          - run:     Run daemon
          - install: Install systemd service needed to use gigacenter without root permissions
          - remove:  Remove binary and systemd service

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

NOTE: Currently it's tested for Aorus 16X. For other models, use it at your own risk!
```

## 🖥️Tested laptops
- Aorus 16X (2024)

## 🛠️Contributing:
If you tested it on another laptop model and it worked fine, please open the corresponding issue/PR

Any help is welcome. Feel free to open issue/PR with any questions or suggestions
