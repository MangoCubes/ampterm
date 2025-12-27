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
    pub fn from_secs_f32(pos: f32) -> Self {
        PosTime(Duration::from_secs_f32(pos))
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
        if duration >= self.last_added_at.0 {
            let interval = duration - self.last_added_at.0;
            self.last_added_at = PlayTime(duration);
            self.playtime.0 += interval;
            // If the PlayTime is 10 and the current speed is 2, that means we have played 10 * 2 = 20
            // seconds of music.
            self.postime.0 += interval.mul_f32(speed);
        } else {
            // Only this can happen is two ways:
            // 1. User have rewinded
            // 2. New song is played
            // In the first case, this is safe as reset is intended to happen anyway. In the second
            // case, this is also safe because the duration is negligibly small.
            self.last_added_at = PlayTime(duration);
            self.playtime = PlayTime(duration);
            self.postime = PosTime(duration.mul_f32(speed));
        };
    }
    pub fn get_now(&self) -> Duration {
        self.postime.0
    }
    /// Jump to a specific point in the music.
    /// The time passed to this should be absolute, calculated from the start with play speed of
    /// 1.0.
    pub fn move_time_to(&mut self, pos: PosTime, speed: f32) -> (Duration, Duration) {
        let pos_duration = pos.0;
        let play = pos.to_playtime(speed).0;
        self.reset();
        (pos_duration, play)
    }
    /// Add given offset to the current time, returning a duration ready to be used by Sink.
    pub fn move_time_by(&mut self, now: Duration, speed: f32, offset: f32) -> (Duration, Duration) {
        // Make one final calculation that covers the last datapoint addition and now
        self.add(now, speed);
        // We now have the real runtime, and how long it has been since the last
        // [`new_time`] refers to the absolute point at which the player should jump to
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
        self.move_time_to(new_time, speed)
    }

    pub fn reset(&mut self) {
        self.last_added_at = PlayTime(Duration::default());
        self.playtime = PlayTime(Duration::default());
        self.postime = PosTime(Duration::default());
    }
}
