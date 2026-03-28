CREATE TABLE endpoint_events (
    endpoint_id   UUID NOT NULL REFERENCES endpoints(id),
    event_type_id UUID NOT NULL REFERENCES event_types(id),
    PRIMARY KEY (endpoint_id, event_type_id)
);
