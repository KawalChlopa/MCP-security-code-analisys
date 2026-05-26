use security_code_analisys::tools::bandit::BanditOutput;


// Bandit tool test
#[tokio::test]
async fn test_bandit_detects_yaml_load() {
    let result = BanditOutput::run_bandit("text/fixtures/python")
        .await
        .expect("Bandit should run");

    println!("{}", result);

    assert!(result.contains("yaml") || result.contains("B506"));

}
