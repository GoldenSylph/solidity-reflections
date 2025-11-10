use reflections_commands::{Command, Verbosity, commands::init::Init, run};
use std::fs;
use temp_env::async_with_vars;
use testdir::testdir;

#[tokio::test]
async fn test_init_clean() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".gitignore"), "# Test gitignore\n").unwrap();

    let cmd: Command = Init::builder().clean(true).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("/scripts/reflections/"));

    // Verify DI framework was copied
    assert!(dir.join("scripts/reflections/di/Autowirable.s.sol").exists());
    assert!(dir.join("scripts/reflections/di/interfaces").exists());
    assert!(dir.join("scripts/reflections/di/wiring").exists());
    assert!(dir.join("scripts/reflections/di/configurations").exists());
    
    // Verify reflections.toml was created
    assert!(dir.join("reflections.toml").exists());
    let config = fs::read_to_string(dir.join("reflections.toml")).unwrap();
    assert!(config.contains("openzeppelin-version"));
    assert!(config.contains("zksync-os-url"));
}

#[tokio::test]
async fn test_init_no_clean() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".gitignore"), "# Test gitignore\n").unwrap();

    let cmd: Command = Init::builder().build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("/scripts/reflections/"));
    
    // Verify reflections.toml exists
    assert!(dir.join("reflections.toml").exists());
}

#[tokio::test]
async fn test_init_no_gitignore() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();

    let cmd: Command = Init::builder().clean(true).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    // .gitignore should now be created
    assert!(dir.join(".gitignore").exists());
    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("/scripts/reflections/"));
}

#[tokio::test]
async fn test_init_existing_reflections_in_gitignore() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".gitignore"), "# Test\n\n# Reflections - DI Framework\n# The entire scripts/reflections/di/ directory is scaffolded by `reflections init`\n# and should not be committed to version control\n/scripts/reflections/\n").unwrap();

    let cmd: Command = Init::builder().build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    let gitignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    // Should not add duplicate entry
    assert_eq!(gitignore.matches("/scripts/reflections/").count(), 1);
}

#[tokio::test]
async fn test_init_custom_config() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();

    let cmd: Command = Init::builder()
        .openzeppelin_version("v5.2.0".to_string())
        .zksync_os_url("https://custom.example.com/zksync".to_string())
        .build()
        .into();
    
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    // Verify custom config was saved
    assert!(dir.join("reflections.toml").exists());
    let config = fs::read_to_string(dir.join("reflections.toml")).unwrap();
    assert!(config.contains("v5.2.0"));
    assert!(config.contains("https://custom.example.com/zksync"));
}

#[tokio::test]
async fn test_init_with_remappings() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    // Create remappings.txt
    fs::write(
        dir.join("remappings.txt"),
        "@openzeppelin/=lib/openzeppelin-contracts/\nforge-std/=lib/forge-std/src/\n"
    ).unwrap();

    let cmd: Command = Init::builder().build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    // Verify framework was scaffolded
    assert!(dir.join("scripts/reflections/di/Autowirable.s.sol").exists());
    
    // Verify imports were remapped in scaffolded files
    let autowirable = fs::read_to_string(dir.join("scripts/reflections/di/Autowirable.s.sol")).unwrap();
    assert!(autowirable.contains("lib/openzeppelin-contracts/") || autowirable.contains("@openzeppelin/"));
}

#[tokio::test]
async fn test_init_clean_removes_previous_scaffolding() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    // Create fake previous scaffolding
    let reflections_dir = dir.join("scripts").join("reflections");
    fs::create_dir_all(&reflections_dir).unwrap();
    fs::write(reflections_dir.join("old_file.txt"), "old content").unwrap();

    let cmd: Command = Init::builder().clean(true).build().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    // Verify old file was removed
    assert!(!dir.join("scripts/reflections/old_file.txt").exists());
    
    // Verify new scaffolding exists
    assert!(dir.join("scripts/reflections/di/Autowirable.s.sol").exists());
}

#[tokio::test]
async fn test_init_updates_existing_config() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    // Create existing config
    fs::write(
        dir.join("reflections.toml"),
        "openzeppelin-version = \"v5.0.0\"\nzksync-os-url = \"https://old.example.com\"\n"
    ).unwrap();

    let cmd: Command = Init::builder()
        .openzeppelin_version("v5.3.0".to_string())
        .build()
        .into();
    
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    assert!(res.is_ok(), "{res:?}");

    // Verify config was updated
    let config = fs::read_to_string(dir.join("reflections.toml")).unwrap();
    assert!(config.contains("v5.3.0"));
}
