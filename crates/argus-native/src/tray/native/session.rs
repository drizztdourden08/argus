//! The live tray icon: its state, its menu, and what the owner's clicks and the request files ask.

use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use super::event_loop::ExitReason;
use super::handlers::Handlers;
use super::taskbar;
use super::view;
use super::win32::{self, InstanceGuard};
use crate::tray::balloon::Balloon;
use crate::tray::error::TrayError;
use crate::tray::gate::OpenGate;
use crate::tray::icon::{IconKind, Ink, render};
use crate::tray::log::TrayLog;
use crate::tray::menu::{Command, entries};
use crate::tray::paths::TrayFiles;
use crate::tray::state::TrayState;
use crate::tray::watch::{self, Request};
use crate::tray::{quit_flag, record};

/// The id the icon library gives the next icon: it counts from 1, one per icon built in this
/// process, and Windows needs it to address the icon for a balloon tip.
static NEXT_ICON_ID: AtomicU32 = AtomicU32::new(1);

/// The two icons in the ink of the taskbar they were made for.
struct Icons {
    ink: Ink,
    running: Icon,
    stopped: Icon,
}

impl Icons {
    fn load(ink: Ink) -> Result<Self, TrayError> {
        let size = win32::small_icon_px();
        let make = |kind| {
            let image = render(kind, ink, size)?;
            Icon::from_rgba(image.pixels, image.width, image.height)
                .map_err(|e| TrayError::Platform(e.to_string()))
        };
        Ok(Self {
            ink,
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
        let icons = Icons::load(taskbar::ink())?;
        let state = TrayState::default();
        let menu = view::build(&entries(&state))?;
        let icon_id = NEXT_ICON_ID.fetch_add(1, Ordering::Relaxed);
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

    /// Draws the icon again in the other ink when the owner switched the taskbar theme.
    fn follow_taskbar(&mut self) {
        let ink = taskbar::ink();
        if ink == self.icons.ink {
            return;
        }
        match Icons::load(ink) {
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
            Command::Restart | Command::Start if self.quitting => {}
            other => self.handlers.fire_command(other),
        }
    }

    fn open(&mut self) {
        if self.gate.allow(Instant::now()) {
            self.handlers.fire_command(Command::Open);
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
        let menu = view::build(&entries(&self.state))?;
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
