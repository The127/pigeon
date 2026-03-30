Feature: Delete Application

  # Note: Cascade deletion of child entities (endpoints, event types, messages)
  # is handled by PostgreSQL ON DELETE CASCADE constraints.
  # See migration 20260329000002_add_cascade_deletes.sql.

  Scenario: Successfully deleting an application
    Given an application named "doomed-app" with uid "app_del_1" exists
    When the delete application command is executed
    Then the application should be deleted successfully
    And the application store should have deleted the application

  Scenario: Rejecting deletion of a non-existent application
    When the delete application command is executed for a non-existent application
    Then the delete command should fail with a not found error
