//! The in-game console which allows changing cvars at runtime.

use macroquad::{
    prelude::*,
    ui::{
        hash, root_ui,
        widgets::{Group, Label},
        Layout, Skin,
    },
};

use crate::cvars::Cvars;

#[derive(Debug, Clone, Default)]
pub struct Console {
    is_open: bool,
    prompt: String,
    prompt_saved: String,
    history: Vec<HistoryLine>,
    /// Where we are in history when using up and down keys.
    history_index: usize,
    /// Where we are in the history view when scrolling using page up and down keys.
    history_view_index: usize,
    input: ConsoleInput,
    input_prev: ConsoleInput,
}

impl Console {
    pub fn new() -> Self {
        Self {
            is_open: false,
            prompt: String::new(),
            prompt_saved: String::new(),
            history: Vec::new(),
            history_index: 0,
            history_view_index: 0,
            input: ConsoleInput::new(),
            input_prev: ConsoleInput::new(),
        }
    }

    pub fn update(&mut self, cvars: &mut Cvars) {
        self.input_prev = self.input;
        self.input = get_input();

        self.open_close();

        if self.is_open {
            self.process_input();
            self.draw_console(cvars);
            if !self.input_prev.enter && self.input.enter && !self.prompt.is_empty() {
                self.process_input_text(cvars);
            }
        }
    }

    /// Open or close the console based on user's input.
    fn open_close(&mut self) {
        let pressed_console = !self.input_prev.console && self.input.console;
        let pressed_escape = !self.input_prev.escape && self.input.escape;
        if !self.is_open && pressed_console {
            self.is_open = true;
            show_mouse(true);
        } else if self.is_open && (pressed_console || pressed_escape) {
            self.is_open = false;
            show_mouse(false);
        }
    }

    /// Sanitize input text, handle cycling through history, etc.
    fn process_input(&mut self) {
        // The semicolon (default console bind) gets typed into the console
        // when opening it (but interestingly not closing).
        // Just disallow it completely we don't allow multiple commands on one line
        // so there's currently no need for it.
        // This has the side effect, that the text cursor moves one char to the right
        // with each open/close cycle but that's OK.
        // LATER A less hacky input system would be great.
        self.prompt = self.prompt.replace(';', "");

        // Detect key pressed based on previous and current state.
        // MQ's UI doesn't seem to hae a built-in way to detecting keyboard events.
        let pressed_up = !self.input_prev.up && self.input.up;
        let pressed_down = !self.input_prev.down && self.input.down;
        let pressed_page_up = !self.input_prev.page_up && self.input.page_up;
        let pressed_page_down = !self.input_prev.page_down && self.input.page_down;

        // Go back in history
        if pressed_up {
            // Save the prompt so that users can go back in history,
            // then come back to present and get what they typed back.
            if self.history_index == self.history.len() {
                self.prompt_saved = self.prompt.clone();
            }

            let search_slice = &self.history[0..self.history_index];
            if let Some(new_index) = search_slice
                .iter()
                .rposition(|hist_line| hist_line.is_input)
            {
                self.history_index = new_index;
                self.prompt = self.history[self.history_index].text.clone();
            }
        }

        // Go forward in history
        if pressed_down && self.history_index < self.history.len() {
            // Since we're starting at history_index+1, the history.len() condition must remain here
            // otherwise the range could start at history.len()+1 and panic.
            let search_slice = &self.history[self.history_index + 1..];
            if let Some(new_index) = search_slice.iter().position(|hist_line| hist_line.is_input) {
                // `position` starts counting from the iterator's start,
                // not from history's start so we add the found index to what we skipped
                // instead of using it directly.
                self.history_index += new_index + 1;
                self.prompt = self.history[self.history_index].text.clone();
            } else {
                // We're at the end of history, restore the saved prompt.
                self.history_index = self.history.len();
                self.prompt = self.prompt_saved.clone();
            }
        }

        // Scroll history up
        let jump = 10; // LATER configurable
        if pressed_page_up {
            self.history_view_index = self.history_view_index.saturating_sub(jump);
            if self.history_view_index == 0 && !self.history.is_empty() {
                // Keep at least one line in history when possible
                // because scrolling up to an empty view looks weird.
                self.history_view_index = 1;
            }
        }
        if pressed_page_down {
            self.history_view_index = (self.history_view_index + jump).min(self.history.len());
        }
    }

    /// Draw the console and the UI elements it needs.
    fn draw_console(&mut self, cvars: &Cvars) {
        // Draw background
        // Floor aligns to pixels, otherwise text renders poorly.
        let console_height = (screen_height() * cvars.con_height_fraction).floor();
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            console_height,
            Color::new(0.0, 0.0, 0.0, cvars.con_background_alpha),
        );
        draw_line(
            0.0,
            console_height,
            screen_width(),
            console_height,
            1.0,
            RED,
        );

