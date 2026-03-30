Feature: List Event Types by Application

  List event types belonging to an application with pagination.

  Scenario: Listing event types when none exist
    Given an application with no event types
    When the list event types query is executed
    Then the result should be an empty event type list

  Scenario: Listing event types with results
    Given an application with 2 event types
    When the list event types query is executed
    Then the result should contain 2 event types

  Scenario: Listing event types respects pagination
    Given an application with 2 event types
    When the list event types query is executed with offset 0 and limit 1
    Then the result should contain 1 event type with total 2
