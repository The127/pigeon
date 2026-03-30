Feature: List Organizations

  List all organizations with pagination support.

  Scenario: Listing organizations when none exist
    Given no organizations exist
    When the list organizations query is executed
    Then the result should be an empty paginated list

  Scenario: Listing organizations with results
    Given 2 organizations exist
    When the list organizations query is executed
    Then the result should contain 2 organizations

  Scenario: Listing organizations respects pagination
    Given 2 organizations exist
    When the list organizations query is executed with offset 0 and limit 1
    Then the result should contain 1 organization with total 2
