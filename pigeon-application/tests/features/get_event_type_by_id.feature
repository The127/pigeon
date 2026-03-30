Feature: Get Event Type by ID

  Retrieve an event type by its unique identifier.

  Scenario: Successfully retrieving an existing event type
    Given an event type exists in the read store
    When the get event type by id query is executed
    Then the event type should be returned

  Scenario: Returning none for a non-existent event type
    When the get event type by id query is executed for a non-existent id
    Then no event type should be returned
