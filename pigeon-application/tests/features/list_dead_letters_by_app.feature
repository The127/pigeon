Feature: List Dead Letters by Application

  List dead letters for an application with optional filters and pagination.

  Scenario: Listing dead letters when none exist
    Given an application with no dead letters
    When the list dead letters query is executed
    Then the result should be an empty dead letter list

  Scenario: Listing dead letters with results
    Given an application with 2 dead letters
    When the list dead letters query is executed
    Then the result should contain 2 dead letters

  Scenario: Listing dead letters respects pagination
    Given an application with 2 dead letters
    When the list dead letters query is executed with offset 0 and limit 1
    Then the result should contain 1 dead letter with total 2
