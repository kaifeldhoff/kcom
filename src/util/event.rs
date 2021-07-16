use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::TermRead;

pub enum MyEvent {
    Input(Key),
    Click(u16, u16),
    Tick,
}

/// An small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<MyEvent>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Alt('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.events() {
                    match evt {
                        Ok(Event::Key(key)) => {
                            if let Err(_) = tx.send(MyEvent::Input(key)) {
                                return;
                            }
                            if key == config.exit_key {
                                return;
                            }
                        }
                        Ok(Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))) => {
                            if let Err(_) = tx.send(MyEvent::Input(Key::Up)) {
                                return;
                            }
                        }
                        Ok(Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _))) => {
                            if let Err(_) = tx.send(MyEvent::Input(Key::Down)) {
                                return;
                            }
                        }
                        Ok(Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y))) => {
                            if let Err(_) = tx.send(MyEvent::Click(x, y)) {
                                return;
                            }
                        }
                        // Quit on Right-Mouse-Button
                        Ok(Event::Mouse(MouseEvent::Press(MouseButton::Right, _x, _y))) => {
                            if let Err(_) = tx.send(MyEvent::Input(Key::Alt('q'))) {
                                return;
                            }
                        }
                        Err(_) => {}
                        _ => {}
                    }
                }
            })
        };
        let _tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                if true {
                    return;
                }
                let tx = tx.clone();
                loop {
                    tx.send(MyEvent::Tick).unwrap();
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            _input_handle,
            _tick_handle,
        }
    }

    pub fn next(&self) -> Result<MyEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}