        // Draw history
        // This doesn't allow copying but in MQ's UI there's no way to print text
        // which allows copying while preventing editing.
        if self.history_view_index >= 1 {
            let mut i = self.history_view_index - 1;
            let mut y = console_height - cvars.con_history_y_offset;
            loop {
                let text = if self.history[i].is_input {
                    format!("> {}", self.history[i].text)
                } else {
                    self.history[i].text.clone()
                };
                draw_text(
                    &text,
                    cvars.con_history_x,
                    y,
                    cvars.con_history_line_font_size,
                    WHITE,
                );
                if i == 0 || y < 0.0 {
                    break;
                }
                i -= 1;
                y -= cvars.con_history_line_height;
            }
        }

        // Prompt style
        let bg_image = Image::gen_image_color(1, 1, BLANK);
        let style = root_ui()
            .style_builder()
            .background(bg_image)
            .color(BLANK) // This hides the faint rectangle around a Group
            .text_color(WHITE)
            .build();
        let skin = Skin {
            label_style: style.clone(),
            editbox_style: style.clone(),
            group_style: style,
            ..root_ui().default_skin()
        };
        root_ui().push_skin(&skin);

        // Draw prompt - this uses MQ's UI so i don't have to reimplement basic text editing ops.
        let id_prompt = 0;
        let label_y = console_height - cvars.con_prompt_label_y_offset;
        Label::new(">")
            .position(vec2(cvars.con_prompt_label_x, label_y))
            .ui(&mut root_ui());
        // Can't set position on an InputText so we wrap it in a Group.
        let group_y = screen_height() * cvars.con_height_fraction - cvars.con_prompt_group_y_offset;
        Group::new(hash!(), vec2(screen_width() - 8.0, 20.0))
            .position(vec2(cvars.con_prompt_group_x, group_y))
            .layout(Layout::Horizontal)
            .ui(&mut root_ui(), |ui| {
                ui.input_text(id_prompt, "", &mut self.prompt);
            });

        // The prompt should have focus all the time.
        root_ui().set_input_focus(id_prompt);
    }

    /// The user pressed enter - process the line of text
    fn process_input_text(&mut self, cvars: &mut Cvars) {
        let hist_len_old = self.history.len();

        let hist_line = HistoryLine::new(self.prompt.clone(), true);
        self.history.push(hist_line);

        // The actual command parsing logic
        let res = self.process_line(cvars);
        if let Err(msg) = res {
            let hist_line = HistoryLine::new(msg, false);
            self.history.push(hist_line);
        }

        self.prompt = String::new();

        // Entering a new command resets the user's position in history to the end.
        self.history_index = self.history.len();

        // If the view was at the end, keep scrolling down as new lines are added.
        // Otherwise the view's position shouldn't change.
        if self.history_view_index == hist_len_old {
            self.history_view_index = self.history.len();
        }
    }

    /// Parse what the user typed and get or set a cvar
    fn process_line(&mut self, cvars: &mut Cvars) -> Result<(), String> {
        let mut parts = self.prompt.split_whitespace();
        let cvar_name = parts
            .next()
            .ok_or_else(|| "expected cvar name".to_owned())?;
        let cvar_value = match parts.next() {
            Some(val) => val,
            None => {
                let val = cvars.get_string(cvar_name)?;
                let hist_line = HistoryLine::new(val, false);
                self.history.push(hist_line);
                return Ok(());
            }
        };
        if let Some(rest) = parts.next() {
            return Err(format!("expected only cvar name and value, found {}", rest));
        }
        cvars.set_str(cvar_name, cvar_value)
    }

    /// Whether the console is open right now.
    ///
    /// Useful for example to ignore game-related input
    /// while the player is typing into console.
    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

#[derive(Debug, Clone)]
struct HistoryLine {
    text: String,
    is_input: bool,
}

impl HistoryLine {
    fn new(text: String, is_input: bool) -> Self {
        Self { text, is_input }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ConsoleInput {
    console: bool,
    escape: bool,
    enter: bool,
    up: bool,
    down: bool,
    page_up: bool,
    page_down: bool,
}

impl ConsoleInput {
    fn new() -> Self {
        Self::default()
    }
}

fn get_input() -> ConsoleInput {
    let mut input = ConsoleInput::new();
    if are_keys_pressed(&[KeyCode::GraveAccent, KeyCode::Semicolon]) {
        input.console = true;
    }
    if are_keys_pressed(&[KeyCode::Escape]) {
        input.escape = true;
    }
    if are_keys_pressed(&[KeyCode::Enter, KeyCode::KpEnter]) {
        input.enter = true;
    }
    if are_keys_pressed(&[KeyCode::Up]) {
        input.up = true;
    }
    if are_keys_pressed(&[KeyCode::Down]) {
        input.down = true;
    }
    if are_keys_pressed(&[KeyCode::PageUp]) {
        input.page_up = true;
    }
    if are_keys_pressed(&[KeyCode::PageDown]) {
        input.page_down = true;
    }
    input
}

fn are_keys_pressed(key_codes: &[KeyCode]) -> bool {
    for &key_code in key_codes {
        if is_key_pressed(key_code) {
            return true;
        }
    }
    false
}
