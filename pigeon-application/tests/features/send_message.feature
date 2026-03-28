Feature: Send Message

  Scenario: Successfully sending a message with matching endpoints
    Given an application with two enabled endpoints for event type "user.created"
    When the send message command is executed with event type "user.created"
    Then the message should be created
    And 2 attempts should be created
    And the message should not be a duplicate

  Scenario: Duplicate message is idempotent
    Given an application with a previously sent message with idempotency key "key-1"
    When the send message command is executed with idempotency key "key-1"
    Then the message should be a duplicate
    And 0 attempts should be created

  Scenario: Sending to an app with no matching endpoints
    Given an application with no endpoints for event type "order.placed"
    When the send message command is executed with event type "order.placed"
    Then the message should be created
    And 0 attempts should be created
    And the message should not be a duplicate

  Scenario: Rejecting a non-object payload
    Given an application with no endpoints for event type "test.event"
    When the send message command is executed with a non-object payload
    Then the send message command should fail with a validation error
