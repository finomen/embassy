//! Consumer Electronics Control (CEC)
#![macro_use]

use defmt::println;

use embassy_hal_internal::{Peripheral, PeripheralRef};
use embassy_time::Timer;
use embassy_usb_synopsys_otg::otg_v1::vals;
use stm32_metapac::RCC;
use crate::gpio::{AfType, AnyPin, Flex, OutputType, Pin, Pull, Speed};

use crate::pac::cec::{regs, Cec as Regs};
use crate::rcc;
use crate::rcc::{RccInfo, SealedRccPeripheral};
use crate::time::Hertz;
use crate::_generated::interrupt::typelevel::Interrupt;

/// CEC configuration.
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {

}

impl Default for Config {
    fn default() -> Self {
        Self {
        }
    }
}

impl Config {

}
/// CEC driver.
pub struct Cec<'d> {
    pub(crate) info: &'static Info,
    kernel_clock: Hertz,
    //_phantom: PhantomData<M>,
    cec: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d> Cec<'d> {
    pub fn new<T: Instance>(
        _peri: impl Peripheral<P = T> + 'd,
        cecPin: impl Peripheral<P = impl Pin> + 'd,
        config: Config,
    ) -> Self {
        let mut this = Self {
            info: T::info(),
            kernel_clock: T::frequency(),
            cec: None,
            //_phantom: PhantomData,
        };



        let mut flex = Flex::new(cecPin);
        flex.set_as_af_unchecked(1, AfType::output_pull(
            OutputType::OpenDrain,
            Speed::Low,
            Pull::None,
        ));

        this.enable_and_init(config);


        this
    }

    fn enable_and_init(&mut self, config: Config) {
        rcc::enable_and_reset::<crate::peripherals::CEC>();

        crate::pac::RCC.apbenr1().modify(|x|{x.set_pwren(true)});
        crate::pac::RCC.apbenr1().modify(|x|{x.set_cecen(true)});
        crate::pac::PWR.cr1().modify(|x|{x.set_dbp(true)});


        crate::pac::RCC.ccipr().modify(|x|{x.set_cecsel(rcc::mux::Cecsel::HSI_DIV_488)});


        unsafe { crate::interrupt::typelevel::CEC::enable(); }

        let regs = self.info.regs;
        let mut cfgr = regs::Cfgr(0);
        cfgr.set_sft(0);
        cfgr.set_rxtol(false);
        cfgr.set_brestp(true);
        cfgr.set_bregen(false);
        cfgr.set_lbpegen(false);
        cfgr.set_brdnogen(false);
        cfgr.set_sftopt(false);
        cfgr.set_lstn(true);
        cfgr.set_oar(0); // TODO: OWN ADDRESS
        regs.cfgr().write_value(cfgr);
        let ier = regs::Ier(0x1fff);
        regs.ier().write_value(ier);
        let mut cr = regs::Cr(0);
        cr.set_cecen(true);

        regs.cr().write_value(cr);

        println!("C HSI48 ON={} RDY={}",RCC.cr().read().hsi48on(), RCC.cr().read().hsi48rdy());

        println!("PWR.CR1: {=u32:x}", crate::pac::PWR.cr1().read().0);
        println!("PWR.CR2: {=u32:x}", crate::pac::PWR.cr2().read().0);
        println!("PWR.CR3: {=u32:x}", crate::pac::PWR.cr3().read().0);
        println!("PWR.CR4: {=u32:x}", crate::pac::PWR.cr4().read().0);
        println!("RCC.CR: {=u32:x}", crate::pac::RCC.cr().read().0);
        println!("RCC.ICSCR: {=u32:x}", crate::pac::RCC.icscr().read().0);
        println!("RCC.CFGR: {=u32:x}", crate::pac::RCC.cfgr().read().0);
        println!("RCC.PLLCFGR: {=u32:x}", crate::pac::RCC.pllcfgr().read().0);
        //println!("RCC.IOPENR: {=u32:x}", crate::pac::RCC.iopenr().read().0);
        println!("RCC.AHBENR: {=u32:x}", crate::pac::RCC.ahbenr().read().0);
        println!("RCC.APBENR1: {=u32:x}", crate::pac::RCC.apbenr1().read().0);
        println!("RCC.APBENR2: {=u32:x}", crate::pac::RCC.apbenr2().read().0);

        println!("RCC.CCIPR: {=u32:x}", crate::pac::RCC.ccipr().read().0);

        println!("CR: {=u32:x}", regs.cr().read().0);
        println!("CFGR: {=u32:x}", regs.cfgr().read().0);
        println!("IER: {=u32:x}", regs.ier().read().0);
        println!("ISR: {=u32:x}", regs.isr().read().0);
    }

    pub fn poll(&mut self) {
        let regs = self.info.regs;
        println!("ISR: {=u32:x}", regs.isr().read().0);
        println!("RXDR: {=u32:x}", regs.rxdr().read().0);
        //println!("TXDR: {=u32:x}", regs.txdr().read().0);
    }
}

trait RegsExt {
    fn  cfg<W>(&self) -> *mut W;
}

impl RegsExt for Regs {
    fn  cfg<W>(&self) -> *mut W {
        let cr = self.cfgr();
        cr.as_ptr() as *mut W
    }
}

pub(crate) struct Info {
    pub(crate) regs: Regs,
    pub(crate) rcc: RccInfo,
}

struct State {}

impl State {
    const fn new() -> Self {
        Self {}
    }
}

peri_trait!();

pin_trait!(CecPin, Instance);

foreach_peripheral!(
    (cec, $inst:ident) => {
        peri_trait_impl!($inst, Info {
            regs: crate::pac::$inst,
            rcc: crate::peripherals::$inst::RCC_INFO,
        });
    };
);

/*

<lvl> PWR.CR1: 308
<lvl> PWR.CR2: 100
<lvl> PWR.CR3: 8000
<lvl> PWR.CR4: 0
<lvl> RCC.CR: 3c30500
<lvl> RCC.ICSCR: 408a
<lvl> RCC.CFGR: 12
                 0x36021003
<lvl> RCC.PLLCFGR: 33031003
<lvl> RCC.AHBENR: 103
<lvl> RCC.APBENR1: 11000000
<lvl> RCC.APBENR2: 10001
<lvl> CR: 1
<lvl> CFGR: 80000010
<lvl> IER: 1fff
<lvl> ISR: 0
<lvl> Loop
<lvl> ISR: 0
<lvl> RXDR: 0

 */