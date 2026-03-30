Feature: List Messages by Application

  List messages for an application with optional filters and pagination.

  Scenario: Listing messages when none exist
    Given an application with no messages
    When the list messages query is executed
    Then the result should be an empty message list

  Scenario: Listing messages with results
    Given an application with 2 messages
    When the list messages query is executed
    Then the result should contain 2 messages

  Scenario: Listing messages respects pagination
    Given an application with 2 messages
    When the list messages query is executed with offset 0 and limit 1
    Then the result should contain 1 message with total 2
