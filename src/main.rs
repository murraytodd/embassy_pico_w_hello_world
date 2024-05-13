#![no_std]
#![no_main]

mod netsetup;
use embassy_net::{Config, IpEndpoint};

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, SendError, UdpSocket};
use embassy_net::{Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio;
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use rand_core::RngCore;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[cortex_m_rt::pre_init]
unsafe fn before_main() {
    // Soft-reset doesn't clear spinlocks. Clear the one used by critical-section
    // before we hit main to avoid deadlocks when using a debugger
    embassy_rp::pac::SIO.spinlock(31).write_value(1);
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program start");
    let p = embassy_rp::init(Default::default());

    // PICO W WIFI CHIP SETUP
    // Setup PIO-based SPI needed to communicate with the CYW43 networking chip
    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // PICO W WIFI NETWORKING SERVICES SETUP
    let config = Config::dhcpv4(netsetup::dhcp_with_host_name());
    let seed: u64 = RoscRng.next_u64();
    warn!("Random seed value seeded to 0x{=u64:#X}", seed);

    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new(); // Increase this if you start getting full socket ring errors.
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<4>::new()),
        seed,
    ));
    let mac_addr = stack.hardware_address();
    info!("Hardware configured. MAC Address is {}", mac_addr);

    unwrap!(spawner.spawn(net_task(stack))); // Start networking services thread

    loop {
        // JOIN THE WIFI NETWORK
        match control
            .join_wpa2(netsetup::WIFI_NETWORK, netsetup::WIFI_PASSWORD)
            .await
        {
            Ok(_) => {
                info!("Successfully joined {}", netsetup::WIFI_NETWORK);
                break;
            }
            Err(err) => {
                info!("Join failed with status={}", err.status);
            }
        }
    }

    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    match stack.config_v4() {
        Some(a) => info!("IP Address appears to be: {}", a.address),
        None => info!("No IP address assigned!"),
    }
    info!("DHCP is now up!");

    let server_addr = stack
        .dns_query(netsetup::SERVER_NAME, embassy_net::dns::DnsQueryType::A)
        .await;

    match server_addr {
        Ok(ref add) => info!("event server resolved to {}", add),
        Err(e) => info!("error resolving event server: {}", e),
    }
    let dest = server_addr.unwrap().first().unwrap().clone(); // this looks crazy to me

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut _buf = [0; 4096];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let destination = IpEndpoint {
        addr: dest,
        port: 9932,
    };

    socket.bind(9400).unwrap();

    let mut led = Output::new(p.PIN_22, Level::Low);

    loop {
        match socket.send_to("message".as_bytes(), destination).await {
            Ok(_) => info!("Message sent into the ether..."),
            Err(SendError::NoRoute) => info!("UDP No Route to Destination"),
            Err(SendError::SocketNotBound) => error!("UDP Socket not bound"),
        }

        info!("external led on, onboard off!");
        led.set_high();
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_secs(1)).await;

        info!("external led off, onboard on!");
        led.set_low();
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
