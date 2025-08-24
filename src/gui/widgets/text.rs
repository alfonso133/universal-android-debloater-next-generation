#![allow(
    clippy::disallowed_types,
    reason = "this is the replacement that enforces advanced shaping for disallowed [`iced::widget::Text`]"
)]

use iced::advanced::text::IntoFragment;
use iced::widget::text::Catalog;
use iced::widget::Text;

// Creates a new Text widget with advanced shaping.
pub fn text<'a, Theme, Renderer>(text: impl IntoFragment<'a>) -> Text<'a, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
{
    Text::new(text).shaping(iced::widget::text::Shaping::Advanced)
}
