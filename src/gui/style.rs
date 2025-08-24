use iced::widget::scrollable::Rail;
use iced::widget::{
    button, checkbox, container, scrollable, text,
};
use iced::{Background, Border, Color, Shadow, Theme};

#[derive(Default, Debug, Clone, Copy)]
pub enum Container {
    #[default]
    Invisible,
    Frame,
    BorderedFrame,
    Tooltip,
    Background,
}

impl Container {
    pub fn get_default_style() -> impl Fn(&Theme) -> container::Style {
        Container::Invisible.get_style()
    }
    
    pub fn get_style(&self) -> impl Fn(&Theme) -> container::Style + use<'_> {
        let container_type = self.clone();
        move |theme: &Theme| {
            match container_type {
                Container::Invisible => container::transparent(theme),
                Container::Frame => container::Style::default()
                    .background(theme.extended_palette().background.strong.color)
                    .color(theme.extended_palette().background.strong.text)
                    .border(Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 5.0.into(),
                    }),
                Container::BorderedFrame => container::Style::default()
                    .background(theme.extended_palette().background.strong.color)
                    .color(theme.extended_palette().background.strong.text)
                    .border(Border {
                        color: theme.palette().danger,
                        width: 1.0,
                        radius: 5.0.into(),
                    })
                    .shadow(Shadow::default()),
                Container::Tooltip => container::Style::default()
                    .background(theme.extended_palette().background.strong.color)
                    .color(theme.extended_palette().background.strong.text)
                    .border(Border {
                        color: theme.palette().primary,
                        width: 1.0,
                        radius: 8.0.into(),
                    })
                    .shadow(Shadow::default()),
                Container::Background => container::Style::default()
                    .background(theme.palette().background)
                    .color(theme.palette().text)
                    .border(Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 5.0.into(),
                    })
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    #[default]
    Primary,
    Unavailable,
    SelfUpdate,
    UninstallPackage,
    RestorePackage,
    NormalPackage,
    SelectedPackage,
    Hidden,
}

impl Button {
    pub fn get_default_style() -> impl Fn(&Theme, button::Status) -> button::Style {
        Button::Primary.get_style()
    }
    
