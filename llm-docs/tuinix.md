# Tuinix Crate Documentation

## Crate Summary

### Overview

`tuinix` is a lightweight Rust library for building terminal user interface (TUI) applications on Unix systems with minimal dependencies. The crate provides a clean, efficient API for creating terminal-based applications with only `libc` as a dependency.

### Key Use Cases and Benefits

- **Minimal Dependencies**: Only requires `libc`, making it suitable for resource-constrained environments
- **Terminal Management**: Complete control over terminal state including raw mode and alternate screen buffer
- **Event-Driven Input**: Unified handling of keyboard, mouse, and terminal resize events
- **Efficient Rendering**: Differential frame updates that only redraw changed portions of the screen
- **Styled Output**: ANSI color support and text styling (bold, italic, underline, etc.)
- **Flexible Integration**: Works with blocking I/O or integrates with external event loops like `mio`

### Design Philosophy

The library is designed around a frame-based rendering model where applications:
1. Create a `TerminalFrame` representing the desired terminal state
2. Write styled content to the frame using standard Rust formatting macros
3. Draw the frame to the terminal, which automatically calculates and applies only necessary updates
4. Poll for events (input, resize) and respond by drawing new frames

This approach separates the logic of "what to display" from "how to display it", making TUI applications easier to reason about and test.

---

## API Reference

### Core Types

#### `tuinix::Terminal`

The main terminal interface for building TUI applications.

```rust
pub struct tuinix::Terminal { /* fields omitted */ }
```

**Key Responsibilities:**
- Manages terminal state (raw mode, alternate screen)
- Handles input event processing
- Renders frames to the terminal
- Detects terminal resize events

**Lifecycle:**
- Only one `Terminal` instance can exist at a time
- Automatically restores terminal state on drop
- Installs panic handler to restore terminal state on panic

**Methods:**

```rust
pub fn tuinix::Terminal::new() -> std::io::Result<tuinix::Terminal>
```
Creates a new terminal interface. Enables raw mode, alternate screen, and hides cursor. Returns error if another `Terminal` exists or if stdin/stdout are not terminals.

```rust
pub fn tuinix::Terminal::size(&self) -> tuinix::TerminalSize
```
Returns the current terminal dimensions.

```rust
pub fn tuinix::Terminal::input_fd(&self) -> std::os::fd::RawFd
```
Returns the file descriptor for terminal input (stdin).

```rust
pub fn tuinix::Terminal::output_fd(&self) -> std::os::fd::RawFd
```
Returns the file descriptor for terminal output (stdout).

```rust
pub fn tuinix::Terminal::signal_fd(&self) -> std::os::fd::RawFd
```
Returns the file descriptor that receives SIGWINCH (resize) signal notifications.

```rust
pub fn tuinix::Terminal::enable_mouse_input(&mut self) -> std::io::Result<()>
```
Enables mouse event reporting. Mouse events will be received as `TerminalInput::Mouse` variants.

```rust
pub fn tuinix::Terminal::disable_mouse_input(&mut self) -> std::io::Result<()>
```
Disables mouse event reporting. Called automatically on drop.

```rust
pub fn tuinix::Terminal::poll_event(
    &mut self,
    additional_readfds: &[std::os::fd::RawFd],
    additional_writefds: &[std::os::fd::RawFd],
    timeout: Option<std::time::Duration>,
) -> std::io::Result<Option<tuinix::TerminalEvent>>
```
Waits for terminal events using `libc::select()`. Returns `Ok(Some(event))` on event, `Ok(None)` on timeout, or `Err` on I/O error.

```rust
pub fn tuinix::Terminal::read_input(&mut self) -> std::io::Result<Option<tuinix::TerminalInput>>
```
Reads and parses the next input event. Blocks by default unless `tuinix::set_nonblocking()` has been called on `input_fd()`.

```rust
pub fn tuinix::Terminal::wait_for_resize(&mut self) -> std::io::Result<tuinix::TerminalSize>
```
Blocks until a terminal resize event occurs and returns the new size. Use with `signal_fd()` for non-blocking operation.

