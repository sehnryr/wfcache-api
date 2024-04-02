use std::io::{stdout, Error, ErrorKind, Result, Stdout};
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::interval;

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct Tui {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    task: Option<JoinHandle<()>>,
    event_rx: UnboundedReceiver<Event>,
    event_tx: UnboundedSender<Event>,
    frame_rate: usize,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = unbounded_channel();

        Ok(Self {
            terminal: None,
            task: None,
            event_rx,
            event_tx,
            frame_rate: 30,
        })
    }

    pub fn frame_rate(mut self, frame_rate: usize) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn enter(&mut self) -> Result<()> {
        if self.task.is_some() {
            return Err(Error::new(ErrorKind::Other, "Tui is already running"));
        }

        self.terminal = Some(init()?);
        self.task = Some(self.run());
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if self.task.is_none() {
            return Err(Error::new(ErrorKind::Other, "Tui is not running"));
        }

        self.task.take().unwrap().abort();
        self.terminal = None;
        restore()?;
        Ok(())
    }

    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        if let Some(terminal) = self.terminal.as_mut() {
            terminal.draw(f)?;
        }
        Ok(())
    }

    pub fn run(&self) -> JoinHandle<()> {
        let frame_duration = Duration::from_secs_f64(1.0 / self.frame_rate as f64);
        let _event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut frame_interval = interval(frame_duration);
            loop {
                let frame_delay = frame_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(event)) => {
                                match event {
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            _event_tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    _ => {},
                                }
                            },
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            },
                            None => {},
                        }
                    },
                    _ = frame_delay => {
                        _event_tx.send(Event::Render).unwrap();
                    }
                }
            }
        })
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.event_rx
            .recv()
            .await
            .ok_or(Error::new(ErrorKind::Other, "Unable to get event"))
    }
}

/// Initialize the terminal
pub fn init() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
