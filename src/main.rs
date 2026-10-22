use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode,
        enable_raw_mode
    },
};

use ratatui::{
    Terminal, backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use std::{io, time::Duration};
use walkdir::WalkDir;
use std::path::PathBuf;

struct Application {
    tracks: Vec<PathBuf>,
    state: ListState,
    //selected_track: usize,
}

impl Application {
    fn new(music_dir: &str) -> Self {
        let mut tracks = Vec::new();

        for t in WalkDir::new(music_dir)
            .into_iter()
            .filter_map(|e| e.ok()){
            
            let path = t.path();
            if path.is_file(){

                if let Some(ext) = path.extension()
                    .and_then(|e| e.to_str()){
                    
                    if matches!(ext.to_lowercase().as_str(), "flac" | "mp3" | "wav"){
                        tracks.push(path.to_path_buf());

                    }
                }
            }
        }
        let mut state = ListState::default();

        if !tracks.is_empty(){
            state.select(Some(0));
        }
        Self{
            tracks,
            state
        }
    }

    fn next(&mut self){
        if self.tracks.is_empty() {
            return;
        }
        let i = match self.state.selected(){
            Some(i) => {
                if i >= self.tracks.len()-1 {
                    0
                }
                else {
                    i+1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.tracks.is_empty(){
            return;
        }
        let i = match self.state.selected(){
            Some(i) => {
                if i == 0{
                    self.tracks.len()-1
                }
                else{
                    i-1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<(), io::Error> {
    let music_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("."));

    let mut appln = Application::new(&music_path);
    
    // term setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // appl loop (main)
    let res = app(&mut term, &mut appln);

    // do restore terminal on exit
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen
    )?;
    term.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn app<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    appln: &mut Application,
    ) -> io::Result<()> {

    loop {
        term.draw(
            |f| {
                let size = f.area();

                // main layout using chunks - Title, Track, view
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ]).split(size);
                
                // title block
                let title = Paragraph::new(" Rusty CLI Player ")
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title("Core"));
                f.render_widget(title, chunks[0]);
                
                let items: Vec<ListItem> = appln.tracks
                    .iter()
                    .map(
                        |p| {
                            let name = p.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown Track");
                            ListItem::new(name.to_string())
                        }
                    ).collect();
                
                /*
                let playlist_contents = if appln.tracks.is_empty() {
                    String::from("No supported audio files found")

                }
                else{
                    format!("Loaded {} tracks", appln.tracks.len())
                };
                
                // placeholder
                let main_placeholder = Paragraph::new(playlist_contents)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Playlist"));
                f.render_widget(main_placeholder, chunks[1]);

                */

                let playlist_title = format!(" Playlist ({} tracks) ", appln.tracks.len());
                let playlist_widget = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(playlist_title))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    ).highlight_symbol(">> ");

                f.render_stateful_widget(
                    playlist_widget,
                    chunks[1],
                    &mut appln.state
                );

                // status bar
                let current_sel = appln.state.selected().unwrap_or(0);
                let status_text = if appln.tracks.is_empty() {
                    String::from(" Status: Idle | No tracks found ")

                }
                else {
                    format!(" Status: Idle | Selected: {}/{} ", current_sel+1, appln.tracks.len())
                };
                let status = Paragraph::new(status_text)
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(status, chunks[2]);
            }
        )?;

        // keyboard input handler
        // polling 100ms
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                /*
                if let KeyCode::Char('q') = key.code {
                    break;
                }
                if let KeyCode::UP = event::read()?{
                    if !appln.tracks.is_empty() && appln.selected_track < appln.tracks.len() - 1 {
                        appln.selected_track+=1;
                    }
                }
                if let KeyCode::DOWN = event::read()? {
                    if appln.selected_track > 0 {
                        appln.selected_track-=1;
                    }
                }
                */
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => appln.next(),
                    //{
                        //if !appln.tracks.is_empty() && appln.selected_track < appln.tracks.len() - 1 {
                        //    appln.selected_track += 1;
                        //}
                    //}
                    KeyCode::Up => appln.previous(),
                    //{
                    //    if appln.selected_track > 0 {
                    //        appln.selected_track -= 1;
                    //    }
                    //}
                    _ => {}
                }
            } 
        }
    }

    Ok(())
}