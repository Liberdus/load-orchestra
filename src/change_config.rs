use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use serde_json::Value;
use std::error::Error;
use std::io::{self, stdout};
use std::time::Duration;

struct JsonBrowserState {
    expanded_nodes: Vec<bool>, // Track expanded state of JSON nodes
    selected_index: usize,     // Currently selected line index
    scroll_offset: usize,      // Offset for scrolling
}

impl JsonBrowserState {
    fn new() -> Self {
        Self {
            expanded_nodes: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    fn toggle_node(&mut self, index: usize) {
        if index < self.expanded_nodes.len() {
            self.expanded_nodes[index] = !self.expanded_nodes[index];
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset -= 1;
        }
    }

    fn move_down(&mut self, max_index: usize, terminal_height: usize) {
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
        if self.selected_index >= self.scroll_offset + terminal_height {
            self.scroll_offset += 1;
        }
    }
}

/// Creates a JSON object that only includes the direct ancestor path to the specified leaf.
fn create_change_object(path: &[String], new_value: Value) -> Value {
    let mut result = Value::Object(serde_json::Map::new());
    let mut current = &mut result;

    for key in path.iter().take(path.len() - 1) {
        // Ensure current is an object and add the intermediate keys
        if !current.is_object() {
            *current = Value::Object(serde_json::Map::new());
        }

        current = current
            .as_object_mut()
            .unwrap()
            .entry(key.clone())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
    }

    // Set the final key to the new value
    if let Some(last_key) = path.last() {
        if let Value::Object(map) = current {
            map.insert(last_key.clone(), new_value);
        }
    }

    result
}

/// Displays a prompt to the user and captures input.
fn ratatui_prompt<B: Backend>(
    terminal: &mut Terminal<B>,
    prompt_message: &str,
) -> Result<Value, Box<dyn Error>> {
    let mut input = String::new();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title("Input Prompt")
                .borders(Borders::ALL);

            let prompt = Paragraph::new(format!("{}\n> {}", prompt_message, input))
                .block(block)
                .style(Style::default());

            f.render_widget(prompt, size);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        let v = parse_dynamic_value(&input.trim());

                        return Ok(v);
                    }
                    KeyCode::Esc => {
                        return Ok(serde_json::Value::Null);
                    }
                    _ => {}
                }
            }
        }
    }
}
fn parse_dynamic_value(input: &str) -> Value {
    // Check for boolean
    if input.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if input.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    }
    else if input.eq_ignore_ascii_case("null") {
        Value::Null
    }
    // Check for number
    else if let Ok(number) = input.parse::<u128>() {
        Value::Number(serde_json::Number::from_u128(number).unwrap())
    }
    // Fallback to string
    else {
        Value::String(input.to_string())
    }
}
/// Renders JSON recursively while dynamically tracking indices and resizing expanded_nodes.
/// Returns the lines to render, along with the path to each node.
fn render_json<'a>(
    value: &'a Value,
    indent: usize,
    expanded_nodes: &mut Vec<bool>,
    parent_expanded: bool,
    index_counter: &mut usize,
    path: Vec<String>,
) -> Vec<(String, Vec<String>, bool, &'a Value)> {
    let mut lines = Vec::new();
    if !parent_expanded {
        return lines;
    }

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if expanded_nodes.len() <= *index_counter {
                    expanded_nodes.push(false);
                }

                let prefix = if matches!(val, Value::Object(_) | Value::Array(_)) {
                    if expanded_nodes[*index_counter] {
                        "▼"
                    } else {
                        "▶"
                    }
                } else {
                    " "
                };

                let new_path = {
                    let mut p = path.clone();
                    p.push(key.clone());
                    p
                };

                let is_leaf = !matches!(val, Value::Object(_) | Value::Array(_));
                lines.push((
                    format!(
                        "{}{} {}: {}",
                        " ".repeat(indent),
                        prefix,
                        key,
                        if is_leaf { val.to_string() } else { "".to_string() }
                    ),
                    new_path.clone(),
                    is_leaf,
                    val,
                ));

                let current_index = *index_counter;
                *index_counter += 1;

                if expanded_nodes[current_index] {
                    lines.extend(render_json(
                        val,
                        indent + 2,
                        expanded_nodes,
                        true,
                        index_counter,
                        new_path.clone(),
                    ));
                }
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                if expanded_nodes.len() <= *index_counter {
                    expanded_nodes.push(false);
                }

                let prefix = if matches!(val, Value::Object(_) | Value::Array(_)) {
                    if expanded_nodes[*index_counter] {
                        "▼"
                    } else {
                        "▶"
                    }
                } else {
                    " "
                };

                let new_path = {
                    let mut p = path.clone();
                    p.push(i.to_string());
                    p
                };

                let is_leaf = !matches!(val, Value::Object(_) | Value::Array(_));
                lines.push((
                    format!(
                        "{}{} [{}]: {}",
                        " ".repeat(indent),
                        prefix,
                        i,
                        if is_leaf { val.to_string() } else { "".to_string() }
                    ),
                    new_path.clone(),
                    is_leaf,
                    val,
                ));

                let current_index = *index_counter;
                *index_counter += 1;

                if expanded_nodes[current_index] {
                    lines.extend(render_json(
                        val,
                        indent + 2,
                        expanded_nodes,
                        true,
                        index_counter,
                        new_path.clone(),
                    ));
                }
            }
        }
        _ => {}
    }

    lines
}

pub fn init(json: Value) -> Result<Option<Value>, Box<dyn Error>> {
    clear_screen();

    let mut state = JsonBrowserState::new();
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)].as_ref())
                .split(f.size());

            let mut index_counter = 0;
            let json_lines = render_json(&json, 0, &mut state.expanded_nodes, true, &mut index_counter, vec![]);

            let terminal_height = f.size().height as usize;
            let visible_lines = &json_lines[state.scroll_offset..std::cmp::min(
                state.scroll_offset + terminal_height,
                json_lines.len(),
            )];

            let items: Vec<ListItem> = visible_lines
                .iter()
                .enumerate()
                .map(|(i, (line, _, _, _))| {
                    let actual_index = state.scroll_offset + i;
                    let style = if actual_index == state.selected_index {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(line.clone()).style(style)
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Node Settings"));
            f.render_widget(list, chunks[0]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let mut index_counter = 0;
                let json_lines = render_json(&json, 0, &mut state.expanded_nodes, true, &mut index_counter, vec![]);

                match key.code {
                    KeyCode::Up => state.move_up(),
                    KeyCode::Down => state.move_down(
                        state.expanded_nodes.len() - 1,
                        terminal.size()?.height as usize,
                    ),
                    KeyCode::Enter => {
                        if let Some((_, path, is_leaf, value)) = json_lines.get(state.selected_index) {
                            if *is_leaf {
                                let new_value = ratatui_prompt(&mut terminal, &format!("Enter new value for {}:", path.join("->")))?;
                                if new_value.is_null() {
                                    continue;
                                }
                                let new_json = create_change_object(&path, new_value);
                                disable_raw_mode()?;
                                clear_screen();
                                return Ok(Some(new_json));
                            } else {
                                state.toggle_node(state.selected_index);
                            }
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(None)
}

fn clear_screen() {
    execute!(stdout(), Clear(ClearType::All)).expect("Failed to clear terminal");
}
