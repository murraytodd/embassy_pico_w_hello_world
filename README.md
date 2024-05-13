# Async Embedded Development for the Raspberry Pi Pico W in Rust

Features (developed in order of commits) : 

## Standard "blink" hello world based on an external LED

It is very common to start with a blinking example on embedded devices (Arduinos, etc.) because lots of
boards come with an on-board LED that's attached to one of the standard GPIOs. This is the case with the
Raspberry Pi Pico, but on the Pico W they had to rearrange which pins were used for what in order to 
support the on-board WIFI chip. In doing so, the on-board LED is actually attached to a GPIO that belongs
to the CYW43 chip instead of the main RP2040 chip!

To keep things simple, I maintained the basic blink example, but had to wire an external LED (along with
the mandatory resistor) to one of the RP2040 GPIOs. Looking at the 
[pinout](https://datasheets.raspberrypi.com/picow/PicoW-A4-Pinout.pdf) I noticed that GPIO22 was the only
pin not used by other peripherals (spi, i2c, uart, etc.) so I went ahead and chose that.

## Onboard "blink" via the CYW43

As mentioned, the onboard LED is attached to a GPIO (0) on the CYW43 networking chip. This is actually
a pretty complicated task because sending instructions to the CYW43 requires an SPI communication on an
SPI channel other than the standard SPI0 or SPI1 channels built into the RP2040. (I'm guessing they didn't 
want to waste those channels on such a simple and singular use case.)

So to make things complicated, we have to use the RP2040's build in PIO (programmable I/O) state machines
to create a makeshift SPI channel. That's pretty complex for a hello world example, sort of defeating the
purpose of "hello world", but okay, we're going to do it anyway! Besides, some of the complex setup in 
the code is needed anyway for our networking examples that will come next.

## Networking configuration with DCHP and Hostname

This chunk of functionality brings all the networking goodness up. (Not a small task!) We start by 
joining the WIFI network, then we ask the DHCP server to provide the network configuration. As an added
detail, we enable an optional feature to provide the DHCP server the name of our device&mdash;defaulting to 
'picow'. (We do this to enable other devices to find the Pico's IP address via a network router's
internal/local DNS services, something offered by most local network routers these days.) We finally test 
the resolution of another server on the network. This will be a server that we later try to send 
information to. We're not doing any sending or receiving yet&mdash;we're just verifying that we can use
DNS to find the right IP address.

Note that all configuration names and passwords are stored in the `netsetup.rs` file. You should change
this file directly to suit your needs.