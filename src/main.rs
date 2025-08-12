use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    style::Color,
    widgets::Block,
    text::Text,
};

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
            
            // Calculate square position and size
            let square_size = 20; // Size in terminal characters
            let center_x = area.width / 2;
            let center_y = area.height / 2;
            let start_x = center_x - square_size / 2;
            let start_y = center_y - square_size / 2;
            
            // Create the square using UTF-8 box drawing characters
            let mut lines = Vec::new();
            
            for y in 0..square_size {
                let mut line = String::new();
                for x in 0..square_size {
                    if y == 0 && x == 0 {
                        line.push('┌'); // Top-left corner
                    } else if y == 0 && x == square_size - 1 {
                        line.push('┐'); // Top-right corner
                    } else if y == square_size - 1 && x == 0 {
                        line.push('└'); // Bottom-left corner
                    } else if y == square_size - 1 && x == square_size - 1 {
                        line.push('┘'); // Bottom-right corner
                    } else if y == 0 || y == square_size - 1 {
                        line.push('─'); // Horizontal line
                    } else if x == 0 || x == square_size - 1 {
                        line.push('│'); // Vertical line
                    } else {
                        line.push(' '); // Empty space inside
                    }
                }
                lines.push(line);
            }
            
            // Render each line at the correct position
            for (i, line) in lines.iter().enumerate() {
                let y_pos = start_y + i as u16;
                if y_pos < area.height {
                    let text = Text::from(line.clone()).style(Color::Cyan);
                    let line_area = ratatui::layout::Rect {
                        x: start_x,
                        y: y_pos,
                        width: square_size as u16,
                        height: 1,
                    };
                    frame.render_widget(text, line_area);
                }
            }
            
            // Add a title
            let title_text = Text::from("Smooth Square with UTF-8 Box Drawing Characters").style(Color::White);
            let title_area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: area.width,
                height: 1,
            };
            frame.render_widget(title_text, title_area);
        })?;
        
        // Handle events
        if handle_events()? {
            break;
        }
    }
    
    Ok(())
}

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                _ => {}
            }
        }
        _ => {}
    }
    Ok(false)
}
