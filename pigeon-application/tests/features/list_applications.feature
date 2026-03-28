Feature: List Applications

  Scenario: Listing when no applications exist
    When the list applications query is executed with offset 0 and limit 10
    Then the result should contain 0 items
    And the total count should be 0

  Scenario: Listing existing applications
    Given the following applications exist:
      | name   | uid      |
      | app-a  | uid_a    |
      | app-b  | uid_b    |
    When the list applications query is executed with offset 0 and limit 10
    Then the result should contain 2 items
    And the total count should be 2
