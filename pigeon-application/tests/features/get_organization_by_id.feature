Feature: Get Organization by ID

  Retrieve an organization by its unique identifier.

  Scenario: Successfully retrieving an existing organization
    Given an organization exists in the read store
    When the get organization by id query is executed
    Then the organization should be returned

  Scenario: Returning none for a non-existent organization
    When the get organization by id query is executed for a non-existent id
    Then no organization should be returned
