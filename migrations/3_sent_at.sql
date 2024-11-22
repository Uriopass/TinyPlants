ALTER TABLE water DROP COLUMN notif_sent;
ALTER TABLE water ADD  COLUMN notif_sent_at integer default null;