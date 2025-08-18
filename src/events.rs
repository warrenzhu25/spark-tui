use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Quit,
}

pub struct EventHandler {
    last_tick: Instant,
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            last_tick: Instant::now(),
            tick_rate,
        }
    }

    pub fn next(&mut self) -> anyhow::Result<AppEvent> {
        let timeout = self.tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        return Ok(AppEvent::Quit);
                    }
                    Ok(AppEvent::Key(key))
                }
                _ => Ok(AppEvent::Tick),
            }
        } else {
            if self.last_tick.elapsed() >= self.tick_rate {
                self.last_tick = Instant::now();
                Ok(AppEvent::Tick)
            } else {
                Ok(AppEvent::Tick)
            }
        }
    }
}