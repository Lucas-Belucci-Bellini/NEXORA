//! World clock and calendar.
//!
//! Implements `TIME AND CALENDAR SYSTEM.md`. The rule that shapes the whole
//! module is its first invariant: *one authoritative world clock per simulation
//! context*, and simulation reads world time, never wall-clock time. A system
//! that samples the host clock cannot be replayed, so nothing here can observe
//! it.

use crate::error::{Domain, Error, Recovery, Result};

/// A point in world time, measured in simulation ticks since the world epoch.
///
/// Ticks are the atomic unit: everything larger (seconds, days, seasons) is
/// derived through a [`CalendarConfig`], so a world with an unusual day length
/// still orders and persists correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WorldTime(pub u64);

/// A span of world time in ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WorldDuration(pub u64);

impl WorldTime {
    /// The world epoch, tick zero.
    pub const EPOCH: Self = Self(0);

    /// Raw tick count.
    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.0
    }

    /// Advance by a duration.
    ///
    /// # Errors
    ///
    /// Returns an error when the tick counter would overflow. At a typical tick
    /// rate this is unreachable in practice; it is checked rather than wrapped
    /// because a wrapped clock would silently reorder history.
    pub fn checked_add(self, duration: WorldDuration) -> Result<Self> {
        self.0.checked_add(duration.0).map(Self).ok_or_else(|| {
            Error::new(
                Domain::Time,
                "world-clock",
                "world time overflowed the tick counter",
            )
            .with_recovery(Recovery::Manual)
            .fatal()
            .with_context("now", self.0.to_string())
            .with_context("delta", duration.0.to_string())
        })
    }

    /// The span from `earlier` to `self`, saturating at zero.
    #[must_use]
    pub const fn since(self, earlier: Self) -> WorldDuration {
        WorldDuration(self.0.saturating_sub(earlier.0))
    }
}

impl WorldDuration {
    /// A zero-length span.
    pub const ZERO: Self = Self(0);

    /// Build a duration from ticks.
    #[must_use]
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Raw tick count.
    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.0
    }
}

/// The named time scales the engine converts between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeScale {
    /// One simulation tick.
    Tick,
    /// One in-world second.
    Second,
    /// One in-world minute.
    Minute,
    /// One in-world hour.
    Hour,
    /// One in-world day.
    Day,
    /// One in-world month.
    Month,
    /// One in-world season.
    Season,
    /// One in-world year.
    Year,
}

/// Calendar shape for a dimension or world.
///
/// A dimension may expose its own calendar, but the engine still stores ticks as
/// the canonical representation for ordering and persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarConfig {
    ticks_per_second: u32,
    seconds_per_minute: u32,
    minutes_per_hour: u32,
    hours_per_day: u32,
    days_per_month: u32,
    months_per_year: u32,
    seasons_per_year: u32,
}

/// Default simulation tick rate, in ticks per in-world second.
pub const DEFAULT_TICKS_PER_SECOND: u32 = 20;

