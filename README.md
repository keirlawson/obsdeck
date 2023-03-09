# obsdeck

A little program for my personal OBS macros

## Installation

Enable running as non-root:

```sh
sudo cp 40-streamdeck.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```