```rust
pub fn tuinix::Terminal::set_cursor(&mut self, position: Option<tuinix::TerminalPosition>)
```
Sets the cursor position to display after the next frame draw. `None` hides the cursor.

```rust
pub fn tuinix::Terminal::draw<W>(&mut self, frame: tuinix::TerminalFrame<W>) -> std::io::Result<()>
```
Renders a frame to the terminal, only updating changed portions for efficiency.

---

#### `tuinix::TerminalFrame`

A frame buffer representing the terminal display state.

```rust
pub struct tuinix::TerminalFrame<W = tuinix::FixedCharWidthEstimator> { /* fields omitted */ }
```

Implements `std::fmt::Write`, allowing use of `write!()` and `writeln!()` macros.

**Methods:**

```rust
pub fn tuinix::TerminalFrame::new(size: tuinix::TerminalSize) -> tuinix::TerminalFrame<W>
where W: Default
```
Creates a new frame with the specified dimensions and default character width estimator.

```rust
pub fn tuinix::TerminalFrame::with_char_width_estimator(
    size: tuinix::TerminalSize,
    char_width_estimator: W,
) -> tuinix::TerminalFrame<W>
```
Creates a new frame with a custom character width estimator.

```rust
pub fn tuinix::TerminalFrame::size(&self) -> tuinix::TerminalSize
```
Returns the frame dimensions.

```rust
pub fn tuinix::TerminalFrame::cursor(&self) -> tuinix::TerminalPosition
```
Returns the current cursor position within the frame (where the next character would be written).

```rust
pub fn tuinix::TerminalFrame::draw<X>(
    &mut self,
    position: tuinix::TerminalPosition,
    frame: &tuinix::TerminalFrame<X>,
)
```
Draws another frame onto this frame at the specified position. Properly handles character collision and wide characters.

---

### Geometry Types

#### `tuinix::TerminalSize`

Dimensions of a terminal or frame.

```rust
pub struct tuinix::TerminalSize {
    pub rows: usize,
    pub cols: usize,
}
```

**Constants:**

```rust
pub const tuinix::TerminalSize::EMPTY: tuinix::TerminalSize
```
A size with zero rows and columns.

**Methods:**

```rust
pub const fn tuinix::TerminalSize::rows_cols(rows: usize, cols: usize) -> tuinix::TerminalSize
```

```rust
pub const fn tuinix::TerminalSize::is_empty(self) -> bool
```

```rust
pub const fn tuinix::TerminalSize::contains(self, position: tuinix::TerminalPosition) -> bool
```

```rust
pub const fn tuinix::TerminalSize::to_region(self) -> tuinix::TerminalRegion
```

---

#### `tuinix::TerminalPosition`

Position within a terminal (row and column coordinates).

```rust
pub struct tuinix::TerminalPosition {
    pub row: usize,
    pub col: usize,
}
```

**Constants:**

```rust
pub const tuinix::TerminalPosition::ZERO: tuinix::TerminalPosition
```
Origin position (0, 0).

**Methods:**

```rust
pub const fn tuinix::TerminalPosition::row_col(row: usize, col: usize) -> tuinix::TerminalPosition
```

```rust
pub const fn tuinix::TerminalPosition::row(row: usize) -> tuinix::TerminalPosition
```
Creates a position at the beginning of the specified row (column = 0).

```rust
pub const fn tuinix::TerminalPosition::col(col: usize) -> tuinix::TerminalPosition
```
Creates a position in the first row at the specified column (row = 0).

**Traits:** Implements `Add`, `AddAssign`, `Sub`, `SubAssign` for position arithmetic.

---

#### `tuinix::TerminalRegion`

A rectangular region within a terminal.

```rust
pub struct tuinix::TerminalRegion {
    pub position: tuinix::TerminalPosition,
    pub size: tuinix::TerminalSize,
}
```

**Methods:**

