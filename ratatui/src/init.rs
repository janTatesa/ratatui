use std::io::{self, Stdout, stdout};

use ratatui_core::terminal::{Terminal, TerminalOptions};
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::execute;
use ratatui_crossterm::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

/// A type alias for the default terminal type.
///
/// This is a [`Terminal`] using the [`CrosstermBackend`] which writes to [`Stdout`]. This is a
/// reasonable default for most applications. To use a different backend or output stream, instead
/// use [`Terminal`] and a [backend][`crate::backend`] of your choice directly.
pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Run a closure with a terminal initialized with reasonable defaults for most applications.
///
/// This function creates a new [`DefaultTerminal`] with [`init`] and then runs the given closure
/// with a mutable reference to the terminal. After the closure completes, the terminal is restored
/// to its original state with [`restore`].
///
/// This function is a convenience wrapper around [`init`] and [`restore`], and is useful for simple
/// applications that need a terminal with reasonable defaults for the entire lifetime of the
/// application.
///
/// # Examples
///
/// A simple example where the app logic is contained in the closure:
///
/// ```rust,no_run
/// use crossterm::event;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     ratatui::run(|terminal| {
///         loop {
///             terminal.draw(|frame| frame.render_widget("Hello, world!", frame.area()))?;
///             if event::read()?.is_key_press() {
///                 break Ok(());
///             }
///         }
///     })
/// }
/// ```
///
/// A more complex example where the app logic is contained in a separate function:
///
/// ```rust,no_run
/// use crossterm::event;
///
/// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
///
/// fn main() -> Result<()> {
///     ratatui::run(app)
/// }
///
/// fn app(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
///     const GREETING: &str = "Hello, world!";
///     loop {
///         terminal.draw(|frame| frame.render_widget(format!("{GREETING}"), frame.area()))?;
///         if matches!(event::read()?, event::Event::Key(_)) {
///             break Ok(());
///         }
///     }
/// }
/// ```
///
/// Once the app logic becomes more complex, it may be beneficial to move the app logic into a
/// separate struct. This allows the app logic to be split into multiple methods with each having
/// access to the state of the app. This can make the app logic easier to understand and maintain.
///
/// ```rust,no_run
/// use crossterm::event;
///
/// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
///
/// fn main() -> Result<()> {
///     let mut app = App::new();
///     ratatui::run(|terminal| app.run(terminal))
/// }
///
/// struct App {
///     should_quit: bool,
///     name: String,
/// }
///
/// impl App {
///     fn new() -> Self {
///         Self {
///             should_quit: false,
///             name: "world".to_string(),
///         }
///     }
///
///     fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
///         while !self.should_quit {
///             terminal.draw(|frame| frame.render_widget("Hello, world!", frame.area()))?;
///             self.handle_events()?;
///         }
///         Ok(())
///     }
///
///     fn render(&mut self, frame: &mut ratatui::Frame) -> Result<()> {
///         let greeting = format!("Hello, {}!", self.name);
///         frame.render_widget(greeting, frame.area());
///         Ok(())
///     }
///
///     fn handle_events(&mut self) -> Result<()> {
///         if event::read()?.is_key_press() {
///             self.should_quit = true;
///         }
///         Ok(())
///     }
/// }
/// ```
pub fn run<F, R>(f: F) -> R
where
    F: FnOnce(&mut DefaultTerminal) -> R,
{
    let mut terminal = init();
    let result = f(&mut terminal);
    restore();
    result
}

/// Initialize a terminal with reasonable defaults for most applications.
///
/// This will create a new [`DefaultTerminal`] and initialize it with the following defaults:
///
/// - Backend: [`CrosstermBackend`] writing to [`Stdout`]
/// - Raw mode is enabled
/// - Alternate screen buffer enabled
/// - A panic hook is installed that restores the terminal before panicking. Ensure that this method
///   is called after any other panic hooks that may be installed to ensure that the terminal is
///   restored before those hooks are called.
///
/// For more control over the terminal initialization, use [`Terminal::new`] or
/// [`Terminal::with_options`].
///
/// Ensure that this method is called *after* your app installs any other panic hooks to ensure the
/// terminal is restored before the other hooks are called.
///
/// Generally, use this function instead of [`try_init`] to ensure that the terminal is restored
/// correctly if any of the initialization steps fail. If you need to handle the error yourself, use
/// [`try_init`] instead.
///
/// # Panics
///
/// This function will panic if any of the following steps fail:
///
/// - Enabling raw mode
/// - Entering the alternate screen buffer
/// - Creating the terminal fails due to being unable to calculate the terminal size
///
/// # Examples
///
/// ```rust,no_run
/// let terminal = ratatui::init();
/// ```
pub fn init() -> DefaultTerminal {
    try_init().expect("failed to initialize terminal")
}

/// Try to initialize a terminal using reasonable defaults for most applications.
///
/// This function will attempt to create a [`DefaultTerminal`] and initialize it with the following
/// defaults:
///
/// - Raw mode is enabled
/// - Alternate screen buffer enabled
/// - A panic hook is installed that restores the terminal before panicking.
/// - A [`Terminal`] is created using [`CrosstermBackend`] writing to [`Stdout`]
///
/// If any of these steps fail, the error is returned.
///
/// Ensure that this method is called *after* your app installs any other panic hooks to ensure the
/// terminal is restored before the other hooks are called.
///
/// Generally, you should use [`init`] instead of this function, as the panic hook installed by this
/// function will ensure that any failures during initialization will restore the terminal before
/// panicking. This function is provided for cases where you need to handle the error yourself.
///
/// # Examples
///
/// ```no_run
/// let terminal = ratatui::try_init()?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn try_init() -> io::Result<DefaultTerminal> {
    set_panic_hook();
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

