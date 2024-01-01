use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::io;
use tui_textarea::{Input, Key, TextArea};

// TODO: add licenses
// TODO: support clicking the text areas
// DISCUSS: can we have a history? Like a TI-83 or WolframAlpha or something
// TODO: allow changing any two fields and let the third populate.
// - DISCUSS: how do we know which two are fixed and which is calculated?

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textareas: Vec<TextArea> = ["initial", "final", "percent change"]
        .iter()
        .map(|title| {
            let mut textarea = TextArea::default();
            textarea.set_block(Block::default().borders(Borders::ALL).title(*title));
            // Turn off the default style of underlined text
            textarea.set_cursor_line_style(Style::default());
            textarea
        })
        .collect();

    // By default, the last chunk of the computed layout is expanded to fill the remaining space. To avoid this behavior, add an unused Constraint::Min(0) as the last constraint.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Room for the text boxes
            Constraint::Min(0),    // Fills remaining space
        ]);

    let mut which = 0;
    activate(&mut textareas[0]);
    inactivate(&mut textareas[1]);
    inactivate(&mut textareas[2]);

    loop {
        term.draw(|f| {
            let main_layout = main_layout.split(f.size());
            let text_layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(main_layout[0]);

            for (textarea, layout) in textareas.iter().zip(text_layouts.iter()) {
                let widget = textarea.widget();
                f.render_widget(widget, *layout);
            }
        })?;
        match crossterm::event::read()?.into() {
            // Quit
            Input { key: Key::Esc, .. } => break,

            // Change selected textarea
            // FIXME: shift-tab used to work, but now it doesn't
            Input {
                key: Key::Tab,
                shift: true,
                ..
            } => {
                inactivate(&mut textareas[which]);
                which = (which - 1) % textareas.len();
                activate(&mut textareas[which]);
            }
            Input {
                key: Key::Tab,
                shift: false,
                ..
            } => {
                inactivate(&mut textareas[which]);
                which = (which + 1) % textareas.len();
                activate(&mut textareas[which]);
            }

            // Disallow newlines
            Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Char('j'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {}

            // Pass inputs through to the textarea
            input => {
                // TextArea::input returns if the input modified its text.
                if textareas[which].input(input) {
                    // If that last character does not result in a valid float,
                    // remove it on the user's behalf
                    match is_float(textareas[which].lines()) {
                        true => {
                            calculate(&mut textareas.as_mut_slice());
                        }
                        false => {
                            textareas[which].delete_char();
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Left textarea: {:?}", textareas[0].lines());
    println!("Right textarea: {:?}", textareas[1].lines());
    Ok(())
}

fn calculate(textareas: &mut [TextArea]) {
    let [initial_textarea, final_textarea, percent_changed_textarea] = textareas else {
        todo!()
    };
    let initial_value = initial_textarea.lines().join("\n").parse::<f64>();
    let final_value = final_textarea.lines().join("\n").parse::<f64>();
    if let (Ok(initial_value), Ok(final_value)) = (initial_value, final_value) {
        let percent_changed: f64 = (final_value - initial_value) / initial_value.abs();
        // Delete all existing text
        percent_changed_textarea.move_cursor(tui_textarea::CursorMove::Down);
        percent_changed_textarea.delete_line_by_head();
        // Replace with the new text
        percent_changed_textarea.insert_str(percent_changed.to_string());
    }
}

fn is_float(lines: &[String]) -> bool {
    let float: Result<f64, _> = lines.join("\n").parse();
    float.is_ok()
}

fn inactivate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_style(textarea.style().add_modifier(Modifier::HIDDEN));
    textarea.set_block(
        textarea
            .block()
            .cloned()
            .unwrap()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray)),
    );
}

fn activate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_style(textarea.style().add_modifier(Modifier::REVERSED));
    textarea.set_block(
        textarea
            .block()
            .cloned()
            .unwrap()
            .borders(Borders::ALL)
            .style(Style::default()),
    );
}
