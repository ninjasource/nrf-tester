#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(alloc_error_handler)]

mod setup;
use setup::*;

use core::mem;
use cortex_m_rt::entry;
use defmt::{info, panic, unwrap};

use embassy::executor::Executor;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use futures::pin_mut;
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::{raw, Softdevice};

use embassy::interrupt::InterruptExt;
use embassy::util::Steal;
use embassy_nrf::{interrupt, peripherals, rtc};

static EXECUTOR: Forever<Executor> = Forever::new();

#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[nrf_softdevice::gatt_server(uuid = "9e7312e0-2354-11eb-9f10-fbc30a62cf38")]
struct FooService {
    #[characteristic(uuid = "9e7312e0-2354-11eb-9f10-fbc30a63cf38", read, write, notify)]
    foo: u16,
}

async fn run_bluetooth(sd: &'static Softdevice, server: &FooService) {
    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x09, 0x18,
    ];

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        };
        let conn = unwrap!(peripheral::advertise(sd, adv, &config).await);

        info!("advertising done!");

        let res = gatt_server::run(&conn, server, |e| match e {
            FooServiceEvent::FooWrite(val) => {
                info!("wrote foo level: {}", val);
                if let Err(e) = server.foo_notify(&conn, val + 1) {
                    info!("send notification error: {:?}", e);
                }
            }
            FooServiceEvent::FooNotificationsEnabled => info!("notifications enabled"),
            FooServiceEvent::FooNotificationsDisabled => info!("notifications disabled"),
        })
        .await;

        if let Err(e) = res {
            info!("gatt_server run exited with error: {:?}", e);
        }
    }
}

#[embassy::task]
async fn bluetooth_task(sd: &'static Softdevice) {
    let server: FooService = unwrap!(gatt_server::register(sd));

    loop {
        info!("Bluetooth ON!");

        let bluetooth_fut = run_bluetooth(sd, &server);

        let timer_fut = async {
            info!("About to wait for timer");
            Timer::after(Duration::from_ticks(64000)).await;
            info!("Timer fired!");
        };

        pin_mut!(bluetooth_fut);
        pin_mut!(timer_fut);

        // Select the two futures.
        futures::future::select(bluetooth_fut, timer_fut).await;
        info!("Done waiting");
    }
}

static RTC: Forever<rtc::RTC<peripherals::RTC1>> = Forever::new();
static ALARM: Forever<rtc::Alarm<peripherals::RTC1>> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Started");

    // setup Timer
    unsafe { embassy_nrf::system::configure(Default::default()) };
    let irq = interrupt::take!(RTC1);
    irq.set_priority(interrupt::Priority::Level3); // levels 0-1 are reserved for the softdevice
    let rtc = unsafe { embassy_nrf::peripherals::RTC1::steal() };
    let rtc = RTC.put(rtc::RTC::new(rtc, irq));
    rtc.start();
    unsafe { embassy::time::set_clock(rtc) };
    let alarm = ALARM.put(rtc.alarm0());

    // config for softdevice
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
            rc_ctiv: 0,
            rc_temp_ctiv: 0,
            accuracy: 7,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let (sdp, _p) = take_peripherals();
    let sd = Softdevice::enable(sdp, &config);

    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);
    executor.run(|spawner| {
        unwrap!(spawner.spawn(softdevice_task(sd)));
        unwrap!(spawner.spawn(bluetooth_task(sd)));
    });
}