/// Initialize a terminal with the given options and reasonable defaults.
///
/// This function allows the caller to specify a custom [`Viewport`] via the [`TerminalOptions`]. It
/// will create a new [`DefaultTerminal`] and initialize it with the given options and the following
/// defaults:
///
/// [`Viewport`]: crate::Viewport
///
/// - Raw mode is enabled
/// - A panic hook is installed that restores the terminal before panicking.
///
/// Unlike [`init`], this function does not enter the alternate screen buffer as this may not be
/// desired in all cases. If you need the alternate screen buffer, you should enable it manually
/// after calling this function.
///
/// For more control over the terminal initialization, use [`Terminal::with_options`].
///
/// Ensure that this method is called *after* your app installs any other panic hooks to ensure the
/// terminal is restored before the other hooks are called.
///
/// Generally, use this function instead of [`try_init_with_options`] to ensure that the terminal is
/// restored correctly if any of the initialization steps fail. If you need to handle the error
/// yourself, use [`try_init_with_options`] instead.
///
/// # Panics
///
/// This function will panic if any of the following steps fail:
///
/// - Enabling raw mode
/// - Creating the terminal fails due to being unable to calculate the terminal size
///
/// # Examples
///
/// ```rust,no_run
/// use ratatui::{TerminalOptions, Viewport};
///
/// let options = TerminalOptions {
///     viewport: Viewport::Inline(5),
/// };
/// let terminal = ratatui::init_with_options(options);
/// ```
pub fn init_with_options(options: TerminalOptions) -> DefaultTerminal {
    try_init_with_options(options).expect("failed to initialize terminal")
}

/// Try to initialize a terminal with the given options and reasonable defaults.
///
/// This function allows the caller to specify a custom [`Viewport`] via the [`TerminalOptions`]. It
/// will attempt to create a [`DefaultTerminal`] and initialize it with the given options and the
/// following defaults:
///
/// [`Viewport`]: crate::Viewport
///
/// - Raw mode is enabled
/// - A panic hook is installed that restores the terminal before panicking.
///
/// Unlike [`try_init`], this function does not enter the alternate screen buffer as this may not be
/// desired in all cases. If you need the alternate screen buffer, you should enable it manually
/// after calling this function.
///
/// If any of these steps fail, the error is returned.
///
/// Ensure that this method is called *after* your app installs any other panic hooks to ensure the
/// terminal is restored before the other hooks are called.
///
/// Generally, you should use [`init_with_options`] instead of this function, as the panic hook
/// installed by this function will ensure that any failures during initialization will restore the
/// terminal before panicking. This function is provided for cases where you need to handle the
/// error yourself.
///
/// # Examples
///
/// ```no_run
/// use ratatui::{TerminalOptions, Viewport};
///
/// let options = TerminalOptions {
///     viewport: Viewport::Inline(5),
/// };
/// let terminal = ratatui::try_init_with_options(options)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn try_init_with_options(options: TerminalOptions) -> io::Result<DefaultTerminal> {
    set_panic_hook();
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::with_options(backend, options)
}

/// Restores the terminal to its original state.
///
/// This function should be called before the program exits to ensure that the terminal is
/// restored to its original state.
///
/// This function will attempt to restore the terminal to its original state by performing the
/// following steps:
///
/// 1. Raw mode is disabled.
/// 2. The alternate screen buffer is left.
///
/// If either of these steps fail, the error is printed to stderr and ignored.
///
/// Use this function over [`try_restore`] when you don't need to handle the error yourself, as
/// ignoring the error is generally the correct behavior when cleaning up before exiting. If you
/// need to handle the error yourself, use [`try_restore`] instead.
///
/// # Examples
///
/// ```rust,no_run
/// ratatui::restore();
/// ```
pub fn restore() {
    if let Err(err) = try_restore() {
        // There's not much we can do if restoring the terminal fails, so we just print the error
        std::eprintln!("Failed to restore terminal: {err}");
    }
}

/// Restore the terminal to its original state.
///
/// This function will attempt to restore the terminal to its original state by performing the
/// following steps:
///
/// 1. Raw mode is disabled.
/// 2. The alternate screen buffer is left.
///
/// If either of these steps fail, the error is returned.
///
/// Use [`restore`] instead of this function when you don't need to handle the error yourself, as
/// ignoring the error is generally the correct behavior when cleaning up before exiting. If you
/// need to handle the error yourself, use this function instead.
///
/// # Examples
///
/// ```no_run
/// ratatui::try_restore()?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn try_restore() -> io::Result<()> {
    // disabling raw mode first is important as it has more side effects than leaving the alternate
    // screen buffer
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Sets a panic hook that restores the terminal before panicking.
///
/// Replaces the panic hook with a one that will restore the terminal state before calling the
/// original panic hook. This ensures that the terminal is left in a good state when a panic occurs.
fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(alloc::boxed::Box::new(move |info| {
        restore();
        hook(info);
    }));
}
