Feature: Delete Organization

  Scenario: Successfully deleting an organization
    Given an organization named "doomed-org" with slug "doomed-org" exists
    When the delete organization command is executed
    Then the organization should be deleted successfully
    And the organization store should have deleted the organization

  Scenario: Rejecting deletion of a non-existent organization
    When the delete organization command is executed for a non-existent organization
    Then the delete organization command should fail with a not found error
