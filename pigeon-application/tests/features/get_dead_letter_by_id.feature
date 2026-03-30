Feature: Get Dead Letter by ID

  Retrieve a dead letter by its unique identifier.

  Scenario: Successfully retrieving an existing dead letter
    Given a dead letter exists in the read store
    When the get dead letter by id query is executed
    Then the dead letter should be returned

  Scenario: Returning none for a non-existent dead letter
    When the get dead letter by id query is executed for a non-existent id
    Then no dead letter should be returned
