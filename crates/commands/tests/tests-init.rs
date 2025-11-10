use reflections_commands::{Command, Verbosity, commands::init::Init, run};
use reflections_core::config::ConfigLocation;
use std::fs;
use temp_env::async_with_vars;
use testdir::testdir;

#[tokio::test]
async fn test_init_clean() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".gitignore"), "# Test gitignore\n").unwrap();
    
    let cmd: Command =
        Init::builder().clean(true).config_location(ConfigLocation::Reflections).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");
    
    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("/reflections-output"));
}

#[tokio::test]
async fn test_init_no_clean() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".gitignore"), "# Test gitignore\n").unwrap();
    
    let cmd: Command = Init::builder().config_location(ConfigLocation::Reflections).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");
    
    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("/reflections-output"));
}

#[tokio::test]
async fn test_init_no_gitignore() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    let cmd: Command =
        Init::builder().clean(true).config_location(ConfigLocation::Reflections).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");
    assert!(!dir.join(".gitignore").exists());
}

#[tokio::test]
async fn test_init_existing_reflections_in_gitignore() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".gitignore"), "# Test\n\n# Reflections\n/reflections-output\n").unwrap();
    
    let cmd: Command =
        Init::builder().config_location(ConfigLocation::Foundry).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");
    
    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    // Should not add duplicate entry
    assert_eq!(gitignore.matches("/reflections-output").count(), 1);
}
