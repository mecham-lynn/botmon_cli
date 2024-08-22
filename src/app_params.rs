use argh::FromArgs;
use chrono::Duration;

#[derive(Debug, FromArgs)]
/// cli utility to get bot/queue stats and information
pub struct AppParams {
    /// time for the app to refresh for new stats in seconds
    /// the max amount of time is 10 minutes. The minimum is 10 seconds. If an invalid number is passed in 
    /// the duration will be set to 10 seconds 
    #[argh(option, short='r', from_str_fn(num_to_duration))]
    pub refresh_time: Duration,
    
    #[argh(option, short='c')]
    /// path to a file that contains the leo-config file. It should contain all the details needed to interact
    /// with the LeoBus in question
    pub config_path: Option<String>,
    
    #[argh(option, short='b')]
    /// the actual key for the bus from the configuration file. 
    /// If not provided a select screen will display where a bus can be chosen.
    pub bus: Option<String>,
    
    #[argh(switch, short='d')]
    /// enables debug mode, which enables the State View
    pub debug: bool
    
}

fn num_to_duration(value: &str) -> Result<Duration, String> {
    match value.parse::<i64>() {
        Ok(secs) => if secs > 600 {
                Ok(Duration::seconds(600))
            } else if secs < 10 {
                Ok(Duration::seconds(10))
            } else {
                Ok(Duration::seconds(secs))
            }
        Err(_) => Ok(Duration::seconds(10))
    }
}
