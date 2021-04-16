/*
TODO

Fix: try to keep selection on changed filter

Add: Keys: Pg-Up/Down, Del(one dir back)
Add: Breadcrumb click-able

*/
extern crate failure;
extern crate termion;
extern crate tui;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{PathBuf, Component};

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{SelectableList, Widget};
use tui::Terminal;

mod util;
mod widgets;

use widgets::label::Label;
use util::filesystem::get_current_dir;
use util::filesystem::get_dir_content;
use util::event::{MyEvent, Events};

const CMD_FILE: &'static str = ".kcom.cmd";
const DEFAULT_FILTER: &'static str = "";

struct App {
    window_width: u16,
    selected: Option<usize>,
    top_col: usize,
    offset: usize,
    current_dir: PathBuf,
    all_subdirs: Vec<String>,
    subdirs: Vec<String>,
    all_files: Vec<String>,
    files: Vec<String>,
    list_length: usize,
    chunk_height: usize,
    show_hidden: bool,
    filter: Option<String>,
}

impl App {
    fn new() -> Result<App, failure::Error> {
        let current_dir = get_current_dir()?;
        let files = vec!();
        let subdirs = vec!();
        let all_files = vec!();
        let all_subdirs = vec!();
        Ok ( App {
            window_width: 0,
            selected: Some(0),
            top_col: 0,
            offset: 0,
            current_dir,
            all_subdirs,
            subdirs,
            all_files,
            files,
            list_length: 0,
            chunk_height: 0,
            show_hidden: false,
            filter: None,
        })
    }
    fn update_from_filesystem(&mut self) -> Result<(), failure::Error> {
        let (mut all_subdirs, mut all_files, symlinks) = {
            match get_dir_content(&self.current_dir) {
                Ok((r1, r2, r3)) => (r1, r2, r3),
                Err(e) => {
                    (vec!(format!("{:?}", e)), vec!(), vec!())
                }
            }
        };

        all_subdirs.sort();
        self.all_subdirs = all_subdirs;

        all_files.extend(symlinks);
        all_files.sort();
        self.all_files = all_files;
        Ok(())
    }
    fn build_displayed_items(&mut self) -> Result<(), failure::Error> {
        let filter = match self.filter {
            Some(ref f) => f,
            None => DEFAULT_FILTER,
        };
        let mut subdirs = self.all_subdirs
            .iter()
            .filter(|d| self.show_hidden || !d.starts_with('.'))
            .filter(|d| d.contains(filter))
            .map(|i| i.to_owned())
            .collect::<Vec<String>>();
        if let Some(_) = &self.current_dir.parent() {
            subdirs.insert(0, "../".to_string());
        }
        self.subdirs = subdirs;

        let mut files = self.all_files
            .iter()
            .filter(|f| self.show_hidden || !f.starts_with("."))
            .filter(|f| f.contains(filter))
            .map(|i| i.to_owned())
            .collect::<Vec<String>>();
        for item in files.iter_mut() {
            item.insert_str(0, " - ")
        }
        self.files = files;

        self.list_length = self.subdirs.len() + self.files.len();

        if let Some(selected) = self.selected {
            if selected > self.list_length - 1 {
                self.selected = Some(self.list_length - 1);
            }
        }
        Ok(())
    }
    fn change_dir(&mut self) -> Result<(), failure::Error> {
        let mut last_dir: Option<String> = None;
        if let Some(s) = self.selected {
            if let Some(ref subdir) = self.subdirs.get(s) {
                if *subdir == &"../".to_string() {
                    let basedir = self.current_dir
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    last_dir = Some(basedir);
                    self.current_dir = self.current_dir
                        .parent()
                        .unwrap()
                        .to_path_buf();
                } else {
                    self.current_dir = self.current_dir.join(subdir);
                }
            } else {
                // file is selected
                return Ok(())
            }
        }
        self.filter = None;
        self.update_from_filesystem()?;
        self.build_displayed_items()?;
        self.selected = {
            match last_dir {
                Some(dir) => {
                    let index = self.subdirs
                        .iter()
                        .position(|ref r| *r == &dir)
                        .unwrap();
                    Some(index)
                },
                None => Some(0),
            }
        };
        Ok(())
    }
    fn _current_path_as_string(&self) -> String {
        format!("{}", &self.current_dir.display())
    }
    fn _breadcrumb_length (&self) -> usize {
        self.current_dir.to_string_lossy().chars().count()
    }
    fn breadcrumb_lines(&self) -> Vec<String> {
        let path_as_string = self.current_dir.to_string_lossy();
        let mut lines = vec!();
        let mut item = "".to_string();
        let mut iter_chars = path_as_string.chars();
        let mut col = 0;
        loop {
            match iter_chars.next() {
                Some(c) => {
                    if col < self.window_width {
                        item.push(c);
                        col += 1;
                    } else {
                        lines.push(item.clone());
                        item.clear();
                        item.push(c);
                        col = 1;
                    }
                },
                None => {
                    if !item.is_empty() {
                        lines.push(item.clone());
                    }
                    break
                }
            }
        }
        lines
    }
    fn breadcrumb_chdir(&mut self, x: u16, y: u16)
        -> Result<(), failure::Error>
    {
        let pos = ((x-1) + (y-1)*self.window_width) as usize;
        let mut len = 0;
        let mut clicked_path = PathBuf::from("/");
        for comp in self.current_dir.components() {
            match comp {
                Component::RootDir => len += 1,
                Component::Normal(p) => {
                    len += p.to_string_lossy().chars().count() + 1;
                    clicked_path = clicked_path.join(p);
                },
                _ => (),
            }
            if len >= pos {
                break
            }
        }
        self.current_dir = clicked_path;
        self.filter = None;
        self.update_from_filesystem()?;
        self.build_displayed_items()?;
        Ok(())
    }
    fn refresh_list_offset(&mut self) {
        self.offset = if let Some(selected) = self.selected {
            if selected >= self.chunk_height {
                selected - self.chunk_height + 1
            } else {
                0
            }
        } else {
            0
        };
    }
    fn write_cmd_file(&mut self) -> Result<(), failure::Error> {
        let cmd = format!("cd \"{}\"", self.current_dir.to_string_lossy());
        let out_filename = PathBuf::from(env::var("HOME")?).join(CMD_FILE);
        let mut out_file = fs::File::create(out_filename)?;
        out_file.write_all(cmd.as_bytes())?;
        Ok(())
    }
}

