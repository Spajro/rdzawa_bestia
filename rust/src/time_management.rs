pub fn default_time_manager(time: u64) ->u64 {
    (time as f64 / 40.0).floor() as u64
}