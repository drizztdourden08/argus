//! The fields every kind has in common about how it is shown. An item type, a sub-item and a
//! record each declare their own fields in `shown_struct!` and get these appended, so the list is
//! written once. A struct cannot flatten them: serde does not refuse unknown keys through a
//! flattened field.

/// Declares a registry struct and appends the common fields of a kind: its view, avatar, tabs,
/// card, links, signal strip, grouping and primary action.
macro_rules! shown_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident { $($body:tt)* }
    ) => {
        $(#[$meta])*
        pub struct $name {
            $($body)*
            /// Whether a list draws it as a card or a row.
            pub view: View,
            /// Where its avatar comes from: a property, `=owner` or `=lead`.
            pub assignee: String,
            /// Its pane tabs, in order.
            pub tabs: Vec<String>,
            /// What its card shows.
            pub card: Card,
            /// The links it carries.
            pub links: Vec<Link>,
            /// The providers of the signal strip of its card, in order.
            #[serde(default)]
            pub signals: Vec<String>,
            /// The link field a list groups it by.
            #[serde(default)]
            pub group_by: Option<String>,
            /// The primary action of its card.
            #[serde(default)]
            pub action: Option<String>,
        }
    };
}
