//! Restrict local secrets before reading or writing them. Never silently fall back
//! to inherited permissions when Windows ACL configuration fails.
use std::{fs, io, path::Path};

pub(crate) fn prepare_directory(path: &Path) -> io::Result<()> {
    if let (Ok(target), Ok(shared_temp)) = (
        fs::canonicalize(path),
        fs::canonicalize(std::env::temp_dir()),
    ) {
        if target == shared_temp {
            return Err(io::Error::other(
                "Private storage requires its own directory, not the shared temporary directory",
            ));
        }
    }
    fs::create_dir_all(path)?;
    restrict(path, true)
}

pub(crate) fn restrict_existing_file(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => restrict(path, false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(crate) fn open_for_write(path: &Path) -> io::Result<fs::File> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Missing private storage directory"))?;
    prepare_directory(parent)?;
    restrict_existing_file(path)?;
    // New files inherit only the private directory's entries. Existing files
    // are repaired before being opened/truncated, including leftover temp files.
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    restrict(path, false)?;
    Ok(file)
}

#[cfg(not(windows))]
fn restrict(path: &Path, directory: bool) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.is_dir() != directory {
        return Err(io::Error::other(
            "Private storage must be a regular file or directory",
        ));
    }
    fs::set_permissions(
        path,
        fs::Permissions::from_mode(if directory { 0o700 } else { 0o600 }),
    )
}

#[cfg(windows)]
fn restrict(path: &Path, directory: bool) -> io::Result<()> {
    use std::{
        os::windows::{ffi::OsStrExt, fs::MetadataExt},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SetNamedSecurityInfoW,
                SDDL_REVISION_1, SE_FILE_OBJECT,
            },
            GetSecurityDescriptorDacl, DACL_SECURITY_INFORMATION,
            PROTECTED_DACL_SECURITY_INFORMATION,
        },
    };
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_attributes() & 0x400 != 0 || metadata.is_dir() != directory {
        return Err(io::Error::other(
            "Private storage must not be a reparse point or unexpected file type",
        ));
    }
    let sid = current_user_sid()?;
    let inheritance = if directory { "OICI" } else { "" };
    let sddl: Vec<u16> = format!(
        "D:P(A;{inheritance};FA;;;SY)(A;{inheritance};FA;;;BA)(A;{inheritance};FA;;;{sid})"
    )
    .encode_utf16()
    .chain(Some(0))
    .collect();
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let mut descriptor = ptr::null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let result = (|| {
            let mut present = 0;
            let mut defaulted = 0;
            let mut dacl = ptr::null_mut();
            if GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted) == 0 {
                return Err(io::Error::last_os_error());
            }
            if present == 0 || dacl.is_null() {
                return Err(io::Error::other("Private storage ACL is missing"));
            }
            let error = SetNamedSecurityInfoW(
                name.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                dacl,
                ptr::null_mut(),
            );
            if error != 0 {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
            Ok(())
        })();
        LocalFree(descriptor);
        result
    }
}