    pub fn get_style(&self) -> impl Fn(&Theme, button::Status) -> button::Style + use<'_> {
        |theme: &Theme, status: button::Status| {
            match status {
                button::Status::Active => self.get_active_style(theme),
                button::Status::Hovered => self.get_hovered_style(theme),
                button::Status::Pressed => self.get_active_style(theme),
                button::Status::Disabled => self.get_disabled_style(theme),
            }
        }
    }

    fn get_active_style(&self, theme: &Theme) -> button::Style {
        let base_style = |bg: Option<Color>, mc| button::Style {
            background: Some(bg.unwrap_or(theme.extended_palette().background.strong.color).into()),
            text_color: mc,
            border: Border {
                color: Color {
                    a: 0.5,
                    ..mc
                },
                width: 1.0,
                radius: 2.0.into(),
            },
            shadow: Shadow::default(),
        };
        match self {
            Button::Primary | Button::SelfUpdate => base_style(None, theme.palette().primary),
            Button::Unavailable | Button::UninstallPackage => base_style(None, theme.palette().danger),
            Button::RestorePackage => base_style(None, theme.extended_palette().secondary.base.color),
            Button::NormalPackage => button::Style {
                background: Some(theme.extended_palette().background.strong.color.into()),
                text_color: theme.extended_palette().background.strong.text,
                border: Border {
                    color: theme.palette().background,
                    width: 0.0,
                    radius: 5.0.into(),
                },
                shadow: Shadow::default(),
            },
            Button::SelectedPackage => button::Style {
                background: Some(Background::Color(Color {a: 0.25, ..theme.palette().primary})),
                text_color: theme.palette().primary,
                border: Border {
                    color: theme.palette().primary,
                    width: 0.0,
                    radius: 5.0.into(),
                },
                shadow: Shadow::default(),
            },
            Button::Hidden => button::Style {
                background: Some(Color::TRANSPARENT.into()),
                text_color: Color::TRANSPARENT,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 5.0.into(),
                },
                shadow: Shadow::default(),
            }
        }
    }

    fn get_hovered_style(&self, theme: &Theme) -> button::Style {
        let active_style = self.get_active_style(theme);
        let base_style = |bg| button::Style {
            background: Some(Background::Color(Color { a: 0.25, ..bg })),
            ..active_style
        };

        match self {
            Button::Primary | Button::SelfUpdate | Button::NormalPackage | Button::SelectedPackage => base_style(theme.palette().primary),
            Button::RestorePackage => base_style(theme.extended_palette().secondary.base.color),
            Button::Unavailable | Button::UninstallPackage => base_style(theme.palette().danger),
            Button::Hidden => base_style(Color::TRANSPARENT)
        }
    }

    fn get_disabled_style(&self, theme: &Theme) -> button::Style {
        let active_style = self.get_active_style(theme);
        let base_style = |bg| button::Style {
            background: Some(Background::Color(Color { a: 0.05, ..bg })),
            text_color: Color {
                a: 0.50,
                ..bg
            },
            ..active_style
        };

        match self {
            Button::RestorePackage | Button::Primary => base_style(theme.palette().primary),
            Button::UninstallPackage => base_style(theme.palette().danger),
            _ => active_style,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Scrollable {
    #[default]
    Description,
    Packages,
}

impl Scrollable {
    pub fn get_style(&self) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style + use<'_> {
        let scrollable_type = self.clone();
        move |theme: &Theme, _status: scrollable::Status| {
            let scroller_color = match scrollable_type {
                Scrollable::Description => theme.extended_palette().background.weak,
                Scrollable::Packages => theme.extended_palette().background.strong
            }.color;
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: Rail {
                    background: Some(Color::TRANSPARENT.into()),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    scroller: scrollable::Scroller {
                        color: scroller_color,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                    },
                },
                horizontal_rail: Rail {
                    background: Some(Color::TRANSPARENT.into()),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    scroller: scrollable::Scroller {
                        color: scroller_color,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                    },
                },
                gap: None,
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CheckBox {
    #[default]
    PackageEnabled,
    PackageDisabled,
    SettingsEnabled,
    SettingsDisabled,
}

impl CheckBox {
    pub fn get_style(&self) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style + use<'_> {
        |theme: &Theme, status: checkbox::Status| {
            match status {
                checkbox::Status::Active { is_checked } => self.get_active_style(theme, is_checked),
                checkbox::Status::Hovered { is_checked: _is_checked } => checkbox::Style {
                    background: theme.extended_palette().background.weak.color.into(),
                    icon_color: theme.palette().primary,
                    border: Border {
                        color: theme.palette().primary,
                        width: 2.0,
                        radius: 5.0.into(),
                    },
                    text_color: Some(theme.extended_palette().background.weak.text),
                },
                checkbox::Status::Disabled { is_checked } => self.get_active_style(theme, is_checked),
            }
        }
    }

    fn get_active_style(&self, theme: &Theme, _is_checked: bool) -> checkbox::Style {
        match self {
            CheckBox::PackageEnabled => checkbox::Style {
                background: theme.palette().background.into(),
                icon_color: theme.palette().primary,
                border: Border {
                    color: theme.palette().background,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                text_color: Some(theme.palette().text),
            },
            CheckBox::PackageDisabled => checkbox::Style {
                background: Background::Color(Color {
                    a: 0.55,
                    ..theme.palette().background
                }),
                icon_color: theme.palette().primary,
                border: Border {
                    color: theme.palette().primary,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                text_color: Some(theme.palette().primary),
            },
            CheckBox::SettingsEnabled => checkbox::Style {
                background: theme.palette().background.into(),
                icon_color: theme.palette().primary,
                border: Border {
                    color: theme.palette().primary,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                text_color: Some(theme.palette().text),
            },
            CheckBox::SettingsDisabled => checkbox::Style {
                background: theme.extended_palette().background.strong.color.into(),
                icon_color: theme.palette().primary,
                border: Border {
                    color: theme.palette().primary,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                text_color: Some(theme.palette().text),
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum Text {
    #[default]
    Default,
    Ok,
    Danger,
    Commentary,
    Color(Color),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

impl Text {
    pub fn get_style(&self) -> impl Fn(&Theme) -> text::Style + use<'_> {
        let text_type = self.clone();
        move |theme: &Theme| {
            match text_type {
                Text::Default => text::Style::default(),
                Text::Ok => text::Style { color: Some(theme.extended_palette().secondary.base.text) },
                Text::Danger => text::Style { color: Some(theme.palette().danger) },
                Text::Commentary => text::Style { color: Some(theme.palette().text) },
                Text::Color(c) => text::Style { color: Some(c.clone()) },
            }
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {

}
