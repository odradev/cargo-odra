Feature: Odra Backend
  Scenario: When I run "cargo odra backend list" an empty list is shown
    Given odra set up
    When I run "cargo odra backend list"
    Then I see "No backend configured"

  Scenario: When I run "cargo odra backend list" a list of backends is shown
    Given odra set up
    When I run "cargo odra backend add -p casper"
    And I run "cargo odra backend list"
    Then I see "casper"

  Scenario: When I add backend as a github repo, I see it in the list
    Given odra set up
    When I run "cargo odra backend add -p casper -b develop -n casper-gh -r https://github.com/odradev/odra-casper.git"
    And I run "cargo odra backend list"
    Then I see "casper-gh"
    And I see "develop"
    And I see "https://github.com/odradev/odra-casper.git"