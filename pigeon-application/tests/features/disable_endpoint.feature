Feature: Disable Endpoint

  Internal saga command that disables an endpoint after
  repeated delivery failures.

  Scenario: Successfully disabling an enabled endpoint
    Given an enabled endpoint exists
    When the disable endpoint command is executed
    Then the endpoint should be disabled
    And an endpoint updated event should be emitted

  Scenario: Disabling an already-disabled endpoint is a no-op
    Given a disabled endpoint exists
    When the disable endpoint command is executed
    Then the command should succeed without changes

  Scenario: Disabling a non-existent endpoint fails
    When a non-existent endpoint is disabled
    Then the disable should fail with a not found error
