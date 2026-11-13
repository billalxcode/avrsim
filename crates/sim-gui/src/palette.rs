//! Command palette catalog and state.

/// One executable palette command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Save,
    Open,
    LoadHex,
    RunPause,
    Step,
    Reset,
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    Delete,
    ToggleGrid,
    ToggleLibrary,
    ToggleInspector,
    ToggleLegend,
    ToggleWaveform,
    ZoomReset,
    SelectTool,
    WireTool,
}

impl Command {
    /// All commands shown by the palette.
    pub const ALL: [Self; 20] = [
        Self::Save,
        Self::Open,
        Self::LoadHex,
        Self::RunPause,
        Self::Step,
        Self::Reset,
        Self::Undo,
        Self::Redo,
        Self::Copy,
        Self::Paste,
        Self::Duplicate,
        Self::Delete,
        Self::ToggleGrid,
        Self::ToggleLibrary,
        Self::ToggleInspector,
        Self::ToggleLegend,
        Self::ToggleWaveform,
        Self::ZoomReset,
        Self::SelectTool,
        Self::WireTool,
    ];

    /// Returns the display label for the current simulation state.
    #[must_use]
    pub fn label(self, running: bool) -> &'static str {
        if self == Self::RunPause {
            return if running {
                "Pause simulation"
            } else {
                "Run simulation"
            };
        }
        self.static_label()
    }

    const fn static_label(self) -> &'static str {
        match self {
            Self::Copy => "Copy selection",
            Self::Paste => "Paste selection",
            Self::Duplicate => "Duplicate selection",
            _ => self.edit_label(),
        }
    }

    const fn edit_label(self) -> &'static str {
        match self {
            Self::Save => "Save project",
            Self::Open => "Open project",
            Self::LoadHex => "Load HEX firmware",
            Self::RunPause => "Run simulation",
            Self::Step => "Step instruction",
            Self::Reset => "Reset simulation",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            _ => self.view_label(),
        }
    }

    const fn view_label(self) -> &'static str {
        match self {
            Self::Delete => "Delete selection",
            Self::ToggleGrid => "Toggle grid",
            Self::ToggleLibrary => "Toggle library",
            Self::ToggleInspector => "Toggle inspector",
            Self::ToggleLegend => "Toggle net legend",
            Self::ToggleWaveform => "Toggle waveform",
            Self::ZoomReset => "Reset zoom",
            Self::SelectTool => "Select tool",
            Self::WireTool => "Wire tool",
            _ => "Unknown command",
        }
    }
}

/// Palette visibility and keyboard selection state.
#[derive(Default)]
pub struct State {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub focus: bool,
}

impl State {
    /// Toggles the palette and resets its query when opening.
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
            return;
        }
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.focus = true;
    }

    /// Closes the palette.
    pub fn close(&mut self) {
        self.open = false;
        self.focus = false;
    }

    /// Moves the selection with wrapping.
    pub fn move_selection(&mut self, count: usize, delta: isize) {
        self.selected = step_selection(self.selected, count, delta);
    }
}

/// Returns commands whose labels contain the query, ignoring case.
#[must_use]
pub fn matching(query: &str, running: bool) -> Vec<Command> {
    let needle = query.trim().to_lowercase();
    Command::ALL
        .into_iter()
        .filter(|command| {
            needle.is_empty() || command.label(running).to_lowercase().contains(&needle)
        })
        .collect()
}

/// Steps a selection index with wrapping.
#[must_use]
pub fn step_selection(index: usize, count: usize, delta: isize) -> usize {
    if count == 0 {
        return 0;
    }
    (index as isize + delta).rem_euclid(count as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::{Command, State, matching, step_selection};

    #[test]
    fn filters_case_insensitive() {
        assert_eq!(matching("SAVE", false), vec![Command::Save]);
        assert!(matching("missing", false).is_empty());
        assert_eq!(matching("", false).len(), Command::ALL.len());
    }

    #[test]
    fn run_label_depends_on_state() {
        assert!(matching("pause", false).is_empty());
        assert_eq!(matching("pause", true), vec![Command::RunPause]);
        assert_eq!(matching("run", false), vec![Command::RunPause]);
    }

    #[test]
    fn selection_wraps_both_ways() {
        assert_eq!(step_selection(0, 3, -1), 2);
        assert_eq!(step_selection(2, 3, 1), 0);
        assert_eq!(step_selection(0, 0, 1), 0);
    }

    #[test]
    fn toggle_resets_query_and_selection() {
        let mut state = State::default();
        state.query.push_str("save");
        state.selected = 4;
        state.toggle();
        assert!(state.open);
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(state.focus);
        state.toggle();
        assert!(!state.open);
    }
}
