//! Keybinding model: config-driven key -> Action resolution.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(default = "vec_one_q")]
    pub quit: Vec<String>,
    #[serde(default = "vec_one_question")]
    pub help: Vec<String>,
    #[serde(default = "vec_one_t")]
    pub next_theme: Vec<String>,
    #[serde(default = "vec_one_shift_t")]
    pub prev_theme: Vec<String>,
    #[serde(default = "vec_one_l")]
    pub next_layout: Vec<String>,
    #[serde(default = "vec_one_f")]
    pub toggle_fullscreen: Vec<String>,
    #[serde(default = "vec_one_shift_f")]
    pub cycle_fullscreen: Vec<String>,
    #[serde(default = "vec_one_slash")]
    pub search: Vec<String>,
    #[serde(default = "vec_one_ctrl_p")]
    pub command_palette: Vec<String>,
    #[serde(default = "vec_one_escape")]
    pub cancel: Vec<String>,
    #[serde(default = "vec_one_k")]
    pub kill_process: Vec<String>,
    #[serde(default = "vec_one_up")]
    pub process_up: Vec<String>,
    #[serde(default = "vec_one_down")]
    pub process_down: Vec<String>,
    #[serde(default = "vec_one_s")]
    pub cycle_sort: Vec<String>,
}

fn vec_one_q() -> Vec<String> {
    vec!["q".into()]
}
fn vec_one_question() -> Vec<String> {
    vec!["?".into()]
}
fn vec_one_t() -> Vec<String> {
    vec!["t".into()]
}
fn vec_one_shift_t() -> Vec<String> {
    vec!["T".into()]
}
fn vec_one_l() -> Vec<String> {
    vec!["l".into()]
}
fn vec_one_f() -> Vec<String> {
    vec!["f".into()]
}
fn vec_one_shift_f() -> Vec<String> {
    vec!["F".into()]
}
fn vec_one_slash() -> Vec<String> {
    vec!["/".into()]
}
fn vec_one_ctrl_p() -> Vec<String> {
    vec!["ctrl+p".into(), "ctrl+P".into()]
}
fn vec_one_escape() -> Vec<String> {
    vec!["escape".into()]
}
fn vec_one_k() -> Vec<String> {
    vec!["k".into()]
}
fn vec_one_up() -> Vec<String> {
    vec!["up".into()]
}
fn vec_one_down() -> Vec<String> {
    vec!["down".into()]
}
fn vec_one_s() -> Vec<String> {
    vec!["s".into()]
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            quit: vec_one_q(),
            help: vec_one_question(),
            next_theme: vec_one_t(),
            prev_theme: vec_one_shift_t(),
            next_layout: vec_one_l(),
            toggle_fullscreen: vec_one_f(),
            cycle_fullscreen: vec_one_shift_f(),
            search: vec_one_slash(),
            command_palette: vec_one_ctrl_p(),
            cancel: vec_one_escape(),
            kill_process: vec_one_k(),
            process_up: vec_one_up(),
            process_down: vec_one_down(),
            cycle_sort: vec_one_s(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Quit,
    ToggleHelp,
    NextTheme,
    PreviousTheme,
    RandomTheme,
    NextLayout,
    ToggleFullscreen,
    CycleFullscreen,
    Search,
    OpenCommandPalette,
    Cancel,
    SelectTheme(usize),
    SelectLayout(usize),
    NavigateThemes,
    NavigateLayouts,
    KillProcess,
    ProcessUp,
    ProcessDown,
    SortByCpu,
}

impl Action {
    /// Whether executing this action changes persisted config (theme/layout).
    /// Used to avoid rewriting `config.json` on every palette Enter.
    pub fn persists(&self) -> bool {
        matches!(
            self,
            Action::Quit
                | Action::NextTheme
                | Action::PreviousTheme
                | Action::RandomTheme
                | Action::NextLayout
                | Action::SelectTheme(_)
                | Action::SelectLayout(_)
        )
    }
}

impl Keybindings {
    pub fn resolve(&self, key_str: &str) -> Option<Action> {
        if self.quit.contains(&key_str.to_string()) {
            return Some(Action::Quit);
        }
        if self.help.contains(&key_str.to_string()) {
            return Some(Action::ToggleHelp);
        }
        if self.next_theme.contains(&key_str.to_string()) {
            return Some(Action::NextTheme);
        }
        if self.prev_theme.contains(&key_str.to_string()) {
            return Some(Action::PreviousTheme);
        }
        if self.next_layout.contains(&key_str.to_string()) {
            return Some(Action::NextLayout);
        }
        if self.toggle_fullscreen.contains(&key_str.to_string()) {
            return Some(Action::ToggleFullscreen);
        }
        if self.cycle_fullscreen.contains(&key_str.to_string()) {
            return Some(Action::CycleFullscreen);
        }
        if self.search.contains(&key_str.to_string()) {
            return Some(Action::Search);
        }
        if self.command_palette.contains(&key_str.to_string()) {
            return Some(Action::OpenCommandPalette);
        }
        if self.cancel.contains(&key_str.to_string()) {
            return Some(Action::Cancel);
        }
        if self.kill_process.contains(&key_str.to_string()) {
            return Some(Action::KillProcess);
        }
        if self.process_up.contains(&key_str.to_string()) {
            return Some(Action::ProcessUp);
        }
        if self.process_down.contains(&key_str.to_string()) {
            return Some(Action::ProcessDown);
        }
        if self.cycle_sort.contains(&key_str.to_string()) {
            return Some(Action::SortByCpu);
        }
        None
    }
}
