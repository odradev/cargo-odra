Feature: Cargo Odra

  Scenario:
    When I run "cargo odra"
    Then I see "Krzysztof"
    And error code is 2