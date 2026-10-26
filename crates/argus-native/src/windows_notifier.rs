//! The Windows notifier: registers the app identity and shows toasts through `WinRT`.

use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager, ToastNotifier};
use windows::core::HSTRING;
use windows_registry::CURRENT_USER;

use crate::error::NotifyError;
use crate::identity::Identity;
use crate::notifier::Notifier;
use crate::payload::{GROUP, Payload};
use crate::toast::Toast;

/// Shows toasts under the `AppUserModelID` of an `Identity`.
///
/// A click on a toast needs no running Argus: the toast carries a protocol launch, so Windows opens
/// the page in the default browser itself.
pub struct WindowsNotifier {
    notifier: ToastNotifier,
}

impl WindowsNotifier {
    /// Registers the identity for the current user, then opens the `WinRT` notifier for it. The
    /// registration is idempotent and needs no administrator rights.
    ///
    /// # Errors
    ///
    /// `Platform` when the registry key cannot be written or `WinRT` refuses the identity.
    pub fn new(identity: &Identity) -> Result<Self, NotifyError> {
        register(identity)?;
        let notifier =
            ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(&identity.aumid))
                .map_err(platform)?;
        Ok(Self { notifier })
    }
}

impl Notifier for WindowsNotifier {
    fn show(&self, toast: &Toast) -> Result<(), NotifyError> {
        let payload = Payload::build(toast)?;
        let notification = notification(&payload).map_err(platform)?;
        self.notifier.Show(&notification).map_err(platform)
    }
}

fn register(identity: &Identity) -> Result<(), NotifyError> {
    let key = CURRENT_USER
        .create(identity.registry_path())
        .map_err(platform)?;
    for (name, value) in identity.registry_values() {
        key.set_string(name, value).map_err(platform)?;
    }
    Ok(())
}

fn notification(payload: &Payload) -> windows::core::Result<ToastNotification> {
    let document = XmlDocument::new()?;
    document.LoadXml(&HSTRING::from(&payload.xml))?;
    let notification = ToastNotification::CreateToastNotification(&document)?;
    notification.SetGroup(&HSTRING::from(GROUP))?;
    if let Some(tag) = &payload.tag {
        notification.SetTag(&HSTRING::from(tag))?;
    }
    Ok(notification)
}

fn platform(error: impl std::fmt::Display) -> NotifyError {
    NotifyError::Platform(error.to_string())
}
