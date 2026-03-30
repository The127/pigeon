Feature: Retry Attempt

  Manually retry a failed delivery attempt, resetting it
  to pending so the delivery worker picks it up again.

  Scenario: Successfully retrying a failed attempt
    Given a failed delivery attempt exists
    When the retry attempt command is executed
    Then the attempt status should be pending

  Scenario: Retrying a non-failed attempt fails
    Given a pending delivery attempt exists
    When the retry attempt command is executed
    Then the retry should fail with a validation error

  Scenario: Retrying a non-existent attempt fails
    When a non-existent attempt is retried
    Then the retry should fail with a not found error
