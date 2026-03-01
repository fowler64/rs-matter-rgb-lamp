use core::cell::{Cell, RefCell};

#[cfg(feature = "defmt")]
use defmt::{debug, error};
#[cfg(feature = "log")]
use log::debug;

use rs_matter_embassy::matter::dm::Cluster;
use rs_matter_embassy::matter::dm::clusters::on_off::{self, OnOffHooks, StartUpOnOffEnum};
use rs_matter_embassy::matter::error::Error;
use rs_matter_embassy::matter::tlv::Nullable;
use rs_matter_embassy::matter::with;

use esp_hal::gpio::{Level, Output};

use embassy_time::Timer;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LightHandler<'a> {
    light_output: RefCell<Output<'a>>,
    // OnOff Attributes
    on_off: Cell<bool>,
    start_up_on_off: Cell<Option<StartUpOnOffEnum>>,
}

impl<'a> LightHandler<'a> {
    pub fn new(light_output: Output<'a>) -> Self {
        Self {
            light_output: RefCell::new(light_output),
            on_off: Cell::new(true),
            start_up_on_off: Cell::new(None),
        }
    }
}

impl<'a> OnOffHooks for LightHandler<'a> {
    const CLUSTER: Cluster<'static> = on_off::FULL_CLUSTER
        .with_revision(6)
        .with_features(on_off::Feature::LIGHTING.bits())
        .with_attrs(with!(
            required;
            on_off::AttributeId::OnOff
            | on_off::AttributeId::GlobalSceneControl
            | on_off::AttributeId::OnTime
            | on_off::AttributeId::OffWaitTime
            | on_off::AttributeId::StartUpOnOff
        ))
        .with_cmds(with!(
            on_off::CommandId::Off
                | on_off::CommandId::On
                | on_off::CommandId::Toggle
                | on_off::CommandId::OffWithEffect
                | on_off::CommandId::OnWithRecallGlobalScene
                | on_off::CommandId::OnWithTimedOff
        ));

    fn on_off(&self) -> bool {
        self.on_off.get()
    }

    // todo this method should probably return an error `.map_err(|_| Error::new(ErrorCode::Busy))`
    fn set_on_off(&self, on: bool) {
        debug!("Setting device to {}", on);
        self.light_output.borrow_mut().set_level(match on {
            true => Level::High,
            false => Level::Low,
        });
        self.on_off.set(on);
        debug!("OnOff state set to: {}", on);
    }

    fn start_up_on_off(&self) -> Nullable<on_off::StartUpOnOffEnum> {
        match self.start_up_on_off.get() {
            Some(value) => Nullable::some(value),
            None => Nullable::none(),
        }
    }

    fn set_start_up_on_off(&self, value: Nullable<on_off::StartUpOnOffEnum>) -> Result<(), Error> {
        self.start_up_on_off.set(value.into_option());
        Ok(())
    }

    async fn handle_off_with_effect(&self, _effect: on_off::EffectVariantEnum) {
        // no effect
    }

    async fn run<F: Fn(on_off::OutOfBandMessage)>(&self, _: F) {
        loop {
            Timer::after_secs(10).await;
        }
    }
}