fn run() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let mut app = App::new()?;
    app.update_from_filesystem()?;
    app.build_displayed_items()?;

    let mut cur_size = terminal.size()?;
    loop {

        // check for resize
        let size = terminal.size()?;
        if cur_size != size {
            cur_size = size;
            terminal.resize(cur_size)?;
        }

        terminal.draw(|mut f| {
            app.window_width = cur_size.width;
            let breadcrumb_lines = app.breadcrumb_lines();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(breadcrumb_lines.len() as u16),
                        Constraint::Max(100),
                        // Constraint::Length(1),
                    ].as_ref()
                )
                .split(cur_size);

            // calc offset as in Widget::draw
            app.chunk_height = chunks[1].height as usize;
            app.refresh_list_offset();

            // fill items to fit widget-width
            app.top_col = chunks[1].y as usize;

            let style = Style::default().fg(Color::White).bg(Color::Black);
            SelectableList::default()
                // .block(Block::default().borders(Borders::NONE))
                .items(&[&app.subdirs[..], &app.files[..]].concat())
                .select(app.selected)
                // .style(style)
                .highlight_style(
                    style.fg(Color::Black)
                        .bg(Color::Cyan))
                .render(&mut f, chunks[1]);

            // Header
            let header_lines = {
                if let Some(ref f) = app.filter {
                    vec!(format!("Filter: {}", f))
                } else {
                    breadcrumb_lines
                }
            };
            Label::default()
                .style(style.fg(Color::LightYellow))
                .text(header_lines)
                .render(&mut f, chunks[0]);

        })?;

        match events.next()? {
            MyEvent::Input(input) => match input {
                Key::Alt('q') => {
                    app.write_cmd_file()?;
                    break;
                }
                Key::Down => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected >= app.list_length - 1 {
                            Some(selected)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Up => {
                    app.selected = if let Some(selected) = app.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(selected)
                        }
                    } else {
                        Some(0)
                    }
                },
                Key::Home => {
                    // if below first file, move to top of files
                    // , otherwise move to top of list
                    if let Some(selected) = app.selected {
                        if selected > app.subdirs.len() {
                            app.selected = Some(app.subdirs.len());
                        } else {
                            app.selected = Some(0);
                        }
                    } else {
                        // selection unknown -> move to top of list
                        app.selected = Some(0);
                    }
                },
                Key::End => {
                    // if above last subdirs, move to end of subdirs
                    // , otherwise move to end of list
                    if let Some(selected) = app.selected {
                        if selected < app.subdirs.len() - 1 {
                            app.selected = Some(app.subdirs.len()-1);
                        } else {
                            app.selected = Some(app.list_length-1);
                        }
                    } else {
                        // selection unknown -> move to end of list
                        app.selected = Some(app.list_length-1);
                    }
                },
                Key::F(1) => {
                    app.show_hidden = !app.show_hidden;
                    app.build_displayed_items()?;
                }
                Key::Char('\n') => {
                    app.change_dir()?;
                },
                Key::Char(c) => {
                    if let Some(ref mut filter) = app.filter {
                        filter.push(c);
                    } else {
                        app.filter = Some(c.to_string())
                    }
                    app.build_displayed_items()?;
                },
                Key::Backspace => {
                    if app.filter.is_some() {
                        if let Some(ref mut filter) = app.filter {
                            let _ = filter.pop();
                        }
                        if app.filter == Some("".to_string()) {
                            app.filter = None
                        }
                        app.build_displayed_items()?;
                    } else {
                        // Todo: got up parent-dir
                    }
                },
                Key::Esc => {
                    app.filter = None;
                    app.build_displayed_items()?;
                }
                _ => {}
            },
            MyEvent::Click(x_u16, y_u16) => {
                // inside listbox?
                let y = y_u16 as usize;
                if y > app.top_col {
                    app.selected = Some(y - app.top_col + app.offset - 1);
                    app.change_dir()?;
                } else {
                    app.breadcrumb_chdir(x_u16, y_u16)?;
                }
                // let s = format!("top = {}; y = {}", app.top_col, y);
                // let mut item = app.items.get_mut(0).unwrap();
                // *item = s;
            }
            MyEvent::Tick => { // just redraw
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Error: {:?}", e);
    }
    println!("Done");
}
