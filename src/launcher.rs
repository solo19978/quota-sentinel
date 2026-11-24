use crate::DesktopApp;
use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    sync::Mutex,
};
use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::{
        System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE},
        UI::{
            Shell::{Common::COMDLG_FILTERSPEC, ApplicationActivationManager, FileOpenDialog,
                IApplicationActivationManager, IFileOpenDialog, ShellExecuteW, AO_NONE,
                FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, SIGDN_FILESYSPATH},
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
};

static LAUNCH: Mutex<()> = Mutex::new(());

fn executable_name(provider: DesktopApp) -> &'static str {
    match provider {
        DesktopApp::Claude => "Claude.exe",
        DesktopApp::Codex => "ChatGPT.exe",
    }
}

fn executable_path(provider: DesktopApp, path: &Path) -> bool {
    // A UNC path can contact another computer before the executable is launched.
    let mut parts = path.components();
    let local_drive = matches!(
        (parts.next(), parts.next()),
        (Some(std::path::Component::Prefix(prefix)), Some(std::path::Component::RootDir))
            if matches!(prefix.kind(), std::path::Prefix::Disk(_))
    );
    local_drive && path.file_name().and_then(OsStr::to_str)
        .is_some_and(|file| file.eq_ignore_ascii_case(executable_name(provider)))
}

struct Apartment;

impl Apartment {
    fn enter() -> windows::core::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?; }
        Ok(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize(); }
    }
}

fn activate_packaged(provider: DesktopApp) -> windows::core::Result<()> {
    // Fixed package registrations verified on Windows. Other installations use the picker.
    let id = match provider {
        DesktopApp::Claude => w!("Claude_pzs8sxrjxfjjc!Claude"),
        DesktopApp::Codex => w!("OpenAI.Codex_2p2nqsd0c76g0!App"),
    };
    unsafe {
        let manager: IApplicationActivationManager =
            CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_INPROC_SERVER)?;
        manager.ActivateApplication(id, PCWSTR::null(), AO_NONE)?;
    }
    Ok(())
}

fn choose(provider: DesktopApp) -> windows::core::Result<Option<PathBuf>> {
    let name = HSTRING::from(executable_name(provider));
    let title = match provider {
        DesktopApp::Claude => w!("Locate Claude Desktop"),
        DesktopApp::Codex => w!("Locate ChatGPT Desktop"),
    };
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetTitle(title)?;
        dialog.SetOptions(dialog.GetOptions()? | FOS_FORCEFILESYSTEM | FOS_FILEMUSTEXIST)?;
        dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
            pszName: PCWSTR(name.as_ptr()),
            pszSpec: PCWSTR(name.as_ptr()),
        }])?;
        match dialog.Show(None) {
            Err(error) if error.code().0 as u32 == 0x800704C7 => return Ok(None),
            result => result?,
        }
        let path = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let result = OsString::from_wide(path.as_wide());
        CoTaskMemFree(Some(path.0.cast()));
        Ok(Some(PathBuf::from(result)))
    }
}

fn shell_open(target: &OsStr) -> bool {
    let file: Vec<u16> = target.encode_wide().chain([0]).collect();
    if file[..file.len() - 1].contains(&0) {
        return false;
    }
    let result = unsafe {
        ShellExecuteW(
            None, w!("open"), PCWSTR(file.as_ptr()), PCWSTR::null(),
            PCWSTR::null(), SW_SHOWNORMAL,
        )
    };
    result.0 as isize > 32
}
fn preference(provider: DesktopApp) -> Option<PathBuf> {
    let file = match provider {
        DesktopApp::Claude => "claude-desktop-path",
        DesktopApp::Codex => "chatgpt-desktop-path",
    };
    Some(dirs::config_dir()?.join("io.github.solo19978.quota-sentinel").join(file))
}

pub fn launch(provider: DesktopApp) -> Result<&'static str, &'static str> {
    let _guard = LAUNCH.try_lock().map_err(|_| "An app launcher is already open.")?;
    let _apartment = Apartment::enter().map_err(|_| "Could not initialize the Windows launcher.")?;
    let saved = preference(provider);
    if let Some(path) = saved.as_ref().and_then(|file| std::fs::read_to_string(file).ok()) {
        let path = Path::new(&path);
        if executable_path(provider, path) && path.is_file() && shell_open(path.as_os_str()) {
            return Ok("opened");
        }
    }
    if activate_packaged(provider).is_ok() {
        return Ok("opened");
    }
    let Some(selected) = choose(provider).map_err(|_| "Could not open the app picker.")? else {
        return Ok("cancelled");
    };
    let path = selected.as_path();
    if !executable_path(provider, path) || !path.is_file() {
        return Err(match provider {
            DesktopApp::Claude => "Choose Claude.exe on a local drive.",
            DesktopApp::Codex => "Choose ChatGPT.exe on a local drive.",
        });
    }
    if !shell_open(path.as_os_str()) {
        return Err("Windows could not open the selected app.");
    }
    if let Some(file) = saved {
        if let Some(directory) = file.parent() {
            let _ = std::fs::create_dir_all(directory)
                .and_then(|_| std::fs::write(&file, selected.to_string_lossy().as_bytes()));
        }
    }
    Ok("opened")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_matching_local_executable() {
        assert!(executable_path(DesktopApp::Claude, Path::new(r"C:\Apps\Claude.exe")));
        assert!(!executable_path(DesktopApp::Codex, Path::new(r"C:\Apps\Claude.exe")));
        assert!(!executable_path(DesktopApp::Claude, Path::new(r"\\server\Apps\Claude.exe")));
        assert!(!executable_path(DesktopApp::Codex, Path::new("ChatGPT.exe")));
    }
}
