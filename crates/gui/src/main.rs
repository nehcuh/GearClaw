mod app;
mod chat_view;
mod input_bar;
mod log_panel;
mod log_store;
mod monitor_view;
mod multiline_input;
mod settings_view;
mod sidebar;
mod status_bar;
mod text_input;
mod theme;

use gpui::*;
use tracing_subscriber::prelude::*;

use app::{DesktopApp, Quit, SendMessage};
use multiline_input::InsertNewline;
use text_input::{
    Backspace, CopyText, CutText, Delete, End, Home, Left, PasteText, Right, SelectAll, SelectLeft,
    SelectRight, ShowCharacterPalette,
};
use theme::{Theme, ThemeMode};
use log_store::{LogStore, GuiLogLayer};

fn main() {
    // Initialize logging system
    let log_store = LogStore::new();
    let gui_layer = GuiLogLayer::new(log_store.entries.clone());
    tracing_subscriber::registry()
        .with(gui_layer)
        .with(tracing_subscriber::fmt::layer().with_target(true).with_ansi(false))
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")))
        .init();

    Application::new().run(move |cx: &mut App| {
        // Initialize theme from system appearance
        let appearance = cx.window_appearance();
        cx.set_global(Theme::for_appearance(appearance, ThemeMode::System));

        // Register log store as global
        cx.set_global(log_store.clone());

        // Register key bindings
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-v", PasteText, None),
            KeyBinding::new("cmd-c", CopyText, None),
            KeyBinding::new("cmd-x", CutText, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
            KeyBinding::new("enter", SendMessage, None),
            KeyBinding::new("enter", InsertNewline, Some("MultiLineInput")),
            KeyBinding::new("cmd-q", Quit, None),
        ]);

        // Quit action
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("GearClaw".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| DesktopApp::new(cx)),
            )
            .unwrap();

        // Focus the text input and observe appearance changes
        window
            .update(cx, |view, window, cx| {
                let input_focus = view.input.focus_handle(cx);
                window.focus(&input_focus, cx);
                cx.activate(true);

                // Listen for system appearance changes
                let _subscription = cx.observe_window_appearance(window, |_this, window, cx| {
                    let current_mode = theme::mode(cx);
                    if current_mode == ThemeMode::System {
                        let appearance = window.appearance();
                        cx.set_global(Theme::for_appearance(appearance, ThemeMode::System));
                    }
                });
                // Keep the subscription alive by storing it
                // (it will live as long as the view)
                std::mem::forget(_subscription);
            })
            .unwrap();
    });
}
