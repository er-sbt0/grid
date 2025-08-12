use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    style::Color,
    widgets::{
        canvas::{Canvas, Rectangle},
        Block,
    },
    symbols::Marker,
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
            
            // Create a canvas with a smooth square
            let canvas = Canvas::default()
                .block(Block::bordered().title("Smooth Square"))
                .marker(Marker::HalfBlock) // Use HalfBlock for smoother appearance
                .x_bounds([0.0, 100.0])    // Set coordinate bounds
                .y_bounds([0.0, 100.0])
                .paint(|ctx| {
                    // Draw a centered square
                    let square_size = 30.0;
                    let center_x = 50.0;
                    let center_y = 50.0;
                    
                    ctx.draw(&Rectangle {
                        x: center_x - square_size / 2.0,
                        y: center_y - square_size / 2.0,
                        width: square_size,
                        height: square_size,
                        color: Color::Cyan,
                    });
                });
            
            frame.render_widget(canvas, area);
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
