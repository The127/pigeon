Feature: Get Event Type Stats

  Retrieve delivery statistics for an event type.

  Scenario: Successfully retrieving event type statistics
    Given an event type with delivery statistics
    When the get event type stats query is executed
    Then the event type statistics should be returned
