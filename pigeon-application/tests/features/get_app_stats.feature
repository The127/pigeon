Feature: Get Application Stats

  Retrieve delivery statistics for an application.

  Scenario: Successfully retrieving application statistics
    Given an application with delivery statistics
    When the get app stats query is executed
    Then the application statistics should be returned
