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

