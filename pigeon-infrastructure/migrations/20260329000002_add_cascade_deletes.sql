-- Add CASCADE deletes for endpoint and application child tables.
-- Without these, deleting an endpoint or application that has
-- attempts/dead-letters/projections fails with FK violations.

-- attempts.endpoint_id → endpoints(id)
ALTER TABLE attempts DROP CONSTRAINT attempts_endpoint_id_fkey;
ALTER TABLE attempts ADD CONSTRAINT attempts_endpoint_id_fkey
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE;

-- attempts.message_id → messages(id)
ALTER TABLE attempts DROP CONSTRAINT attempts_message_id_fkey;
ALTER TABLE attempts ADD CONSTRAINT attempts_message_id_fkey
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE;

-- dead_letters.endpoint_id → endpoints(id)
ALTER TABLE dead_letters DROP CONSTRAINT dead_letters_endpoint_id_fkey;
ALTER TABLE dead_letters ADD CONSTRAINT dead_letters_endpoint_id_fkey
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE;

-- dead_letters.message_id → messages(id)
ALTER TABLE dead_letters DROP CONSTRAINT dead_letters_message_id_fkey;
ALTER TABLE dead_letters ADD CONSTRAINT dead_letters_message_id_fkey
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE;

-- dead_letters.app_id → applications(id)
ALTER TABLE dead_letters DROP CONSTRAINT dead_letters_app_id_fkey;
ALTER TABLE dead_letters ADD CONSTRAINT dead_letters_app_id_fkey
    FOREIGN KEY (app_id) REFERENCES applications(id) ON DELETE CASCADE;

-- endpoint_events.endpoint_id → endpoints(id)
ALTER TABLE endpoint_events DROP CONSTRAINT endpoint_events_endpoint_id_fkey;
ALTER TABLE endpoint_events ADD CONSTRAINT endpoint_events_endpoint_id_fkey
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE;

-- endpoint_events.event_type_id → event_types(id)
ALTER TABLE endpoint_events DROP CONSTRAINT endpoint_events_event_type_id_fkey;
ALTER TABLE endpoint_events ADD CONSTRAINT endpoint_events_event_type_id_fkey
    FOREIGN KEY (event_type_id) REFERENCES event_types(id) ON DELETE CASCADE;

-- endpoint_delivery_summary.endpoint_id → endpoints(id)
ALTER TABLE endpoint_delivery_summary DROP CONSTRAINT endpoint_delivery_summary_endpoint_id_fkey;
ALTER TABLE endpoint_delivery_summary ADD CONSTRAINT endpoint_delivery_summary_endpoint_id_fkey
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE;

-- message_delivery_status.message_id → messages(id)
ALTER TABLE message_delivery_status DROP CONSTRAINT message_delivery_status_message_id_fkey;
ALTER TABLE message_delivery_status ADD CONSTRAINT message_delivery_status_message_id_fkey
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE;

-- messages.app_id → applications(id)
ALTER TABLE messages DROP CONSTRAINT messages_app_id_fkey;
ALTER TABLE messages ADD CONSTRAINT messages_app_id_fkey
    FOREIGN KEY (app_id) REFERENCES applications(id) ON DELETE CASCADE;

-- messages.event_type_id → event_types(id)
ALTER TABLE messages DROP CONSTRAINT messages_event_type_id_fkey;
ALTER TABLE messages ADD CONSTRAINT messages_event_type_id_fkey
    FOREIGN KEY (event_type_id) REFERENCES event_types(id) ON DELETE CASCADE;

-- endpoints.app_id → applications(id)
ALTER TABLE endpoints DROP CONSTRAINT endpoints_app_id_fkey;
ALTER TABLE endpoints ADD CONSTRAINT endpoints_app_id_fkey
    FOREIGN KEY (app_id) REFERENCES applications(id) ON DELETE CASCADE;

-- event_types.app_id → applications(id)
ALTER TABLE event_types DROP CONSTRAINT event_types_app_id_fkey;
ALTER TABLE event_types ADD CONSTRAINT event_types_app_id_fkey
    FOREIGN KEY (app_id) REFERENCES applications(id) ON DELETE CASCADE;
