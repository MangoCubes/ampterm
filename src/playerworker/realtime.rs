use std::time::Duration;

use derive_deref::Deref;

/// We introduce two concepts: PlayTime and PosTime. PlayTime represents how long we have been
/// playing a specific music. As a result, no amount of speedup affects the rate at which PlayTime
/// increases. On the other hand, PosTime represents the position within the music. As a result,
/// playing at rate > 1 makes the PosTime to gain faster than usual.
#[derive(Deref)]
pub struct PlayTime(Duration);

#[derive(Deref)]
pub struct PosTime(Duration);
impl PosTime {
    fn to_playtime(&self, speed: f32) -> PlayTime {
        PlayTime(self.0.div_f32(speed))
    }
}

pub struct RealTime {
    playtime: PlayTime,
    postime: PosTime,
    last_added_at: PlayTime,
}

impl RealTime {
    pub fn new() -> Self {
        Self {
            playtime: PlayTime(Duration::default()),
            postime: PosTime(Duration::default()),
            last_added_at: PlayTime(Duration::default()),
        }
    }
    /// Add a new datapoint
    pub fn add(&mut self, duration: Duration, speed: f32) {
        let interval = {
            if duration > self.last_added_at.0 {
                let ret = duration - self.last_added_at.0;
                self.last_added_at = PlayTime(duration);
                ret
            } else {
                Duration::default()
            }
        };
        self.playtime.0 += interval;
        // If the PlayTime is 10 and the current speed is 2, that means we have played 10 * 2 = 20
        // seconds of music.
        self.postime.0 += interval.mul_f32(speed);
    }
    pub fn get_now(&self) -> Duration {
        self.postime.0
    }
    pub fn avg_speed(&self) -> f32 {
        self.postime.0.div_duration_f32(self.playtime.0)
    }
    /// Add given offset to the current time, returning a duration ready to be used by Sink.
    pub fn move_time_by(&mut self, now: Duration, speed: f32, offset: f32) -> (Duration, Duration) {
        // Make one final calculation
        self.add(now, speed);
        let avg = self.avg_speed();
        // We now have the real runtime, and how long it has been since the last
        let new_time = if offset >= 0.0 {
            let offset_duration = Duration::from_secs_f32(offset);
            PosTime(self.postime.0 + offset_duration)
        } else {
            let offset_duration = Duration::from_secs_f32(-offset);
            if self.postime.0 > offset_duration {
                PosTime(self.postime.0 - offset_duration)
            } else {
                PosTime(Duration::default())
            }
        };
        let pos = new_time.0;
        let play = new_time.to_playtime(avg).0;
        self.last_added_at = PlayTime(Duration::default());
        self.playtime = PlayTime(Duration::default());
        self.postime = PosTime(Duration::default());
        (pos, play)
    }
}
