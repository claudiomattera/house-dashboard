Display Configuration
====

The project was developed using a 2.2 in TFT Display HAT v0.2a with the following features:

* Dimensions: 65mm×56.5mm (standard Raspberry Pi HAT size),
* Resolution: 320×240, 2.2 inch, High PPI display screen,
* 6 buttons,
* IR function,
* 1× I²C Out Connector.


Framebuffer Configuration
----

The display will be accessible as a framebuffer device on `/dev/fb1`.

*   Enable the I²C interface by adding the following line to `/boot/config.txt`

        dtparam=i2c_arm=on, spi=on

*   Configure the module by adding the following line to `/etc/modules`

        fbtft_device name=pitft rotate=270 speed=48000000 fps=30

*   Enable the console on the screen by adding the following parameters to the kernel command line in `/boot/cmdline.txt`

        fbcon=map:10 fbcon=font:VGA8x8


Buttons Configuration
----

The display comes with 6 buttons, corresponding to the following GPIO pins:

* 23
* 22
* 24
* 5
* 4 (side)
* 17 (side)

They can be interacted with through GPIO, or they can be configured to generate key event as a regular keyboard.

For the latter, create an overlay that defines the associated keys (an example is available at [`./buttons/display-buttons.dts`](./buttons/display-buttons.dts)).
Compile by running `make` in that directory, and copy the dtbo file to `/boot/overlays/display-buttons.dtbo` and add the following line to `/boot/config.txt`

    dtoverlay=display-buttons

Test whether the buttons are generating events by running `evtest`:

    No device specified, trying to scan all of /dev/input/event*
    Not running as root, no devices may be available.
    Available devices:
    /dev/input/event0:      soc:powerbtn
    Select the device event number [0-0]: 0
    Input driver version is 1.0.1
    Input device ID: bus 0x19 vendor 0x1 product 0x1 version 0x100
    Input device name: "soc:powerbtn"
    Supported events:
      Event type 0 (EV_SYN)
      Event type 1 (EV_KEY)
        Event code 116 (KEY_POWER)
    Properties:
    Testing ... (interrupt to exit)
    Event: time 1496937683.446926, type 1 (EV_KEY), code 116 (KEY_POWER), value 1
    Event: time 1496937683.446926, -------------- EV_SYN ------------
    Event: time 1496937683.806920, type 1 (EV_KEY), code 116 (KEY_POWER), value 0
    Event: time 1496937683.806920, -------------- EV_SYN ------------

Optionally, system-wide actions can be configured to react to specific buttons by adding tags to them.
Fetch the `ID_PATH` for the button by running `udevadm info /dev/input/event0`

    P: /devices/platform/soc/soc:powerbtn/input/input0/event0
    N: input/event0
    S: input/by-path/platform-soc:powerbtn-event
    E: BACKSPACE=guess
    E: DEVLINKS=/dev/input/by-path/platform-soc:powerbtn-event
    E: DEVNAME=/dev/input/event0
    E: DEVPATH=/devices/platform/soc/soc:powerbtn/input/input0/event0
    E: ID_INPUT=1
    E: ID_INPUT_KEY=1
    E: ID_PATH=platform-soc:powerbtn
    E: ID_PATH_TAG=platform-soc_powerbtn
    E: LIBINPUT_DEVICE_GROUP=19/1/1/100:gpio-keys
    E: MAJOR=13
    E: MINOR=64
    E: SUBSYSTEM=input
    E: USEC_INITIALIZED=48119
    E: XKBLAYOUT=gb
    E: XKBMODEL=pc105

Create a udev rule in `/etc/udev/rules.d/powerbtn.rules` to add a tag to the button

    ACTION=="remove", GOTO="powerbtn_end"

    SUBSYSTEM=="input", KERNEL=="event*", ENV{ID_PATH}=="platform-soc:powerbtn", ATTRS{keys}=="*", TAG+="power-switch" 

    LABEL="powerbtn_end"

Confirm that the tags are applied by running `udevadm info /dev/input/event0 | grep TAGS`

    E: TAGS=:power-switch:

See the [documentation for the `HandlePowerKey` options](https://www.freedesktop.org/software/systemd/man/logind.conf.html) for logind.


For more information refer to [this discussion](https://www.raspberrypi.org/forums/viewtopic.php?t=185571) on the Raspberry Pi forum.


InfraRed Received Configuration
----

*   Enable the LIRC overlay by adding the following line to `/boot/config.txt`

        dtoverlay=lirc-rpi, gpio_in_pin=26

*   Edit LRIC configuration file `/etc/lirc/hardware.conf`

        LIRCD_ARGS="--uinput"
        DRIVER="default"
        DEVICE="/dev/lirc0"
        MODULES="lirc_rpi"
