Feature: List Endpoints by Application

  List endpoints belonging to an application with pagination.

  Scenario: Listing endpoints when none exist
    Given an application with no endpoints
    When the list endpoints query is executed
    Then the result should be an empty endpoint list

  Scenario: Listing endpoints with results
    Given an application with 2 endpoints
    When the list endpoints query is executed
    Then the result should contain 2 endpoints

  Scenario: Listing endpoints respects pagination
    Given an application with 2 endpoints
    When the list endpoints query is executed with offset 0 and limit 1
    Then the result should contain 1 endpoint with total 2