impl CalendarConfig {
    /// Validate and construct a calendar.
    ///
    /// # Errors
    ///
    /// Returns an error when any field is zero, when the year does not divide
    /// evenly into seasons, or when a full year would overflow `u64` ticks.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ticks_per_second: u32,
        seconds_per_minute: u32,
        minutes_per_hour: u32,
        hours_per_day: u32,
        days_per_month: u32,
        months_per_year: u32,
        seasons_per_year: u32,
    ) -> Result<Self> {
        let fields = [
            ("ticks_per_second", ticks_per_second),
            ("seconds_per_minute", seconds_per_minute),
            ("minutes_per_hour", minutes_per_hour),
            ("hours_per_day", hours_per_day),
            ("days_per_month", days_per_month),
            ("months_per_year", months_per_year),
            ("seasons_per_year", seasons_per_year),
        ];
        for (name, value) in fields {
            if value == 0 {
                return Err(
                    invalid("calendar field must be greater than zero").with_context("field", name)
                );
            }
        }

        let days_per_year = days_per_month
            .checked_mul(months_per_year)
            .ok_or_else(|| invalid("days per year overflows"))?;
        if days_per_year % seasons_per_year != 0 {
            return Err(invalid("the year must divide evenly into seasons")
                .with_context("days_per_year", days_per_year.to_string())
                .with_context("seasons_per_year", seasons_per_year.to_string()));
        }

        let candidate = Self {
            ticks_per_second,
            seconds_per_minute,
            minutes_per_hour,
            hours_per_day,
            days_per_month,
            months_per_year,
            seasons_per_year,
        };
        // Reject a calendar whose own year cannot be expressed, rather than
        // discovering it much later during a long simulation.
        candidate.checked_ticks_per(TimeScale::Year)?;
        Ok(candidate)
    }

    /// The engine default: 20 ticks/s, 60/60/24, 30-day months, 12 months, 4 seasons.
    ///
    /// # Panics
    ///
    /// Never in practice: the constants are checked by
    /// `default_calendar_is_valid`.
    #[must_use]
    pub fn earthlike() -> Self {
        Self::new(DEFAULT_TICKS_PER_SECOND, 60, 60, 24, 30, 12, 4)
            .expect("the built-in earthlike calendar must be valid")
    }

    /// Simulation ticks per in-world second.
    #[must_use]
    pub const fn ticks_per_second(self) -> u32 {
        self.ticks_per_second
    }

    /// In-world seconds per minute.
    #[must_use]
    pub const fn seconds_per_minute(self) -> u32 {
        self.seconds_per_minute
    }

    /// In-world minutes per hour.
    #[must_use]
    pub const fn minutes_per_hour(self) -> u32 {
        self.minutes_per_hour
    }

    /// In-world hours per day.
    #[must_use]
    pub const fn hours_per_day(self) -> u32 {
        self.hours_per_day
    }

    /// In-world days per month.
    #[must_use]
    pub const fn days_per_month(self) -> u32 {
        self.days_per_month
    }

    /// In-world months per year.
    #[must_use]
    pub const fn months_per_year(self) -> u32 {
        self.months_per_year
    }

    /// In-world seasons per year.
    #[must_use]
    pub const fn seasons_per_year(self) -> u32 {
        self.seasons_per_year
    }

    /// Ticks per unit of `scale`.
    ///
    /// # Errors
    ///
    /// Returns an error when the scale's tick count overflows `u64`.
    pub fn checked_ticks_per(self, scale: TimeScale) -> Result<u64> {
        let mul = |a: u64, b: u32| -> Result<u64> {
            a.checked_mul(u64::from(b))
                .ok_or_else(|| invalid("calendar scale overflows u64 ticks"))
        };
        let second = u64::from(self.ticks_per_second);
        let minute = mul(second, self.seconds_per_minute)?;
        let hour = mul(minute, self.minutes_per_hour)?;
        let day = mul(hour, self.hours_per_day)?;
        let month = mul(day, self.days_per_month)?;
        let year = mul(month, self.months_per_year)?;
        // Seasons divide the year evenly; `new` guarantees the division is exact.
        let season = year / u64::from(self.seasons_per_year);
        Ok(match scale {
            TimeScale::Tick => 1,
            TimeScale::Second => second,
            TimeScale::Minute => minute,
            TimeScale::Hour => hour,
            TimeScale::Day => day,
            TimeScale::Month => month,
            TimeScale::Season => season,
            TimeScale::Year => year,
        })
    }

    /// Ticks per unit of `scale`, for calendars already known to be valid.
    ///
    /// # Panics
    ///
    /// Panics only if the calendar was constructed without validation, which
    /// [`CalendarConfig::new`] prevents.
    #[must_use]
    pub fn ticks_per(self, scale: TimeScale) -> u64 {
        self.checked_ticks_per(scale)
            .expect("a validated calendar cannot overflow")
    }

    /// Break a world time into calendar fields.
    #[must_use]
    pub fn date_of(self, time: WorldTime) -> CalendarDate {
        let ticks = time.ticks();
        let per_second = self.ticks_per(TimeScale::Second);
        let per_day = self.ticks_per(TimeScale::Day);
        let per_month = self.ticks_per(TimeScale::Month);
        let per_year = self.ticks_per(TimeScale::Year);
        let per_season = self.ticks_per(TimeScale::Season);

        let tick_of_day = ticks % per_day;
        let second_of_day = tick_of_day / per_second;

        CalendarDate {
            year: ticks / per_year,
            month: (ticks % per_year) / per_month,
            day: (ticks % per_month) / per_day,
            hour: second_of_day
                / u64::from(self.seconds_per_minute)
                / u64::from(self.minutes_per_hour),
            minute: (second_of_day / u64::from(self.seconds_per_minute))
                % u64::from(self.minutes_per_hour),
            second: second_of_day % u64::from(self.seconds_per_minute),
            tick_of_second: tick_of_day % per_second,
            season: ((ticks % per_year) / per_season) as u32,
        }
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Time, "calendar", message).with_recovery(Recovery::Reject)
}