```rust
pub const fn tuinix::TerminalRegion::is_empty(self) -> bool
pub const fn tuinix::TerminalRegion::contains(self, position: tuinix::TerminalPosition) -> bool
pub const fn tuinix::TerminalRegion::top_left(self) -> tuinix::TerminalPosition
pub const fn tuinix::TerminalRegion::top_right(self) -> tuinix::TerminalPosition
pub const fn tuinix::TerminalRegion::bottom_left(self) -> tuinix::TerminalPosition
pub const fn tuinix::TerminalRegion::bottom_right(self) -> tuinix::TerminalPosition
```

**Region manipulation methods:**

```rust
pub const fn tuinix::TerminalRegion::take_top(self, rows: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::take_bottom(self, rows: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::take_left(self, cols: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::take_right(self, cols: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::drop_top(self, rows: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::drop_bottom(self, rows: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::drop_left(self, cols: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::drop_right(self, cols: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::drop(self, amount: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::expand_top(self, rows: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::expand_bottom(self, rows: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::expand_left(self, cols: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::expand_right(self, cols: usize) -> tuinix::TerminalRegion
pub const fn tuinix::TerminalRegion::expand(self, amount: usize) -> tuinix::TerminalRegion
```

---

### Style Types

#### `tuinix::TerminalStyle`

Styling options for terminal text output.

```rust
pub struct tuinix::TerminalStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub dim: bool,
    pub strikethrough: bool,
    pub fg_color: Option<tuinix::TerminalColor>,
    pub bg_color: Option<tuinix::TerminalColor>,
}
```

**Important:** Each style completely overrides the previous style. To apply multiple attributes, chain them on a single `TerminalStyle` instance.

**Constants:**

```rust
pub const tuinix::TerminalStyle::RESET: tuinix::TerminalStyle
```
Resets all styling to default.

**Methods:**

```rust
pub const fn tuinix::TerminalStyle::new() -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::bold(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::italic(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::underline(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::blink(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::reverse(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::dim(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::strikethrough(self) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::fg_color(self, color: tuinix::TerminalColor) -> tuinix::TerminalStyle
pub const fn tuinix::TerminalStyle::bg_color(self, color: tuinix::TerminalColor) -> tuinix::TerminalStyle
```

**Traits:** Implements `std::fmt::Display` (produces ANSI escape sequences) and `std::str::FromStr`.

---

#### `tuinix::TerminalColor`

RGB color value for terminal output.

```rust
pub struct tuinix::TerminalColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
```

**Constants:**

```rust
pub const tuinix::TerminalColor::BLACK: tuinix::TerminalColor          // RGB(0, 0, 0)
pub const tuinix::TerminalColor::RED: tuinix::TerminalColor            // RGB(255, 0, 0)
pub const tuinix::TerminalColor::GREEN: tuinix::TerminalColor          // RGB(0, 255, 0)
pub const tuinix::TerminalColor::YELLOW: tuinix::TerminalColor         // RGB(255, 255, 0)
pub const tuinix::TerminalColor::BLUE: tuinix::TerminalColor           // RGB(0, 0, 255)
pub const tuinix::TerminalColor::MAGENTA: tuinix::TerminalColor        // RGB(255, 0, 255)
pub const tuinix::TerminalColor::CYAN: tuinix::TerminalColor           // RGB(0, 255, 255)
pub const tuinix::TerminalColor::WHITE: tuinix::TerminalColor          // RGB(255, 255, 255)
pub const tuinix::TerminalColor::BRIGHT_BLACK: tuinix::TerminalColor   // RGB(128, 128, 128)
pub const tuinix::TerminalColor::BRIGHT_RED: tuinix::TerminalColor     // RGB(255, 100, 100)
pub const tuinix::TerminalColor::BRIGHT_GREEN: tuinix::TerminalColor   // RGB(100, 255, 100)
pub const tuinix::TerminalColor::BRIGHT_YELLOW: tuinix::TerminalColor  // RGB(255, 255, 100)
pub const tuinix::TerminalColor::BRIGHT_BLUE: tuinix::TerminalColor    // RGB(100, 100, 255)
pub const tuinix::TerminalColor::BRIGHT_MAGENTA: tuinix::TerminalColor // RGB(255, 100, 255)
pub const tuinix::TerminalColor::BRIGHT_CYAN: tuinix::TerminalColor    // RGB(100, 255, 255)
pub const tuinix::TerminalColor::BRIGHT_WHITE: tuinix::TerminalColor   // RGB(255, 255, 255)
```

