Feature: Cargo new

  @serial
  Scenario: If we run cargo odra new it will create a new project
    Given clean workspace
    When I run "cargo odra new -n project"
    Then error code is 0
    And folder named project exists

  @serial
  Scenario: If we run cargo odra new two times, it will fail
    Given clean workspace
    When I run "cargo odra new -n project"
    And I run "cargo odra new -n project"
    Then error code is 101

  @serial
  Scenario: If we run cargo odra init in non-empty folder, it will fail
    Given clean workspace
    When I run "cargo odra new -n project"
    And I run "cargo odra init -n project" in project folder
    Then error code is 101

  @serial
  Scenario: If we run cargo odra init in empty folder, it will create a new project
    Given clean workspace
    When I create a new folder called project
    And I run "cargo odra init -n project" in project folder
    Then error code is 0