/// World time broken into calendar fields.
///
/// All fields except `year` are zero-based indices, so `month == 0` is the first
/// month. Presentation layers add one; the simulation never should.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDate {
    /// Years since the world epoch.
    pub year: u64,
    /// Month index within the year.
    pub month: u64,
    /// Day index within the month.
    pub day: u64,
    /// Hour index within the day.
    pub hour: u64,
    /// Minute index within the hour.
    pub minute: u64,
    /// Second index within the minute.
    pub second: u64,
    /// Tick index within the second.
    pub tick_of_second: u64,
    /// Season index within the year.
    pub season: u32,
}

/// The authoritative clock for one simulation context.
///
/// There is deliberately no way to set the time backwards: monotonicity is an
/// invariant of the system, not a convention callers are asked to respect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldClock {
    now: WorldTime,
    calendar: CalendarConfig,
}

impl WorldClock {
    /// Start a clock at the world epoch.
    #[must_use]
    pub const fn new(calendar: CalendarConfig) -> Self {
        Self {
            now: WorldTime::EPOCH,
            calendar,
        }
    }

    /// Restore a clock at a persisted time.
    #[must_use]
    pub const fn resumed_at(calendar: CalendarConfig, now: WorldTime) -> Self {
        Self { now, calendar }
    }

    /// The current world time.
    #[must_use]
    pub const fn now(&self) -> WorldTime {
        self.now
    }

    /// The calendar in force.
    #[must_use]
    pub const fn calendar(&self) -> CalendarConfig {
        self.calendar
    }

    /// The current calendar date.
    #[must_use]
    pub fn date(&self) -> CalendarDate {
        self.calendar.date_of(self.now)
    }

    /// Advance the clock.
    ///
    /// # Errors
    ///
    /// Returns an error when the tick counter would overflow.
    pub fn advance(&mut self, delta: WorldDuration) -> Result<WorldTime> {
        self.now = self.now.checked_add(delta)?;
        Ok(self.now)
    }