**Methods:**

```rust
pub const fn tuinix::TerminalColor::new(r: u8, g: u8, b: u8) -> tuinix::TerminalColor
```

---

### Input Types

#### `tuinix::TerminalInput`

User input event (keyboard or mouse).

```rust
pub enum tuinix::TerminalInput {
    Key(tuinix::KeyInput),
    Mouse(tuinix::MouseInput),
}
```

---

#### `tuinix::KeyInput`

Keyboard input event.

```rust
pub struct tuinix::KeyInput {
    pub ctrl: bool,
    pub alt: bool,
    pub code: tuinix::KeyCode,
}
```

---

#### `tuinix::KeyCode`

Key code representing which key was pressed.

```rust
pub enum tuinix::KeyCode {
    Enter,
    Escape,
    Backspace,
    Tab,
    BackTab,
    Delete,
    Insert,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Char(char),
}
```

---

#### `tuinix::MouseInput`

Mouse input event.

```rust
pub struct tuinix::MouseInput {
    pub event: tuinix::MouseEvent,
    pub position: tuinix::TerminalPosition,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}
```

---

#### `tuinix::MouseEvent`

Types of mouse events.

```rust
pub enum tuinix::MouseEvent {
    LeftPress,
    LeftRelease,
    RightPress,
    RightRelease,
    MiddlePress,
    MiddleRelease,
    Drag,
    ScrollUp,
    ScrollDown,
}
```

---

#### `tuinix::TerminalEvent`

Event returned by `Terminal::poll_event()`.

```rust
pub enum tuinix::TerminalEvent {
    Resize(tuinix::TerminalSize),
    Input(tuinix::TerminalInput),
    FdReady { fd: std::os::fd::RawFd, readable: bool },
}
```

---

### Character Width Estimation

#### `tuinix::EstimateCharWidth`

Trait for estimating the display width of characters.

```rust
pub trait tuinix::EstimateCharWidth {
    fn estimate_char_width(&self, c: char) -> usize;
}
```

**Limitations:** Cannot accurately handle tab characters (width depends on cursor position) or zero-width combining characters.

---

#### `tuinix::FixedCharWidthEstimator`

Simple character width estimator that assumes all visible characters have width 1.

```rust
pub struct tuinix::FixedCharWidthEstimator;
```

Assigns width 0 to control characters and width 1 to all others. Does not handle wide characters like CJK correctly.

---

### Utility Functions

```rust
pub fn tuinix::set_nonblocking(fd: std::os::fd::RawFd) -> std::io::Result<()>
```
Sets a file descriptor to non-blocking mode by adding the `O_NONBLOCK` flag.

```rust
pub fn tuinix::try_nonblocking<T>(result: std::io::Result<T>) -> std::io::Result<Option<T>>
```
Converts `ErrorKind::WouldBlock` errors to `Ok(None)` for easier handling of non-blocking I/O.

```rust
pub fn tuinix::try_uninterrupted<T>(result: std::io::Result<T>) -> std::io::Result<Option<T>>
```
Converts `ErrorKind::Interrupted` errors to `Ok(None)` for easier handling of interrupted system calls.

---

## Examples and Common Patterns

### Basic Terminal Application

```rust
fn main() -> std::io::Result<()> {
    let mut terminal = tuinix::Terminal::new()?;
    
    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
    write!(frame, "Hello, World!")?;
    terminal.draw(frame)?;
    
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}
```

### Styled Text Output

