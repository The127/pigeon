Feature: Auto-disable failing endpoints

  Scenario: Endpoint is disabled after reaching failure threshold
    Given an endpoint with 5 consecutive failures and a threshold of 5
    When the DeadLettered event is processed
    Then the endpoint should be disabled

  Scenario: Endpoint stays enabled below threshold
    Given an endpoint with 3 consecutive failures and a threshold of 5
    When the DeadLettered event is processed
    Then the endpoint should remain enabled

  Scenario: Already disabled endpoint is not re-disabled
    Given an already disabled endpoint with 5 consecutive failures and a threshold of 5
    When the DeadLettered event is processed
    Then the endpoint should remain disabled

  Scenario: Auto-disable is skipped when threshold is zero
    Given an endpoint with 10 consecutive failures and a threshold of 0
    When the DeadLettered event is processed
    Then the endpoint should remain enabled