    /// Advance by whole units of a scale.
    ///
    /// # Errors
    ///
    /// Returns an error when the resulting span or time overflows.
    pub fn advance_by(&mut self, scale: TimeScale, count: u64) -> Result<WorldTime> {
        let per_unit = self.calendar.checked_ticks_per(scale)?;
        let ticks = per_unit
            .checked_mul(count)
            .ok_or_else(|| invalid("requested advance overflows the tick counter"))?;
        self.advance(WorldDuration::from_ticks(ticks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_calendar_is_valid() {
        let calendar = CalendarConfig::earthlike();
        assert_eq!(calendar.ticks_per(TimeScale::Second), 20);
        assert_eq!(calendar.ticks_per(TimeScale::Minute), 20 * 60);
        assert_eq!(calendar.ticks_per(TimeScale::Hour), 20 * 60 * 60);
        assert_eq!(calendar.ticks_per(TimeScale::Day), 20 * 60 * 60 * 24);
        assert_eq!(calendar.ticks_per(TimeScale::Month), 20 * 60 * 60 * 24 * 30);
        assert_eq!(
            calendar.ticks_per(TimeScale::Year),
            20 * 60 * 60 * 24 * 30 * 12
        );
        assert_eq!(
            calendar.ticks_per(TimeScale::Season),
            calendar.ticks_per(TimeScale::Year) / 4
        );
    }

    #[test]
    fn time_is_monotonic_across_advances() {
        let mut clock = WorldClock::new(CalendarConfig::earthlike());
        let mut previous = clock.now();
        for step in 1..500u64 {
            let now = clock.advance(WorldDuration::from_ticks(step)).unwrap();
            assert!(
                now > previous,
                "clock went backwards: {now:?} after {previous:?}"
            );
            previous = now;
        }
    }

    #[test]
    fn calendar_fields_round_trip_through_ticks() {
        let calendar = CalendarConfig::earthlike();
        let per_year = calendar.ticks_per(TimeScale::Year);
        let per_day = calendar.ticks_per(TimeScale::Day);
        let per_hour = calendar.ticks_per(TimeScale::Hour);

        // 3 years, 2 months, 5 days, 7 hours, 1 tick.
        let ticks = 3 * per_year
            + 2 * calendar.ticks_per(TimeScale::Month)
            + 5 * per_day
            + 7 * per_hour
            + 1;
        let date = calendar.date_of(WorldTime(ticks));

        assert_eq!(date.year, 3);
        assert_eq!(date.month, 2);
        assert_eq!(date.day, 5);
        assert_eq!(date.hour, 7);
        assert_eq!(date.minute, 0);
        assert_eq!(date.second, 0);
        assert_eq!(date.tick_of_second, 1);
        assert_eq!(date.season, 0);
    }

    #[test]
    fn seasons_advance_across_the_year() {
        let calendar = CalendarConfig::earthlike();
        let per_season = calendar.ticks_per(TimeScale::Season);
        for season in 0..4u32 {
            let date = calendar.date_of(WorldTime(u64::from(season) * per_season));
            assert_eq!(date.season, season);
        }
        // Wrapping into the next year resets the season index.
        assert_eq!(calendar.date_of(WorldTime(4 * per_season)).season, 0);
        assert_eq!(calendar.date_of(WorldTime(4 * per_season)).year, 1);
    }

    #[test]
    fn pause_and_resume_preserve_the_clock() {
        let calendar = CalendarConfig::earthlike();
        let mut clock = WorldClock::new(calendar);
        clock.advance_by(TimeScale::Day, 12).unwrap();
        let saved = clock.now();

        // Simulating a save/restart: the resumed clock continues, it does not reset.
        let resumed = WorldClock::resumed_at(calendar, saved);
        assert_eq!(resumed.now(), saved);
        assert_eq!(resumed.date().day, clock.date().day);
    }

    #[test]
    fn invalid_calendars_are_rejected() {
        assert!(CalendarConfig::new(0, 60, 60, 24, 30, 12, 4).is_err());
        assert!(CalendarConfig::new(20, 60, 60, 24, 30, 12, 0).is_err());
        // 365 days do not divide into 4 equal seasons.
        assert!(CalendarConfig::new(20, 60, 60, 24, 365, 1, 4).is_err());
        // 364 days do.
        assert!(CalendarConfig::new(20, 60, 60, 24, 364, 1, 4).is_ok());
    }

    #[test]
    fn overflow_is_reported_rather_than_wrapped() {
        let calendar = CalendarConfig::earthlike();
        let mut clock = WorldClock::resumed_at(calendar, WorldTime(u64::MAX - 1));
        let err = clock
            .advance(WorldDuration::from_ticks(10))
            .expect_err("must not wrap");
        assert!(err.is_fatal());
        assert_eq!(err.domain(), Domain::Time);
        // The clock did not move.
        assert_eq!(clock.now(), WorldTime(u64::MAX - 1));
    }

    #[test]
    fn durations_measure_elapsed_time() {
        let start = WorldTime(100);
        let later = WorldTime(350);
        assert_eq!(later.since(start), WorldDuration::from_ticks(250));
        // Going the other way saturates instead of underflowing.
        assert_eq!(start.since(later), WorldDuration::ZERO);
    }
}
