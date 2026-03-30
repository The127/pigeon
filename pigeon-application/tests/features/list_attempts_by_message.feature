Feature: List Attempts by Message

  Retrieve all delivery attempts for a specific message.

  Scenario: Listing attempts when none exist
    Given a message with no delivery attempts
    When the list attempts by message query is executed
    Then the result should be an empty attempt list

  Scenario: Listing attempts with results
    Given a message with 2 delivery attempts
    When the list attempts by message query is executed
    Then the result should contain 2 attempts
