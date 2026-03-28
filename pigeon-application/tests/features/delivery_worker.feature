Feature: Delivery Worker

  Scenario: Successful webhook delivery
    Given a pending attempt for endpoint "https://example.com/hook"
    When the worker processes the batch
    Then the attempt should be marked as succeeded
    And the response code should be 200
    And the duration should be recorded

  Scenario: 2xx status codes are treated as success
    Given a pending attempt on attempt number 1 with max retries 3
    And the endpoint returns status 201
    When the worker processes the batch
    Then the attempt should be marked as succeeded

  Scenario: 4xx status codes trigger retry
    Given a pending attempt on attempt number 1 with max retries 3
    And the endpoint returns status 400
    When the worker processes the batch
    Then the attempt should be marked for retry

  Scenario: Failed delivery with retries remaining
    Given a pending attempt on attempt number 1 with max retries 3
    And the endpoint returns status 500
    When the worker processes the batch
    Then the attempt should be marked for retry
    And a next_attempt_at should be computed with exponential backoff

  Scenario: Failed delivery exhausts retries and dead letters
    Given a pending attempt on attempt number 3 with max retries 3
    And the endpoint returns status 500
    When the worker processes the batch
    Then the attempt should be marked as failed
    And a dead letter should be created

  Scenario: Network error triggers retry
    Given a pending attempt on attempt number 1 with max retries 3
    And the endpoint is unreachable
    When the worker processes the batch
    Then the attempt should be marked for retry

  Scenario: Network error on last retry dead letters
    Given a pending attempt on attempt number 3 with max retries 3
    And the endpoint is unreachable
    When the worker processes the batch
    Then the attempt should be marked as failed
    And a dead letter should be created

  Scenario: Empty queue returns zero
    Given no pending attempts
    When the worker processes the batch
    Then zero attempts should be processed

  Scenario: Multiple attempts in a single batch
    Given 3 pending attempts in the queue
    When the worker processes the batch
    Then 3 attempts should be processed
    And all attempts should be marked as succeeded
