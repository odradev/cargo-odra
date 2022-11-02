Feature: Cargo Odra Test
  Scenario: cargo odra test runs tests on mock-vm
    Given odra set up
    When I run "cargo odra test"
    Then error code is 0
    And I see "Running cargo test..."
    And I see "test result: ok"

  Scenario: cargo odra test -b casper fails due to missing configuration
    Given odra set up
    When I run "cargo odra test -b casper"
    Then error code is 4
    And I see "No backend configured"

  Scenario: cargo odra test -b casper runs tests on casper vm
    Given odra set up
    When I run "cargo odra backend add -p casper"
    And I run "rustup target add wasm32-unknown-unknown"
    And I run "cargo odra test -b casper"
    Then error code is 0
    And I see "Generating wasm files"
    And I see "Building backend library"
    And I see "Running cargo test..."
    And I see "test result: ok"