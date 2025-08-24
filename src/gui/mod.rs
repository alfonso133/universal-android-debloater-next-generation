pub mod style;
pub mod views;
pub mod widgets;

use crate::core::adb;
use crate::core::sync::{get_devices_list, initial_load, Phone};
use crate::core::theme::OS_COLOR_SCHEME;
use crate::core::uad_lists::UadListState;
use crate::core::update::{get_latest_release, Release, SelfUpdateState, SelfUpdateStatus};
use crate::core::utils::{string_to_theme, NAME};

use iced::advanced::graphics::image::image_rs::ImageFormat;
use iced::window::icon;
use iced::{font, Task};
use views::about::{About as AboutView, Message as AboutMessage};
use views::list::{List as AppsView, LoadingState as ListLoadingState, Message as AppsMessage};
use views::settings::{Message as SettingsMessage, Settings as SettingsView};
use widgets::navigation_menu::nav_menu;

use iced::widget::column;
use iced::{
    window::Settings as Window, Alignment, Element, Length, Settings,
};

#[cfg(feature = "self-update")]
use std::path::PathBuf;

#[cfg(feature = "self-update")]
use crate::core::update::{download_update_to_temp_file, remove_file, BIN_NAME};

#[derive(Default, Debug, Clone)]
enum View {
    #[default]
    List,
    About,
    Settings,
}

#[derive(Default, Clone)]
pub struct UpdateState {
    self_update: SelfUpdateState,
    uad_list: UadListState,
}

//gui status
#[derive(Default)]
pub struct UadGui {
    view: View,
    apps_view: AppsView,
    about_view: AboutView,
    settings_view: SettingsView,
    devices_list: Vec<Phone>,
    /// index of `devices_list`
    selected_device: Option<Phone>,
    update_state: UpdateState,
    nb_running_async_adb_commands: u32,
    adb_satisfied: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Navigation Panel
    AboutPressed,
    SettingsPressed,
    AppsPress,
    DeviceSelected(Phone),
    AboutAction(AboutMessage),
    AppsAction(AppsMessage),
    SettingsAction(SettingsMessage),
    RefreshButtonPressed,
    RebootButtonPressed,
    LoadDevices(Vec<Phone>),
    #[cfg(feature = "self-update")]
    _NewReleaseDownloaded(Result<(PathBuf, PathBuf), ()>),
    GetLatestRelease(Result<Option<Release>, ()>),
    FontLoaded(Result<(), font::Error>),
    Nothing,
    ADBSatisfied(bool),
}

pub struct GuiConfig;

impl GuiConfig {
    fn theme(state: &UadGui) -> iced::Theme {
        string_to_theme(&state.settings_view.general.theme)
    }
}

impl UadGui {
    pub fn start() -> iced::Result {
        let logo: &[u8] = match *OS_COLOR_SCHEME {
            // remember to keep `Unspecified` in sync with `src/core/theme`
            dark_light::Mode::Dark | dark_light::Mode::Unspecified => {
                include_bytes!("../../resources/assets/logo-dark.png")
            }
            dark_light::Mode::Light => {
                include_bytes!("../../resources/assets/logo-light.png")
            }
        };

        iced::application("Universal Android Debloater Next Generation", UadGui::update, UadGui::view)
            .font(include_bytes!("../../resources/assets/icons.ttf").as_slice())
            .settings(Settings {
                id: Some(String::from(NAME)),
                default_text_size: iced::Pixels(16.0),
                ..Settings::default()
            })
            .window(Window {
                size: iced::Size {
                    width: 950.0,
                    height: 700.0,
                },
                resizable: true,
                decorations: true,
                icon: icon::from_file_data(logo, Some(ImageFormat::Png)).ok(),
                ..Window::default()
            })
            .theme(GuiConfig::theme)
            .run_with(|| {
                (
                    UadGui::default(),
                    Task::batch([
                        // Used in crate::gui::widgets::navigation_menu::ICONS. Name is `icomoon`.
                        font::load(include_bytes!("../../resources/assets/icons.ttf").as_slice())
                            .map(Message::FontLoaded),
                        Task::perform(initial_load(), Message::ADBSatisfied),
                        Task::perform(get_devices_list(), Message::LoadDevices),
                        Task::perform(
                            async move { get_latest_release() },
                            Message::GetLatestRelease,
                        ),
                    ]),
                )
            })
    }

