Feature: Send Test Event

  Send a test event to a specific endpoint to verify
  that webhook delivery is working correctly.

  Scenario: Successfully sending a test event
    Given an application with the pigeon.test event type and an endpoint
    When the send test event command is executed
    Then a test message should be created
    And a delivery attempt should be created for the endpoint

  Scenario: Sending a test event when event type is missing fails
    Given an application without the pigeon.test event type
    When the send test event command is executed
    Then the send test event should fail with an internal error
