use crate::network_command::{default_command::DefaultCommand, echo::Echo, get_online_users::GetOnlineUsers, online_users::OnlineUsers, set_echo_availability::SetEchoAvailability, skip::Skip, user_data::UserData};

pub trait NetworkCommandID {
    const ID: u8;
}

impl NetworkCommandID for DefaultCommand {
    const ID: u8 = 0;
}
impl NetworkCommandID for Skip {
    const ID: u8 = 1;
}
impl NetworkCommandID for Echo {
    const ID: u8 = 2;
}
impl NetworkCommandID for OnlineUsers {
    const ID: u8 = 3;
}
impl NetworkCommandID for UserData {
    const ID: u8 = 4;
}
impl NetworkCommandID for GetOnlineUsers {
    const ID: u8 = 5;
}
impl NetworkCommandID for SetEchoAvailability {
    const ID: u8 = 6;
}