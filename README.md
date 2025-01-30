# GigaCenter
Monitor system info, control fan mode, and battery threshold of your Gigabyte laptop on Linux

## ⚠️Disclaimer
**This project is not affiliated with Gigabyte (or Aorus) in any way**

Currently the software is only tested on **Aorus 16X (2024)**. If you have another laptop model, use it at your own risk. We are now responsible for any damage to your hardware

## 🚀Features
- Hardware properties monitoring
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

## 🖥️Tested laptops
- Aorus 16X (2024)

## 🛠️Contributing:
If you tested it on another laptop model and it worked fine, please open the corresponding issue/PR

Any help is welcome. Feel free to open issue/PR with any questions or suggestions
