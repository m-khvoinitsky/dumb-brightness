# Dumb Brightness

Very simple program to control brightness of your screen, keyboard or whatever exposed via sysfs (Linux only)
## How it works?

It just reads files `brightness` and `max_brightness` (which are usually found somewhere in sysfs), and then writes calculated value back to `brightness` according to command line arguments. It also shows fancy [Desktop Notification](https://developer.gnome.org/notification-spec/) with progress bar if your notification server supports it.

It avoids running multiple instances of itself (via flock) and properly updates previous notification, both separately for different working directories.

## Usage
### Basic usage
```sh
cd /sys/class/backlight/intel_backlight
dumb-brightness --increase 10
dumb-brightness --decrease 10
```

Instead of changing current working directory, you can specify it with `-w`, `--working-directory` argument:
```sh
dumb-brightness -w /sys/class/backlight/intel_backlight --increase 10
dumb-brightness -w /sys/class/backlight/intel_backlight --decrease 10
```

### More features: notification icon, title, duration and smooth transition:
```sh
dumb-brightness -w /sys/class/backlight/intel_backlight --icon display-brightness-symbolic --decrease 10 --steps 10 --step-interval 10 --title 'Screen Brightness --duration 1.5'
dumb-brightness -w /sys/devices/platform/applesmc.768/leds/smc::kbd_backlight --icon keyboard-brightness-symbolic --increase 10 --steps 10 --step-interval 10 --title 'Keyboard Brightness'
```

### Using with [Sway](https://swaywm.org/) (it should work with [i3](https://i3wm.org/) too):
(real world example for MacBook Pro 2015)
```
bindsym XF86MonBrightnessUp exec --no-startup-id "dumb-brightness -w /sys/class/backlight/intel_backlight --increase 10 --steps 10 --step-interval 10 --icon display-brightness-symbolic --title 'Screen Brightness'"
bindsym XF86MonBrightnessDown exec --no-startup-id "dumb-brightness -w /sys/class/backlight/intel_backlight --decrease 10 --steps 10 --step-interval 10 --icon display-brightness-symbolic --title 'Screen Brightness'"
bindsym XF86KbdBrightnessUp exec --no-startup-id "dumb-brightness -w /sys/devices/platform/applesmc.768/leds/smc::kbd_backlight --increase 10 --steps 10 --step-interval 10 --icon keyboard-brightness-symbolic --title 'Keyboard Brightness'"
bindsym XF86KbdBrightnessDown exec --no-startup-id "dumb-brightness -w /sys/devices/platform/applesmc.768/leds/smc::kbd_backlight --decrease 10 --steps 10 --step-interval 10 --icon keyboard-brightness-symbolic --title 'Keyboard Brightness'"
```
