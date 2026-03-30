Feature: List Audit Log

  List audit log entries for an organization with optional filters
  and pagination.

  Scenario: Listing audit entries when none exist
    Given an organization with no audit entries
    When the list audit log query is executed
    Then the result should be an empty audit log list

  Scenario: Listing audit entries with results
    Given an organization with 2 audit entries
    When the list audit log query is executed
    Then the result should contain 2 audit entries

  Scenario: Listing audit entries respects pagination
    Given an organization with 2 audit entries
    When the list audit log query is executed with offset 0 and limit 1
    Then the result should contain 1 audit entry with total 2
