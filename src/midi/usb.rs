use embassy_futures::join::join;
use embassy_usb::{class::midi::MidiClass, driver::EndpointError, Builder};
use esp_backtrace as _;
use esp_hal::{
    get_core,
    otg_fs::{
        asynch::{Config, Driver},
        Usb,
    },
};
use esp_println::println;
use midi_msg::MidiMsg;

use crate::midi::MIDI_EVENTS;

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

fn create_usb_config<'d>() -> embassy_usb::Config<'d> {
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("TNG");
    config.product = Some("ESP32S3 Synth");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}

#[embassy_executor::task]
pub async fn handle_usb(usb: Usb<'static>) {
    println!("starting usb handler on {:?}", get_core());
    let mut ep_out_buffer = [0u8; 1024];
    let config = Config::default();
    let driver = Driver::new(usb, &mut ep_out_buffer, config);

    let usb_config = create_usb_config();
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut builder = Builder::new(
        driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut class = MidiClass::new(&mut builder, 1, 1, 64);
    let mut usb = builder.build();

    let usb_fut = usb.run();

    // Use the Midi class!
    let midi_fut = async {
        loop {
            class.wait_connection().await;
            println!("Connected");
            let _ = midi_print(&mut class).await;
            println!("Disconnected");
        }
    };
    join(usb_fut, midi_fut).await;
}

async fn midi_print<'d>(class: &mut MidiClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        println!("Waiting for data");
        let n = class.read_packet(&mut buf).await?;

        if let Ok((msg, _)) = MidiMsg::from_midi(&buf[1..n]) {
            println!("MIDI: {:?}", msg);
            MIDI_EVENTS.send(msg).await;
        } else {
            println!("MIDI: error parsing event: {:x?}", &buf[1..n]);
        }
    }
}
