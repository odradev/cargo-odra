Feature: Cargo new

  Scenario: If we run cargo odra new it will create a new project
    When I run "cargo odra new -n project"
    Then error code is 0
    And folder named project exists

  Scenario: If we run cargo odra new two times, it will fail
    When I run "cargo odra new -n testnew"
    And I run "cargo odra new -n testnew"
    Then error code is 101

  Scenario: If we run cargo odra init in non-empty folder, it will fail
    When I run "cargo odra init -n project"
    And I run "cargo odra init -n project"
    Then error code is 101

  Scenario: If we run cargo odra init in empty folder, it will create a new project
    When I create a new folder called project
    And I run "cargo odra init -n project"
    Then error code is 0