Feature: Get Endpoint by ID

  Retrieve an endpoint by its unique identifier.

  Scenario: Successfully retrieving an existing endpoint
    Given an endpoint exists in the read store
    When the get endpoint by id query is executed
    Then the endpoint should be returned

  Scenario: Returning none for a non-existent endpoint
    When the get endpoint by id query is executed for a non-existent id
    Then no endpoint should be returned
