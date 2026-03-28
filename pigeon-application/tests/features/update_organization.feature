Feature: Update Organization

  Scenario: Successfully updating an organization name
    Given an existing organization named "old-name" with slug "old-slug"
    When the update organization command is executed with name "new-name"
    Then the organization should be updated with name "new-name"
    And the organization store should have saved the organization

  Scenario: Rejecting update with an empty name
    Given an existing organization named "old-name" with slug "old-slug-2"
    When the update organization command is executed with name ""
    Then the update organization command should fail with a validation error

  Scenario: Rejecting update with a whitespace-only name
    Given an existing organization named "old-name" with slug "old-slug-ws"
    When the update organization command is executed with name "   "
    Then the update organization command should fail with a validation error

  Scenario: Rejecting update with a version conflict
    Given an existing organization named "old-name" with slug "old-slug-3"
    When the update organization command is executed with a stale version
    Then the update organization command should fail with a conflict error

  Scenario: Rejecting update for a non-existent organization
    When the update organization command is executed for a non-existent organization
    Then the update organization command should fail with a not found error
