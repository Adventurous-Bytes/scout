use crate::models::Artifact;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Represents an artifact item in the UI with selection state
#[derive(Clone, Debug)]
pub struct ArtifactItem {
    pub artifact: Artifact,
    pub selected: bool,
    pub exists_locally: bool,
}

impl ArtifactItem {
    pub fn new(artifact: Artifact, output_dir: &str) -> Self {
        let filename = std::path::Path::new(&artifact.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&artifact.file_path);
        let local_path = std::path::Path::new(output_dir).join(filename);
        let exists_locally = local_path.exists();

        Self {
            artifact,
            selected: false,
            exists_locally,
        }
    }

    /// Get display text for the artifact
    pub fn display_text(&self) -> String {
        let id = self.artifact.id.map(|i| i.to_string()).unwrap_or_else(|| "N/A".to_string());
        let filename = std::path::Path::new(&self.artifact.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.artifact.file_path);
        let modality = self.artifact.modality.as_deref().unwrap_or("unknown");
        let created = self.artifact.created_at.as_deref().unwrap_or("N/A");
        
        format!("[{}] {} | {} | {} | Device: {}", id, filename, modality, created, self.artifact.device_id)
    }
}

/// UI state for artifact selection
pub struct ArtifactSelector {
    pub items: Vec<ArtifactItem>,
    pub all_items: Vec<ArtifactItem>,
    pub list_state: ListState,
    pub output_dir: String,
    pub mode: SelectorMode,
    pub filter_mode: FilterMode,
}

#[derive(Clone, PartialEq)]
pub enum SelectorMode {
    Selecting,
    Downloading,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterMode {
    All,
    HideDownloaded,
    ShowOnlyDownloaded,
}

impl ArtifactSelector {
    pub fn new(artifacts: Vec<Artifact>, output_dir: String) -> Self {
        let all_items: Vec<ArtifactItem> = artifacts
            .into_iter()
            .map(|a| ArtifactItem::new(a, &output_dir))
            .collect();
        let items = all_items.clone();
        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            items,
            all_items,
            list_state,
            output_dir,
            mode: SelectorMode::Selecting,
            filter_mode: FilterMode::All,
        }
    }

    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|item| item.selected).count()
    }

    pub fn selected_artifacts(&self) -> Vec<&Artifact> {
        self.items
            .iter()
            .filter(|item| item.selected)
            .map(|item| &item.artifact)
            .collect()
    }

    pub fn toggle_filter(&mut self) {
        self.filter_mode = match self.filter_mode {
            FilterMode::All => FilterMode::HideDownloaded,
            FilterMode::HideDownloaded => FilterMode::ShowOnlyDownloaded,
            FilterMode::ShowOnlyDownloaded => FilterMode::All,
        };
        self.apply_filter();
        if !self.items.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn apply_filter(&mut self) {
        self.items = match self.filter_mode {
            FilterMode::All => self.all_items.clone(),
            FilterMode::HideDownloaded => self.all_items.iter().filter(|item| !item.exists_locally).cloned().collect(),
            FilterMode::ShowOnlyDownloaded => self.all_items.iter().filter(|item| item.exists_locally).cloned().collect(),
        };
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_selection(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.items.len() {
                let artifact_id = self.items[selected].artifact.id;
                self.items[selected].selected = !self.items[selected].selected;
                
                if let Some(all_item) = self.all_items.iter_mut().find(|item| item.artifact.id == artifact_id) {
                    all_item.selected = self.items[selected].selected;
                }
            }
        }
    }

    pub fn select_all(&mut self) {
        for item in &mut self.items {
            item.selected = true;
        }
        for item in &mut self.all_items {
            if self.items.iter().any(|i| i.artifact.id == item.artifact.id) {
                item.selected = true;
            }
        }
    }

    pub fn deselect_all(&mut self) {
        for item in &mut self.items {
            item.selected = false;
        }
        for item in &mut self.all_items {
            item.selected = false;
        }
    }
}

/// Render the UI
pub fn render_ui(f: &mut Frame, state: &ArtifactSelector) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Artifact list
            Constraint::Length(3), // Status/instructions
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("Artifact Download - Select artifacts to download")
        .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Artifact list
    let items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let prefix = if item.selected { "[✓] " } else { "[ ] " };
            let status_indicator = if item.exists_locally {
                Span::styled(" [Downloaded]", Style::default().fg(Color::Cyan))
            } else {
                Span::styled(" [Not Downloaded]", Style::default().fg(Color::Gray))
            };
            let style = if state.list_state.selected() == Some(i) {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else if item.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(item.display_text(), style),
                status_indicator,
            ]))
        })
        .collect();

    let filter_text = match state.filter_mode {
        FilterMode::All => "All",
        FilterMode::HideDownloaded => "Hide Downloaded",
        FilterMode::ShowOnlyDownloaded => "Only Downloaded",
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Artifacts ({} selected, {} shown, Filter: {})", state.selected_count(), state.items.len(), filter_text)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED));

    f.render_stateful_widget(list, chunks[1], &mut state.list_state.clone());

    // Footer with instructions
        let instructions = match state.mode {
        SelectorMode::Selecting => {
            format!(
                "Output: {} | ↑↓: Navigate | Space: Select | Enter: Download | 'a': Select All | 'f': Filter | 'q': Quit",
                state.output_dir
            )
        }
        SelectorMode::Downloading => {
            format!("Downloading {} artifacts...", state.selected_count())
        }
    };

    let footer = Paragraph::new(instructions)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(footer, chunks[2]);
}

/// Run the artifact selector UI
pub async fn run_artifact_selector(
    artifacts: Vec<Artifact>,
    output_dir: String,
) -> Result<Vec<Artifact>, Box<dyn std::error::Error>> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = ArtifactSelector::new(artifacts, output_dir);

    loop {
        terminal.draw(|f| render_ui(f, &state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match state.mode {
                SelectorMode::Selecting => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Char(' ') => {
                            state.toggle_selection();
                        }
                        KeyCode::Up => {
                            state.previous();
                        }
                        KeyCode::Down => {
                            state.next();
                        }
                        KeyCode::PageUp => {
                            for _ in 0..10 {
                                state.previous();
                            }
                        }
                        KeyCode::PageDown => {
                            for _ in 0..10 {
                                state.next();
                            }
                        }
                        KeyCode::Char('a') => {
                            if state.selected_count() == state.items.len() {
                                state.deselect_all();
                            } else {
                                state.select_all();
                            }
                        }
                        KeyCode::Char('f') => {
                            state.toggle_filter();
                        }
                        KeyCode::Enter => {
                            if state.selected_count() > 0 {
                                state.mode = SelectorMode::Downloading;
                                terminal.draw(|f| render_ui(f, &state))?;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                SelectorMode::Downloading => {
                    break;
                }
            }
        }
    }

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;

    if matches!(state.mode, SelectorMode::Downloading) {
        Ok(state.selected_artifacts().into_iter().cloned().collect())
    } else {
        Ok(vec![])
    }
}
