//! The live tray icon: its state, its menu, and what the owner's clicks and the request files ask.
// test-tag: critical

use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use super::event_loop::ExitReason;
use super::handlers::Handlers;
use super::opener;
use super::taskbar;
use super::view;
use super::win32::{self, InstanceGuard};
use crate::tray::accounts::AccountLines;
use crate::tray::balloon::Balloon;
use crate::tray::error::TrayError;
use crate::tray::gate::OpenGate;
use crate::tray::icon::{IconKind, Ink, render};
use crate::tray::log::TrayLog;
use crate::tray::menu::{Command, entries_with};
use crate::tray::paths::TrayFiles;
use crate::tray::state::TrayState;
use crate::tray::watch::{self, Request};
use crate::tray::{quit_flag, record};

/// The next value of the icon library's counter, which Windows needs to address the icon for a
/// balloon tip. The counter starts at 1 and every icon takes two values: the builder takes one
/// for the public id, then the icon takes the next as the id Windows knows it by.
static NEXT_ICON_ID: AtomicU32 = AtomicU32::new(1);
/// How many values of the counter one icon takes.
const IDS_PER_ICON: u32 = 2;

/// The two icons in the ink of the taskbar and the pixel size of the icon slot they were made for.
struct Icons {
    ink: Ink,
    size: u32,
    running: Icon,
    stopped: Icon,
}

impl Icons {
    fn load(ink: Ink, size: u32) -> Result<Self, TrayError> {
        let make = |kind| {
            let image = render(kind, ink, size)?;
            Icon::from_rgba(image.pixels, image.width, image.height)
                .map_err(|e| TrayError::Platform(e.to_string()))
        };
        Ok(Self {
            ink,
            size,
            running: make(IconKind::Running)?,
            stopped: make(IconKind::Stopped)?,
        })
    }

    fn of(&self, kind: IconKind) -> Icon {
        match kind {
            IconKind::Running => self.running.clone(),
            IconKind::Stopped => self.stopped.clone(),
        }
    }
}

pub struct Session {
    tray: TrayIcon,
    icons: Icons,
    icon_id: u32,
    state: TrayState,
    accounts: AccountLines,
    quitting: bool,
    files: TrayFiles,
    log: TrayLog,
    handlers: Arc<Handlers>,
    gate: OpenGate,
    _instance: InstanceGuard,
}

