UPDATE wakes SET thread = 'lead' WHERE thread = 'coordinator';
UPDATE wakes SET thread = 'lead-hosted' WHERE thread = 'coordinator-hosted';
UPDATE wake_faults SET thread = 'lead' WHERE thread = 'coordinator';
UPDATE wake_faults SET thread = 'lead-hosted' WHERE thread = 'coordinator-hosted';
UPDATE wake_faults SET cleared_by = 'lead' WHERE cleared_by = 'coordinator';
UPDATE wake_faults SET cleared_by = 'lead-hosted' WHERE cleared_by = 'coordinator-hosted';
