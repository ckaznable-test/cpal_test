use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let target_sample_rate = SampleRate(22050);
    let config = supported_configs_range
        .find(|config| {
            config.max_sample_rate() > target_sample_rate
                && config.min_sample_rate() < target_sample_rate
                && config.sample_format() == SampleFormat::F32
                && config.channels() == 2
        })
        .expect("no supported config found")
        .with_sample_rate(target_sample_rate);

    run(&device, &config.into())
}

fn run(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.2
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data(output: &mut [f32], channels: usize, next_sample: &mut dyn FnMut() -> f32) {
    for frame in output.chunks_mut(channels) {
        let value: f32 = next_sample();
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
