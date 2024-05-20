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

## Sending data packets via UDP to a server

In my opinion, this is the fundamental task that a person would want to buy a Raspberry Pi Pico W for: 
to periodically gather some information, such as sensor readings, and to send it back to some server
wirelessly. This makes it possible to be "untethered" and not need physical connection via USB cables
or UART wires or anything like that.

I had previously created an example where the remote server was using the TCP protocol, but frankly, 
I don't need the fault tollerance of TCP. Depending on the application, I might not care if there's a
running service somewhere ready to receive my messages. And that's the beauty of UDP: as long as
there's theoretically a route to send UDP packets with, that's good enough for me!

Here's a simple python UDP server program that I ran on the remote machine:

```python
import socket

# Create a UDP socket
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

# Bind the socket to the port
server_address = ("0.0.0.0", 9932)
sock.bind(server_address)

# Listen for incoming messages
while True:
    data, address = sock.recvfrom(1024)

    # Print the message and address
    print("Received message: %s" % data)
    print("From address: %s" % (address,))
```

## Basic I2C Operations

I have a device (BMP180 pressure monitor) that I used as a reference. It has a basic command that
you can use to query the chip ID where the expected value (0x55) is known. (It's meant to help
verify you're talking to the device you're expecting.) This was meant to test the basic ability
to configure the I2C bus and perform a basic write and read.

Rather than using the pins used in the embassy_rp example (associated with I2C1) I used GPIO pins
4 and 5 which are assocated with I2C0.