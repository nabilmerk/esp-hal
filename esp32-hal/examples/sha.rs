//! Demonstrates the use of the SHA peripheral and compares the speed of
//! hardware-accelerated and pure software hashing.

#![no_std]
#![no_main]

use esp32_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    sha::{Sha, ShaMode},
    timer::TimerGroup,
    Rng,
    Rtc,
};
use esp_backtrace as _;
use esp_println::{print, println};
use nb::block;
use sha2::Digest;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt = timer_group0.wdt;

    let mut rng = Rng::new(peripherals.RNG);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();

    let shamode = [
        ShaMode::SHA1,
        ShaMode::SHA256,
        ShaMode::SHA384,
        ShaMode::SHA512,
    ];
    let mut hw_hasher = Sha::new(
        peripherals.SHA,
        ShaMode::SHA256,
        &mut system.peripheral_clock_control,
    );
    let mut sw_hasher_sha1 = sha1::Sha1::new();
    let mut sw_hasher_sha256 = sha2::Sha256::new();
    let mut sw_hasher_sha384 = sha2::Sha384::new();
    let mut sw_hasher_sha512 = sha2::Sha512::new();

    let mut src = [0_u8; 1024];
    rng.read(src.as_mut_slice()).unwrap();
    // println!("SHA256 input {:02X?}", src);

    // Short hashes can be created by decreasing the output buffer to the desired
    // length
    let mut output = [0u8; 32];

    println!("Beginning stress tests...");
    for mode in shamode {
        hw_hasher.setmode(mode);
        print!(
            "Testing length from 0 to {:?} bytes for {:?}...",
            src.len(),
            mode
        );
        for i in 0..src.len() + 1 {
            let (nsrc, _) = src.split_at(i);
            let mut remaining = nsrc;
            while remaining.len() > 0 {
                remaining = block!(hw_hasher.update(remaining)).unwrap();
            }
            block!(hw_hasher.finish(output.as_mut_slice())).unwrap();
            match mode {
                ShaMode::SHA1 => {
                    sw_hasher_sha1.update(nsrc);
                    let soft_result = sw_hasher_sha1.finalize_reset();
                    for (a, b) in output.iter().zip(soft_result) {
                        assert_eq!(
                            *a, b,
                            "Sha1 failed during the {:?}th test with {:02X?} not equal to {:02X?}",
                            i, *a, b
                        );
                    }
                }
                ShaMode::SHA256 => {
                    sw_hasher_sha256.update(nsrc);
                    let soft_result = sw_hasher_sha256.finalize_reset();
                    for (a, b) in output.iter().zip(soft_result) {
                        assert_eq!(
                            *a, b,
                            "Sha256 failed during the {:?}th test with {:02X?} not equal to {:02X?}",
                            i, *a, b
                        );
                    }
                }
                ShaMode::SHA384 => {
                    sw_hasher_sha384.update(nsrc);
                    let soft_result = sw_hasher_sha384.finalize_reset();
                    for (a, b) in output.iter().zip(soft_result) {
                        assert_eq!(
                            *a, b,
                            "Sha384 failed during the {:?}th test with {:02X?} not equal to {:02X?}",
                            i, *a, b
                        );
                    }
                }
                ShaMode::SHA512 => {
                    sw_hasher_sha512.update(nsrc);
                    let soft_result = sw_hasher_sha512.finalize_reset();
                    for (a, b) in output.iter().zip(soft_result) {
                        assert_eq!(
                            *a, b,
                            "Sha512 failed during the {:?}th test with {:02X?} not equal to {:02X?}",
                            i, *a, b
                        );
                    }
                }
            }
        }
        print!("ok");
    }
    println!("Finished stress tests!");

    loop {}
}
