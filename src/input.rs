use alloc::{boxed::Box, vec, vec::Vec};
use embassy_time::{Duration, Ticker};
use esp_hal::{
    analog::adc::{
        Adc, AdcCalScheme, AdcChannel, AdcConfig, AdcPin, Attenuation, CalibrationAccess,
    },
    gpio::AnalogPin,
    peripheral::Peripheral,
    peripherals::ADC1,
};

use crate::{
    filters::traits::{Filter, Filterable},
    midi::send_control,
};

/// Simple first order
#[derive(Clone)]
pub struct AdcFilter {
    y: u16,
    a: u32,
    b: u32,
}

impl AdcFilter {
    pub fn new(alpha: f32) -> Self {
        // a = 2^16 * alpha
        let a = (u16::MAX as f32 * alpha) as u32;
        // b = 2^16 * (1 - alpha)
        let b = u16::MAX as u32 - a;
        Self { y: 0, a, b }
    }
}

impl Filter for AdcFilter {
    type In = u16;
    type Out = u16;
    fn filter(&mut self, x: u16) -> u16 {
        // y = (a * x + b * y) / 2^16
        // shifting right by 16 is equivalent to dividing by 2^16
        // calculation must be as u32 to avoid overflow
        self.y = ((self.a * x as u32 + self.b * self.y as u32) >> 16) as u16;
        self.y
    }
}

#[derive(Clone)]
pub struct AdcGate {
    trigger_threshold: u16,
    sustain_threshold: u16,
    last: u16,
    active: bool,
}

impl AdcGate {
    pub fn new(trigger_threshold: u16, sustain_threshold: u16) -> Self {
        Self {
            trigger_threshold,
            sustain_threshold,
            last: 0,
            active: false,
        }
    }
}

impl Filter for AdcGate {
    type In = u16;
    type Out = Option<u16>;
    fn filter(&mut self, x: u16) -> Option<u16> {
        let diff = x.abs_diff(self.last);
        if diff > self.trigger_threshold {
            self.active = true;
        } else if diff < self.sustain_threshold {
            self.active = false;
        }

        if self.active {
            self.last = x;
            Some(x)
        } else {
            None
        }
    }
}

/// Filtered and gated analog input to create events from repeated measurements.
///
/// Analog measurements are inherently noisy. This input instance filters the noisy measurements
/// and only produces a new value if the difference to the previously produced value exceeds the
/// trigger threshold. After a value has been produced, the gate is open, i.e. polling the input
/// produces a value, until a measurement falls below the sustain threshold.
pub struct AnalogInput<ADC, P, CS>
where
    ADC: CalibrationAccess,
    P: AdcChannel + AnalogPin,
    CS: AdcCalScheme<ADC>,
{
    filter: AdcFilter,
    gate: AdcGate,
    pin: AdcPin<P, ADC, CS>,
}

#[derive(Clone, Copy)]
pub struct AnalogInputConfig {
    /// Filter coefficient, range: (0 - 1]
    ///
    /// `alpha` = 1: no filtering
    pub alpha: f32,
    pub trigger_threshold: u16,
    pub sustain_threshold: u16,
}

impl<ADC, P, CS> AnalogInput<ADC, P, CS>
where
    ADC: CalibrationAccess,
    P: AdcChannel + AnalogPin,
    CS: AdcCalScheme<ADC>,
{
    pub fn new(adc_config: &mut AdcConfig<ADC>, pin: P, config: AnalogInputConfig) -> Self {
        Self {
            filter: AdcFilter::new(config.alpha),
            gate: AdcGate::new(config.trigger_threshold, config.sustain_threshold),
            pin: adc_config.enable_pin_with_cal::<_, CS>(pin, Attenuation::Attenuation11dB),
        }
    }
}

/// Helper trait to abstract AnalogInput
pub trait AdcPoll<ADC: CalibrationAccess> {
    fn poll(&mut self, adc: &mut Adc<'_, ADC>) -> Option<u16>;
}

impl<ADC, P, CS> AdcPoll<ADC> for AnalogInput<ADC, P, CS>
where
    ADC: CalibrationAccess,
    P: AdcChannel + AnalogPin,
    CS: AdcCalScheme<ADC>,
{
    fn poll(&mut self, adc: &mut Adc<'_, ADC>) -> Option<u16> {
        adc.read_oneshot(&mut self.pin)
            .ok()
            .and_then(|x| x.apply(&mut self.filter).apply(&mut self.gate))
    }
}

pub struct AnalogInputBuilder<'d> {
    adc_config: AdcConfig<ADC1>,
    input_config: AnalogInputConfig,
    inputs: Vec<(Box<dyn AdcPoll<ADC1> + 'd>, u8)>,
}

/// Builder type for analog inputs
impl<'d> AnalogInputBuilder<'d> {
    /// Construct a new analog input builder
    ///
    /// `config` - filter and gate configuration for analog input
    pub fn new(config: AnalogInputConfig) -> Self {
        Self {
            adc_config: AdcConfig::new(),
            input_config: config,
            inputs: vec![],
        }
    }

    /// Add a new analog input
    ///
    /// `pin` - GPIO pin
    /// `control` - MIDI control code
    pub fn add(mut self, pin: impl AdcChannel + AnalogPin + 'd, control: u8) -> Self {
        type AdcCal = esp_hal::analog::adc::AdcCalBasic<ADC1>;
        let input = AnalogInput::<_, _, AdcCal>::new(&mut self.adc_config, pin, self.input_config);
        self.inputs.push((Box::new(input), control));
        self
    }

    /// Build the adc instance and all inputs
    ///
    /// `adc1` - the ADC1 peripheral
    pub fn build(
        self,
        adc1: impl Peripheral<P = ADC1> + 'd,
    ) -> (Adc<'d, ADC1>, Vec<(Box<dyn AdcPoll<ADC1> + 'd>, u8)>) {
        let adc = Adc::new(adc1, self.adc_config);
        (adc, self.inputs)
    }
}

/// Polls inputs cyclically at fixed time intervals `poll_interval`.
///
/// t = 0*T input[0]
/// t = 1*T input[1]
/// t = 2*T input[2]
/// t = 3*T input[0]
/// ...
pub async fn produce_midi_on_analog_input_change<'d>(
    adc: &mut Adc<'d, ADC1>,
    inputs: &mut [(Box<dyn AdcPoll<ADC1> + 'd>, u8)],
    poll_interval: Duration,
) {
    let mut tick = Ticker::every(poll_interval);
    loop {
        for (input, control) in inputs.iter_mut() {
            tick.next().await;
            if let Some(v) = input.poll(adc) {
                let value = (v / 32) as u8;
                send_control(*control, value).await;
            }
        }
    }
}
