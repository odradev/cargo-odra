Feature: Cargo Odra Test
  @serial
  Scenario: cargo odra test runs tests on mock-vm
    Given odra set up in project folder
    When I run "cargo odra test" in project folder
    Then error code is 0
    And I see "Running cargo test..."
    And I see "test result: ok"

  @serial
  Scenario: cargo odra test -b casper fails due to missing configuration
    Given odra set up in project folder
    When I run "cargo odra test -b casper" in project folder
    Then error code is 4
    And I see "No backend configured"

  @serial
  Scenario: cargo odra test -b casper runs tests on casper vm
    Given odra set up in project folder
    When I run "cargo odra backend add -p casper" in project folder
    And I run "rustup target add wasm32-unknown-unknown" in project folder
    And I run "cargo odra test -b casper" in project folder
    Then error code is 0
    And I see "Generating wasm files"
    And I see "Building backend library"
    And I see "Running cargo test..."
    And I see "test result: ok"