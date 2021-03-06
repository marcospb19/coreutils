use std::{
    fs::{metadata, remove_file},
    io,
};

use super::*;

#[test]
fn touch_create_empty_files() {
    let matches = ArgMatches::new();
    let flags = TouchFlags::from_matches(&matches);
    let files = vec!["file1.rs", "file2.rs"];

    touch(&files, flags);

    assert_eq!(metadata("file1.rs").is_ok(), true);
    assert_eq!(metadata("file2.rs").is_ok(), true);
    remove_test_files(&files).unwrap();
}

#[test]
fn touch_update_existing_files() {
    let matches = ArgMatches::new();
    let flags = TouchFlags::from_matches(&matches);
    let files = vec!["file3.rs", "file4.rs"];

    File::create("file3.rs").unwrap();
    let file1_metadata = metadata("file3.rs").unwrap();
    let file1_mtime = FileTime::from_last_modification_time(&file1_metadata);
    let file1_atime = FileTime::from_last_access_time(&file1_metadata);

    // update and create files
    touch(&files, flags);

    let new_file1_metadata = metadata("file3.rs").unwrap();
    let new_file1_mtime = FileTime::from_last_modification_time(&new_file1_metadata);
    let new_file1_atime = FileTime::from_last_access_time(&new_file1_metadata);
    // check that file1 modification time has changed
    assert_ne!(file1_mtime, new_file1_mtime);

    // check that file1 access time has changed
    assert_ne!(file1_atime, new_file1_atime);

    remove_test_files(&files).unwrap();
}

#[test]
fn touch_update_only_access_time() {
    let matches = cli::create_app().get_matches_from(vec!["touch", "-a", "file5.rs", "file6.rs"]);

    let flags = TouchFlags::from_matches(&matches);

    let files: Vec<_> = matches.values_of("FILE").unwrap().collect();

    File::create(&files[0]).unwrap();

    let mut file1_metadata = metadata(&files[0]).unwrap();
    let file1_atime = FileTime::from_last_access_time(&file1_metadata);

    // update and create files
    touch(&files, flags);

    file1_metadata = metadata(&files[0]).unwrap();
    let new_file1_atime = FileTime::from_last_access_time(&file1_metadata);

    // check that first file access time has changed
    assert_ne!(file1_atime, new_file1_atime);
    remove_test_files(&files).unwrap();
}

#[test]
fn touch_update_only_modification_time() {
    let matches = cli::create_app().get_matches_from(vec!["touch", "-m", "file7.rs", "file8.rs"]);

    let flags = TouchFlags::from_matches(&matches);

    let files: Vec<_> = matches.values_of("FILE").unwrap().collect();

    File::create(&files[0]).unwrap();

    let mut file1_metadata = metadata(&files[0]).unwrap();
    let file1_mtime = FileTime::from_last_modification_time(&file1_metadata);

    // update and create files
    touch(&files, flags);

    file1_metadata = metadata(&files[0]).unwrap();
    let new_file1_mtime = FileTime::from_last_modification_time(&file1_metadata);

    // check that first file modification time has changed
    assert_ne!(file1_mtime, new_file1_mtime);

    remove_test_files(&files).unwrap();
}

#[test]
fn touch_update_time_with_date() {
    let matches = cli::create_app().get_matches_from(vec![
        "touch",
        "-d=2009-01-03 03:13:00",
        "file9.rs",
        "file10.rs",
    ]);

    let flags = TouchFlags::from_matches(&matches);

    let files: Vec<_> = matches.values_of("FILE").unwrap().collect();

    File::create(&files[0]).unwrap();

    // update and create files
    touch(&files, flags);

    let file1_metadata = metadata(&files[0]).unwrap();
    let file1_mtime = FileTime::from_last_modification_time(&file1_metadata);

    let time = time::OffsetDateTime::from_unix_timestamp(file1_mtime.unix_seconds());

    // check modification and access time is equal 2009-01-03 03:13:00
    assert_eq!(
        time,
        PrimitiveDateTime::parse("2009-01-03 03:13:00", "%Y-%m-%d %H:%M:%S").unwrap().assume_utc()
    );
    remove_test_files(&files).unwrap();
}

fn touch(files: &[&str], flags: TouchFlags) {
    let (new_atime, new_mtime) = new_filetimes(flags).unwrap_or_else(|err| {
        eprintln!("touch: {}", err);
        process::exit(1);
    });

    for filename in files {
        // if file already exist in the current directory
        let file_metadata =
            if flags.no_deref { fs::symlink_metadata(&filename) } else { fs::metadata(&filename) };

        if file_metadata.is_err() && !flags.no_create {
            match File::create(&filename) {
                Ok(_) => (),
                Err(e) => eprintln!("touch: Failed to create file {}: {}", &filename, e),
            }
        } else {
            // Ok to unwrap cause it was checked in the first condition of the if-elseif-else
            // expression.
            update_time(&filename, new_atime, new_mtime, &file_metadata.unwrap(), flags);
        }
    }
}

fn remove_test_files(files: &[&str]) -> io::Result<()> {
    for filename in files {
        remove_file(&filename)?;
    }
    Ok(())
}
