#![no_std]
#![no_main]

mod rand;

use esp8266_hal::gpio::{Gpio0, Gpio2, Input, InterruptMode, Output, PullUp, PushPull};
use esp8266_hal::interrupt::{enable_interrupt, InterruptType};
use esp8266_hal::target::Peripherals;
use esp8266_hal::{prelude::*, timer};
use esp_println::println;
use panic_halt as _;
use xtensa_lx::interrupt;

static mut LED: Option<Led> = None;
static mut BUTTON: Option<Gpio0<Input<PullUp>>> = None;
static mut TIMER1: Option<Timer> = None;
static mut INTERRUPT_COUNTER: u32 = 0;
static mut START_NEW_GAME: bool = true;

struct Timer {
    inner: timer::Timer1,
}

impl Timer {
    fn new(inner: timer::Timer1) -> Self {
        Self { inner }
    }

    fn start(&mut self) {
        self.inner.enable_interrupts();
        self.inner.start(esp8266_hal::time::Milliseconds(1));
    }

    fn stop(&mut self) {
        self.inner.disable_interrupts();
        self.inner.cancel().unwrap()
    }
}

struct Led {
    inner: Gpio2<Output<PushPull>>,
}

impl Led {
    fn new(inner: Gpio2<Output<PushPull>>) -> Self {
        Self { inner }
    }

    fn disable(&mut self) {
        self.inner.set_high().unwrap();
    }

    fn enable(&mut self) {
        self.inner.set_low().unwrap();
    }

    #[allow(unused)]
    fn is_enabled(&self) -> bool {
        self.inner.is_set_low().unwrap()
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_set_high().unwrap()
    }
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = dp.GPIO.split();
    let mut led = Led::new(pins.gpio2.into_push_pull_output());
    let mut button = pins.gpio0.into_pull_up_input();
    let (timer1, mut timer2) = dp.TIMER.timers();

    led.disable();
    button.set_interrupt_mode(InterruptMode::NegativeEdge);
    unsafe {
        LED = Some(led);
        BUTTON = Some(button);
        TIMER1 = Some(Timer::new(timer1));
    }

    enable_interrupt(InterruptType::GPIO);
    enable_interrupt(InterruptType::TIMER1);

    println!("\n\n");

    let mut rng = rand::XorRand::new(1);
    loop {
        if !interrupt::free(|_| unsafe { START_NEW_GAME }) {
            timer2.delay_ms(1000);
            continue;
        }

        println!("start");
        let delay = rng.next_between(500, 3000);
        println!("delaying for {}ms", delay);
        timer2.delay_ms(delay);

        interrupt::free(|_| unsafe {
            println!("enabling LED");
            LED.as_mut().unwrap().enable();
            START_NEW_GAME = false;
            TIMER1.as_mut().unwrap().start();
        })
    }
}

#[interrupt(timer1)]
fn timer() {
    interrupt::free(|_| unsafe {
        INTERRUPT_COUNTER += 1;
    });
}

#[interrupt(gpio)]
fn button() {
    interrupt::free(|_| unsafe {
        println!("button press interrupt");
        BUTTON.as_mut().unwrap().clear_interrupt();

        if LED.as_mut().unwrap().is_disabled() {
            println!("too soon");
            return;
        }

        let interruptions = INTERRUPT_COUNTER;
        TIMER1.as_mut().unwrap().stop();
        INTERRUPT_COUNTER = 0;
        LED.as_mut().unwrap().disable();

        println!("you pressed the button after {}ms", interruptions);
        START_NEW_GAME = true;
    });
}
