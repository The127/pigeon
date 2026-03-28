Feature: Event Type CRUD

  Scenario: Successfully creating an event type
    Given a request to create an event type named "user.created" for an application
    When the create event type command is executed
    Then the event type should be created with name "user.created"
    And the event type should have a generated id
    And the event type store should contain the event type

  Scenario: Rejecting an event type with an empty name
    Given a request to create an event type named "" for an application
    When the create event type command is executed
    Then the create event type command should fail with a validation error

  Scenario: Rejecting an event type with a whitespace-only name
    Given a request to create an event type named "   " for an application
    When the create event type command is executed
    Then the create event type command should fail with a validation error

  Scenario: Successfully updating an event type
    Given an existing event type named "user.created"
    When the update event type command is executed with name "user.updated"
    Then the event type should be updated with name "user.updated"
    And the event type store should have saved the event type

  Scenario: Rejecting update with an empty name
    Given an existing event type named "user.created"
    When the update event type command is executed with name ""
    Then the update event type command should fail with a validation error

  Scenario: Rejecting update with a whitespace-only name
    Given an existing event type named "user.created"
    When the update event type command is executed with name "   "
    Then the update event type command should fail with a validation error

  Scenario: Update event type with version conflict
    Given an existing event type named "user.created"
    When the update event type command is executed with a stale version
    Then the update event type command should fail with a conflict error

  Scenario: Successfully deleting an event type
    Given an event type named "user.created" exists
    When the delete event type command is executed
    Then the event type should be deleted successfully
    And the event type store should have deleted the event type

  Scenario: Deleting a non-existent event type
    When the delete event type command is executed for a non-existent event type
    Then the delete event type command should fail with a not found error
