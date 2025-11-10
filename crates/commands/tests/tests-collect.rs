use reflections_commands::{Command, Verbosity, commands::collect::Collect, run};
use std::fs;
use temp_env::async_with_vars;
use testdir::testdir;

/// Helper to create a Collect command with sensible defaults
fn collect_cmd() -> Collect {
    Collect::builder()
        .artifacts_dir("out".to_string())
        .output("abis.json".to_string())
        .tag("@custom:swagger".to_string())
        .build()
}

#[tokio::test]
async fn test_collect_no_artifacts_dir() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();

    let cmd: Command = collect_cmd().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    
    // Should fail with error about missing artifacts directory
    assert!(res.is_err());
}

#[tokio::test]
async fn test_collect_with_simple_artifacts() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    // Create out directory structure
    let out_dir = dir.join("out");
    let contract_dir = out_dir.join("Counter.sol");
    fs::create_dir_all(&contract_dir).unwrap();
    
    // Create a simple artifact JSON
    let artifact = r#"{
        "abi": [
            {
                "type": "function",
                "name": "increment",
                "inputs": [],
                "outputs": [],
                "stateMutability": "nonpayable"
            },
            {
                "type": "function",
                "name": "count",
                "inputs": [],
                "outputs": [
                    {
                        "type": "uint256",
                        "name": "",
                        "internalType": "uint256"
                    }
                ],
                "stateMutability": "view"
            }
        ],
        "metadata": "{\"compiler\":{\"version\":\"0.8.0\"},\"output\":{\"devdoc\":{\"custom\":{\"swagger\":\"Core\"}},\"userdoc\":{}}}"
    }"#;
    
    fs::write(contract_dir.join("Counter.json"), artifact).unwrap();

    let cmd: Command = collect_cmd().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    
    assert!(res.is_ok(), "{res:?}");
    
    // Verify output file was created
    let output_path = dir.join("abis.json");
    assert!(output_path.exists());
    
    // Verify content
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Counter"));
    assert!(content.contains("Core"));
    assert!(content.contains("grouped"));
}

#[tokio::test]
async fn test_collect_with_custom_tag() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    let out_dir = dir.join("out");
    let contract_dir = out_dir.join("Token.sol");
    fs::create_dir_all(&contract_dir).unwrap();
    
    let artifact = r#"{
        "abi": [
            {
                "type": "function",
                "name": "transfer",
                "inputs": [],
                "outputs": [],
                "stateMutability": "nonpayable"
            }
        ],
        "metadata": "{\"output\":{\"devdoc\":{\"title\":\"MyToken\"},\"userdoc\":{}}}"
    }"#;
    
    fs::write(contract_dir.join("Token.json"), artifact).unwrap();

    let mut cmd = collect_cmd();
    cmd.tag = "@title".to_string();
    
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd.into(), Verbosity::default()),
    )
    .await;
    
    assert!(res.is_ok(), "{res:?}");
    
    let content = fs::read_to_string(dir.join("abis.json")).unwrap();
    assert!(content.contains("Token"));
    assert!(content.contains("MyToken"));
}

#[tokio::test]
async fn test_collect_without_groups() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    let out_dir = dir.join("out");
    let contract_dir = out_dir.join("Simple.sol");
    fs::create_dir_all(&contract_dir).unwrap();
    
    // Artifact without group metadata
    let artifact = r#"{
        "abi": [
            {
                "type": "function",
                "name": "doSomething",
                "inputs": [],
                "outputs": [],
                "stateMutability": "nonpayable"
            }
        ],
        "metadata": "{\"output\":{\"devdoc\":{},\"userdoc\":{}}}"
    }"#;
    
    fs::write(contract_dir.join("Simple.json"), artifact).unwrap();

    let cmd: Command = collect_cmd().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    
    assert!(res.is_ok(), "{res:?}");
    
    let content = fs::read_to_string(dir.join("abis.json")).unwrap();
    assert!(content.contains("Simple"));
    assert!(content.contains("ungrouped"));
}

#[tokio::test]
async fn test_collect_multiple_contracts_same_group() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    let out_dir = dir.join("out");
    
    // Create first contract
    let contract1_dir = out_dir.join("TokenA.sol");
    fs::create_dir_all(&contract1_dir).unwrap();
    let artifact1 = r#"{
        "abi": [{"type": "function", "name": "transfer", "inputs": [], "outputs": [], "stateMutability": "nonpayable"}],
        "metadata": "{\"output\":{\"devdoc\":{\"custom\":{\"swagger\":\"Tokens\"}},\"userdoc\":{}}}"
    }"#;
    fs::write(contract1_dir.join("TokenA.json"), artifact1).unwrap();
    
    // Create second contract with same group
    let contract2_dir = out_dir.join("TokenB.sol");
    fs::create_dir_all(&contract2_dir).unwrap();
    let artifact2 = r#"{
        "abi": [{"type": "function", "name": "mint", "inputs": [], "outputs": [], "stateMutability": "nonpayable"}],
        "metadata": "{\"output\":{\"devdoc\":{\"custom\":{\"swagger\":\"Tokens\"}},\"userdoc\":{}}}"
    }"#;
    fs::write(contract2_dir.join("TokenB.json"), artifact2).unwrap();

    let cmd: Command = collect_cmd().into();
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd, Verbosity::default()),
    )
    .await;
    
    assert!(res.is_ok(), "{res:?}");
    
    let content = fs::read_to_string(dir.join("abis.json")).unwrap();
    assert!(content.contains("TokenA"));
    assert!(content.contains("TokenB"));
    assert!(content.contains("Tokens"));
    
    // Verify both are in same group
    let collection: serde_json::Value = serde_json::from_str(&content).unwrap();
    let grouped = collection.get("grouped").unwrap();
    let tokens_group = grouped.get("Tokens").unwrap().as_array().unwrap();
    assert_eq!(tokens_group.len(), 2);
}

#[tokio::test]
async fn test_collect_custom_output_path() {
    let dir = testdir!();
    fs::create_dir_all(&dir).unwrap();
    
    let out_dir = dir.join("out");
    let contract_dir = out_dir.join("Test.sol");
    fs::create_dir_all(&contract_dir).unwrap();
    
    let artifact = r#"{
        "abi": [{"type": "constructor", "inputs": [], "stateMutability": "nonpayable"}],
        "metadata": "{\"output\":{\"devdoc\":{},\"userdoc\":{}}}"
    }"#;
    
    fs::write(contract_dir.join("Test.json"), artifact).unwrap();

    let mut cmd = collect_cmd();
    cmd.output = "custom-abis.json".to_string();
    
    let res = async_with_vars(
        [("REFLECTIONS_PROJECT_ROOT", Some(dir.to_string_lossy().as_ref()))],
        run(cmd.into(), Verbosity::default()),
    )
    .await;
    
    assert!(res.is_ok(), "{res:?}");
    assert!(dir.join("custom-abis.json").exists());
}
