#![no_std]
#![no_main]

mod rand;

use esp8266_hal::gpio::{Gpio0, Gpio2, Input, InterruptMode, Output, PullUp, PushPull};
use esp8266_hal::interrupt::{enable_interrupt, InterruptType};
use esp8266_hal::prelude::*;
use esp8266_hal::target::Peripherals;
use esp_println::println;
use panic_halt as _;
use xtensa_lx::mutex::{CriticalSectionMutex, Mutex};

static LED: CriticalSectionMutex<Option<Gpio2<Output<PushPull>>>> = CriticalSectionMutex::new(None);
static BUTTON: CriticalSectionMutex<Option<Gpio0<Input<PullUp>>>> = CriticalSectionMutex::new(None);
static INTERRUPT_COUNTER: CriticalSectionMutex<u32> = CriticalSectionMutex::new(0);

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = dp.GPIO.split();
    let mut led = pins.gpio2.into_push_pull_output();
    let mut button = pins.gpio0.into_pull_up_input();
    let (mut timer1, mut timer2) = dp.TIMER.timers();

    led.set_high().unwrap();
    (&LED).lock(|led_locked| *led_locked = Some(led));
    button.set_interrupt_mode(InterruptMode::NegativeEdge);
    (&BUTTON).lock(|btn| *btn = Some(button));

    timer1.disable_interrupts();
    enable_interrupt(InterruptType::GPIO);
    enable_interrupt(InterruptType::TIMER1);

    let mut rng = rand::XorRand::new(1);
    let delay = rng.next_between(500, 3000);
    println!("\n\n");
    println!("delaying for {}ms", delay);
    timer2.delay_ms(delay);
    println!("delaying finished");

    (&LED).lock(|led| led.as_mut().unwrap().set_low().unwrap());
    timer1.enable_interrupts();
    timer1.start(esp8266_hal::time::Milliseconds(1));
    loop {}
}

#[interrupt(gpio)]
fn button() {
    println!("button press interrupt");
    (&LED).lock(|led| led.as_mut().unwrap().set_high().unwrap());
    (&BUTTON).lock(|btn| btn.as_mut().unwrap().clear_interrupt());
    println!("took {}ms", (&INTERRUPT_COUNTER).lock(|n| *n));
}

#[interrupt(timer1)]
fn timer() {
    (&INTERRUPT_COUNTER).lock(|n| *n += 1);
}