#[cfg(windows)]
fn current_user_sid() -> io::Result<String> {
    use std::ptr;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, LocalFree},
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TokenUser, TOKEN_QUERY,
            TOKEN_USER,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    unsafe {
        let mut token = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = (|| {
            let mut length = 0;
            GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut length);
            if length == 0 {
                return Err(io::Error::last_os_error());
            }
            // TOKEN_USER contains pointers; keep the returned buffer aligned.
            let mut buffer = vec![0usize; (length as usize).div_ceil(size_of::<usize>())];
            if GetTokenInformation(
                token,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                length,
                &mut length,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
            let mut text = ptr::null_mut();
            if ConvertSidToStringSidW(user.User.Sid, &mut text) == 0 {
                return Err(io::Error::last_os_error());
            }
            let mut count = 0;
            while *text.add(count) != 0 {
                count += 1;
            }
            let sid = String::from_utf16_lossy(std::slice::from_raw_parts(text, count));
            LocalFree(text.cast());
            Ok(sid)
        })();
        CloseHandle(token);
        result
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{io::Write, process::Command};

    #[test]
    fn repairs_inherited_permissions_on_directory_existing_file_and_temp_file() {
        let root = std::env::temp_dir().join(format!("cli-hub-acl-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let grant = Command::new("icacls.exe")
            .arg(&root)
            .args(["/grant", "*S-1-1-0:(OI)(CI)R"])
            .output()
            .unwrap();
        assert!(grant.status.success());
        let file = root.join("preferences.json");
        let temp = root.join("preferences.json.tmp");
        fs::write(&file, "existing").unwrap();
        fs::write(&temp, "old temporary content").unwrap();
        prepare_directory(&root).unwrap();
        restrict_existing_file(&file).unwrap();
        open_for_write(&temp)
            .unwrap()
            .write_all(b"private")
            .unwrap();
        let new_file = root.join("terminal-host.json.tmp");
        open_for_write(&new_file)
            .unwrap()
            .write_all(b"new private")
            .unwrap();
        let expected = [
            "S-1-5-18".to_string(),
            "S-1-5-32-544".to_string(),
            current_user_sid().unwrap(),
        ];
        for path in [&root, &file, &temp, &new_file] {
            assert_private_acl(path, &expected);
        }
        assert_eq!(fs::read_to_string(&file).unwrap(), "existing");
        assert_eq!(fs::read_to_string(&temp).unwrap(), "private");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_unexpected_file_types_without_modifying_target() {
        assert!(restrict_existing_file(&std::env::temp_dir()).is_err());
        assert!(prepare_directory(&std::env::temp_dir()).is_err());
    }

    fn assert_private_acl(path: &Path, expected: &[String]) {
        use std::{os::windows::ffi::OsStrExt, ptr};
        use windows_sys::Win32::{
            Foundation::LocalFree,
            Security::{
                Authorization::{ConvertSidToStringSidW, GetNamedSecurityInfoW, SE_FILE_OBJECT},
                GetAce, GetSecurityDescriptorControl, ACCESS_ALLOWED_ACE,
                DACL_SECURITY_INFORMATION, SE_DACL_PROTECTED,
            },
        };
        let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        unsafe {
            let mut descriptor = ptr::null_mut();
            let mut acl = ptr::null_mut();
            assert_eq!(
                GetNamedSecurityInfoW(
                    name.as_ptr(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut acl,
                    ptr::null_mut(),
                    &mut descriptor
                ),
                0
            );
            let mut control = 0;
            let mut revision = 0;
            assert_ne!(
                GetSecurityDescriptorControl(descriptor, &mut control, &mut revision),
                0
            );
            assert_ne!(control & SE_DACL_PROTECTED, 0);
            assert_eq!((*acl).AceCount as usize, expected.len());
            let mut actual = Vec::new();
            for index in 0..(*acl).AceCount {
                let mut ace = ptr::null_mut();
                assert_ne!(GetAce(acl, index as u32, &mut ace), 0);
                let entry = &*ace.cast::<ACCESS_ALLOWED_ACE>();
                assert_eq!(entry.Header.AceType, 0); // ACCESS_ALLOWED_ACE_TYPE
                assert_eq!(entry.Mask, 0x1f01ff); // FILE_ALL_ACCESS
                let sid = (&entry.SidStart as *const u32).cast_mut().cast();
                let mut text = ptr::null_mut();
                assert_ne!(ConvertSidToStringSidW(sid, &mut text), 0);
                let mut length = 0;
                while *text.add(length) != 0 {
                    length += 1;
                }
                actual.push(String::from_utf16_lossy(std::slice::from_raw_parts(
                    text, length,
                )));
                LocalFree(text.cast());
            }
            LocalFree(descriptor);
            for sid in expected {
                assert!(actual.contains(sid), "Unexpected ACL: {actual:?}");
            }
        }
    }
}
