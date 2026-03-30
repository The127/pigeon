Feature: Get Endpoint Stats

  Retrieve delivery statistics for an endpoint.

  Scenario: Successfully retrieving endpoint statistics
    Given an endpoint with delivery statistics
    When the get endpoint stats query is executed
    Then the endpoint statistics should be returned
