use opfs::{
    DirectoryHandle as _,
    persistent::{self, DirectoryHandle},
};
use weapon::opfs::UserDirectory;

#[derive(Debug)]
pub(crate) struct Directories {
    pub data_directory_handle: DirectoryHandle,
    pub current_user_directory_handle: UserDirectory,
    pub user_events_directory_handle: DirectoryHandle,
    pub weapon_directory_handle: DirectoryHandle,
}

pub(crate) async fn get_directories(
    user_id: &Option<String>,
) -> Result<Directories, persistent::Error> {
    let root = opfs::persistent::app_specific_dir().await?;
    let create = opfs::GetDirectoryHandleOptions { create: true };

    let data = root
        .get_directory_handle_with_options("data", &create)
        .await?;

    let weapon = root
        .get_directory_handle_with_options(".weapon", &create)
        .await?;

    let user_events = weapon
        .get_directory_handle_with_options("user-events", &create)
        .await?;

    let user = if let Some(user_id) = user_id {
        UserDirectory::new(&user_events, user_id).await?
    } else {
        UserDirectory::new(&user_events, "logged-out-unknown-user").await?
    };

    Ok(Directories {
        data_directory_handle: data,
        user_events_directory_handle: user_events,
        current_user_directory_handle: user,
        weapon_directory_handle: weapon,
    })
}
