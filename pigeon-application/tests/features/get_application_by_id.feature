Feature: Get Application By ID

  Scenario: Finding an existing application
    Given an application named "my-app" with uid "app_get_1" has been created
    When the get application by id query is executed
    Then the query should return the application with name "my-app"

  Scenario: Returning none for a non-existent application
    When the get application by id query is executed for a non-existent id
    Then the query should return no application