    #[allow(clippy::too_many_lines)]
    fn update(state: &mut UadGui, msg: Message) -> Task<Message> {
        match msg {
            Message::LoadDevices(devices_list) => {
                state.selected_device = match &state.selected_device {
                    Some(s_device) => {
                        // Try to reload last selected phone
                        devices_list
                            .iter()
                            .find(|phone| phone.adb_id == s_device.adb_id)
                            .cloned()
                    }
                    None => devices_list.first().cloned(),
                };
                state.devices_list = devices_list;

                #[expect(unused_must_use, reason = "side-effect")]
                {
                    let _ = UadGui::update(state, Message::SettingsAction(SettingsMessage::LoadDeviceSettings));
                }

                UadGui::update(state, Message::AppsAction(AppsMessage::LoadUadList(true)))
            }
            Message::AppsPress => {
                state.view = View::List;
                Task::none()
            }
            Message::AboutPressed => {
                state.view = View::About;
                state.update_state.self_update = SelfUpdateState::default();
                Task::perform(
                    async move { get_latest_release() },
                    Message::GetLatestRelease,
                )
            }
            Message::SettingsPressed => {
                state.view = View::Settings;
                Task::none()
            }
            Message::RefreshButtonPressed => {
                state.apps_view = AppsView::default();
                #[expect(unused_must_use, reason = "side-effect")]
                {
                    UadGui::update(state, Message::AppsAction(AppsMessage::ADBSatisfied(
                        state.adb_satisfied,
                    )));
                }
                Task::perform(get_devices_list(), Message::LoadDevices)
            }
            Message::RebootButtonPressed => {
                state.apps_view = AppsView::default();
                let serial = match &state.selected_device {
                    Some(d) => d.adb_id.clone(),
                    _ => String::default(),
                };
                state.selected_device = None;
                state.devices_list = vec![];
                Task::perform(
                    async { adb::ACommand::new().shell(serial).reboot() },
                    |_| Message::Nothing,
                )
            }
            Message::AppsAction(msg) => state
                .apps_view
                .update(
                    &mut state.settings_view,
                    &mut state.selected_device.clone().unwrap_or_default(),
                    &mut state.update_state.uad_list,
                    msg,
                )
                .map(Message::AppsAction),
            Message::SettingsAction(msg) => {
                match msg {
                    SettingsMessage::RestoringDevice(ref output) => {
                        state.nb_running_async_adb_commands -= 1;
                        state.view = View::List;

                        #[expect(unused_must_use, reason = "side-effect")]
                        {
                            state.apps_view.update(
                                &mut state.settings_view,
                                &mut state.selected_device.clone().unwrap_or_default(),
                                &mut state.update_state.uad_list,
                                AppsMessage::RestoringDevice(output.clone()),
                            );
                        }
                        if state.nb_running_async_adb_commands == 0 {
                            return UadGui::update(state, Message::RefreshButtonPressed);
                        }
                    }
                    SettingsMessage::MultiUserMode(toggled) if toggled => {
                        for user in state.apps_view.phone_packages.clone() {
                            for (i, _) in user.iter().filter(|&pkg| pkg.selected).enumerate() {
                                for u in state
                                    .selected_device
                                    .as_ref()
                                    .expect("Device should be selected")
                                    .user_list
                                    .iter()
                                    .filter(|&u| !u.protected)
                                {
                                    state.apps_view.phone_packages[u.index][i].selected = true;
                                }
                            }
                        }
                    }
                    _ => (),
                }
                state.settings_view
                    .update(
                        &state.selected_device.clone().unwrap_or_default(),
                        &state.apps_view.phone_packages,
                        &mut state.nb_running_async_adb_commands,
                        msg,
                        state.apps_view.selected_user,
                    )
                    .map(Message::SettingsAction)
            }
            Message::AboutAction(msg) => {
                state.about_view.update(msg.clone());

                match msg {
                    AboutMessage::UpdateUadLists => {
                        state.update_state.uad_list = UadListState::Downloading;
                        state.apps_view.loading_state = ListLoadingState::DownloadingList;
                        UadGui::update(state, Message::AppsAction(AppsMessage::LoadUadList(true)))
                    }
                    AboutMessage::DoSelfUpdate => {
                        #[cfg(feature = "self-update")]
                        if let Some(release) = state.update_state.self_update.latest_release.as_ref()
                        {
                            state.update_state.self_update.status = SelfUpdateStatus::Updating;
                            state.apps_view.loading_state = ListLoadingState::_UpdatingUad;
                            return Task::perform(
                                download_update_to_temp_file(BIN_NAME, release.clone()),
                                Message::_NewReleaseDownloaded,
                            )
                        } else {
                            return Task::none()
                        }
                        #[cfg(not(feature = "self-update"))]
                        Task::none()
                    }
                    AboutMessage::UrlPressed(_) => Task::none(),
                }
            }
            Message::DeviceSelected(s_device) => {
                state.selected_device = Some(s_device.clone());
                state.view = View::List;
                info!("{:-^65}", "-");
                info!(
                    "ANDROID_SDK: {} | DEVICE: {}",
                    s_device.android_sdk, s_device.model
                );
                info!("{:-^65}", "-");
                state.apps_view.loading_state = ListLoadingState::FindingPhones;

                #[expect(unused_must_use, reason = "side-effects")]
                {
                    UadGui::update(state, Message::SettingsAction(SettingsMessage::LoadDeviceSettings));
                    UadGui::update(state, Message::AppsAction(AppsMessage::ToggleAllSelected(false)));
                    UadGui::update(state, Message::AppsAction(AppsMessage::ClearSelectedPackages));
                }
                UadGui::update(state, Message::AppsAction(AppsMessage::LoadPhonePackages((
                    state.apps_view.uad_lists.clone(),
                    UadListState::Done,
                ))))
            }
            #[cfg(feature = "self-update")]
            Message::_NewReleaseDownloaded(res) => {
                debug!("{NAME} update has been downloaded!");

                if let Ok((relaunch_path, cleanup_path)) = res {
                    let mut args: Vec<_> = std::env::args().skip(1).collect();

                    // Remove the `--self-update-temp` arg from args if it exists,
                    // since we need to pass it cleanly. Otherwise new process will
                    // fail during arg parsing.
                    if let Some(idx) = args.iter().position(|a| a == "--self-update-temp") {
                        args.remove(idx);
                        // Remove path passed after this arg
                        args.remove(idx);
                    }

                    match std::process::Command::new(relaunch_path)
                        .args(args)
                        .arg("--self-update-temp")
                        .arg(&cleanup_path)
                        .spawn()
                    {
                        Ok(_) => {
                            if let Err(e) = remove_file(cleanup_path) {
                                error!("Could not remove temp update file: {e}");
                            }
                            std::process::exit(0)
                        }
                        Err(error) => {
                            if let Err(e) = remove_file(cleanup_path) {
                                error!("Could not remove temp update file: {e}");
                            }
                            error!("Failed to update {NAME}: {error}");
                        }
                    }
                } else {
                    error!("Failed to update {NAME}!");
                    #[expect(unused_must_use, reason = "side-effect")]
                    {
                        UadGui::update(state, Message::AppsAction(AppsMessage::UpdateFailed));
                        state.update_state.self_update.status = SelfUpdateStatus::Failed;
                    }
                }
                Task::none()
            }
            Message::GetLatestRelease(release) => {
                match release {
                    Ok(r) => {
                        state.update_state.self_update.status = SelfUpdateStatus::Done;
                        state.update_state.self_update.latest_release = r;
                    }
                    Err(()) => state.update_state.self_update.status = SelfUpdateStatus::Failed,
                }
                Task::none()
            }
            Message::FontLoaded(result) => {
                if let Err(error) = result {
                    error!("Couldn't load font: {error:?}");
                }

                Task::none()
            }
            Message::ADBSatisfied(result) => {
                state.adb_satisfied = result;
                UadGui::update(state, Message::AppsAction(AppsMessage::ADBSatisfied(
                    state.adb_satisfied,
                )))
            }
            Message::Nothing => Task::none(),
        }
    }

    fn view(state: &UadGui) -> Element<Message> {

        
        let navigation_container = nav_menu(
            &state.devices_list,
            state.selected_device.clone(),
            &state.apps_view,
            &state.update_state.self_update
        );

        let selected_device = state.selected_device.clone().unwrap_or_default();
        let main_container = match state.view {
            View::List => state
                .apps_view
                .view(&state.settings_view, &selected_device)
                .map(Message::AppsAction),
            View::About => state
                .about_view
                .view(&state.update_state)
                .map(Message::AboutAction),
            View::Settings => state
                .settings_view
                .view(&selected_device, &state.apps_view)
                .map(Message::SettingsAction),
        };

        column![navigation_container, main_container]
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }

}