```rust
fn draw_styled_content(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
    
    let title_style = tuinix::TerminalStyle::new()
        .bold()
        .fg_color(tuinix::TerminalColor::CYAN);
    
    let error_style = tuinix::TerminalStyle::new()
        .bold()
        .fg_color(tuinix::TerminalColor::RED);
    
    writeln!(frame, "{}Title Text{}", title_style, tuinix::TerminalStyle::RESET)?;
    write!(frame, "{}Error message{}", error_style, tuinix::TerminalStyle::RESET)?;
    
    terminal.draw(frame)
}
```

### Event Loop with Timeout

```rust
fn event_loop() -> std::io::Result<()> {
    let mut terminal = tuinix::Terminal::new()?;
    
    loop {
        let timeout = std::time::Duration::from_millis(100);
        match terminal.poll_event(&[], &[], Some(timeout))? {
            Some(tuinix::TerminalEvent::Input(tuinix::TerminalInput::Key(key))) => {
                if let tuinix::KeyCode::Char('q') = key.code {
                    break;
                }
            }
            Some(tuinix::TerminalEvent::Resize(size)) => {
                let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(size);
                write!(frame, "Resized to {}x{}", size.cols, size.rows)?;
                terminal.draw(frame)?;
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### Mouse Input Handling

```rust
fn handle_mouse_input() -> std::io::Result<()> {
    let mut terminal = tuinix::Terminal::new()?;
    terminal.enable_mouse_input()?;
    
    loop {
        if let Some(tuinix::TerminalEvent::Input(tuinix::TerminalInput::Mouse(mouse))) = 
            terminal.poll_event(&[], &[], Some(std::time::Duration::from_millis(100)))? 
        {
            match mouse.event {
                tuinix::MouseEvent::LeftPress => {
                    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
                    write!(
                        frame,
                        "Clicked at row {}, col {}", 
                        mouse.position.row, 
                        mouse.position.col
                    )?;
                    terminal.draw(frame)?;
                }
                tuinix::MouseEvent::ScrollUp => { /* handle scroll */ }
                _ => {}
            }
        }
    }
}
```

### Composing Frames

```rust
fn compose_frames(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    let size = terminal.size();
    let mut main_frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(size);
    
    let mut header_frame: tuinix::TerminalFrame = 
        tuinix::TerminalFrame::new(tuinix::TerminalSize::rows_cols(3, size.cols));
    write!(header_frame, "Header Content")?;
    
    let mut content_frame: tuinix::TerminalFrame = 
        tuinix::TerminalFrame::new(tuinix::TerminalSize::rows_cols(size.rows - 3, size.cols));
    write!(content_frame, "Main Content")?;
    
    main_frame.draw(tuinix::TerminalPosition::ZERO, &header_frame);
    main_frame.draw(tuinix::TerminalPosition::row(3), &content_frame);
    
    terminal.draw(main_frame)
}
```

### Non-blocking I/O with Custom Event Loop

```rust
fn nonblocking_example() -> std::io::Result<()> {
    let mut terminal = tuinix::Terminal::new()?;
    
    let stdin_fd = terminal.input_fd();
    let signal_fd = terminal.signal_fd();
    
    tuinix::set_nonblocking(stdin_fd)?;
    tuinix::set_nonblocking(signal_fd)?;
    
    loop {
        match tuinix::try_nonblocking(terminal.read_input())? {
            Some(tuinix::TerminalInput::Key(key)) => {
                if let tuinix::KeyCode::Char('q') = key.code {
                    break;
                }
            }
            None => {}
            _ => {}
        }
        
        if let Some(new_size) = tuinix::try_nonblocking(terminal.wait_for_resize())? {
            let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(new_size);
            write!(frame, "Terminal resized!")?;
            terminal.draw(frame)?;
        }
        
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    Ok(())
}
```

### Using TerminalRegion for Layout

```rust
fn region_layout_example(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    let region = terminal.size().to_region();
    
    let header_region = region.take_top(3);
    let footer_region = region.take_bottom(2);
    let content_region = region.drop_top(3).drop_bottom(2);
    
    let left_panel = content_region.take_left(content_region.size.cols / 2);
    let right_panel = content_region.drop_left(content_region.size.cols / 2);
    
    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
    
    let mut header: tuinix::TerminalFrame = tuinix::TerminalFrame::new(header_region.size);
    write!(header, "Header")?;
    frame.draw(header_region.position, &header);
    
    terminal.draw(frame)
}
```

### Error Handling Pattern

```rust
fn robust_terminal_app() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = tuinix::Terminal::new()?;
    
    let result = (|| -> std::io::Result<()> {
        loop {
            let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
            write!(frame, "Press 'q' to quit")?;
            terminal.draw(frame)?;
            
            if let Some(tuinix::TerminalEvent::Input(tuinix::TerminalInput::Key(key))) = 
                terminal.poll_event(&[], &[], Some(std::time::Duration::from_millis(100)))? 
            {
                if let tuinix::KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
        Ok(())
    })();
    
    // Terminal automatically restored on drop
    result?;
    Ok(())
}
```

### Keyboard Modifier Handling

```rust
fn handle_keyboard_modifiers(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    loop {
        if let Some(tuinix::TerminalEvent::Input(tuinix::TerminalInput::Key(key))) = 
            terminal.poll_event(&[], &[], Some(std::time::Duration::from_millis(100)))? 
        {
            let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
            
            match key.code {
                tuinix::KeyCode::Char('c') if key.ctrl => {
                    write!(frame, "Ctrl+C pressed - exiting")?;
                    terminal.draw(frame)?;
                    break;
                }
                tuinix::KeyCode::Char(c) if key.alt => {
                    write!(frame, "Alt+{} pressed", c)?;
                    terminal.draw(frame)?;
                }
                tuinix::KeyCode::Char(c) => {
                    write!(frame, "Pressed: {}", c)?;
                    terminal.draw(frame)?;
                }
                tuinix::KeyCode::Up if key.ctrl => {
                    write!(frame, "Ctrl+Up arrow")?;
                    terminal.draw(frame)?;
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}
```

### Cursor Positioning

```rust
fn show_cursor_at_position(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    let size = terminal.size();
    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(size);
    
    // Write some text
    write!(frame, "Enter your name: ")?;
    
    // Position the cursor right after the prompt
    terminal.set_cursor(Some(frame.cursor()));
    
    // Draw and show the cursor
    terminal.draw(frame)?;
    
    // Later, hide cursor
    terminal.set_cursor(None);
    
    Ok(())
}
```

### Multi-line Text with Wrapping

```rust
fn display_multi_line_text(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
    
    let title = "Welcome";
    let content = vec![
        "This is a multi-line",
        "terminal user interface",
        "demonstration.",
    ];
    
    let title_style = tuinix::TerminalStyle::new()
        .bold()
        .fg_color(tuinix::TerminalColor::GREEN);
    
    writeln!(frame, "{}{}{}", title_style, title, tuinix::TerminalStyle::RESET)?;
    writeln!(frame)?; // blank line
    
    for line in content {
        writeln!(frame, "{}", line)?;
    }
    
    terminal.draw(frame)?;
    
    Ok(())
}
```

### Color Palette Display

```rust
fn show_color_palette(terminal: &mut tuinix::Terminal) -> std::io::Result<()> {
    let mut frame: tuinix::TerminalFrame = tuinix::TerminalFrame::new(terminal.size());
    
    let colors = vec![
        ("BLACK", tuinix::TerminalColor::BLACK),
        ("RED", tuinix::TerminalColor::RED),
        ("GREEN", tuinix::TerminalColor::GREEN),
        ("YELLOW", tuinix::TerminalColor::YELLOW),
        ("BLUE", tuinix::TerminalColor::BLUE),
        ("MAGENTA", tuinix::TerminalColor::MAGENTA),
        ("CYAN", tuinix::TerminalColor::CYAN),
        ("WHITE", tuinix::TerminalColor::WHITE),
    ];
    
    for (name, color) in colors {
        let style = tuinix::TerminalStyle::new().fg_color(color);
        write!(frame, "{}{:<15}{} ", style, name, tuinix::TerminalStyle::RESET)?;
    }
    
    terminal.draw(frame)?;
    Ok(())
}
```
