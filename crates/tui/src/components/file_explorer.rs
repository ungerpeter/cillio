use block::Title;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use layout::Offset;
use ratatui::{prelude::*, widgets::*};
use std::path::PathBuf;
use style::palette::tailwind::SLATE;
use symbols::border;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct FileExplorer {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    offset: Offset,
    cwd: PathBuf,
    file_list: FileList,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_and_set_files(&mut self) -> Result<()> {
        if self.cwd == PathBuf::default() {
            self.cwd = std::env::current_dir()?;
        }
        let (mut dirs, mut none_dirs): (Vec<_>, Vec<_>) = std::fs::read_dir(&self.cwd)?
            .filter_map(|entry| {
                entry.ok().map(|e| {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = if is_dir {
                        format!("{}/", e.file_name().to_string_lossy())
                    } else {
                        e.file_name().to_string_lossy().into_owned()
                    };

                    FileItem { name, path, is_dir }
                })
            })
            .partition(|file| file.is_dir);

        dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));
        none_dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));

        if let Some(parent) = self.cwd.parent() {
            let mut files = Vec::with_capacity(1 + dirs.len() + none_dirs.len());

            files.push(FileItem {
                name: "../".to_owned(),
                path: parent.to_path_buf(),
                is_dir: true,
            });

            files.extend(dirs);
            files.extend(none_dirs);

            self.file_list.items = files
        } else {
            let mut files = Vec::with_capacity(dirs.len() + none_dirs.len());

            files.extend(dirs);
            files.extend(none_dirs);

            self.file_list.items = files;
        };

        Ok(())
    }

    fn select_none(&mut self) {
        self.file_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.file_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.file_list.state.select_previous();
    }
}

impl Component for FileExplorer {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                self.get_and_set_files()?;
                if !self.file_list.items.is_empty() && self.file_list.state.selected().is_none() {
                    self.file_list.state.select_first();
                }
            }
            Action::MoveRight => {
                self.offset.x += 1;
            }
            Action::MoveLeft => {
                if self.offset.x >= 1 {
                    self.offset.x -= 1;
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Enter => {
                if let Some(selected) = self.file_list.state.selected() {
                    let file = &self.file_list.items[selected];
                    if file.is_dir {
                        self.cwd = file.path.clone();
                        self.get_and_set_files()?;
                        self.select_none();
                    } else {
                        return Ok(Some(Action::SetGraphConfigPath(file.path.clone())));
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let title = Title::from(" File explorer ".bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .border_set(border::ROUNDED);
        let items = self.file_list.items.iter().map(|file| file.name.as_str());
        frame.render_stateful_widget(
            List::new(items)
                .highlight_style(SELECTED_STYLE)
                .block(block),
            area.offset(self.offset),
            &mut self.file_list.state,
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileList {
    items: Vec<FileItem>,
    state: ListState,
}

impl Default for FileList {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileItem {
    name: String,
    path: PathBuf,
    is_dir: bool,
}
