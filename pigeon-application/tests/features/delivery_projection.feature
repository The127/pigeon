Feature: Delivery Projection

  Scenario: MessageCreated initializes message delivery status
    When a MessageCreated event with 3 attempts is processed
    Then the message delivery status should show 3 attempts created
    And all delivery counters should be zero

  Scenario: AttemptSucceeded updates endpoint and message projections
    Given a message delivery status exists
    When an AttemptSucceeded event is processed
    Then the endpoint summary should show 1 success and 0 consecutive failures
    And the message delivery status should show 1 succeeded

  Scenario: AttemptFailed with no retry updates both projections
    Given a message delivery status exists
    When an AttemptFailed event with will_retry false is processed
    Then the endpoint summary should show 1 failure and 1 consecutive failure
    And the message delivery status should show 1 failed

  Scenario: AttemptFailed with retry only updates endpoint summary
    Given a message delivery status exists
    When an AttemptFailed event with will_retry true is processed
    Then the endpoint summary should show 1 failure and 1 consecutive failure
    And the message delivery status should show 0 failed

  Scenario: Consecutive successes reset failure count
    Given an endpoint with 3 consecutive failures in the summary
    When an AttemptSucceeded event is processed for that endpoint
    Then the endpoint summary should show 0 consecutive failures

  Scenario: DeadLettered increments dead letter count
    Given a message delivery status exists
    When a DeadLettered event is processed
    Then the message delivery status should show 1 dead lettered
