Feature: Get Message by ID

  Retrieve a message with delivery status by its unique identifier.

  Scenario: Successfully retrieving an existing message
    Given a message exists in the read store
    When the get message by id query is executed
    Then the message should be returned with status counts

  Scenario: Returning none for a non-existent message
    When the get message by id query is executed for a non-existent id
    Then no message should be returned
