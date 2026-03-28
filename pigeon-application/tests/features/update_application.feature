Feature: Update Application

  Scenario: Successfully updating an application name
    Given an existing application named "old-name" with uid "app_123"
    When the update application command is executed with name "new-name"
    Then the application should be updated with name "new-name"
    And the application store should have saved the application

  Scenario: Rejecting update with an empty name
    Given an existing application named "old-name" with uid "app_456"
    When the update application command is executed with name ""
    Then the update command should fail with a validation error

  Scenario: Rejecting update with a whitespace-only name
    Given an existing application named "old-name" with uid "app_ws"
    When the update application command is executed with name "   "
    Then the update command should fail with a validation error

  Scenario: Rejecting update with a version conflict
    Given an existing application named "old-name" with uid "app_789"
    When the update application command is executed with a stale version
    Then the update command should fail with a conflict error

  Scenario: Rejecting update for a non-existent application
    When the update application command is executed for a non-existent application
    Then the update command should fail with a not found error