impl Session {
    /// Takes the single-instance mutex, draws the icon and writes `tray.json`. Must run on the
    /// thread that pumps messages.
    pub fn start(files: TrayFiles, handlers: Arc<Handlers>) -> Result<Self, TrayError> {
        win32::allow_dark_menus();
        let instance = InstanceGuard::acquire(&files.instance_name())?;
        let icons = Icons::load(taskbar::ink(), win32::small_icon_px(win32::dpi(None)))?;
        let state = TrayState::default();
        let menu = view::build(&entries_with(&state, &AccountLines::default()))?;
        let icon_id = NEXT_ICON_ID.fetch_add(IDS_PER_ICON, Ordering::Relaxed) + 1;
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icons.of(state.icon()))
            .with_tooltip(state.tooltip())
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| TrayError::Platform(e.to_string()))?;
        watch::clear_requests(&files);
        record::write(&files, process::id())?;
        record::set_tooltip(&files, process::id(), &state.tooltip())?;
        Ok(Self {
            tray,
            icons,
            icon_id,
            state,
            accounts: AccountLines::default(),
            quitting: false,
            log: TrayLog::new(files.clone()),
            files,
            handlers,
            gate: OpenGate::default(),
            _instance: instance,
        })
    }

    /// Shows a new state. Ignored while Quit is carried out.
    pub fn apply_state(&mut self, state: TrayState) {
        if !self.quitting && state != self.state {
            self.show(state);
        }
    }

    /// Shows the accounts section. Ignored while Quit is carried out.
    pub fn apply_accounts(&mut self, accounts: AccountLines) {
        if self.quitting || accounts == self.accounts {
            return;
        }
        self.accounts = accounts;
        if let Err(error) = self.paint_menu() {
            self.log.line(&error.to_string());
        }
    }

    /// Shows a balloon tip.
    pub fn balloon(&self, balloon: &Balloon) {
        let window = self.tray.window_handle();
        if !win32::show_balloon(window, self.icon_id, &balloon.title, &balloon.text) {
            self.log.line("balloon failed");
        }
    }

    /// Takes what the other processes and the owner asked since the last look. `Some` when the
    /// tray must close.
    pub fn tick(&mut self) -> Option<ExitReason> {
        self.follow_taskbar();
        for request in watch::poll(&self.files) {
            match request {
                Request::Stop => return Some(ExitReason::StopRequested),
                Request::Quit => self.quit(),
                Request::Balloon(balloon) => self.balloon(&balloon),
            }
        }
        None
    }

    /// Handles the menu clicks and icon clicks that wait.
    pub fn drain_events(&mut self) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(command) = Command::from_id(&event.id.0) {
                self.on_command(command);
            }
        }
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if opens_page(&event) {
                self.open();
            }
        }
    }

    /// Draws the icon again when the owner switched the taskbar theme (the other ink) or the
    /// scale of the screen changed, or the tray moved to a screen of another scale (the size of
    /// the icon slot): the exact size is drawn, never a stretched one.
    fn follow_taskbar(&mut self) {
        let ink = taskbar::ink();
        let size = win32::small_icon_px(win32::dpi(Some(self.tray.window_handle())));
        if ink == self.icons.ink && size == self.icons.size {
            return;
        }
        match Icons::load(ink, size) {
            Ok(icons) => self.icons = icons,
            Err(error) => return self.log.line(&error.to_string()),
        }
        let shown = self.icons.of(self.state.icon());
        if let Err(error) = self.tray.set_icon(Some(shown)) {
            self.log.line(&error.to_string());
        }
    }

    fn on_command(&mut self, command: Command) {
        match command {
            Command::Quit => self.quit(),
            Command::Open => self.open(),
            Command::OpenNotifications | Command::SwitchDesktop(_) => self.show_page(&command),
            Command::Restart | Command::Start | Command::CancelSwitch if self.quitting => {}
            other => self.fire(&other),
        }
    }

    fn open(&mut self) {
        if self.gate.allow(Instant::now()) {
            self.show_page(&Command::Open);
        } else {
            self.log
                .line("open: ignored, the page was asked for less than 2 s ago");
        }
    }

    /// Hands the foreground right of this click on, while the tray still holds it, then asks
    /// for the page.
    fn show_page(&self, command: &Command) {
        if let Err(why) = opener::pass_foreground() {
            self.log.line(&format!(
                "{}: the browser may open behind: {why}",
                command.id()
            ));
        }
        self.fire(command);
    }

    fn fire(&self, command: &Command) {
        if !self.handlers.fire_command(command.clone()) {
            self.log
                .line(&format!("{}: nothing handles the menu yet", command.id()));
        }
    }

    fn quit(&mut self) {
        if self.quitting {
            return;
        }
        self.quitting = true;
        if let Err(error) = quit_flag::write(&self.files) {
            self.log
                .line(&format!("could not write quit.flag: {error}"));
        }
        self.show(self.state.closing());
        self.handlers.fire_quit();
    }

    fn show(&mut self, state: TrayState) {
        let previous = std::mem::replace(&mut self.state, state);
        self.log.line(&state.status_label());
        if let Err(error) = self.paint(previous.icon() != state.icon()) {
            self.log.line(&error.to_string());
        }
    }

    fn paint(&self, icon_changed: bool) -> Result<(), TrayError> {
        let platform = |e: tray_icon::Error| TrayError::Platform(e.to_string());
        self.tray
            .set_tooltip(Some(self.state.tooltip()))
            .map_err(platform)?;
        let _noted = record::set_tooltip(&self.files, process::id(), &self.state.tooltip());
        if icon_changed {
            let icon = self.icons.of(self.state.icon());
            self.tray.set_icon(Some(icon)).map_err(platform)?;
        }
        self.paint_menu()
    }

    fn paint_menu(&self) -> Result<(), TrayError> {
        let menu = view::build(&entries_with(&self.state, &self.accounts))?;
        self.tray.set_menu(Some(Box::new(menu)));
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        record::remove_if_owned(&self.files, process::id());
    }
}

fn opens_page(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use tray_icon::dpi::PhysicalPosition;
    use tray_icon::{MouseButton, MouseButtonState, Rect, TrayIconEvent, TrayIconId};

    use super::opens_page;

    fn click(button: MouseButton, state: MouseButtonState) -> TrayIconEvent {
        TrayIconEvent::Click {
            id: TrayIconId::new("t"),
            position: PhysicalPosition::new(0.0, 0.0),
            rect: Rect::default(),
            button,
            button_state: state,
        }
    }

    #[test]
    fn a_left_click_released_and_a_left_double_click_open_the_page() {
        assert!(opens_page(&click(MouseButton::Left, MouseButtonState::Up)));
        assert!(!opens_page(&click(
            MouseButton::Left,
            MouseButtonState::Down
        )));
        assert!(!opens_page(&click(
            MouseButton::Right,
            MouseButtonState::Up
        )));
        let double = TrayIconEvent::DoubleClick {
            id: TrayIconId::new("t"),
            position: PhysicalPosition::new(0.0, 0.0),
            rect: Rect::default(),
            button: MouseButton::Left,
        };
        assert!(opens_page(&double));
    }
}
