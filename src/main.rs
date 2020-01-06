extern crate notify_rust;
extern crate nix;
use std::iter::FromIterator;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dumb-brightness", about = "Very simple program to control brightness of your screen, keyboard or whatever exposed via sysfs (Linux only)", author = "Mikhail Khvoinitsky")]
struct Opt {
    /// Verbose mode
    #[structopt(short, long)]
    verbose: bool,

    /// Increase brightness of PERCENT
    #[structopt(long, value_name = "PERCENT", display_order(1))]
    increase: Option<f64>,

    /// Decrease brightness of PERCENT
    #[structopt(long, value_name = "PERCENT", display_order(1), conflicts_with = "increase")]
    decrease: Option<f64>,

    /// Notification duration
    #[structopt(long, default_value = "1", display_order(10), value_name = "SECONDS")]
    duration: f64,

    /// Notification icon
    #[structopt(long, default_value = "display-brightness-symbolic", display_order(11), value_name = "ICON")]
    icon: std::string::String,

    /// Notification title
    #[structopt(long, default_value = "Brightness", display_order(12), value_name = "ICON")]
    title: std::string::String,

    /// Steps for smooth transition (1 to disable)
    #[structopt(long, default_value = "1", display_order(20), value_name = "NUMBER")]
    steps: u64,

    /// Step interval
    #[structopt(long, default_value = "0", display_order(21), value_name = "MILLISECONDS")]
    step_interval: u64,

    /// Working directory
    #[structopt(short, long, value_name = "PATH")]
    working_directory: Option<std::string::String>,
}

fn main() {
    let opt = Opt::from_args();

    if opt.working_directory.is_some() {
        std::env::set_current_dir(opt.working_directory.unwrap()).unwrap();
    };

    let mut tmp = std::env::temp_dir();
    tmp.push(format!("{}_{}", "dumb-brightness", std::env::current_dir().unwrap().to_str().unwrap().replace("/", "_")));
    let tmpfile_base = tmp.to_str().unwrap();

    let lockfile_path = format!("{}_{}", tmpfile_base, "lock");
    let lockfile = std::fs::File::create(&lockfile_path).unwrap();
    nix::fcntl::flock(std::os::unix::io::AsRawFd::as_raw_fd(&lockfile), nix::fcntl::FlockArg::LockExclusiveNonblock).expect("Already running");

    let error_msg = "Unable to open 'brightness' and\\or 'max_brightness' files, you should cd into appropriate sysfs directory or specify it with -w option";
    let max_brightness: f64 = std::fs::read_to_string("max_brightness").expect(error_msg).trim().parse().unwrap();
    let old_brightness: f64 = std::fs::read_to_string("brightness").expect(error_msg).trim().parse().unwrap();

    let factor: f64 = if opt.increase.is_some() {
        opt.increase.unwrap()
    } else {
        0f64 - opt.decrease.unwrap()
    } / 100.0;

    let new_brightness_raw = old_brightness + (max_brightness * factor);
    let new_brightness = if opt.increase.is_some() {
        f64::min(new_brightness_raw, max_brightness)
    } else {
        f64::max(new_brightness_raw, 0f64)
    }.round();
    let prev_notification_id_filename = format!("{}_{}", tmpfile_base, "notification-id");
    let mut prev_id: u32 = match std::fs::read_to_string(&prev_notification_id_filename) {
        Ok(prev_notification_id_str) => match prev_notification_id_str.trim().parse::<u32>() {
            Ok(prev_notification_id) => prev_notification_id,
            Err(_) => 0,
        },
        Err(_) => 0,
    };

    let number_of_steps = opt.steps;
    let step_size = (new_brightness - old_brightness) / number_of_steps as f64;

    let mut value_steps = std::vec::Vec::from_iter( (1..number_of_steps + 1).map(|step_number| {
        if step_number == number_of_steps {
            new_brightness
        } else {
            (old_brightness + (step_size * step_number as f64)).round()
        }
    }));
    value_steps.dedup();
    let value_steps_len = value_steps.len();

    for (new_brightness_live, new_percent_live, sleep_at_the_end) in value_steps.into_iter().enumerate().map(|(i, step_value)| {
        (step_value, (100f64 * step_value / max_brightness).round(), i + 1 != value_steps_len)
    }) {
        if opt.verbose {
            println!("old: {}, target: {}, setting currently: {} {}%, max: {}.", old_brightness, new_brightness, new_brightness_live, new_percent_live, max_brightness)
        }
        std::fs::write("brightness", new_brightness_live.round().to_string()).unwrap();

        let notification = notify_rust::Notification::new()
            .summary(opt.title.as_str())
            .body(format!("{}%", new_percent_live).as_str())
            .icon(opt.icon.as_str())
            .hint(notify_rust::NotificationHint::CustomInt("value".to_string(), new_percent_live as i32))
            .timeout(notify_rust::Timeout::Milliseconds((opt.duration * 1000f64) as u32))
            .id(prev_id)
            .show().unwrap();
        prev_id = notification.id();

        if sleep_at_the_end {
            std::thread::sleep(std::time::Duration::from_millis(opt.step_interval));
        }
    }

    std::fs::write(&prev_notification_id_filename, prev_id.to_string()).unwrap();

    std::fs::remove_file(&lockfile_path).unwrap();
}
