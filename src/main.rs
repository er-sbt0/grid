use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{style::Color, text::Text};

fn main() -> std::io::Result<()> {
    // Initialize the terminal
    let mut terminal = ratatui::init();

    let result = run(&mut terminal);

    // Restore the terminal
    ratatui::restore();

    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Define tree structure: (x, y, label)
            let nodes = vec![
                (5, 10, "Root"), // Root node
                (15, 6, "A"),    // Left child
                (15, 14, "B"),   // Right child
                (25, 4, "A1"),   // Left-left grandchild
                (25, 8, "A2"),   // Left-right grandchild
                (25, 12, "B1"),  // Right-left grandchild
                (25, 16, "B2"),  // Right-right grandchild
            ];

            // Define connections: (from_node_index, to_node_index)
            let connections = vec![
                (0, 1), // Root -> A
                (0, 2), // Root -> B
                (1, 3), // A -> A1
                (1, 4), // A -> A2
                (2, 5), // B -> B1
                (2, 6), // B -> B2
            ];

            // Clear the screen and draw title
            let title_text =
                Text::from("Tree Graph with Smooth Squares and Grid Lines").style(Color::White);
            let title_area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: area.width,
                height: 1,
            };
            frame.render_widget(title_text, title_area);

            // Draw connections first (so they appear behind nodes)
            for (from_idx, to_idx) in &connections {
                let from_node = &nodes[*from_idx];
                let to_node = &nodes[*to_idx];
                draw_connection(frame, from_node, to_node, area);
            }

            // Draw nodes
            for (x, y, label) in &nodes {
                draw_node(frame, *x, *y, label, area);
            }
        })?;

        // Handle events
        if handle_events()? {
            break;
        }
    }

    Ok(())
}

fn draw_node(frame: &mut ratatui::Frame, x: u16, y: u16, label: &str, area: ratatui::layout::Rect) {
    let node_size = 3; // 3x3 square

    // Ensure the node fits within the terminal area
    if x + node_size >= area.width || y + node_size >= area.height {
        return;
    }

    // Draw the square node
    let lines = ["┌─┐", "│ │", "└─┘"];

    for (i, line) in lines.iter().enumerate() {
        let y_pos = y + i as u16;
        if y_pos < area.height {
            let text = Text::from(line.to_string()).style(Color::Cyan);
            let line_area = ratatui::layout::Rect {
                x,
                y: y_pos,
                width: node_size as u16,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }

    // Draw the label below the node
    if y + node_size + 1 < area.height {
        let label_text = Text::from(label.to_string()).style(Color::Yellow);
        let label_area = ratatui::layout::Rect {
            x,
            y: y + node_size as u16,
            width: label.len() as u16,
            height: 1,
        };
        frame.render_widget(label_text, label_area);
    }
}

fn draw_connection(
    frame: &mut ratatui::Frame,
    from: &(u16, u16, &str),
    to: &(u16, u16, &str),
    area: ratatui::layout::Rect,
) {
    let from_x = from.0 + 2; // Connect from right edge of source node (middle of 3-wide node)
    let from_y = from.1 + 1; // Connect from middle of source node (middle of 3-high node)
    let to_x = to.0; // Connect to left edge of target node
    let to_y = to.1 + 1; // Connect to middle of target node

    // Draw horizontal line from source to intermediate point
    let mid_x = (from_x + to_x) / 2;

    // Horizontal line from source node
    for x in (from_x + 1)..mid_x {
        if x < area.width && from_y < area.height {
            let text = Text::from("─").style(Color::Green);
            let line_area = ratatui::layout::Rect {
                x,
                y: from_y,
                width: 1,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }

    // Vertical line (if needed)
    let min_y = from_y.min(to_y);
    let max_y = from_y.max(to_y);

    if from_y != to_y {
        for y in min_y..=max_y {
            if mid_x < area.width && y < area.height {
                let char = if y == min_y && from_y < to_y {
                    "┐"
                } else if y == max_y && from_y > to_y {
                    "┘"
                } else if y == min_y && from_y > to_y {
                    "┌"
                } else if y == max_y && from_y < to_y {
                    "└"
                } else {
                    "│"
                };

                let text = Text::from(char).style(Color::Green);
                let line_area = ratatui::layout::Rect {
                    x: mid_x,
                    y,
                    width: 1,
                    height: 1,
                };
                frame.render_widget(text, line_area);
            }
        }
    }

    // Horizontal line to target node
    for x in (mid_x + 1)..to_x {
        if x < area.width && to_y < area.height {
            let text = Text::from("─").style(Color::Green);
            let line_area = ratatui::layout::Rect {
                x,
                y: to_y,
                width: 1,
                height: 1,
            };
            frame.render_widget(text, line_area);
        }
    }
}

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}
