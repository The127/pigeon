Feature: Replay Dead Letter

  Replay a dead-lettered message, creating a new delivery attempt
  so the webhook can be retried.

  Scenario: Successfully replaying a dead letter
    Given a dead letter exists that has not been replayed
    When the replay dead letter command is executed
    Then the dead letter should be marked as replayed
    And a new delivery attempt should be created

  Scenario: Replaying an already-replayed dead letter fails
    Given a dead letter exists that has already been replayed
    When the replay dead letter command is executed
    Then the replay should fail with a validation error

  Scenario: Replaying a non-existent dead letter fails
    When a non-existent dead letter is replayed
    Then the replay should fail with a not found error
