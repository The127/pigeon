Feature: Retrigger Message

  Retrigger an existing message to create new delivery attempts
  for currently matching endpoints.

  Scenario: Successfully retriggering a message with matching endpoints
    Given a message exists with a matching enabled endpoint
    When the retrigger message command is executed
    Then new delivery attempts should be created
    And the attempt count should match the number of matching endpoints

  Scenario: Retriggering a message with no matching endpoints fails
    Given a message exists with no matching endpoints
    When the retrigger message command is executed
    Then the retrigger should fail with a validation error

  Scenario: Retriggering skips endpoints that already have attempts
    Given a message exists with an endpoint that already has an attempt
    When the retrigger message command is executed
    Then the retrigger should fail with a validation error

  Scenario: Retriggering a non-existent message fails
    When a non-existent message is retriggered
    Then the retrigger should fail with a not found error
